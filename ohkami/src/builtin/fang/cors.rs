#![allow(non_snake_case)]

use crate::{header::append, Fang, FangProc, IntoResponse, Method, Request, Response, Status};


/// # Builtin fang for CORS config
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::builtin::fang::CORS;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with((
///         CORS::new("https://foo.bar.org")
///             .AllowMethods(&[Method::GET, Method::POST])
///             .AllowHeaders(&["Content-Type", "X-Requested-With"])
///             .AllowCredentials()
///             .MaxAge(86400),
///     ), (
///         "/api".GET(|| async {
///             "Hello, CORS!"
///         }),
///     )).howl("localhost:8080").await
/// }
/// ```
#[derive(Clone)]
pub struct CORS {
    pub(crate) AllowOrigin:      AccessControlAllowOrigin,
    pub(crate) AllowCredentials: bool,
    pub(crate) AllowMethods:     Option<&'static [Method]>,
    pub(crate) AllowHeaders:     Option<&'static [&'static str]>,
    pub(crate) ExposeHeaders:    Option<&'static [&'static str]>,
    pub(crate) MaxAge:           Option<u32>,
}

#[derive(Clone)]
pub(crate) enum AccessControlAllowOrigin {
    Any,
    Only(&'static str),
} impl AccessControlAllowOrigin {
    #[inline(always)] pub(crate) const fn is_any(&self) -> bool {
        match self {
            Self::Any => true,
            _ => false,
        }
    }

    #[inline(always)] pub(crate) const fn from_literal(lit: &'static str) -> Self {
        match lit.as_bytes() {
            b"*"   => Self::Any,
            origin => Self::Only(unsafe{std::str::from_utf8_unchecked(origin)}),
        }
    }

    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Any          => "*",
            Self::Only(origin) => origin,
        }
    }
}

impl CORS {
    /// Create `CORS` fang for specified `AllowOrigin` as `Access-Control-Allow-Origin` header.\
    /// (Both `"*"` and a speciffic origin are available)
    #[allow(non_snake_case)]
    pub const fn new(AllowOrigin: &'static str) -> Self {
        use Method::*;

        Self {
            AllowOrigin:      AccessControlAllowOrigin::from_literal(AllowOrigin),
            AllowCredentials: false,
            AllowMethods:     Some(&[GET, HEAD, PUT, POST, DELETE, PATCH]),
            AllowHeaders:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }

    pub const fn AllowCredentials(mut self) -> Self {
        if self.AllowOrigin.is_any() {
            panic!("\
                The value of the 'Access-Control-Allow-Origin' header in the response \
                must not be the wildcard '*' when the request's credentials mode is 'include'.\
            ")
        }
        self.AllowCredentials = true;
        self
    }
    pub fn AllowMethods(mut self, methods: &'static [Method]) -> Self {
        if methods.len() > 0 {
            self.AllowMethods = Some(methods);
        }
        self
    }
    pub fn AllowHeaders(mut self, headers: &'static [&'static str]) -> Self {
        if headers.len() > 0 {
            self.AllowHeaders = Some(headers);
        }
        self
    }
    pub fn ExposeHeaders(mut self, headers: &'static [&'static str]) -> Self {
        if headers.len() > 0 {
            self.ExposeHeaders = Some(headers);
        }
        self
    }
    pub fn MaxAge(mut self, delta_seconds: u32) -> Self {
        self.MaxAge = Some(delta_seconds);
        self
    }
}

impl<Inner: FangProc> Fang<Inner> for CORS {
    type Proc = CORSProc<Inner>;
    fn chain(&self, inner: Inner) -> Self::Proc {
        CORSProc { inner, cors: self.clone() }
    }
}

pub struct CORSProc<Inner: FangProc> {
    cors:  CORS,
    inner: Inner,
}
/* Based on https://github.com/honojs/hono/blob/main/src/middleware/cors/index.ts; MIT */
impl<Inner: FangProc> FangProc for CORSProc<Inner> {
    async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
        let mut res = self.inner.bite(req).await.into_response();

        let mut h = res.headers.set();

        h = h.AccessControlAllowOrigin(self.cors.AllowOrigin.as_str());
        if self.cors.AllowOrigin.is_any() {
            h = h.Vary("Origin");
        }
        if self.cors.AllowCredentials {
            h = h.AccessControlAllowCredentials("true");
        }
        if let Some(expose_headers) = &self.cors.ExposeHeaders {
            h = h.AccessControlExposeHeaders(expose_headers.join(","));
        }

        if req.method.isOPTIONS() {
            if let Some(max_age) = self.cors.MaxAge {
                h = h.AccessControlMaxAge(max_age.to_string());
            }
            if let Some(allow_methods) = self.cors.AllowMethods {
                let methods_string = allow_methods.iter()
                    .map(Method::as_str)
                    .fold(String::new(), |mut ms, m| {ms.push_str(m); ms});
                h = h.AccessControlAllowMethods(methods_string);
            }
            if let Some(allow_headers_string) = match self.cors.AllowHeaders {
                Some(hs) => Some(hs.join(",")),
                None     => req.headers.AccessControlRequestHeaders().map(String::from),
            } {
                h = h.AccessControlAllowHeaders(allow_headers_string)
                    .Vary(append("Access-Control-Request-Headers"));
            }

            h.ContentType(None).ContentLength(None);
            res.status = Status::NoContent;
        }

        res
    }
}
