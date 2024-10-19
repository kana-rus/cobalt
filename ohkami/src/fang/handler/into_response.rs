#![allow(non_snake_case)]

use whttp::{Response, Status};


/// A trait implemented by types that can be used as a return value of a handler.
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// struct MyResponse {
///     message: String,
/// }
/// impl IntoResponse for MyResponse {
///     fn into_response(self) -> Response {
///         Response::OK().with_text(self.message)
///     }
/// }
/// 
/// async fn handler() -> MyResponse {
///     MyResponse {
///         message: String::from("Hello!")
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new(
///         "/".GET(handler)
///     ).howl("localhost:5050").await
/// }
/// ```
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    #[inline] fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Status {
    #[inline(always)] fn into_response(self) -> Response {
        Response::of(self)
    }
}

impl<T:IntoResponse, E:IntoResponse> IntoResponse for Result<T, E> {
    #[inline(always)] fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for std::convert::Infallible {
    #[cold]
    fn into_response(self) -> Response {
        unsafe {std::hint::unreachable_unchecked()}
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::OK()
    }
}

macro_rules! text_response {
    ($( $t:ty )*) => {$(
        impl IntoResponse for $t {
            #[inline(always)]
            fn into_response(self: $t) -> Response {
                Response::OK().with_text(self)
            }
        }
    )*};
} text_response! {
    &'static str
    String
    std::borrow::Cow<'static, str>
}
