#![cfg(feature="__rt__")]

#[cfg(test)]
mod _test;

pub(crate) mod build;

pub use build::{Route, Routes};

use crate::router::TrieRouter;
use crate::fang::Fangs;
use std::sync::Arc;

#[cfg(feature="__rt_native__")]
use {crate::{__rt__, Session}, ohkami_lib::signal};

/// # Ohkami - a robust wolf who serves your web app
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// # use ohkami::serde::Serialize;
/// # use ohkami::typed::status::{OK, Created};
/// # use ohkami::format::JSON;
/// # use ohkami::{Fang, FangProc};
/// 
/// struct Auth;
/// impl<I: FangProc> Fang<I> for Auth {
///     /* 〜 */
/// #   type Proc = AuthProc;
/// #   fn chain(&self, inner: I) -> Self::Proc {
/// #       AuthProc
/// #   }
/// # }
/// # struct AuthProc;
/// # impl FangProc for AuthProc {
/// #     async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
/// #         Response::NotImplemented()
/// #     }
/// # }
/// 
/// # #[derive(Serialize)]
/// # struct User {
/// #     id:   usize,
/// #     name: String,
/// #     age:  Option<usize>,
/// # }
/// # 
/// # enum APIError {
/// #     UserNotFound
/// # }
/// # impl IntoResponse for APIError {
/// #     fn into_response(self) -> Response {
/// #         match self {
/// #             Self::UserNotFound => Response::InternalServerError()
/// #         }
/// #     }
/// # }
/// # 
/// # async fn health_check() -> impl IntoResponse {
/// #     Status::NoContent
/// # }
/// # 
/// # async fn create_user() -> Created<JSON<User>> {
/// #     Created(JSON(User {
/// #         id:   42,
/// #         name: String::from("ohkami"),
/// #         age:  None,
/// #     }))
/// # }
/// # 
/// # async fn get_user_by_id(id: usize) -> Result<JSON<User>, APIError> {
/// #     Ok(JSON(User {
/// #         id,
/// #         name: String::from("ohkami"),
/// #         age:  Some(2),
/// #     }))
/// # }
/// # 
/// # async fn update_user(id: usize) -> impl IntoResponse {
/// #     Status::OK
/// # }
/// 
/// fn my_ohkami() -> Ohkami {
///     let api_ohkami = Ohkami::with((Auth,), (
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     ));
/// 
///     Ohkami::new((
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ))
/// }
/// ```
/// 
/// <br>
/// 
/// #### handler schema：
/// `async ({path_params}?, {FromRequest type}s...) -> {IntoResponse type}`
/// 
/// #### path_params：
/// A tuple of types that implement `FromParam` trait.\
/// If the path contains only one parameter, then you can omit the tuple.\
/// (In current ohkami, at most *2* path params can be handled.)
/// 
/// <br>
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// struct MyParam;
/// impl<'p> ohkami::FromParam<'p> for MyParam {
///     type Error = std::convert::Infallible;
///     fn from_param(param: std::borrow::Cow<'p, str>) -> Result<Self, Self::Error> {
///         Ok(MyParam)
///     }
/// }
/// 
/// async fn handler_1(param: (MyParam,)) -> Response {
///     todo!()
/// }
/// 
/// async fn handler_2(str_param: &str) -> Response {
///     todo!()
/// }
/// ```
pub struct Ohkami {
    pub(crate) routes: TrieRouter,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Option<Arc<dyn Fangs>>,
}


impl Ohkami {
    /// Create new `Ohkami` on the routing.
    /// 
    /// ---
    ///
    /// `routes` is a routing item or a tuple of them :
    /// 
    /// ```
    /// # use ohkami::Route;
    /// #
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// )
    /// # ;
    /// ```
    /// 
    /// ---
    /// 
    /// Handler is an _**async**_ function :
    /// 
    /// > `({path params}, {FromRequest values},...) -> {IntoResponse value}`
    ///
    /// `{path params}` is a `FromParam` value or a tuple of them
    pub fn new(routes: impl build::Routes) -> Self {
        let mut router = TrieRouter::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  None,
        }
    }

    /// Create new ohkami with the fangs on the routing.
    /// 
    /// ---
    ///
    /// `fangs: impl Fangs` is an tuple of `Fang` items.
    /// 
    /// **NOTE**: You can omit tuple when `fangs` contains only one `Fang`.
    /// 
    /// <br>
    /// 
    /// ---
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// 
    /// #[derive(Clone)]
    /// struct AuthFang;
    /// impl FangAction for AuthFang {
    ///     //...
    /// }
    /// 
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// Ohkami::with(AuthFang, (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// ))
    /// # ;
    /// ```
    pub fn with(fangs: impl Fangs + 'static, routes: impl build::Routes) -> Self {
        let mut router = TrieRouter::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  Some(Arc::new(fangs)),
        }
    }

    #[cfg(feature="__rt_native__")]
    /// Start serving at `address`!
    /// 
    /// `address` is：
    /// 
    /// - `tokio::net::ToSocketAddrs` if using `tokio`
    /// - `async_std::net::ToSocketAddrs` if using `async-std`
    /// - `smol::net::AsyncToSocketAddrs` if using `smol`
    /// - `std::net::ToSocketAddrs` if using `glommio`
    /// 
    /// *note* : Keep-Alive timeout is 42 seconds and this is not
    /// configureable by user (it'll be in future version...)
    /// 
    /// <br>
    /// 
    /// ---
    /// 
    /// *example.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::typed::status;
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// async fn health_check() -> status::NoContent {
    ///     status::NoContent
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new((
    ///         "/".GET(hello),
    ///         "/healthz".GET(health_check),
    ///     )).howl("localhost:5000").await
    /// }
    /// ```
    /// 
    /// ---
    /// 
    /// *example_glommio.rs*
    /// ```ignore
    /// use ohkami::prelude::*;
    /// use ohkami::util::num_cpus;
    /// use glommio::{LocalExecutorPoolBuilder, PoolPlacement, CpuSet};
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// fn main() {
    ///     LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
    ///         num_cpus::get(), CpuSet::online().ok()
    ///     )).on_all_shards(|| {
    ///         Ohkami::new((
    ///             "/user/:id"
    ///                 .GET(echo_id),
    ///         )).howl("0.0.0.0:3000")
    ///     }).unwrap().join_all();
    /// }
    /// ```
    pub async fn howl(self, address: impl __rt__::ToSocketAddrs) {
        let router = Arc::new(self.into_router().into_radix());

        #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_smol"))]
        let listener = __rt__::TcpListener::bind(address).await.expect("Failed to bind TCP listener");
        #[cfg(any(feature="rt_glommio"))]
        let listener = __rt__::TcpListener::bind(address).expect("Failed to bind TCP listener");
        
        let (close_tx, close_rx) = signal::watch::channel(());

        let ctrl_c = signal::ctrl_c();
        let (ctrl_c_tx, ctrl_c_rx) = signal::watch::channel(());

        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
        __rt__::spawn(async {
            ctrl_c.await.expect("Something was wrong around Ctrl-C");
            drop(ctrl_c_rx);
        });
        #[cfg(any(feature="rt_smol",feature="rt_glommio"))]
        __rt__::spawn(async {
            ctrl_c.await.expect("Something was wrong around Ctrl-C");
            drop(ctrl_c_rx);
        }).detach();

        #[cfg(feature="rt_tokio")] {
            loop {
                __rt__::select! {
                    accept = listener.accept() => {
                        let Ok((connection, addr)) = accept else {continue};

                        let session = Session::new(
                            router.clone(),
                            connection,
                            addr.ip()
                        );

                        let close_rx = close_rx.clone();
                        __rt__::spawn(async {
                            session.manage().await;
                            drop(close_rx)
                        });
                    }
                    _ = ctrl_c_tx.closed() => {
                        crate::DEBUG!("Recieved Ctrl-C, trying graceful shutdown");
                        drop(listener);
                        break
                    }
                }
            }
        }

        #[cfg(feature="rt_async-std")] {
            loop {
                __rt__::select! {
                    accept = __rt__::FutureExt::fuse(listener.accept()) => {
                        let Ok((connection, addr)) = accept else {continue};

                        let session = Session::new(
                            router.clone(),
                            connection,
                            addr.ip()
                        );

                        let close_rx = close_rx.clone();
                        __rt__::spawn(async {
                            session.manage().await;
                            drop(close_rx)
                        });
                    }
                    _ = __rt__::FutureExt::fuse(ctrl_c_tx.closed()) => {
                        crate::DEBUG!("Recieved Ctrl-C, trying graceful shutdown");
                        drop(listener);
                        break
                    }
                }
            }
        }
        
        #[cfg(feature="rt_smol")] {
            loop {
                __rt__::select! {
                    accept = __rt__::FutureExt::fuse(listener.accept()) => {
                        let Ok((connection, addr)) = accept else {continue};

                        let session = Session::new(
                            router.clone(),
                            connection,
                            addr.ip()
                        );

                        let close_rx = close_rx.clone();
                        __rt__::spawn(async {
                            session.manage().await;
                            drop(close_rx)
                        }).detach();
                    }
                    _ = __rt__::FutureExt::fuse(ctrl_c_tx.closed()) => {
                        crate::DEBUG!("Recieved Ctrl-C, trying graceful shutdown");
                        drop(listener);
                        break
                    }
                }
            }
        }

        #[cfg(feature="rt_glommio")] {
            loop {
                __rt__::select! {
                    accept = __rt__::FutureExt::fuse(listener.accept()) => {
                        let Ok(connection) = accept else {continue};
                        let Ok(addr) = connection.peer_addr() else {continue};
        
                        let session = Session::new(
                            router.clone(),
                            connection,
                            addr.ip()
                        );

                        let close_rx = close_rx.clone();
                        __rt__::spawn(async {
                            session.manage().await;
                            drop(close_rx)
                        }).detach();
                    }
                    _ = __rt__::FutureExt::fuse(ctrl_c_tx.closed()) => {
                        crate::DEBUG!("Recieved Ctrl-C, trying graceful shutdown");
                        drop(listener);
                        break
                    }
                }
            }
        }

        crate::DEBUG!("Waiting {} session(s) to finish...", close_tx.receiver_count());
        drop(close_rx);
        close_tx.closed().await;
    }

    #[cfg(feature="rt_worker")]
    #[doc(hidden)]
    pub async fn __worker__(self,
        req: ::worker::Request,
        env: ::worker::Env,
        ctx: ::worker::Context,
    ) -> ::worker::Response {
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Called `#[ohkami::worker]`; req: {req:?}");

        let mut ohkami_req = crate::Request::init();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::init`");

        let mut ohkami_req = unsafe {std::pin::Pin::new_unchecked(&mut ohkami_req)};
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Put request in `Pin`");

        let take_over = ohkami_req.as_mut().take_over(req, env, ctx).await;
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::take_over`: {ohkami_req:?}");

        let ohkami_res = match take_over {
            Ok(()) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` succeed");

                let router = self.into_router();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `Ohkami::into_router`");

                let router = router.into_radix();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `TrieRouter::into_radix` (without compressions)");
                
                let mut res = router.handle(&mut ohkami_req).await;
                res.complete();
                res
            }
            Err(e) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` returned an error response: {e:?}");
                e
            }
        };
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Successfully generated ohkami::Response: {ohkami_res:?}");

        let res = ohkami_res.into();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Response` --into--> `worker::Response`: {res:?}");

        res
    }
}

impl Ohkami {
    pub(crate) fn into_router(self) -> TrieRouter {
        let Self { routes: mut router, fangs } = self;

        if let Some(fangs) = fangs {
            router.apply_fangs(router.id(), fangs);
        }

        #[cfg(feature="DEBUG")]
        println!("{router:#?}");

        router
    }
}
