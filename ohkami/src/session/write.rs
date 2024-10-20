use crate::Response;
use crate::__rt__::TcpStream;

pub(super) enum Upgrade {
    None,

    #[cfg(feature="ws")]
    WebSocket()
}

pub(super) async fn write(mut res: Response, conn: &mut TcpStream) -> Upgrade {
    todo!()
}
