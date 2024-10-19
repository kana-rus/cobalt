use super::FangProcCaller;
use std::future::Future;
use whttp::Response;


pub(super) use dispatch::*;

#[cfg(not(feature="rt_worker"))]
mod dispatch {
    pub trait SendSyncOnNative: Send + Sync {}
    impl<T: Send + Sync> SendSyncOnNative for T {}

    #[allow(unused)]
    pub trait SendOnNative: Send {}
    impl<T: Send> SendOnNative for T {}
}
#[cfg(feature="rt_worker")]
mod dispatch {
    pub trait SendSyncOnNative {}
    impl<T> SendSyncOnNative for T {}

    pub trait SendOnNative {}
    impl<T> SendOnNative for T {}
}

#[allow(unused)]
pub(crate) trait ResponseFuture: Future<Output = Response> + SendOnNative {}
impl<R: Future<Output = Response> + SendOnNative> ResponseFuture for R {}

pub(crate) trait FPCBound: FangProcCaller + SendSyncOnNative {}
impl<T: FangProcCaller + SendSyncOnNative> FPCBound for T {}
