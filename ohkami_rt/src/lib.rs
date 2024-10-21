#[cfg(feature="tokio")]
pub use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
#[cfg(feature="async-std")]
pub use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
#[cfg(feature="smol")]
pub use smol::net::{TcpListener, TcpStream, AsyncToSocketAddrs as ToSocketAddrs};
#[cfg(feature="glommio")]
pub use {glommio::net::{TcpListener, TcpStream}, std::net::ToSocketAddrs};

pub async fn bind(address: impl ToSocketAddrs) -> TcpListener {
    Result::expect({
        #[cfg(feature="glommio")] {
            TcpListener::bind(address)
        }
        #[cfg(any(feature="tokio",feature="async-std",feature="smol"))] {
            TcpListener::bind(address).await
        }
    }, "failed to bind TCP listener")
}

#[cfg(feature="tokio")]
pub use tokio::task::spawn;
#[cfg(feature="async-std")]
pub use async_std::task::spawn;
#[cfg(feature="smol")]
pub fn spawn<T: Send + 'static>(task: impl std::future::Future<Output = T> + Send + 'static) {
    smol::spawn(task).detach()
}
#[cfg(feature="glommio")]
pub fn spawn<T: 'static>(task: impl std::future::Future<Output = T> + 'static) {
    glommio::spawn_local(task).detach();
}

#[cfg(feature="tokio")]
pub use tokio::time::sleep;
#[cfg(feature="async-std")]
pub use async_std::task::sleep;
#[cfg(feature="smol")]
pub async fn sleep(duration: std::time::Duration) {
    smol::Timer::after(duration).await;
}
#[cfg(feature="glommio")]
pub use glommio::timer::sleep;

#[cfg(feature="tokio")]
pub use tokio::io::AsyncReadExt as Read;
#[cfg(feature="async-std")]
pub use async_std::io::ReadExt as Read;
#[cfg(feature="smol")]
pub use futures_util::AsyncReadExt as Read;
#[cfg(feature="glommio")]
pub use futures_util::AsyncReadExt as Read;

#[cfg(feature="tokio")]
pub use tokio::io::AsyncWriteExt as Write;
#[cfg(feature="async-std")]
pub use async_std::io::WriteExt as Write;
#[cfg(feature="smol")]
pub use futures_util::AsyncWriteExt as Write;
#[cfg(feature="glommio")]
pub use futures_util::AsyncWriteExt as Write;
