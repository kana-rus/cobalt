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
//! - *multi runtimes* are supported：`tokio`, `async-std`, `worker` (Cloudflare Workers)
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
    all(feature="rt_tokio",     feature="rt_async-std"),
    all(feature="rt_tokio",     feature="rt_smol"),
    all(feature="rt_tokio",     feature="rt_glommio"),
    all(feature="rt_tokio",     feature="rt_worker"),
    all(feature="rt_async-std", feature="rt_smol"),
    all(feature="rt_async-std", feature="rt_glommio"),
    all(feature="rt_async-std", feature="rt_worker"),
    all(feature="rt_smol",      feature="rt_glommio"),
    all(feature="rt_smol",      feature="rt_worker"),
    all(feature="rt_glommio",   feature="rt_worker"),
))] compile_error! {"
    Can't activate multiple `rt_*` features at once!
"}

#[cfg(not(feature="DEBUG"))]
#[cfg(all(feature="rt_worker", not(target_arch="wasm32")))]
compile_error! {"
    `rt_worker` must be activated on `wasm32` target!
    (We recommend to touch `.cargo/config.toml`: `[build] target = \"wasm32-unknown-unknown\"`)
"}


#[allow(unused)]
mod __rt__ {
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_smol")]
    pub(crate) use smol::net::{TcpListener, TcpStream, AsyncToSocketAddrs as ToSocketAddrs};
    #[cfg(feature="rt_glommio")]
    pub(crate) use {glommio::net::{TcpListener, TcpStream}, std::net::ToSocketAddrs};

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task::spawn;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task::spawn;
    #[cfg(feature="rt_smol")]
    pub(crate) use smol::spawn;
    #[cfg(feature="rt_glommio")]
    pub(crate) use glommio::spawn_local as spawn;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::time::sleep;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task::sleep;
    #[cfg(feature="rt_smol")]
    pub(crate) async fn sleep(duration: std::time::Duration) {
        smol::Timer::after(duration).await;
    }
    #[cfg(feature="rt_glommio")]
    pub(crate) use glommio::timer::sleep;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;
    #[cfg(feature="rt_smol")]
    pub(crate) use futures_util::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_glommio")]
    pub(crate) use futures_util::AsyncReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;
    #[cfg(feature="rt_smol")]
    pub(crate) use futures_util::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_glommio")]
    pub(crate) use futures_util::AsyncWriteExt as AsyncWriter;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::select;
    #[cfg(feature="rt_async-std")]
    pub(crate) use futures_util::{select, FutureExt};
    #[cfg(feature="rt_smol")]
    pub(crate) use futures_util::{select, FutureExt};
    #[cfg(feature="rt_glommio")]
    pub(crate) use futures_util::{select, FutureExt};
}


pub use ::whttp::{Request, Response, Status, Method, header};

pub use ::ohkami_macros::FromRequest;

pub mod fang;
pub use fang::{Fang, FangProc, FromParam, FromRequest, IntoResponse};

pub mod format;

#[cfg(feature="__rt_native__")]
mod session;
#[cfg(feature="__rt_native__")]
use session::Session;

#[cfg(feature="__rt__")]
mod ohkami;
#[cfg(feature="__rt__")]
pub use ohkami::{Ohkami, Route};

pub mod typed;

#[cfg(all(feature="ws", feature="__rt_native__"))]
pub mod ws;

#[cfg(feature="testing")]
#[cfg(feature="__rt__")]
pub mod testing;

pub mod util;

#[cfg(feature="rt_worker")]
pub use ::ohkami_macros::{worker, bindings};

pub mod prelude {
    pub use ::whttp::{Request, Response};

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
}
