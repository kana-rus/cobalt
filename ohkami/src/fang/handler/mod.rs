mod from_request;
pub use from_request::{FromRequest, FromParam};

mod into_response;
pub use into_response::IntoResponse;

mod into_handler;
pub(crate) use into_handler::IntoHandler;

use super::{FangProcCaller, BoxedFPC};
use super::{SendOnNative, SendSyncOnNative, ResponseFuture};
use whttp::{Request, Response};
use std::{pin::Pin, future::Future};


#[derive(Clone)]
pub struct Handler(BoxedFPC);

const _: () = {
    impl Into<BoxedFPC> for Handler {
        fn into(self) -> BoxedFPC {
            self.0
        }
    }

    impl std::fmt::Debug for Handler {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("{handler}")
        }
    }
};

impl Handler {
    pub(crate) fn new(
        proc: impl Fn(&mut Request) -> Pin<Box<dyn ResponseFuture + '_>> + SendSyncOnNative + 'static
    ) -> Self {
        struct HandlerProc<F>(F);

        const _: () = {
            impl<F> FangProcCaller for HandlerProc<F>
            where
                F: Fn(&mut Request) -> Pin<Box<dyn ResponseFuture + '_>> + SendSyncOnNative + 'static
            {
                #[cfg(not(feature="rt_worker"))]
                fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
                    // SAFETY: trait upcasting
                    // trait upcasting coercion is experimental <https://github.com/rust-lang/rust/issues/65991>
                    unsafe {std::mem::transmute((self.0)(req))}
                }
                #[cfg(feature="rt_worker")]
                fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>> {
                    // SAFETY: trait upcasting
                    // trait upcasting coercion is experimental <https://github.com/rust-lang/rust/issues/65991>
                    unsafe {std::mem::transmute((self.0)(req))}
                }
            }
        };

        Self(BoxedFPC::from_proc(HandlerProc(proc)))
    }
}

#[cfg(feature="rt_worker")]
const _: () = {
    unsafe impl Send for Handler {}
    unsafe impl Sync for Handler {}
};

impl Handler {
    pub(crate) fn default_not_found() -> Self {
        use std::sync::LazyLock;

        static NOT_FOUND: LazyLock<Handler> = LazyLock::new(|| {
            async fn not_found() -> Response {
                Response::NotFound()
            }
            not_found.into_handler()
        });

        Handler((&*NOT_FOUND).0.clone())
    }
}
