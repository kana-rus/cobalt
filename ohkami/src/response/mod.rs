
mod status;
pub use status::Status;

mod headers;
pub use headers::{Headers as ResponseHeaders, SetHeaders};

#[cfg(feature="DEBUG")]
pub use headers::Header as ResponseHeader;

mod content;
pub use content::Content;

mod into_response;
pub use into_response::IntoResponse;

#[cfg(feature="__rt__")] mod cache;

#[cfg(test)] mod _test;
#[cfg(test)] mod _test_headers;

use std::borrow::Cow;
use ohkami_lib::{CowSlice, Slice};

#[cfg(feature="__rt_native__")]
use crate::__rt__::AsyncWrite;
#[cfg(feature="sse")]
use crate::util::StreamExt;


/// # HTTP Response
/// 
/// Composed of
/// 
/// - `status`
/// - `headers`
/// - `content`
/// 
/// <br>
/// 
/// ## Usages
/// 
/// ---
/// 
/// *in_fang.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// #[derive(Clone)]
/// struct SetHeaders;
/// impl FangAction for SetHeaders {
///     async fn back<'a>(&'a self, res: &'a mut Response) {
///         res.headers.set()
///             .Server("ohkami")
///             .Vary("Origin");
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(SetHeaders,
///         "/".GET(|| async {"Hello, ohkami!"})
///     ).howl("localhost:5050").await
/// }
/// ```
/// 
/// ---
/// 
/// *into_response.rs*
/// ```
/// use ohkami::{Response, IntoResponse, Status};
/// 
/// enum AppError {
///     A(String),
///     B(String),
/// }
/// impl IntoResponse for AppError {
///     fn into_response(self) -> Response {
///         match self {
///             Self::A(msg) => Response::InternalServerError().with_text(msg),
///             Self::B(msg) => Response::BadRequest().with_text(msg),
///         }
///     }
/// }
/// 
/// async fn handler(id: usize) -> Result<String, AppError> {
///     if id == 0 {
///         return Err(AppError::B("id must be positive".into()))
///     }
/// 
///     Ok("Hello, Response!".into())
/// }
/// ```
pub struct Response {
    /// HTTP status of this response
    pub status: Status,

    /// Headers of this response
    /// 
    /// - `.{Name}()`, `.get({Name})` to get the value
    /// - `.set().{Name}({action})`, `.set().x({Name}, {action})` to mutate the value
    /// 
    /// ---
    /// 
    /// `{action}`:
    /// - just `{value}` to insert
    /// - `None` to remove
    /// - `header::append({value})` to append
    /// 
    /// `{value}`:
    /// - `String`
    /// - `&'static str`
    /// - `Cow<'static, str>`
    /// - `Some(Cow<'static, str>)`
    pub headers: ResponseHeaders,

    pub(crate) content: Content,
}

impl Response {
    #[inline(always)]
    pub fn of(status: Status) -> Self {
        Self {
            status,
            headers: ResponseHeaders::new(),
            content: Content::None,
        }
    }

    #[cfg(feature="__rt__")]
    /// Complete HTTP spec
    #[inline(always)]
    pub(crate) fn complete(&mut self) {
        self.headers.set().Date(
            // SAFETY: This value is immediately used in sending process and disposed
            unsafe {self::cache::imf_fixdate_now()}
        );

        match &self.content {
            Content::None => {
                match self.status {
                    Status::NoContent => self.headers.set()
                        .ContentLength(None),
                    _ => self.headers.set()
                        .ContentLength("0")
                };
            }

            Content::Payload(bytes) => {
                self.headers.set()
                    .ContentLength(ohkami_lib::num::itoa(bytes.len()));
            }

            #[cfg(feature="sse")]
            Content::Stream(_) => {
                self.headers.set()
                    .ContentLength(None);
            }

            #[cfg(feature="ws")]
            Content::WebSocket(_) => {
                self.headers.set()
                    .ContentLength(None);
            }
        };
    }
}

impl Response {
    #[inline]
    pub fn with_headers(mut self, h: impl FnOnce(SetHeaders)->SetHeaders) -> Self {
        h(self.headers.set());
        self
    }

    pub fn drop_content(&mut self) -> Content {
        let old_content = self.content.take();
        self.headers.set()
            .ContentType(None)
            .ContentLength(None);
        old_content
    }
    pub fn without_content(mut self) -> Self {
        let _ = self.drop_content();
        self
    }

    pub fn set_payload(&mut self,
        content_type: &'static str,
        content:      impl Into<Cow<'static, [u8]>>,
    ) {
        let content = content.into();
        self.headers.set()
            .ContentType(content_type)
            .ContentLength(content.len().to_string());
        self.content = Content::Payload(content.into());
    }
    pub fn with_payload(mut self,
        content_type: &'static str,
        content:      impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        self.set_payload(content_type, content);
        self
    }
    pub fn payload(&self) -> Option<&[u8]> {
        self.content.as_bytes()
    }

    #[inline] pub fn set_text<Text: Into<Cow<'static, str>>>(&mut self, text: Text) {
        let body = text.into();

        self.headers.set()
            .ContentType("text/plain; charset=UTF-8");
        self.content = Content::Payload(match body {
            Cow::Borrowed(str) => CowSlice::Ref(Slice::from_bytes(str.as_bytes())),
            Cow::Owned(string) => CowSlice::Own(string.into_bytes().into()),
        });
    }
    #[inline(always)] pub fn with_text<Text: Into<Cow<'static, str>>>(mut self, text: Text) -> Self {
        self.set_text(text);
        self
    }

    #[inline(always)]
    pub fn set_html<HTML: Into<Cow<'static, str>>>(&mut self, html: HTML) {
        let body = html.into();

        self.headers.set()
            .ContentType("text/html; charset=UTF-8");
        self.content = Content::Payload(match body {
            Cow::Borrowed(str) => CowSlice::Ref(Slice::from_bytes(str.as_bytes())),
            Cow::Owned(string) => CowSlice::Own(string.into_bytes().into()),
        });
    }
    #[inline(always)]
    pub fn with_html<HTML: Into<Cow<'static, str>>>(mut self, html: HTML) -> Self {
        self.set_html(html);
        self
    }

    #[inline(always)]
    pub fn set_json<JSON: serde::Serialize>(&mut self, json: JSON) {
        let body = ::serde_json::to_vec(&json).unwrap();

        self.headers.set()
            .ContentType("application/json");
        self.content = Content::Payload(body.into());
    }
    #[inline(always)]
    pub fn with_json<JSON: serde::Serialize>(mut self, json: JSON) -> Self {
        self.set_json(json);
        self
    }

    /// SAFETY: Argument `json_lit` is **valid JSON**
    pub unsafe fn set_json_lit<JSONLiteral: Into<Cow<'static, str>>>(&mut self, json_lit: JSONLiteral) {
        let body = match json_lit.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };

        self.headers.set()
            .ContentType("application/json");
        self.content = Content::Payload(body.into());
    }
    /// SAFETY: Argument `json_lit` is **valid JSON**
    pub unsafe fn with_json_lit<JSONLiteral: Into<Cow<'static, str>>>(mut self, json_lit: JSONLiteral) -> Self {
        self.set_json_lit(json_lit);
        self
    }
}

#[cfg(feature="sse")]
impl Response {
    #[inline]
    pub fn with_stream<
        T: Into<String>,
        E: std::error::Error,
    >(mut self,
        stream: impl ohkami_lib::Stream<Item = Result<T, E>> + Unpin + Send + 'static
    ) -> Self {
        self.set_stream(stream);
        self
    }

    #[inline]
    pub fn set_stream<
        T: Into<String>,
        E: std::error::Error,
    >(&mut self, stream: impl ohkami_lib::Stream<Item = Result<T, E>> + Unpin + Send + 'static) {
        let stream = Box::pin(stream.map(|res|
            res
            .map(Into::into)
            .map_err(|e| e.to_string())
        ));

        self.headers.set()
            .ContentType("text/event-stream")
            .CacheControl("no-cache, must-revalidate")
            .TransferEncoding("chunked");
        self.content = Content::Stream(stream);
    }
}

#[cfg(feature="ws")]
impl Response {
    #[cfg(feature="__rt_native__")]
    pub(crate) fn with_websocket(mut self, ws: mews::WebSocket) -> Self {
        self.content = Content::WebSocket(ws);
        self
    }
    #[cfg(feature="rt_worker")]
    pub(crate) fn with_websocket(mut self, ws: worker::WebSocket) -> Self {
        self.content = Content::WebSocket(ws);
        self
    }
}

#[cfg(feature="__rt_native__")]
pub(super) enum Upgrade {
    None,

    #[cfg(feature="ws")]
    WebSocket(mews::WebSocket),
}
#[cfg(feature="__rt_native__")]
impl Upgrade {
    #[inline(always)]
    pub(super) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
#[cfg(feature="__rt_native__")]
impl Response {
    #[cfg_attr(not(feature="sse"), inline)]
    pub(crate) async fn send(mut self,
        conn: &mut (impl AsyncWrite + Unpin)
    ) -> Upgrade {
        self.complete();

        match self.content {
            Content::None => {
                let mut buf = Vec::<u8>::with_capacity(
                    self.status.line().len() +
                    self.headers.size
                ); unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await.expect("Failed to send response");
                conn.flush().await.expect("Failed to flush connection");

                Upgrade::None
            }

            Content::Payload(bytes) => {
                let mut buf = Vec::<u8>::with_capacity(
                    self.status.line().len() +
                    self.headers.size +
                    bytes.len()
                ); unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                    crate::push_unchecked!(buf <- bytes);
                }
                conn.write_all(&buf).await.expect("Failed to send response");
                conn.flush().await.expect("Failed to flush connection");

                Upgrade::None
            }

            #[cfg(feature="sse")]
            Content::Stream(mut stream) => {
                let mut buf = Vec::<u8>::with_capacity(
                    self.status.line().len() +
                    self.headers.size
                ); unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await.expect("Failed to send response");
                conn.flush().await.expect("Failed to flush connection");

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Err(msg)  => {
                            crate::warning!("Error in stream: {msg}");
                            break
                        }
                        Ok(chunk) => {
                            let mut message = Vec::with_capacity(
                                /* capacity for a single line */
                                "data: ".len() + chunk.len() + "\n\n".len()
                            );
                            for line in chunk.split('\n') {
                                message.extend_from_slice(b"data: ");
                                message.extend_from_slice(line.as_bytes());
                                message.push(b'\n');
                            }
                            message.push(b'\n');

                            let size_hex_bytes = ohkami_lib::num::hexized_bytes(message.len());

                            let mut chunk = Vec::from(&size_hex_bytes[size_hex_bytes.iter().position(|b| *b!=b'0').unwrap()..]);
                            chunk.extend_from_slice(b"\r\n");
                            chunk.append(&mut message);
                            chunk.extend_from_slice(b"\r\n");

                            #[cfg(feature="DEBUG")]
                            println!("\n[sending chunk]\n{}", chunk.escape_ascii());

                            conn.write_all(&chunk).await.expect("Failed to send response");
                            conn.flush().await.expect("Failed to flush connection");
                        }
                    }
                }
                conn.write_all(b"0\r\n\r\n").await.expect("Failed to send response");
                conn.flush().await.expect("Failed to flush connection");

                Upgrade::None
            }

            #[cfg(all(feature="ws", feature="__rt_native__"))]
            Content::WebSocket(ws) => {
                let mut buf = Vec::<u8>::with_capacity(
                    self.status.line().len() +
                    self.headers.size
                ); unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await.expect("Failed to send response");
                conn.flush().await.expect("Failed to flush connection");

                Upgrade::WebSocket(ws)
            }
        }
    }
}

const _: () = {
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Response")
                .field("status",  &self.status)
                .field("headers", &self.headers)
                .field("content", &self.content)
                .finish()
        }
    }

    impl PartialEq for Response {
        fn eq(&self, other: &Self) -> bool {
            self.status  == other.status  &&
            self.headers == other.headers &&
            self.content == other.content
        }
    }
};

#[cfg(feature="nightly")]
const _: () = {
    use std::{ops::FromResidual, convert::Infallible};

    impl FromResidual<Result<Infallible, Response>> for Response {
        fn from_residual(residual: Result<Infallible, Response>) -> Self {
            unsafe {residual.unwrap_err_unchecked()}
        }
    }

    #[cfg(test)]
    fn try_response() {
        use crate::Request;

        fn payload_serde_json_value(req: &Request) -> Result<::serde_json::Value, Response> {
            let payload = req.payload.as_deref()
                .ok_or_else(Response::BadRequest)?;
            let value = serde_json::from_slice::<serde_json::Value>(payload)
                .map_err(|_| Response::BadRequest())?;
            Ok(value)
        }
    }
};

#[cfg(feature="rt_worker")]
const _: () = {
    impl Into<::worker::Response> for Response {
        #[inline(always)]
        fn into(self) -> ::worker::Response {
            self.content.into_worker_response()
                .with_status(self.status.code())
                .with_headers(self.headers.into())
        }
    }
};
