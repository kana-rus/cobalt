#![cfg(feature="__rt_native__")]

mod read;
mod write;

use self::read::read;
use self::write::{write, Upgrade};

use crate::__rt__::TcpStream;
use crate::util::timeout_in;
use crate::router::RadixRouter;

use std::{any::Any, pin::Pin, sync::Arc, time::Duration};
use std::panic::{AssertUnwindSafe, catch_unwind};

use whttp::{Response, request::parse, header::Connection};


mod env {
    #![allow(unused, non_snake_case)]

    use std::sync::OnceLock;

    pub(crate) fn OHKAMI_KEEPALIVE_TIMEOUT() -> u64 {
        static OHKAMI_KEEPALIVE_TIMEOUT: OnceLock<u64> = OnceLock::new();
        *OHKAMI_KEEPALIVE_TIMEOUT.get_or_init(|| {
            std::env::var("OHKAMI_KEEPALIVE_TIMEOUT").ok()
                .map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
        })
    }

    #[cfg(feature="ws")]
    pub(crate) fn OHKAMI_WEBSOCKET_TIMEOUT() -> u64 {
        static OHKAMI_WEBSOCKET_TIMEOUT: OnceLock<u64> = OnceLock::new();
        *OHKAMI_WEBSOCKET_TIMEOUT.get_or_init(|| {
            std::env::var("OHKAMI_WEBSOCKET_TIMEOUT").ok()
                .map(|v| v.parse().ok()).flatten()
                .unwrap_or(1 * 60 * 60)
        })
    }
}

pub(crate) struct Session {
    router:     Arc<RadixRouter>,
    connection: TcpStream,
    ip:         std::net::IpAddr,
}
impl Session {
    pub(crate) fn new(
        router:     Arc<RadixRouter>,
        connection: TcpStream,
        ip:         std::net::IpAddr
    ) -> Self {
        Self {
            router,
            connection,
            ip
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold] #[inline(never)]
        fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                crate::warning!("[Panicked]: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                crate::warning!("[Panicked]: {msg}");
            } else {
                crate::warning!("[Panicked]");
            }
            crate::Response::InternalServerError()
        }

        match timeout_in(Duration::from_secs(env::OHKAMI_KEEPALIVE_TIMEOUT()), async {
            let mut req = parse::new();
            let mut req = Pin::new(&mut req);
            loop {
                parse::clear(&mut req);
                match read(req.as_mut(), &mut self.connection).await {
                    Ok(Some(())) => {
                        let close = matches!(req.header(Connection), Some("close" | "Close"));

                        let res = match catch_unwind(AssertUnwindSafe({
                            let req = req.as_mut();
                            || self.router.handle(self.ip, req.get_mut())
                        })) {
                            Ok(future) => future.await,
                            Err(panic) => panicking(panic),
                        };
                        let upgrade = write(res, &mut self.connection).await;

                        if !(matches!(upgrade, Upgrade::None)) {break upgrade}
                        if close {break Upgrade::None}
                    }
                    Ok(None) => break Upgrade::None,
                    Err(err) => {write(Response::of(err), &mut self.connection).await;},
                }
            }
        }).await {
            Some(Upgrade::None) | None => {
                crate::DEBUG!("about to shutdown connection");
            }

            #[cfg(feature="ws")]
            Some(Upgrade::WebSocket((config, handler))) => {
                use crate::ws::{Connection, Message, CloseFrame, CloseCode};

                crate::DEBUG!("WebSocket session started");

                let mut conn = Connection::new(self.connection, config);

                let close = timeout_in(Duration::from_secs(env::OHKAMI_WEBSOCKET_TIMEOUT()),
                    handler(conn.clone())
                ).await;

                if !conn.is_closed() {
                    conn.send(Message::Close(Some(match close {
                        Some(_) => {
                            crate::DEBUG!("Closing WebSocket session...");
                            CloseFrame {
                                code:   CloseCode::Normal,
                                reason: None
                            }
                        }
                        None => {
                            crate::warning!("[WARNING] WebSocket session is aborted by `OHKAMI_WEBSOCKET_TIMEOUT` (default to 1 hour, and can be set via environment variable)");
                            CloseFrame {
                                code:   CloseCode::Library(4000),
                                reason: Some("OHKAMI_WEBSOCKET_TIMEOUT".into())
                            }
                        }
                    }))).await.expect("Failed to send close message");
                }

                crate::DEBUG!("WebSocket session finished");
            }
        }
    }
}
