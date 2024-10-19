use crate::prelude::*;
use whttp::header::{WWWAuthenticate, Authorization};


/// # Builtin fang for Basic Auth
/// 
/// - `BasicAuth { username, password }` verifies each request to have the
///   `username` and `password`
/// - `[BasicAuth; N]` verifies each request to have one of the pairs of
///   `username` and `password`
/// 
/// <br>
/// 
/// ## Note
/// - **NEVER** hardcode `username` and `password` in your code
///   if you are pushing your source code to GitHub or other public repository!!!
/// - **NEVER** use this on `http://`!!! The `username` and `password` themselves
///   are not encoded in secure way and MUST passed on `https://`
/// 
/// <br>
/// 
/// *example*
/// ```rust,no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::BasicAuth;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/hello".GET(|| async {"Hello, public!"}),
///         "/private".By(Ohkami::with(
///             BasicAuth {
///                 username: "master of hello",
///                 password: "world"
///             },
///             "/hello".GET(|| async {"Hello, private :)"})
///         ))
///     )).howl("localhost:8888").await
/// }
/// ```
#[derive(Clone)]
pub struct BasicAuth<S>
where
    S: AsRef<str> + Clone + Send + Sync + 'static
{
    pub username: S,
    pub password: S
}

impl<S> BasicAuth<S>
where
    S: AsRef<str> + Clone + Send + Sync + 'static
{
    #[inline]
    fn matches(&self,
        username: &str,
        password: &str
    ) -> bool {
        self.username.as_ref() == username &&
        self.password.as_ref() == password
    }
}

const _: () = {
    fn unauthorized() -> Response {
        Response::Unauthorized()
            .with(WWWAuthenticate, "Basic realm=\"Secure Area\"")
    }

    #[inline]
    fn basic_credential_of(req: &Request) -> Result<String, Response> {
        let credential_base64 = req
            .header(Authorization).ok_or_else(unauthorized)?
            .strip_prefix("Basic ").ok_or_else(unauthorized)?;

        let credential = String::from_utf8(
            ohkami_lib::base64::decode(credential_base64.as_bytes())
        ).map_err(|_| unauthorized())?;

        Ok(credential)
    }

    impl<S> FangAction for BasicAuth<S>
    where
        S: AsRef<str> + Clone + Send + Sync + 'static
    {
        #[inline]
        async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
            let credential = basic_credential_of(req)?;
            let (username, password) = credential.split_once(':')
                .ok_or_else(unauthorized)?;

            self.matches(username, password).then_some(())
                .ok_or_else(unauthorized)?;

            Ok(())
        }
    }

    impl<S, const N: usize> FangAction for [BasicAuth<S>; N]
    where
        S: AsRef<str> + Clone + Send + Sync + 'static
    {
        #[inline]
        async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
            let credential = basic_credential_of(req)?;
            let (username, password) = credential.split_once(':')
                .ok_or_else(unauthorized)?;

            self.iter()
                .map(|candidate| candidate.matches(username, password))
                .any(|matched| matched).then_some(())
                .ok_or_else(unauthorized)?;

            Ok(())
        }
    }
};
