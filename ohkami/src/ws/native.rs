#![cfg(feature="ws")]

pub use mews::{
    Message,
    CloseCode, CloseFrame,
    Config,
    Connection,
    ReadHalf, WriteHalf,
    WebSocket as Session,
    connection,
    split,
};

use crate::{__rt__, FromRequest, IntoResponse, Request, Response};

impl<'ctx> super::WebSocketContext<'ctx> {
    /// create a `WebSocket` with the handler and default `Config`.
    /// use [`upgrade_with`](WebSocketContext::upgrade_with) to provide a custom config.
    /// 
    /// ## handler
    /// 
    /// any 'static `FnOnce(Connection) -> {impl Future<Output = ()> + Send} + Send + Sync`
    pub fn upgrade<H, F, C: mews::connection::UnderlyingConnection>(
        self,
        handler: H
    ) -> WebSocket<C>
    where
        H: FnOnce(Connection<C>) -> F + Send + Sync + 'static,
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.upgrade_with(Config::default(), handler)
    }

    /// create a `WebSocket` with the config and handler.
    /// 
    /// ## handler
    /// 
    /// any 'static `FnOnce(Connection) -> {impl Future<Output = ()> + Send} + Send + Sync`
    pub fn upgrade_with<H, F, C: mews::connection::UnderlyingConnection>(self,
        config:  Config,
        handler: H
    ) -> WebSocket<C>
    where
        H: FnOnce(Connection<C>) -> F + Send + Sync + 'static,
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let (sign, session) = mews::WebSocketContext::new(self.sec_websocket_key)
            .with(config)
            .on_upgrade(handler);
        WebSocket { sign, session }
    }
}

/// # Response for upgrading to WebSocket
/// 
/// Perform handshake with a `WebSocketContext`,
/// establish a WebSocket connection,
/// and run the given handler.
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|mut conn| async move {
///         conn.send("Hello, WebSocket! and bye...").await
///             .expect("failed to send")
///     })
/// }
/// ```
/// 
/// <br>
/// 
/// *split_example.rs*
/// ```
/// # use tokio::{join, spawn};
/// # use tokio::time::{Duration, sleep};
/// # 
/// use ohkami::ws::{WebSocketContext, WebSocket, Message};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|c| async {
///         let (mut r, mut w) = c.split();
///         tokio::join!( /* joining is necessary to prevent resource leak or unsafe situations */
///             tokio::spawn(async move {
///                 while let Some(Message::Text(
///                     text
///                 )) = r.recv().await.expect("failed to recieve") {
///                     println!("[->] {text}");
///                     if text == "close" {break}
///                 }
///             }),
///             tokio::spawn(async move {
///                 for text in [
///                     "abc",
///                     "def",
///                     "ghi",
///                     "jk",
///                     "lmno",
///                     "pqr",
///                     "stuvw",
///                     "xyz"
///                 ] {
///                     println!("[<-] {text}");
///                     w.send(text).await.expect("failed to send text");
///                     sleep(Duration::from_secs(1)).await;
///                 }
///             })
///         );
///     })
/// }
/// ```
pub struct WebSocket<C: mews::connection::UnderlyingConnection = __rt__::TcpStream> {
    sign:    String,
    session: Session<C>,
}
impl IntoResponse for WebSocket {
    fn into_response(self) -> Response {
        Response::SwitchingProtocols().with_headers(|h|h
            .Connection("Upgrade")
            .Upgrade("websocket")
            .SecWebSocketAccept(self.sign)
        ).with_websocket(self.session)
    }
}