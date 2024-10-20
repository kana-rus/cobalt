use crate::__rt__::TcpStream;
use whttp::response::{Response, Body};

pub(super) enum Upgrade {
    None,

    #[cfg(feature="ws")]
    WebSocket()
}

pub(super) async fn write(
    mut res: Response,
    conn: &mut TcpStream
) -> Upgrade {
    match res.take_body() {
        None => 
    }

    todo!()
}
