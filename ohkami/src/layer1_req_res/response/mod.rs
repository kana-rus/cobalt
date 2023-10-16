pub(crate) mod headers; pub(crate) use headers::ResponseHeaders;


use std::borrow::Cow;
use crate::{
    __rt__, __rt__::AsyncWriter,
    layer0_lib::{Status, ContentType, IntoCows},
};


/// # HTTP Response
/// 
/// Generated from `Context`. Handlers have to returns this.
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// async fn hello(c: Context) -> Response {
///     c
///         .OK()           // generate Response
///         .text("Hello!") // set content (text/plain)
/// }
/// ```
pub struct Response {
    pub status:         Status,
    pub(crate) headers: String,
    pub(crate) content: Option<(ContentType, Cow<'static, str>)>,
}

impl Response {
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        let Self { status, headers, content } = self;
        let (status, headers) = (status.as_bytes(), headers.as_bytes());

        match content {
            None => [
                b"HTTP/1.1 ",status,b"\r\n",
                headers,
                b"\r\n"
            ].concat(),

            Some((content_type, body)) => [   
                b"HTTP/1.1 ",status,b"\r\n",
                b"Content-Type: "  ,content_type.as_bytes(),          b"\r\n",
                b"Content-Length: ",body.len().to_string().as_bytes(),b"\r\n",
                headers,
                b"\r\n",
                body.as_bytes()
            ].concat(),
        }
    }
}

impl Response {
    pub(crate) async fn send(self, stream: &mut __rt__::TcpStream) {
        if let Err(e) = stream.write_all(&self.into_bytes()).await {
            panic!("Failed to send response: {e}")
        }
    }
}

impl Response {
    pub fn drop_content(mut self) -> Self {
        self.content.take();
        self
    }

    pub fn text(mut self, text: impl IntoCows<'static>) -> Self {
        self.content.replace((
            ContentType::Text,
            text.into_cow()
        ));
        self
    }
    pub fn html(mut self, html: impl IntoCows<'static>) -> Self {
        self.content.replace((
            ContentType::HTML,
            html.into_cow()
        ));
        self
    }
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        self.content.replace((
            ContentType::JSON,
            Cow::Owned(serde_json::to_string(&json).expect("Failed to serialize json"))
        ));
        self
    }
}

const _: () = {
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.content {
                None => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .finish(),
                Some((_, cow)) => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .field("content", &*cow)
                    .finish(),
            }
        }
    }
};
