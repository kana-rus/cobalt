use crate::{FromRequest, IntoResponse, Request, Response, header::ContentType};
use serde::{Deserialize, Serialize};


pub struct URLEncoded<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for URLEncoded<S> {
    type Error = Response;

    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.header(ContentType)? != "application/x-www-form-urlencoded" {
            return None
        }
        ohkami_lib::serde_urlencoded::from_bytes(req.body()?)
            .map_err(super::reject)
            .map(Self).into()
    }
}

impl<S: Serialize> IntoResponse for URLEncoded<S> {
    fn into_response(self) -> Response {
        Response::OK().with_payload("application/x-www-form-urlencoded",
            ohkami_lib::serde_urlencoded::to_string(&self.0).unwrap().into_bytes()
        )
    }
}
