mod _test;

use std::borrow::Cow;
use std::collections::HashMap;
use std::{pin::Pin, future::Future, format as f};
use byte_reader::Reader;

use crate::{Response, Request, Ohkami, Context};
use crate::layer0_lib::{IntoCows, Status, Method, ContentType};

#[cfg(feature="websocket")]
use {std::sync::Arc, crate::__rt__::Mutex};


pub trait Testing {
    fn oneshot(&self, req: TestRequest) -> TestFuture;
}
pub struct TestFuture(
    Box<dyn Future<Output = TestResponse>>);
impl Future for TestFuture {
    type Output = TestResponse;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {self.map_unchecked_mut(|this| this.0.as_mut())}.poll(cx)
    }
}

#[cfg(feature="websocket")]
pub struct TestWebSocket(Vec<u8>);
#[cfg(feature="websocket")] const _: () = {
    impl TestWebSocket {
        fn new(size: usize) -> Self {
            Self(Vec::with_capacity(size))
        }
    }

    #[cfg(feature="rt_tokio")] const _: () = {
        impl tokio::io::AsyncRead for TestWebSocket {
            fn poll_read(
                self: Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
                buf: &mut tokio::io::ReadBuf<'_>,
            ) -> std::task::Poll<std::io::Result<()>> {
                let mut pin_inner = unsafe {self.map_unchecked_mut(|this| &mut this.0)};

                let amt = std::cmp::min(pin_inner.len(), buf.remaining());
                let (a, b) = pin_inner.split_at(amt);
                buf.put_slice(a);
                *pin_inner = b.to_vec();

                std::task::Poll::Ready(Ok(()))
            }
        }
        impl tokio::io::AsyncWrite for TestWebSocket {
            fn poll_write(
                self: Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
                buf: &[u8],
            ) -> std::task::Poll<Result<usize, std::io::Error>> {
                unsafe {self.map_unchecked_mut(|this| &mut this.0)}
                    .poll_write(cx, buf)
            }
            fn poll_flush(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
                unsafe {self.map_unchecked_mut(|this| &mut this.0)}
                    .poll_flush(cx)
            }
            fn poll_shutdown(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
                unsafe {self.map_unchecked_mut(|this| &mut this.0)}
                    .poll_shutdown(cx)
            }
        }
    };
};

impl Testing for Ohkami {
    fn oneshot(&self, request: TestRequest) -> TestFuture {
        let router = {
            let mut router = self.routes.clone();
            for (methods, fang) in &self.fangs {
                router = router.apply_fang(methods, fang.clone())
            }
            router.into_radix()
        };

        let test_res = async move {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            req.as_mut().read(&mut &request.encode_request()[..]).await;

            let (res, _) = router.handle(Context::new(), &mut req).await;
            TestResponse::new(res)
        };

        TestFuture(Box::new(test_res))
    }
}


pub struct TestRequest {
    method:  Method,
    path:    Cow<'static, str>,
    queries: HashMap<Cow<'static, str>, Cow<'static, str>>,
    headers: HashMap<Cow<'static, str>, Cow<'static, str>>,
    content: Option<Cow<'static, str>>,
} impl TestRequest {
    fn encode_request(self) -> Vec<u8> {
        let Self { method, path, queries, headers, content } = self;

        let queries = queries.into_iter()
            .map(|(k, v)| f!("{k}={v}"))
            .fold(Vec::new(), |mut q, kv| if q.is_empty() {
                q.push(b'?'); q.extend_from_slice(kv.as_bytes()); q
            } else {
                q.push(b'&'); q.extend_from_slice(kv.as_bytes()); q
            });

        let headers = headers.into_iter()
            .map(|(k, v)| f!("{k}: {v}\r\n"))
            .fold(Vec::new(), |mut h, kv| {
                h.extend_from_slice(kv.as_bytes()); h
            });

        [
            method.as_bytes(), b" ", path.as_bytes(), &queries, b" HTTP/1.1\r\n",
            &headers,
            b"\r\n",
            content.unwrap_or(Cow::Borrowed("")).as_bytes()
        ].concat()
    }
} macro_rules! new_test_request {
    ( $($method:ident)* ) => {$(
        #[allow(non_snake_case)]
        impl TestRequest {
            pub fn $method(path: impl IntoCows<'static>) -> Self {
                Self {
                    method:  Method::$method,
                    path:    path.into_cow(),
                    queries: HashMap::new(),
                    headers: HashMap::new(),
                    content: None,
                }
            }
        }
    )*};
} new_test_request! {
    GET PUT POST PATCH DELETE HEAD OPTIONS
} impl TestRequest {
    pub fn query(mut self, key: impl IntoCows<'static>, value: impl IntoCows<'static>) -> Self {
        self.queries.insert(key.into_cow(), value.into_cow());
        self
    }
    pub fn header(mut self, key: impl IntoCows<'static>, value: impl IntoCows<'static>) -> Self {
        self.headers.insert(key.into_cow(), value.into_cow());
        self
    }
}
impl TestRequest {
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        let content       = serde_json::to_string(&json).expect("Failed to serialize json");
        let content_lenth = content.len();

        self.content = Some(Cow::Owned(content));
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
    pub fn json_lit(mut self, json: impl IntoCows<'static>) -> Self {
        let content = json.into_cow();
        let content_lenth = content.len();

        self.content = Some(content);
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
}


pub struct TestResponse {
    pub status:  Status,
    pub headers: ResponseHeaders,
    pub content: Option<ResponseBody>,
} impl TestResponse {
    fn new(response: Response) -> Self {
        let Response { status, headers, content } = response;
        Self {
            status,
            headers: ResponseHeaders::new(headers),
            content: content.map(|(content_type, payload )| ResponseBody { content_type, payload }),
        }
    }
}

pub struct ResponseHeaders(
    std::sync::RwLock<LazyMap>
); enum LazyMap {
    Raw(String),
    Map(HashMap</*lower case*/String, String>),
} impl LazyMap {
    fn eval(&mut self) {
        match self {
            Self::Map(_) => (),
            Self::Raw(string) => {
                let map = {
                    let mut map = HashMap::new();
                    let mut r   = Reader::new(string);

                    while r.peek().is_some() {
                        let key   = r.read_kebab().unwrap();
                        r.consume(": ").unwrap();
                        let value = String::from_utf8(r.read_while(|b| b != &b'\r').to_vec()).unwrap();
                        r.consume("\r\n").unwrap();

                        map.insert(key.to_ascii_lowercase(), value);
                    }

                    map
                };
                *self = Self::Map(map)
            }
        }
    }
} impl ResponseHeaders {
    fn new(raw_headers: String) -> Self {
        Self(std::sync::RwLock::new(
            LazyMap::Raw(raw_headers)
        ))
    }
} impl ResponseHeaders {
    pub fn get(&self, key: &str) -> Option<String> {
        let current = self.0.read().ok()?;
        if let LazyMap::Map(map) = &*current {
            return map.get(&key.to_ascii_lowercase()).map(|s| s.to_string())
        } else {drop(current)}

        let inner = &mut *self.0.write().ok()?;
        inner.eval();
        let LazyMap::Map(map) = inner else {unsafe {std::hint::unreachable_unchecked()}};
        map.get(&key.to_ascii_lowercase()).map(|s| s.to_string())
    }
}

pub struct ResponseBody {
    content_type: ContentType,
    payload:      Cow<'static, str>,
} impl ResponseBody {
    pub fn text(&self) -> Option<&str> {
        matches!(&self.content_type, ContentType::Text)
            .then_some(&self.payload)
    }
    pub fn html(&self) -> Option<&str> {
        matches!(&self.content_type, ContentType::HTML)
            .then_some(&self.payload)
    }
    pub fn json(&self) -> Option<&str> {
        matches!(&self.content_type, ContentType::JSON)
            .then_some(&self.payload)
    }
}
