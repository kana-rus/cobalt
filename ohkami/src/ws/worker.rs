pub use mews::{Message, CloseFrame, CloseCode};

use worker::{WebSocketPair, EventStream, wasm_bindgen_futures};

impl<'req> super::WebSocketContext<'req> {
    pub fn upgrade<H, F>(
        self,
        handler: H
    ) -> WebSocket
    where
        H: FnOnce(Connection) -> F,
        F: std::future::Future<Output = ()> + 'static
    {
        let WebSocketPair {
            client: session,
            server: ws
        } = WebSocketPair::new().expect("failed to create WebSocketPair");

        ws.accept().ok();
        wasm_bindgen_futures::spawn_local(handler(Connection::new(ws)));

        WebSocket(session)
    }

    pub async fn upgrade_durable(
        self,
        durable_object: worker::Stub
    ) -> Result<WebSocket, worker::Error> {
        self.upgrade_durable_with(worker::Request::new_with_init(
            "http://ws",
            worker::RequestInit::new().with_headers(worker::Headers::from_iter([
                ("Upgrade", "websocket")
            ]))
        ).unwrap(), durable_object).await
    }
    pub async fn upgrade_durable_with(
        self,
        req: worker::Request,
        durable_object: worker::Stub
    ) -> Result<WebSocket, worker::Error> {
        durable_object.fetch_with_request(req).await?
            .websocket().map(WebSocket)
            .ok_or_else(|| worker::Error::RustError(format!("given Durable Object stub didn't respond with WebSocket")))
    }
}

pub struct Connection {
    ws:     worker::WebSocket,
    events: Option<EventStream<'static>>
}
impl Connection {
    fn new(ws: worker::WebSocket) -> Self {
        Self { ws, events:None }
    }
}
impl Connection {
    pub async fn recv(&mut self) -> Result<Option<Message>, worker::Error> {
        use std::mem::transmute as unchecked_static;
        use ohkami_lib::StreamExt;
        use worker::{WebsocketEvent, worker_sys::web_sys::MessageEvent, js_sys::Uint8Array};

        if self.events.is_none() {
            crate::DEBUG!("[ws::Connection::recv] initial call: setting events");

            self.events = Some(match self.ws.events() {
                Ok(events) => unsafe {unchecked_static(events)},
                Err(error) => return Err(error)
            });
        }

        match (unsafe {self.events.as_mut().unwrap_unchecked()}).next().await {
            None            => Ok(None),
            Some(Err(err))  => Err(err),
            Some(Ok(event)) => Ok(Some(match event {
                WebsocketEvent::Close(event) => {
                    crate::DEBUG!("[ws::Connection::recv] close");
                    Message::Close(Some(CloseFrame {
                        code:   CloseCode::from_u16(event.code()),
                        reason: Some(event.reason().into())
                    }))
                }
                WebsocketEvent::Message(event) => {
                    let data = AsRef::<MessageEvent>::as_ref(&event).data();
                    if data.is_string() {
                        let data = data.as_string();
                        crate::DEBUG!("[ws::Connection::recv] data.is_string(): `{data:?}`");
                        Message::Text(data.ok_or(worker::Error::BadEncoding)?)
                    } else if data.is_object() {
                        let data = Uint8Array::new(&data).to_vec();
                        crate::DEBUG!("[ws::Connection::recv] not data.is_string() but data.is_object(): `{data:?}`");
                        Message::Binary(data)
                    } else {
                        crate::DEBUG!("[ws::Connection::recv] NOT data.is_object()");
                        return Err(worker::Error::Infallible)
                    }
                }
            }))
        }
    }

    #[inline]
    pub async fn send(&mut self, message: impl Into<Message>) -> Result<(), worker::Error> {
        let message = message.into();
        match message {
            Message::Text(text)         => self.ws.send_with_str(text),
            Message::Binary(binary)     => self.ws.send_with_bytes(binary),
            Message::Close(None)        => self.ws.close::<&str>(None, None),
            Message::Close(Some(frame)) => self.ws.close(Some(frame.code.as_u16()), frame.reason),
            Message::Ping(_) | Message::Pong(_) => Err(worker::Error::RustError((|message| {
                format!("`Connection::send` got `{message:?}`, but sending ping/pong is not supported on `rt_worker`")
            })(message)))
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        // https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/close
        // 
        // > If the connection is already CLOSED, this method does nothing.
        self.ws.close::<&str>(None, None).ok();
    }
}

pub type Session = ::worker::WebSocket;

pub struct WebSocket(Session);
impl crate::IntoResponse for WebSocket {
    fn into_response(self) -> crate::Response {
        crate::Response::SwitchingProtocols().with_websocket(self.0)
        // let `worker` crate and Cloudflare Workers to do around
        // headers and something other
    }
}

/// A utility struct for storing *session* instances of type `S` associated with `worker::WebSocket`s.
/// Primarily used in implementations of a Durable Object with WebSocket.
pub struct SessionMap<S>(
    Vec<(worker::WebSocket, S)>
);
impl<S> SessionMap<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, ws: worker::WebSocket, session: S) -> worker::Result<()> {
        if let Some(previos_session) = self.get_mut(&ws) {
            *previos_session = session
        } else {
            self.0.push((ws, session));
        }
        Ok(())
    }
    pub fn remove(&mut self, ws: &worker::WebSocket) -> Option<S> {
        ws.close::<&str>(None, None).ok();
        if let Some(index) = self.index_of(ws) {
            Some(self.0.remove(index).1)
        } else {
            None
        }
    }
    fn index_of(&self, ws: &worker::WebSocket) -> Option<usize> {
        self.0.iter().position(|(w, _)| w == ws)
    }

    pub fn get(&self, ws: &worker::WebSocket) -> Option<&S> {
        let index = self.index_of(&ws)?;
        let (_, session) = self.0.get(index)?;
        Some(session)
    }
    pub fn get_mut(&mut self, ws: &worker::WebSocket) -> Option<&mut S> {
        let index = self.index_of(&ws)?;
        let (_, session) = self.0.get_mut(index)?;
        Some(session)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(worker::WebSocket, S)> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (worker::WebSocket, S)> {
        self.0.iter_mut()
    }
}