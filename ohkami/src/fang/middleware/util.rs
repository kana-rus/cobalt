use super::super::{Fang, FangProc};
use whttp::{Request, Response};


/// # Fang action - utility wrapper of `Fang`
/// 
/// `FangAction` provides 2 actions:
/// 
/// - `fore` ... *bite* a `&mut Request`, maybe early returning `Err(Response)`, before a handler is called
/// - `back` ... *bite* a `&mut Response` after a handler is called
/// 
/// Both of them perform nothing by default.
/// 
/// <br>
/// 
/// `T: FangAction` automatically implements `Fang` that performs as
/// 
/// ```
/// # use ohkami::{prelude::*, Fang, FangProc};
/// # #[derive(Clone)]
/// # struct DummyProc<
/// #     A: FangAction + Clone,
/// #     I: FangProc + Clone,
/// # > {
/// #     action: A,
/// #     inner:  I,
/// # }
/// # impl<
/// #     A: FangAction + Clone,
/// #     I: FangProc + Clone,
/// # > FangProc for DummyProc<A, I> {
/// async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
///     let Self { action, inner } = self;
///     match action.fore(req).await {
///         Err(e) => e,
///         Ok(()) => {
///             let mut res = inner.bite(req).await;
///             action.back(&mut res).await;
///             res
///         }
///     }
/// }
/// # }
/// ```
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// #[derive(Clone)]
/// struct SimpleLogger;
/// impl FangAction for SimpleLogger {
///     async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
///         println!("[request] {req:?}");
///         Ok(())
///     }
///     async fn back<'a>(&'a self, res: &'a mut Response) {
///         println!("[response] {res:?}");
///     }
/// }
/// ```
pub trait FangAction: Clone + Send + Sync + 'static {
    /// *fore fang*, that bites a request before a handler.
    /// 
    /// **Default**: just return `Ok(())`
    #[allow(unused_variables)]
    fn fore<'a>(&'a self, req: &'a mut Request) -> impl std::future::Future<Output = Result<(), Response>> + Send {
        async {Ok(())}
    }
    /// *back fang*, that bites a response after a handler.
    /// 
    /// **Default**: just return `()`
    #[allow(unused_variables)]
    fn back<'a>(&'a self, res: &'a mut Response) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }
} const _: () = {
    impl<A: FangAction, I: FangProc> Fang<I> for A {
        type Proc = FangActionProc<A, I>;
        fn chain(&self, inner: I) -> Self::Proc {
            FangActionProc {
                action: self.clone(),
                inner
            }
        }
    }

    pub struct FangActionProc<A: FangAction, I: FangProc> {
        action: A,
        inner:  I,
    }
    impl<A: FangAction, I: FangProc> FangProc for FangActionProc<A, I> {
        #[inline(always)]
        async fn bite<'b>(&'b self, ctx: super::super::Context<'b>, req: &'b mut Request) -> Response {
            let Self { action, inner } = self;
            match action.fore(req).await {
                Err(e) => e,
                Ok(()) => {
                    let mut res = inner.bite(ctx, req).await;
                    action.back(&mut res).await;
                    res
                }
            }
        }
    }
};




#[cfg(all(test, feature="testing", any(feature="rt_tokio",feature="rt_async-std")))]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::testing::*;

    #[crate::__rt__::test]
    async fn availablity() {
        use std::sync::{Mutex, OnceLock};

        fn messages() -> &'static Mutex<Vec<String>> {
            static MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
            MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
        }

        #[derive(Clone)]
        struct GreetingFang { name: &'static str }
        const _: () = {
            impl<I: FangProc> Fang<I> for GreetingFang {
                type Proc = GreetingFangProc<I>;
                fn chain(&self, inner: I) -> Self::Proc {
                    GreetingFangProc { fang: self.clone(), inner }
                }
            }

            struct GreetingFangProc<I: FangProc> {
                fang:  GreetingFang,
                inner: I
            }
            impl<I: FangProc> FangProc for GreetingFangProc<I> {
                async fn bite<'b>(&'b self, ctx: crate::Context<'b>, req: &'b mut Request) -> Response {
                    {
                        messages().lock().unwrap().push(format!("Hello, {}!", self.fang.name));
                    }
                    let res = self.inner.bite(ctx, req).await;
                    {
                        messages().lock().unwrap().push(format!("Bye, {}!", self.fang.name));
                    }
                    res
                }
            }
        };

        #[derive(Clone)]
        struct GreetingFangWithAction { name: &'static str }
        impl FangAction for GreetingFangWithAction {
            async fn fore<'b>(&'b self, _req: &'b mut Request) -> Result<(), Response> {
                messages().lock().unwrap().push(format!("Hello, {}!", self.name));
                Ok(())
            }
            async fn back<'b>(&'b self, _res: &'b mut Response) {
                messages().lock().unwrap().push(format!("Bye, {}!", self.name));
            }
        }

        let t = Ohkami::with((
            GreetingFang { name: "Clerk" },
            GreetingFangWithAction { name: "John" },
        ), (
            "/greet".POST(|| async {"Hi, I'm Handler!"}),
        )).test();

        {
            let req = Request::POST("/greet");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*messages().lock().unwrap(), &[
                "Hello, Clerk!",
                "Hello, John!",
                "Bye, John!",
                "Bye, Clerk!",
            ]);
        }
    }
}
