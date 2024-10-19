#![cfg(feature="__rt_native__")]

use std::time::Duration;


/// # Builtin fang for timeout config
/// 
/// <br>
/// 
/// Set timeout of request handling when a request was handled by that `Ohkami`.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::Timeout;
/// use std::time::Duration;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(Timeout::by_secs(10), (
///         "/hello/:sleep".GET(sleeping_hello),
///     )).howl("0.0.0.0:3000").await
/// }
/// 
/// async fn sleeping_hello(sleep: u64) -> &'static str {
///     tokio::time::sleep(Duration::from_secs(sleep)).await;
///     
///     "Hello, I was sleeping ):"
/// }
/// ```
/// ---
#[derive(Clone, Copy)]
pub struct Timeout(Duration);
impl Timeout {
    pub fn by(duration: Duration) -> Self {
        Self(duration)
    }
    pub const fn by_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }
    pub const fn by_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }
    pub fn by_secs_f32(secs: f32) -> Self {
        Self(Duration::from_secs_f32(secs))
    }
    pub fn by_secs_f64(secs: f64) -> Self {
        Self(Duration::from_secs_f64(secs))
    }
}

const _: () = {
    use crate::{Fang, FangProc, Request, Response};

    impl<Inner: FangProc> Fang<Inner> for Timeout {
        type Proc = TimeoutProc<Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            TimeoutProc { time: self.0, inner }
        }
    }

    pub struct TimeoutProc<Inner: FangProc> {
        inner: Inner,
        time:  Duration,
    }
    impl<Inner: FangProc> FangProc for TimeoutProc<Inner> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            crate::util::timeout_in(self.time, self.inner.bite(req)).await
                .unwrap_or_else(|| Response::InternalServerError().with_text("timeout"))
        }
    }
};


#[cfg(all(test, feature="testing", any(feature="rt_tokio",feature="rt_async-std")))]
#[crate::__rt__::test] async fn test_timeout() {
    use crate::prelude::*;
    use crate::testing::*;

    async fn lazy_greeting(
        (name, sleep): (&str, u64)
    ) -> String {
        crate::__rt__::sleep(Duration::from_secs(sleep)).await;

        format!("Hello, {name}!")
    }

    let t = Ohkami::with((
        Timeout::by_secs(2),
    ), (
        "/greet/:name/:sleep".GET(lazy_greeting),
    )).test();


    {
        let req = Request::GET("/greet");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    }
    {
        let req = Request::PUT("/greet/ohkami/1");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    }
    {
        let req = Request::GET("/greet/ohkami/1");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(),   Some("Hello, ohkami!"));
    }
    {
        let req = Request::GET("/greet/ohkami/3");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::InternalServerError);
        assert_eq!(res.text(),   Some("timeout"));
    }
}
