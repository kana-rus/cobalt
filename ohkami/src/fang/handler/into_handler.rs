use super::{Handler, SendOnNative, SendSyncOnNative, ResponseFuture};
use crate::{FromRequest, FromParam, IntoResponse};
use std::{future::Future, pin::Pin};
use whttp::{Request, Response};


pub trait IntoHandler<T> {
    fn into_handler(self) -> Handler;
}

#[inline(never)] #[cold] fn __error__(e: Response) -> Pin<Box<dyn ResponseFuture>> {
    Box::pin(async {e})
}

/* FIXME: omit unsafe... */
#[inline(always)] fn from_request<'fr, 'req, R: FromRequest<'fr>>(
    req: &'req Request
) -> Result<R, Response> {
    <R as FromRequest>::from_request(unsafe {
        std::mem::transmute::<&'req _, &'fr _>(req)
    })
        .ok_or_else(|| Response::BadRequest().with_text("missing something expected in request"))?
        .map_err(IntoResponse::into_response)
}


const _: (/* no args */) = {
    impl<'req, F, Body, Fut> IntoHandler<fn()->Body> for F
    where
        F:    Fn() -> Fut + SendSyncOnNative + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |_| {
                let res = self();
                Box::pin(async move {
                    res.await.into_response()
                })
            })
        }
    }
};

const _: (/* FromParam */) = {
    impl<'req, F, Fut, Body, P1:FromParam<'req>> IntoHandler<fn((P1,))->Body> for F
    where
        F:    Fn(P1) -> Fut + SendSyncOnNative + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match P1::from_raw_param(unsafe {req.path.assume_one_param()}) {
                    Ok(p1) => {
                        let res = self(p1);
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(e) => __error__(e)
                }
            )
        }
    }

    impl<'req, F, Body, Fut, P1:FromParam<'req>> IntoHandler<fn(((P1,),))->Body> for F
    where
        F:    Fn((P1,)) -> Fut + SendSyncOnNative + 'static,
        Body: IntoResponse,
        Fut:  Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                match P1::from_raw_param(unsafe {req.path.assume_one_param()}) {
                    Ok(p1) => {
                        let res = self((p1,));
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(e) => __error__(e)
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>> IntoHandler<fn(((P1, P2),))->Body> for F
    where
        F:   Fn((P1, P2)) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                let (p1, p2) = unsafe {req.path.assume_two_params()};
                match (P1::from_raw_param(p1), P2::from_raw_param(p2)) {
                    (Ok(p1), Ok(p2)) => {
                        let res = self((p1, p2));
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _) | (_, Err(e)) => __error__(e),
                }
            })
        }
    }
};

const _: (/* FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>> IntoHandler<fn(Item1)->Body> for F
    where
        F:   Fn(Item1) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match from_request::<Item1>(req) {
                    Ok(item1) => {
                        let res = self(item1);
                        Box::pin(async move {res.await.into_response()})
                    }
                    Err(e) => __error__(e)
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn(Item1, Item2)->Body> for F
    where
        F:   Fn(Item1, Item2) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match (from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(item1), Ok(item2)) => {
                        let res = self(item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _) |
                    (_, Err(e)) => __error__(e),
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn(Item1, Item2, Item3)->Body> for F
    where
        F:   Fn(Item1, Item2, Item3) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match (from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self(item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _, _) |
                    (_, Err(e), _) |
                    (_, _, Err(e)) => __error__(e),
                }
            )
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>, Item4:FromRequest<'req>> IntoHandler<fn(Item1, Item2, Item3, Item4)->Body> for F
    where
        F:   Fn(Item1, Item2, Item3, Item4) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req|
                match (from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req), from_request::<Item4>(req)) {
                    (Ok(item1), Ok(item2), Ok(item3), Ok(item4)) => {
                        let res = self(item1, item2, item3, item4);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e), _, _,_) |
                    (_, Err(e), _,_) |
                    (_, _, Err(e),_) |
                    (_,_, _, Err(e)) => __error__(e),
                }
            )
        }
    }
};

const _: (/* one FromParam without tuple and FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>> IntoHandler<fn(((P1,),), Item1)->Body> for F
    where
        F:   Fn(P1, Item1) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request(req)) {
                    (Ok(p1), Ok(item1)) => {
                        let res = self(p1, item1);
                        Box::pin(async move {res.await.into_response()})
                    },
                    (Err(e), _) |
                    (_, Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn(((P1,),), Item1, Item2)->Body> for F
    where
        F:   Fn(P1, Item1, Item2) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2)) => {
                        let res = self(p1, item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_) |
                    (_,Err(e),_) |
                    (_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn(((P1,),), Item1, Item2, Item3)->Body> for F
    where
        F:   Fn(P1, Item1, Item2, Item3) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self(p1, item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_) |
                    (_,Err(e),_,_) |
                    (_,_,Err(e),_) |
                    (_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>, Item4:FromRequest<'req>> IntoHandler<fn(((P1,),), Item1, Item2, Item3, Item4)->Body> for F
    where
        F:   Fn(P1, Item1, Item2, Item3, Item4) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req), from_request::<Item4>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2), Ok(item3), Ok(item4)) => {
                        let res = self(p1, item1, item2, item3, item4);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_,_) |
                    (_,Err(e),_,_,_) |
                    (_,_,Err(e),_,_) |
                    (_,_,_,Err(e),_) |
                    (_,_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }
};

const _: (/* one FromParam and FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>> IntoHandler<fn((P1,), Item1)->Body> for F
    where
        F:   Fn((P1,), Item1) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request::<Item1>(req)) {
                    (Ok(p1), Ok(item1)) => {
                        let res = self((p1,), item1);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_) |
                    (_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn((P1,), Item1, Item2)->Body> for F
    where
        F:   Fn((P1,), Item1, Item2) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};

                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2)) => {
                        let res = self((p1,), item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_) |
                    (_,Err(e),_) |
                    (_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn((P1,), Item1, Item2, Item3)->Body> for F
    where
        F:   Fn((P1,), Item1, Item2, Item3) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};
                
                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self((p1,), item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_) |
                    (_,Err(e),_,_) |
                    (_,_,Err(e),_) |
                    (_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>, Item4:FromRequest<'req>> IntoHandler<fn((P1,), Item1, Item2, Item3, Item4)->Body> for F
    where
        F:   Fn((P1,), Item1, Item2, Item3, Item4) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed once before this code
                let p1 = unsafe {req.path.assume_one_param()};
                
                match (P1::from_raw_param(p1), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req), from_request::<Item4>(req)) {
                    (Ok(p1), Ok(item1), Ok(item2), Ok(item3), Ok(item4)) => {
                        let res = self((p1,), item1, item2, item3, item4);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_,_) |
                    (_,Err(e),_,_,_) |
                    (_,_,Err(e),_,_) |
                    (_,_,_,Err(e),_) |
                    (_,_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }
};

const _: (/* two PathParams and FromRequest items */) = {
    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1)->Body> for F
    where
        F:   Fn((P1, P2), Item1) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (FromParam::from_raw_param(p1), FromParam::from_raw_param(p2), from_request::<Item1>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1)) => {
                        let res = self((p1, p2), item1); 
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_) |
                    (_,Err(e),_) |
                    (_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1, Item2)->Body> for F
    where
        F:   Fn((P1, P2), Item1, Item2) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (FromParam::from_raw_param(p1), FromParam::from_raw_param(p2), from_request::<Item1>(req), from_request::<Item2>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2)) => {
                        let res = self((p1, p2), item1, item2);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_) |
                    (_,Err(e),_,_) |
                    (_,_,Err(e),_) |
                    (_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1, Item2, Item3)->Body> for F
    where
        F:   Fn((P1, P2), Item1, Item2, Item3) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (FromParam::from_raw_param(p1), FromParam::from_raw_param(p2), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2), Ok(item3)) => {
                        let res = self((p1, p2), item1, item2, item3);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_,_) |
                    (_,Err(e),_,_,_) |
                    (_,_,Err(e),_,_) |
                    (_,_,_,Err(e),_) |
                    (_,_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }

    impl<'req, F, Fut, Body:IntoResponse, P1:FromParam<'req>, P2:FromParam<'req>, Item1:FromRequest<'req>, Item2:FromRequest<'req>, Item3:FromRequest<'req>, Item4:FromRequest<'req>> IntoHandler<fn((P1, P2), Item1, Item2, Item3, Item4)->Body> for F
    where
        F:   Fn((P1, P2), Item1, Item2, Item3, Item4) -> Fut + SendSyncOnNative + 'static,
        Fut: Future<Output = Body> + SendOnNative + 'static,
    {
        fn into_handler(self) -> Handler {
            Handler::new(move |req| {
                // SAFETY: Due to the architecture of `Router`,
                // `params` has already `append`ed twice before this code
                let (p1, p2) = unsafe {req.path.assume_two_params()};

                match (FromParam::from_raw_param(p1), FromParam::from_raw_param(p2), from_request::<Item1>(req), from_request::<Item2>(req), from_request::<Item3>(req), from_request::<Item4>(req)) {
                    (Ok(p1), Ok(p2), Ok(item1), Ok(item2), Ok(item3), Ok(item4)) => {
                        let res = self((p1, p2), item1, item2, item3, item4);
                        Box::pin(async move {res.await.into_response()})
                    }
                    (Err(e),_,_,_,_,_) |
                    (_,Err(e),_,_,_,_) |
                    (_,_,Err(e),_,_,_) |
                    (_,_,_,Err(e),_,_) |
                    (_,_,_,_,Err(e),_) |
                    (_,_,_,_,_,Err(e)) => __error__(e),
                }
            })
        }
    }
};


#[cfg(test)] #[test] fn handler_args() {
    async fn h0() -> &'static str {""}

    async fn h1(_param: String) -> Response {todo!()}
    async fn h2(_param: &str) -> Response {todo!()}

    struct P;
    impl<'p> FromParam<'p> for P {
        type Error = std::convert::Infallible;
        fn from_param(_param: std::borrow::Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(Self)
        }
    }
    async fn h3(_param: P) -> String {format!("")}

    #[cfg(feature="rt_worker")]
    struct SomeJS {_ptr: *const u8}
    #[cfg(feature="rt_worker")]
    impl<'req> FromRequest<'req> for SomeJS {
        type Error = std::convert::Infallible;
        fn from_request(_: &'req Request) -> Option<Result<Self, Self::Error>> {
            None
        }
    }
    #[cfg(feature="rt_worker")]
    async fn h4(_: SomeJS) -> String {format!("")}

    macro_rules! assert_handlers {
        ( $($function:ident)* ) => {
            $( let _ = IntoHandler::into_handler($function); )*
        };
    }

    assert_handlers! { h0 h1 h2 h3  }

    #[cfg(feature="rt_worker")]
    assert_handlers! { h4 }
}
