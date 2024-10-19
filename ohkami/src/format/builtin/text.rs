use crate::{FromRequest, IntoResponse, Request, Response, header::ContentType};


pub struct Text<T>(pub T);

impl<'req, T: From<&'req str>> FromRequest<'req> for Text<T> {
    type Error = Response;
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if !req.header(ContentType)?.starts_with("text/plain") {
            return None
        }
        std::str::from_utf8(req.body()?)
            .map_err(super::reject)
            .map(|s| Self(T::from(s))).into()
    }
}

impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for Text<T> {
    fn into_response(self) -> Response {
        Response::OK().with_text(self.0)
    }
}
