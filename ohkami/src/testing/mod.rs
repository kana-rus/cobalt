//! Ohkami testing tools
//! 
//! <br>
//! 
//! *test_example.rs*
//! ```
//! use ohkami::prelude::*;
//! use ohkami::testing::*;
//! 
//! fn my_ohkami() -> Ohkami {
//!     Ohkami::new(
//!         "/".GET(|| async {
//!             "Hello, ohkami!"
//!         })
//!     )
//! }
//! 
//! #[cfg(test)]
//! #[tokio::test]
//! async fn test_my_ohkami() {
//!     let t = my_ohkami().test();
//! 
//!     let req = TestRequest::GET("/");
//!     let res = t.oneshot(req).await;
//!     assert_eq!(res.status(), Status::OK);
//!     assert_eq!(res.text(), Some("Hello, ohkami!"));
//! }
//! ```

pub use whttp::{Status};
use whttp::{Request, Response};
use whttp::header::ContentType;

use crate::Ohkami;
use crate::router::RadixRouter;

use std::sync::Arc;


pub trait Testing {
    fn test(self) -> TestingOhkami;
}
impl Testing for Ohkami {
    fn test(self) -> TestingOhkami {
        TestingOhkami(Arc::new(self.into_router().into_radix()))
    }
}

pub struct TestingOhkami(Arc<RadixRouter>);
impl TestingOhkami {
    #[must_use]
    pub async fn oneshot(&self, mut req: Request) -> TestResponse {
        let router = self.0.clone();
        TestResponse(router.handle(crate::util::IP_0000, &mut req).await)
    }
}

pub struct TestResponse(Response);
impl std::ops::Deref for TestResponse {
    type Target = Response;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl TestResponse {
    pub fn text(&self) -> Option<&str> {
        self.header(ContentType)?.starts_with("text/plain").then_some(
            std::str::from_utf8(self.payload()?)
            .expect("can't parse as UTF-8 text")
        )
    }
    pub fn html(&self) -> Option<&str> {
        self.header(ContentType)?.starts_with("text/html").then_some(
            std::str::from_utf8(self.payload()?)
            .expect("can't parse as UTF-8 text")
        )
    }
    pub fn json<'t, T: serde::Deserialize<'t>>(&'t self) -> Option<T> {
        self.header(ContentType)?.starts_with("application/json").then_some(
            ::serde_json::from_slice(self.payload()?)
            .expect(&format!("can't parse as `{}`", std::any::type_name::<T>()))
        )
    }
}
