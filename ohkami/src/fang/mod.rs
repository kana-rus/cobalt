#[cfg(feature="__rt__")]
mod handler;
#[cfg(feature="__rt__")]
pub(crate) use handler::{Handler, IntoHandler};
pub use handler::{FromParam, FromRequest, IntoResponse};

mod middleware;
pub use middleware::{Fangs, util::FangAction};

mod builtin;
pub use builtin::*;

mod dispatch;
pub(self) use dispatch::*;

use whttp::{Request, Response};
use std::{future::Future, pin::Pin, ops::Deref};


/// # Core trait for Ohkami's Fang system
/// 
/// See `FangAction` for simple cases!
/// 
/// <br>
/// 
/// ### required
/// - `type Proc` ー associated type implementing `FangProc`, the behavior as a fang
/// - `fn chain` ー how to associate with `inner: Inner: FangProc`
/// 
/// <br>
/// 
/// *impl_example.rs*
/// ```
/// use ohkami::prelude::*;
/// use ohkami::{Fang, FangProc};
/// 
/// struct HelloFang;
/// const _: () = {
///     struct HelloFangProc<I: FangProc> {
///         inner: I
///     }
///     impl<I: FangProc> FangProc for HelloFangProc<I> {
///         async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
///             println!("Hello, fang!");
///             self.inner.bite(req).await
///         }
///     }
/// 
///     impl<I: FangProc> Fang<I> for HelloFang {
///         type Proc = HelloFangProc<I>;
///         fn chain(&self, inner: I) -> Self::Proc {
///             HelloFangProc { inner }
///         }
///     }
/// };
/// 
/// ```
pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc: SendSyncOnNative + 'static {
    #[cfg(not(feature="rt_worker"))]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send;
    #[cfg(feature="rt_worker")]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response>;

    #[cfg(not(feature="rt_worker"))]
    /// Default: just `Box::pin(self.bite(req))`.
    /// 
    /// Mainly used for override `bite` when itself returns `Pin<Box<dyn Future>>`.
    #[inline(always)]
    fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
        Box::pin(self.bite(req))
    }
    #[cfg(feature="rt_worker")]
    /// Default: just `Box::pin(self.bite(req))`.
    /// 
    /// Mainly used for override `bite` when itself returns `Pin<Box<dyn Future>>`.
    #[inline(always)]
    fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>> {
        Box::pin(self.bite(req))
    }
}

/// `FangProc` but object-safe, returning `Pin<Box<dyn Future>>`.
pub(crate) trait FangProcCaller {
    #[cfg(not(feature="rt_worker"))]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>>;
    #[cfg(feature="rt_worker")]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>>;
}
impl<Proc: FangProc> FangProcCaller for Proc {
    #[cfg(not(feature="rt_worker"))]
    #[inline(always)]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
        self.bite_boxed(req)
    }
    #[cfg(feature="rt_worker")]
    #[inline(always)]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>> {
        self.bite_boxed(req)
    }
}

#[derive(Clone/* copy pointer */)]
pub(crate) struct BoxedFPC(&'static (dyn
    FPCBound + 'static
));
const _: () = {
    impl BoxedFPC {
        pub(crate) fn from_proc(proc: impl FPCBound + 'static) -> Self {
            Self(Box::leak(Box::new(proc)))
        }
    }

    impl Deref for BoxedFPC {
        type Target = dyn FPCBound + 'static;
        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    impl std::fmt::Debug for BoxedFPC {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("{Fang proc}")
        }
    }

    impl FangProc for BoxedFPC {
        #[cfg(not(feature="rt_worker"))]
        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> + Send {
            self.0.call_bite(req)
        }
        #[cfg(feature="rt_worker")]
        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> {
            self.0.call_bite(req)
        }

        #[cfg(not(feature="rt_worker"))]
        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            self.0.call_bite(req)
        }
        #[cfg(feature="rt_worker")]
        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>> {
            self.0.call_bite(req)
        }
    }
};
