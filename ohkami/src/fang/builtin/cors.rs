#![allow(non_snake_case)]

use crate::{Fang, FangProc, Context, Request, Response, Status, Method};
use crate::header::{
    Vary,
    ContentType,
    ContentLength,
    AccessControlAllowCredentials,
    AccessControlAllowHeaders,
    AccessControlAllowMethods,
    AccessControlAllowOrigin,
    AccessControlExposeHeaders,
    AccessControlMaxAge,
    AccessControlRequestHeaders,
};


/// # Builtin fang for CORS config
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::CORS;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with((
///         CORS::new("https://foo.bar.org")
///             .AllowHeaders(["Content-Type", "X-Requested-With"])
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
    pub(crate) AllowMethods:     Option<String>,
    pub(crate) AllowHeaders:     Option<String>,
    pub(crate) ExposeHeaders:    Option<String>,
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
    /// Create `CORS` fang using given `AllowOrigin` as `Access-Control-Allow-Origin` header value.\
    /// (Both `"*"` and a speciffic origin are available)
    #[allow(non_snake_case)]
    pub fn new(AllowOrigin: &'static str) -> Self {
        Self {
            AllowOrigin:      AccessControlAllowOrigin::from_literal(AllowOrigin),
            AllowCredentials: false,
            AllowMethods:     None,
            AllowHeaders:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }

    /* Always use default for now...
    /// Override `Access-Control-Allow-Methods` header value, it's default to
    /// all available methods on the request path.
    pub fn AllowMethods<const N: usize>(mut self, methods: [Method; N]) -> Self {
        self.AllowMethods = Some(methods.map(|m| m.as_str()).join(", "));
        self
    }
    */

    pub fn AllowCredentials(mut self) -> Self {
        if self.AllowOrigin.is_any() {
            #[cfg(debug_assertions)] crate::warning!("\
                [WRANING] \
                'Access-Control-Allow-Origin' header \
                must not have wildcard '*' when the request's credentials mode is 'include' \
            ");
            return self
        }
        self.AllowCredentials = true;
        self
    }
    pub fn AllowHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.AllowHeaders = Some(headers.join(", "));
        self
    }
    pub fn ExposeHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.ExposeHeaders = Some(headers.join(", "));
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
    async fn bite<'b>(&'b self, ctx: Context<'b>, req: &'b mut Request) -> Response {
        let mut res = self.inner.bite(ctx, req).await;

        res.set(AccessControlAllowOrigin, self.cors.AllowOrigin.as_str());
        if self.cors.AllowOrigin.is_any() {
            res.set(Vary, "Origin");
        }
        if self.cors.AllowCredentials {
            res.set(AccessControlAllowCredentials, "true");
        }
        if let Some(expose_headers) = &self.cors.ExposeHeaders {
            res.set(AccessControlExposeHeaders, expose_headers.to_string());
        }

        if req.method() == Method::OPTIONS {
            if let Some(max_age) = self.cors.MaxAge {
                res.set(AccessControlMaxAge, max_age.to_string());
            }
            if let Some(allow_methods) = &self.cors.AllowMethods {
                res.set(AccessControlAllowMethods, allow_methods.to_string());
            }
            if let Some(allow_headers) = self.cors.AllowHeaders.as_deref()
                .or_else(|| req.header(AccessControlRequestHeaders))
            {
                res
                    .set(AccessControlAllowHeaders, allow_headers.to_string())
                    .append(Vary, "Access-Control-Request-Headers");
            }

            /* override default `Not Implemented` response for valid preflight */
            if res.status() == Status::NotImplemented {
                res
                    .set_status(Status::OK)
                    .set(ContentType, None)
                    .set(ContentLength, None);
            }
        }

        #[cfg(feature="DEBUG")]
        println!("After CORS proc: res = {res:#?}");

        res
    }
}




#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(feature="testing")]
#[cfg(test)]
mod test {
    use super::CORS;
    use crate::prelude::*;
    use crate::testing::*;
    use crate::header::{
        Vary,
        AccessControlAllowCredentials,
        AccessControlAllowHeaders,
        AccessControlAllowMethods,
        AccessControlAllowOrigin,
        AccessControlExposeHeaders,
        AccessControlMaxAge,
        AccessControlRequestMethod,
    };

    #[crate::__rt__::test] async fn options_request() {
        let t = Ohkami::with((),
            "/hello".POST(|| async {"Hello!"})
        ).test(); {
            let req = Request::OPTIONS("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = Request::OPTIONS("/hello");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
        }

        let t = Ohkami::with(CORS::new("https://example.x.y.z"),
            "/hello".POST(|| async {"Hello!"})
        ).test(); {
            let req = Request::OPTIONS("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = Request::OPTIONS("/hello");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
        } {
            let req = Request::OPTIONS("/hello")
                .with(AccessControlRequestMethod, "DELETE");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::BadRequest/* Because `DELETE` is not available */);
            assert_eq!(res.text(), None);
        } {
            let req = Request::OPTIONS("/hello")
                .with(AccessControlRequestMethod, "POST");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK/* Becasue `POST` is available */);
            assert_eq!(res.text(), None);
        }
    }

    #[crate::__rt__::test] async fn cors_headers() {
        let t = Ohkami::with(CORS::new("https://example.example"),
            "/".GET(|| async {"Hello!"})
        ).test(); {
            let req = Request::GET("/");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200);
            assert_eq!(res.text(), Some("Hello!"));

            assert_eq!(res.header(AccessControlAllowOrigin), Some("https://example.example"));
            assert_eq!(res.header(AccessControlAllowCredentials), None);
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), None);
            assert_eq!(res.header(AccessControlAllowMethods), None);
            assert_eq!(res.header(AccessControlAllowHeaders), None);
            assert_eq!(res.header(Vary), None);
        }

        let t = Ohkami::with(
            CORS::new("https://example.example")
                .AllowCredentials()
                .AllowHeaders(["Content-Type", "X-Custom"]),
            "/abc"
                .GET(|| async {"Hello!"})
                .PUT(|| async {"Hello!"})
        ).test(); {
            let req = Request::OPTIONS("/abc");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 404/* Because `req` has no `Access-Control-Request-Method` */);
            assert_eq!(res.text(), None);

            assert_eq!(res.header(AccessControlAllowOrigin), Some("https://example.example"));
            assert_eq!(res.header(AccessControlAllowCredentials), Some("true"));
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), None);
            assert_eq!(res.header(AccessControlAllowMethods), None/* Because `req` has no `Access-Control-Request-Method` */);
            assert_eq!(res.header(AccessControlAllowHeaders), Some("Content-Type, X-Custom"));
            assert_eq!(res.header(Vary), Some("Access-Control-Request-Headers"));
        } {
            let req = Request::OPTIONS("/abc")
                .with(AccessControlRequestMethod, "PUT");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200/* Because `req` HAS available `Access-Control-Request-Method` */);
            assert_eq!(res.text(), None);

            assert_eq!(res.header(AccessControlAllowOrigin), Some("https://example.example"));
            assert_eq!(res.header(AccessControlAllowCredentials), Some("true"));
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), None);
            assert_eq!(res.header(AccessControlAllowMethods), Some("GET, PUT, HEAD, OPTIONS")/* Because `req` HAS a `Access-Control-Request-Method` */);
            assert_eq!(res.header(AccessControlAllowHeaders), Some("Content-Type, X-Custom"));
            assert_eq!(res.header(Vary), Some("Access-Control-Request-Headers"));
        } {
            let req = Request::OPTIONS("/abc")
                .with(AccessControlRequestMethod, "DELETE");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 400/* Because `DELETE` is not available */);
            assert_eq!(res.text(), None);

            assert_eq!(res.header(AccessControlAllowOrigin), Some("https://example.example"));
            assert_eq!(res.header(AccessControlAllowCredentials), Some("true"));
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), None);
            assert_eq!(res.header(AccessControlAllowMethods), Some("GET, PUT, HEAD, OPTIONS")/* Because `req` HAS a `Access-Control-Request-Method` */);
            assert_eq!(res.header(AccessControlAllowHeaders), Some("Content-Type, X-Custom"));
            assert_eq!(res.header(Vary), Some("Access-Control-Request-Headers"));
        } {
            let req = Request::PUT("/abc");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200);
            assert_eq!(res.text(), Some("Hello!"));

            assert_eq!(res.header(AccessControlAllowOrigin), Some("https://example.example"));
            assert_eq!(res.header(AccessControlAllowCredentials), Some("true"));
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), None);
            assert_eq!(res.header(AccessControlAllowMethods), None);
            assert_eq!(res.header(AccessControlAllowHeaders), None);
            assert_eq!(res.header(Vary), None);
        }

        let t = Ohkami::with(
            CORS::new("*")
                .AllowHeaders(["Content-Type", "X-Custom"])
                .MaxAge(1024),
            "/".POST(|| async {"Hello!"})
        ).test(); {
            let req = Request::OPTIONS("/");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 404/* Because `req` has no `Access-Control-Request-Method` */);
            assert_eq!(res.text(), None);

            assert_eq!(res.header(AccessControlAllowOrigin), Some("*"));
            assert_eq!(res.header(AccessControlAllowCredentials), None);
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), Some("1024"));
            assert_eq!(res.header(AccessControlAllowMethods), None/* Because `req` has no `Access-Control-Request-Method` */);
            assert_eq!(res.header(AccessControlAllowHeaders), Some("Content-Type, X-Custom"));
            assert_eq!(res.header(Vary), Some("Origin, Access-Control-Request-Headers"));
        } {
            let req = Request::OPTIONS("/")
                .with(AccessControlRequestMethod, "POST");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200);
            assert_eq!(res.text(), None);

            assert_eq!(res.header(AccessControlAllowOrigin), Some("*"));
            assert_eq!(res.header(AccessControlAllowCredentials), None);
            assert_eq!(res.header(AccessControlExposeHeaders), None);
            assert_eq!(res.header(AccessControlMaxAge), Some("1024"));
            assert_eq!(res.header(AccessControlAllowMethods), Some("POST, OPTIONS"));
            assert_eq!(res.header(AccessControlAllowHeaders), Some("Content-Type, X-Custom"));
            assert_eq!(res.header(Vary), Some("Origin, Access-Control-Request-Headers"));
        }
    }
}
