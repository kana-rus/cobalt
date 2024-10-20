use crate::{Request, Status};
use crate::__rt__::{TcpStream, AsyncReader as _};
use std::pin::Pin;
use whttp::{request::parse, header::ContentLength};

const PAYLOAD_LIMIT: usize = 1 << 32;

pub(super) async fn read(
    mut req: Pin<&mut Request>,
    conn: &mut TcpStream
) -> Result<Option<()>, Status> {
    let buf = parse::buf(req.as_mut());

    match conn.read(&mut **buf).await {
        Err(e) => return match e.kind() {
            std::io::ErrorKind::ConnectionReset => Ok(None),
            _ => Err((|err| {
                crate::warning!("failed to read Request: {err}");
                Status::InternalServerError
            })(e))
        },
        Ok(0) => return Ok(None),
        _ => ()
    }

    let mut r = byte_reader::Reader::new(unsafe {
        // tell `r` to refer `buf` detouchedly
        // 
        // SAFETY: `buf` of `Request` is immutable after `parse::buf`
        std::mem::transmute(buf.as_slice())
    });

    unsafe {parse::method(&mut req, r.read_while(|&b| b != b' '))}?;

    r.next_if(|&b| b == b' ').ok_or(Status::BadRequest)?;

    unsafe {parse::path(&mut req, r.read_while(|b| !matches!(b, b' '|b'?')))}?;

    if r.peek() == Some(&b'?') {
        unsafe {parse::query(&mut req, r.read_while(|&b| b != b' '))}?;
    }

    r.next_if(|&b| b == b' ').ok_or(Status::BadRequest)?;

    r.consume("HTTP/1.1\r\n").ok_or(Status::HTTPVersionNotSupported)?;

    while r.consume("\r\n").is_none() {
        let name  = r.read_while(|&b| b != b':');
        r.consume(": ").ok_or(Status::BadRequest)?;
        let value = r.read_while(|&b| b != b'\r');
        r.consume("\r\n").ok_or(Status::BadRequest)?;
        unsafe {parse::header(&mut req, name, value)}?;
    }

    match req
        .header(ContentLength)
        .map(|v| v.bytes().fold(0, |n, b| 10*n + (b - b'0') as usize))
    {
        None | Some(0) => (),
        Some(PAYLOAD_LIMIT..) => return Err(Status::PayloadTooLarge),
        Some(n) => read_body(req, conn, r.remaining(), n).await
    }

    Ok(Some(()))
}

async fn read_body(
    mut req:        Pin<&mut Request>,
    conn:           &mut TcpStream,
    remaining_buf:  &[u8],
    content_length: usize,
) {
    let remaining_buf_len = remaining_buf.len();

    if remaining_buf_len == 0 || remaining_buf[0] == 0 {
        crate::DEBUG!("\n[read_body] case: remaining_buf.is_empty() || remaining_buf[0] == 0\n");

        let mut body = vec![0; content_length];
        conn.read_exact(&mut body).await.unwrap();
        parse::body_own(&mut req, body);

    } else if content_length <= remaining_buf_len {
        crate::DEBUG!("\n[read_body] case: starts_at + size <= BUF_SIZE\n");

        let body = unsafe {remaining_buf.get_unchecked(..content_length)};
        unsafe {parse::body_ref(&mut req, body)}

    } else {
        crate::DEBUG!("\n[read_body] case: else\n");

        let mut body = vec![0; content_length];
        unsafe {body.get_unchecked_mut(..remaining_buf_len)}.copy_from_slice(remaining_buf);
        conn.read_exact(unsafe {body.get_unchecked_mut(remaining_buf_len..)}).await.unwrap();
        parse::body_own(&mut req, body);
    }
}
