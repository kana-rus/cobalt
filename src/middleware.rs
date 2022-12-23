use std::{pin::Pin, future::Future};
use crate::context::Context;

type MiddlewareFunc = Box<dyn Fn(&mut Context) -> Pin<Box<dyn Future<Output=()>>>>;

trait MiddlewareArg {}
trait MiddlewareProcess<Arg: MiddlewareArg> {
    fn into_middleware_func(self) -> MiddlewareFunc;
}
impl MiddlewareArg for () {}
impl<F, Fut> MiddlewareProcess<()> for F
where
    F:   Fn() -> Fut + 'static,
    Fut: Future<Output=()> + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |_| Box::pin(self()))
    }
}
impl MiddlewareArg for (&Context,) {}
impl<F, Fut> MiddlewareProcess<(&Context,)> for F
where
    F:   Fn(&Context) -> Fut + 'static,
    Fut: Future<Output=()> + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |ctx| Box::pin(self(ctx)))
    }
}
impl MiddlewareArg for &Context {}
impl<F, Fut> MiddlewareProcess<&Context> for F
where
    F:   Fn(&mut Context) -> Fut + 'static,
    Fut: Future<Output=()> + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |ctx| Box::pin(self(ctx)))
    }
}

pub struct Middleware(
    Vec<(/*route*/&'static str, MiddlewareFunc)>
); impl Middleware {
    pub fn init() -> Self {
        Self(Vec::new())
    }
}
