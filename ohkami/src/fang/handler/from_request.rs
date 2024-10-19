use crate::{util::ErrorMessage, IntoResponse};
use std::borrow::Cow;
use whttp::{Request, Response};


/// "Retirieved from a `Request`".
/// 
/// ### required
/// - `type Errpr`
/// - `fn from_request`
/// 
/// Of course, you can manually implement for your structs that can be extracted from a request：
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct IsGETRequest(bool);
/// 
/// impl ohkami::FromRequest<'_> for IsGETRequest {
///     type Error = std::convert::Infallible;
///     fn from_request(req: &Request) -> Option<Result<Self, Self::Error>> {
///         Some(Ok(Self(
///             req.method.isGET()
///         )))
///     }
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// NOTE: *MUST NOT impl both `FromRequest` and `FromParam`*.
pub trait FromRequest<'req>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;
    
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>>;

}
const _: () = {
    impl<'req> FromRequest<'req> for &'req Request {
        type Error = std::convert::Infallible;
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req))
        }
    }
    impl<'req, FR: FromRequest<'req>> FromRequest<'req> for Option<FR> {
        type Error = FR::Error;
        #[inline]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            match FR::from_request(req) {
                None     => Some(Ok(None)),
                Some(fr) => Some(fr.map(Some))
            }
        }
    }
};
#[cfg(feature="rt_worker")]
const _: () = {
    impl<'req> FromRequest<'req> for &'req ::worker::Env {
        type Error = std::convert::Infallible;
        #[inline(always)]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req.env()))
        }
    }
    impl<'req> FromRequest<'req> for &'req ::worker::Context {
        type Error = std::convert::Infallible;
        #[inline(always)]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req.context()))
        }
    }
};


/// "Retrieved from a path/query param".
/// 
/// ### required
/// - `type Errpr`
/// - `fn from_param`
/// 
/// NOTE: *MUST NOT impl both `FromRequest` and `FromParam`*.
pub trait FromParam<'p>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;

    /// `param` is already percent-decoded：
    /// 
    /// - `Cow::Borrowed(&'p str)` if not encoded in request
    /// - `Cow::Owned(String)` if encoded and ohkami has decoded
    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error>;

    #[inline(always)]
    fn from_raw_param(raw_param: &'p [u8]) -> Result<Self, Response> {
        Self::from_param(
            ohkami_lib::percent_decode_utf8(raw_param)
                .map_err(|_e| {
                    #[cfg(debug_assertions)] crate::warning!(
                        "Failed to decode percent encoded param `{}`: {_e}",
                        raw_param.escape_ascii()
                    );
                    Response::InternalServerError()
                })?
        ).map_err(IntoResponse::into_response)
    }
} const _: () = {
    impl<'p> FromParam<'p> for String {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(match param {
                Cow::Owned(s)    => s,
                Cow::Borrowed(s) => s.into()
            })
        }
    }
    impl<'p> FromParam<'p> for Cow<'p, str> {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(param)
        }
    }
    impl<'p> FromParam<'p> for &'p str {
        type Error = ErrorMessage;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_) => Err({
                    #[cold] #[inline(never)]
                    fn unexpected(param: &str) -> ErrorMessage {                        
                        crate::warning!("\
                            `&str` can't handle percent encoded parameters. \
                            Use `Cow<'_, str>` or `String` to handle them. \
                        ");
                        ErrorMessage(format!(    
                            "Unexpected path params `{param}`: percent encoded"
                        ))
                    } unexpected(&param)
                }),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = ErrorMessage;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        let digit_bytes = param.as_bytes();
                        (digit_bytes.len() > 0).then_some(())
                            .ok_or_else(|| ErrorMessage(format!("Unexpected path params: Expected a number nut found an empty string")))?;
                        match digit_bytes {
                            [b'0'] => Ok(0),
                            [b'0', ..] => Err((|| ErrorMessage(format!("Unexpected path params `{}`: Expected a number but it starts with '0'", digit_bytes.escape_ascii())))()),
                            _ => {
                                let mut value: $unsigned_int = 0;
                                for d in digit_bytes {
                                    match d {
                                        b'0'..=b'9' => value = value * 10 + (*d - b'0') as $unsigned_int,
                                        _ => return Err((|| ErrorMessage(format!("Unexpected path params `{}`: Expected a number but it contains a non-digit charactor", digit_bytes.escape_ascii())))())
                                    }
                                }
                                Ok(value)
                            }
                        }
                    }
                }
            )*
        };
    } unsigned_integers! { u8, u16, u32, u64, u128, usize }
};
