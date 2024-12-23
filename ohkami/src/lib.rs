#![doc(html_root_url = "https://docs.rs/ohkami/0.21.0")]

/* Execute static tests for sample codes in README */
#![cfg_attr(feature="DEBUG", doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>Ohkami</h1>
//!     Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
//! </div>
//! 
//! <br>
//! 
//! - *macro-less and type-safe* APIs for intuitive and declarative code
//! - *multi runtimes* are supported：`tokio`, `async-std`, `smol`, `nio`, `glommio`, `worker` (Cloudflare Workers)
//! 
//! <div align="right">
//!     <a href="https://github.com/ohkami-rs/ohkami/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
//!     <a href="https://github.com/ohkami-rs/ohkami/actions"><img alt="build check status of ohkami" src="https://github.com/ohkami-rs/ohkami/actions/workflows/CI.yml/badge.svg"/></a>
//!     <a href="https://crates.io/crates/ohkami"><img alt="crates.io" src="https://img.shields.io/crates/v/ohkami" /></a>
//! </div>


#![allow(incomplete_features)]
#![cfg_attr(feature="nightly", feature(
    specialization,
    try_trait_v2,
))]


#[cfg(any(
    all(feature="rt_tokio",      any(feature="rt_async-std", feature="rt_smol",      feature="rt_nio",       feature="rt_glommio",   feature="rt_worker"   )),
    all(feature="rt_async-std",  any(feature="rt_smol",      feature="rt_nio",       feature="rt_glommio",   feature="rt_worker",    feature="rt_tokio"    )),
    all(feature="rt_smol",       any(feature="rt_nio",       feature="rt_glommio",   feature="rt_worker",    feature="rt_tokio",     feature="rt_async-std")),
    all(feature="rt_nio",        any(feature="rt_glommio",   feature="rt_worker",    feature="rt_tokio",     feature="rt_async-std", feature="rt_smol"     )),
    all(feature="rt_glommio",    any(feature="rt_worker",    feature="rt_tokio",     feature="rt_async-std", feature="rt_smol",      feature="rt_nio"      )),
    all(feature="rt_worker",     any(feature="rt_tokio",     feature="rt_async-std", feature="rt_smol",      feature="rt_nio",       feature="rt_glommio"  )),
))] compile_error! {"
    Can't activate multiple `rt_*` features at once!
"}


#[cfg(feature="__rt_native__")]
mod __rt__ {
    #[cfg(test)]
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_smol")]
    pub(crate) use smol::net::{TcpListener, TcpStream, AsyncToSocketAddrs as ToSocketAddrs};
    #[cfg(feature="rt_nio")]
    pub(crate) use {nio::net::{TcpListener, TcpStream}, std::net::ToSocketAddrs};
    #[cfg(feature="rt_glommio")]
    pub(crate) use {glommio::net::{TcpListener, TcpStream}, std::net::ToSocketAddrs};

    pub(crate) async fn bind(address: impl ToSocketAddrs) -> TcpListener {
        let binded = TcpListener::bind(address);
        
        #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))]
        let binded = binded.await;
        
        binded.expect("Failed to bind TCP listener")
    }

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::time::sleep;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task::sleep;
    #[cfg(feature="rt_smol")]
    pub(crate) async fn sleep(duration: std::time::Duration) {
        smol::Timer::after(duration).await;
    }
    #[cfg(feature="rt_nio")]
    pub(crate) use nio::time::sleep;
    #[cfg(feature="rt_glommio")]
    pub(crate) use glommio::timer::sleep;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncRead;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncRead;
    #[cfg(feature="rt_smol")]
    pub(crate) use futures_util::AsyncReadExt as AsyncRead;
    #[cfg(feature="rt_nio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncRead;
    #[cfg(feature="rt_glommio")]
    pub(crate) use futures_util::AsyncReadExt as AsyncRead;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWrite;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWrite;
    #[cfg(feature="rt_smol")]
    pub(crate) use futures_util::AsyncWriteExt as AsyncWrite;
    #[cfg(feature="rt_nio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWrite;
    #[cfg(feature="rt_glommio")]
    pub(crate) use futures_util::AsyncWriteExt as AsyncWrite;

    #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))]
    mod task {
        pub trait Task: std::future::Future<Output: Send + 'static> + Send + 'static {}
        impl<F: std::future::Future<Output: Send + 'static> + Send + 'static> Task for F {}
    }
    #[cfg(any(feature="rt_glommio"))]
    mod task {
        pub trait Task: std::future::Future {}
        impl<F: std::future::Future> Task for F {}
    }
    pub(crate) fn spawn(task: impl task::Task + 'static) {
        #[cfg(feature="rt_tokio")]
        tokio::task::spawn(task);

        #[cfg(feature="rt_async-std")]
        async_std::task::spawn(task);

        #[cfg(feature="rt_smol")]
        smol::spawn(task).detach();

        #[cfg(feature="rt_nio")]
        nio::spawn(task);

        #[cfg(feature="rt_glommio")]
        glommio::spawn_local(task).detach();
    }

    #[cfg(all(test, debug_assertions))]
    pub(crate) mod testing {
        pub(crate) fn block_on(future: impl std::future::Future) {
            #[cfg(feature="rt_tokio")]
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build().unwrap()
                .block_on(future);

            #[cfg(feature="rt_async-std")]
            async_std::task::block_on(future);

            #[cfg(feature="rt_smol")]
            smol::block_on(future);

            #[cfg(feature="rt_nio")]
            nio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build().unwrap()
                .block_on(future);

            #[cfg(feature="rt_glommio")]
            glommio::LocalExecutor::default().run(future);
        }

        pub(crate) const PORT: u16 = {
            #[cfg(feature="rt_tokio")    ] {3001}
            #[cfg(feature="rt_async-std")] {3002}
            #[cfg(feature="rt_smol")     ] {3003}
            #[cfg(feature="rt_nio")      ] {3004}
            #[cfg(feature="rt_glommio")  ] {3005}
        };
    }
}

pub mod util;

mod config;
pub(crate) static CONFIG: config::Config = config::Config::new();

#[cfg(feature="openapi")]
pub mod openapi {
    pub use ::ohkami_openapi::*;
}

#[cfg(debug_assertions)]
#[cfg(feature="__rt__")]
pub mod testing;

mod request;
pub use request::{Request, Method, FromRequest, FromParam, FromBody};
pub use ::ohkami_macros::FromRequest;

mod response;
pub use response::{Response, Status, IntoResponse, IntoBody};

#[cfg(feature="__rt_native__")]
mod session;
#[cfg(feature="__rt_native__")]
use session::Session;

#[cfg(feature="__rt__")]
mod router;

#[cfg(feature="__rt__")]
mod ohkami;
#[cfg(feature="__rt__")]
pub use ohkami::{Ohkami, Route};

pub mod fang;
pub use fang::{Fang, FangProc};

pub mod format;

pub mod header;

pub mod typed;

#[cfg(feature="sse")]
pub mod sse;

#[cfg(feature="ws")]
pub mod ws;

#[cfg(feature="rt_worker")]
pub use ::ohkami_macros::{worker, bindings};

pub mod prelude {
    pub use crate::{Request, Response, IntoResponse, Method, Status};
    pub use crate::util::FangAction;
    pub use crate::serde::{Serialize, Deserialize};
    pub use crate::format::{JSON, Query};
    pub use crate::fang::Memory;

    #[cfg(feature="__rt__")]
    pub use crate::{Route, Ohkami};
}

/// Somthing almost [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json).
/// 
/// ---
/// *not_need_serde_in_your_dependencies.rs*
/// ```
/// use ohkami::serde::{json, Serialize};
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     age:  u8,
/// }
/// 
/// # fn _user() {
/// let user = User {
///     name: String::from("ABC"),
///     age:  200,
/// };
/// assert_eq!(json::to_string(&user).unwrap(), r#"
///     {"age":200,"username":"ABC"}
/// "#);
/// # }
/// ```
/// ---
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
    pub use ::serde_json as json;
}

#[doc(hidden)]
pub mod __internal__ {
    pub use ::serde;

    pub use ohkami_macros::consume_struct;

    pub use crate::fang::Fangs;

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    pub use crate::{
        request::{RequestHeader, RequestHeaders},
        response::{ResponseHeader, ResponseHeaders},
    };
}
