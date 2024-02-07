#![doc(html_root_url = "https://docs.rs/ohkami")]

/* Execute static test for sample codes in README */
#![cfg_attr(feature="DEBUG", doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>ohkami</h1>
//!     ohkami <em>- [狼] wolf in Japanese -</em> is <strong>declarative</strong> web framework for Rust.
//! </div>
//! 
//! - *macro less, declarative APIs*
//! - *multi runtime* support：`tokio`, `async-std`
//! 
//! See our [README](https://github.com/kana-rus/ohkami/blob/main/README.md)
//! and [examples](https://github.com/kana-rus/ohkami/tree/main/examples)
//! for more information！


/*===== crate features =====*/

#[cfg(any(
    all(feature="rt_tokio", feature="rt_async-std")
))] compile_error!("
    Can't activate multiple `rt_*` feature!
");

#[cfg(not(any(
    feature="rt_tokio",
    feature="rt_async-std",
)))] compile_error!("
    Activate 1 of `rt_*` features：
    - rt_tokio
    - rt_async-std
");


/*===== async runtime dependency layer =====*/

mod __rt__ {
    #[allow(unused)]
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(all(feature="websocket", feature="rt_tokio"))]
    pub(crate) use tokio::net::TcpStream;
    #[cfg(all(feature="websocket", feature="rt_async-std"))]
    pub(crate) use async_std::net::TcpStream;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpListener;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpListener;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::Mutex;
    #[allow(unused)]
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::sync::Mutex;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;

    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::stream::StreamExt;
}


/*===== modules =====*/
mod typed;
mod router;
mod ohkami;

#[cfg(feature="testing")]
mod testing;

#[cfg(feature="utils")]
mod utils;

#[cfg(feature="websocket")]
mod x_websocket;


/*===== visibility managements =====*/
pub use layer1_req_res     ::{Request, Method, Response, Status, FromRequestError, FromRequest, FromParam, IntoResponse, Memory};
pub use layer2_fang_handler::{Route, Fang};
pub use ohkami      ::{Ohkami, IntoFang};

/// Passed to `{Request/Response}.headers.set().Name( 〜 )` and
/// append `value` to the header
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// use ohkami::append;
/// 
/// struct AppendServer;
/// impl IntoFang for AppendServer {
///     fn into_fang(self) -> Fang {
///         Fang::back(|res: &mut Response| {
///             res.headers.set()
///                 .Server(append("ohkami"));
///         })
///     }
/// }
/// ```
pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> __internal__::Append {
    __internal__::Append(value.into())
}
    
pub mod prelude {
    pub use crate::{Request, Route, Ohkami, Fang, Response, IntoFang, IntoResponse, Method, Status};

    #[cfg(feature="utils")]
    pub use crate::typed::{OK, Created, NoContent};
}

/// Ohkami testing tools
/// 
/// <br>
/// 
/// *test_example.rs*
/// ```
/// use ohkami::prelude::*;
/// use ohkami::testing::*;
/// 
/// fn my_ohkami() -> Ohkami {
///     Ohkami::new(
///         "/".GET(|| async {
///             "Hello, ohkami!"
///         })
///     )
/// }
/// 
/// #[cfg(test)]
/// #[tokio::test]
/// async fn test_my_ohkami() {
///     let mo = my_ohkami();
/// 
///     let req = TestRequest::GET("/");
///     let res = mo.oneshot(req).await;
///     assert_eq!(res.status, Status::OK);
///     assert_eq!(res.text(), Some("Hello, ohkami!"));
/// }
/// ```
#[cfg(feature="testing")]
pub mod testing {
    pub use crate::testing::*;
}

/// Some utilities for building web app
#[cfg(feature="utils")]
pub mod utils {
    pub use crate::utils::{imf_fixdate_now, unix_timestamp, Text, HTML};
    pub use crate::utils::File;
}

/// Ohkami's buitlin fangs
/// 
/// - `CORS`
/// - `JWT`
#[cfg(feature="utils")]
pub mod fangs {
    pub use crate::utils::{CORS, JWT};
}

/// Somthing that's almost [serde](https://crates.io/crates/serde)
/// 
/// <br>
/// 
/// *not_need_serde_in_your_dependencies.rs*
/// ```
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     age:  u8,
/// }
/// ```
#[cfg(feature="utils")]
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
}

/// Convenient tools to build type-safe handlers
#[cfg(feature="utils")]
pub mod typed {
    pub use ohkami_macros::{ResponseBody, Query, Payload};

    pub use crate::utils::ResponseBody;
    pub use crate::utils::*;
}

#[cfg(feature="websocket")]
pub mod websocket {
    pub use crate::x_websocket::*;
}

#[doc(hidden)]
pub mod __internal__ {
    pub struct Append(pub(crate) std::borrow::Cow<'static, str>);

    #[cfg(feature="utils")]
    pub use ::serde;

    #[cfg(feature="utils")]
    pub use ohkami_macros::consume_struct;

    #[cfg(feature="utils")]
    pub use crate::utils::{
        parse_json,
        parse_formparts,
        parse_urlencoded,
    };

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    pub use crate::layer1_req_res::{RequestHeader, RequestHeaders, ResponseHeader, ResponseHeaders};
}
