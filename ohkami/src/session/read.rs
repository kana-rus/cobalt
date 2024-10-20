use crate::{Request, Status};
use crate::__rt__::TcpStream;
use std::pin::Pin;

pub(super) async fn read(
    mut req: Pin<&mut Request>,
    conn: &mut TcpStream
) -> Result<Option<()>, Status> {
    todo!()
}
