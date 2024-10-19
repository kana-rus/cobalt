use crate::{Response, FromRequest};
use ohkami_lib::serde_urlencoded;
use serde::Deserialize;


pub struct Query<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for Query<S> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        serde_urlencoded::from_bytes(req.query_raw()?)
            .map_err(super::reject)
            .map(Query).into()
    }
}
