use crate::FromRequest;


/// # Memory of a Request
/// 
/// Memorize and retrieve any data within a request.
/// 
/// <br>
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() {
///     let sample_data = Arc::new(String::from("ohkami"));
/// 
///     Ohkami::with(
///         Memory::new(sample_data), // <--
///         (
///             "/hello".GET(hello),
///         )
///     ).howl("0.0.0.0:8080").await
/// }
/// 
/// async fn hello(
///     Memory(name): Memory<'_, String>,
/// ) -> String {
///     format!("Hello, {name}!")
/// }
/// ```
pub struct Memory<'req, Data: Send + Sync + 'static>(pub &'req Data);

impl<'req, Data: Send + Sync + 'static>
FromRequest<'req> for Memory<'req, Data> {
    type Error = std::convert::Infallible;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        match req.memory::<Data>().map(Memory) {
            Some(d) => Some(Ok(d)),
            None => {
                #[cfg(debug_assertions)] {
                    crate::warning!(
                        "`Memory` of type `{}` was not found",
                        std::any::type_name::<Data>()
                    )
                }
                None
            }
        }
    }
}

const _: () = {
    use crate::fang::{Fang, FangProc};

    impl<'req, Data: Clone + Send + Sync + 'static> Memory<'req, Data> {
        #[allow(private_interfaces)]
        pub fn new(data: Data) -> UseMemory<Data> {
            UseMemory(data)
        }
    }

    pub struct UseMemory<Data: Clone + Send + Sync + 'static>(
        Data
    );
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    Fang<Inner> for UseMemory<Data> {
        type Proc = UseMemoryProc<Data, Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            UseMemoryProc { data: self.0.clone(), inner }
        }
    }

    pub struct UseMemoryProc<
        Data:  Clone + Send + Sync + 'static,
        Inner: FangProc,
    > {
        data:  Data,
        inner: Inner,
    }
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    FangProc for UseMemoryProc<Data, Inner> {
        #[cfg(not(feature="rt_worker"))]
        #[inline]
        fn bite<'b>(&'b self, ctx: crate::Context<'b>, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> + Send {
            req.memorize(self.data.clone());
            self.inner.bite(ctx, req)
        }
        #[cfg(feature="rt_worker")]
        #[inline]
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> {
            req.memorize(self.data.clone());
            self.inner.bite(req)
        }
    }
};
