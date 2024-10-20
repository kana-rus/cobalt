#![allow(non_snake_case)]
#![cfg(feature="testing")]
#![cfg(any(feature="rt_tokio",feature="rt_async-std"))] // for `#[__rt__::test]`

use crate::__rt__;
use crate::prelude::*;
use crate::testing::*;


fn my_ohkami() -> Ohkami {
    let health_ohkami = Ohkami::new((
        "/".GET(|| async {"health_check"}),
    ));

    let profiles_ohkami = Ohkami::new((
        "/:username"
            .GET(|username: String| async  move {
                format!("get_profile of user `{username}`")
            }),
        "/:username/follow"
            .POST(|username: String| async move {
                format!("follow_user `{username}`")
            })
            .DELETE(|username: String| async move {
                format!("unfollow_user `{username}`")
            })
    ));

    let articles_ohkami = Ohkami::new((
        "/"
            .GET(|| async {"get_article"})
            .POST(|| async {"post_article"}),
        "/feed"
            .GET(|| async {"get_feed"}),
        "/:slug".By(Ohkami::new((
            "/"
                .GET(|slug: String| async move {
                    format!("get_article {slug}")
                })
                .PUT(|slug: String| async move {
                    format!("put_article {slug}")
                })
                .DELETE(|slug: String| async move {
                    format!("delete_article {slug}")
                }),
            "/comments"
                .POST(|slug: String| async move {
                    format!("post_comments {slug}")
                })
                .GET(|slug: String| async move {
                    format!("get_comments {slug}")
                }),
            "/comments/:id"
                .DELETE(|(slug, id): (String, usize)| async move {
                    format!("delete_comment {slug} / {id}")
                }),
            "/favorite"
                .POST(|slug: String| async move {
                    format!("favorite_article {slug}")
                })
                .DELETE(|slug: String| async move {
                    format!("unfavorite_article {slug}")
                }),
        )))
    ));

    Ohkami::new((
        "/health".By(health_ohkami),
        "/api".By(Ohkami::new((
            "/profiles".By(profiles_ohkami),
            "/articles".By(articles_ohkami),
        ))),
    ))
}

#[crate::__rt__::test] async fn test_handler_registration() {
    let t = my_ohkami().test();

    /* GET /health */

    let req = Request::GET("/health");
    let get_res = t.oneshot(req).await;
    assert_eq!(get_res.text(), Some("health_check"));

    let req = Request::HEAD("/health");
    let head_res = t.oneshot(req).await;
    assert_eq!(head_res.text(), None);

    assert_eq!(
        get_res.
        // {let mut h = get_res.headers().filter(|(name, _)| *name != "Content-Length").collect::<Vec<_>>(); h.sort(); h},
        // {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
    );

    /* GET /api/profiles/:username */

    let req = Request::GET("/api/profiles");
    let get_res = t.oneshot(req).await;
    assert_eq!(get_res.status(), Status::NotFound);

    let req = Request::HEAD("/api/profiles");
    let head_res = t.oneshot(req).await;
    assert_eq!(head_res.text(), None);
    assert_eq!(
        {let mut h = get_res.headers().filter(|(name, _)| *name != "Content-Length").collect::<Vec<_>>(); h.sort(); h},
        {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
    );

    let req = Request::GET("/api/profiles/123");
    let get_res = t.oneshot(req).await;
    assert_eq!(get_res.text(), Some("get_profile of user `123`"));

    let req = Request::HEAD("/api/profiles/123");
    let head_res = t.oneshot(req).await;
    assert_eq!(head_res.text(), None);
    assert_eq!(
        {let mut h = get_res.headers().filter(|(name, _)| *name != "Content-Length").collect::<Vec<_>>(); h.sort(); h},
        {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
    );


    /* POST,DELETE /api/profiles/:username/follow */

    let req = Request::GET("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = Request::POST("/api/profiles/the_user");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = Request::POST("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);

    let req = Request::POST("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("follow_user `the_user`"));

    let req = Request::DELETE("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("unfollow_user `the_user`"));

    /* GET /api/articles/feed */

    let req = Request::GET("/api/articles/feed");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("get_feed"));


    /* GET,PUT,DELETE /api/articles/:slug */

    let req = Request::GET("/api/articles/ohkami123456");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("get_article ohkami123456"));

    let req = Request::PUT("/api/articles/abcdef123");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("put_article abcdef123"));


    /* DELETE /api/articles/:slug/comments/:id */

    let req = Request::DELETE("/api/articles/__prototype__/comments/42");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("delete_comment __prototype__ / 42"));
}


#[crate::__rt__::test] async fn test_fang_registration() {
    use std::sync::{OnceLock, Mutex};
    use crate::{Fang, FangProc};

    fn N() -> &'static Mutex<usize> {
        static N: OnceLock<Mutex<usize>> = OnceLock::new();
        N.get_or_init(|| Mutex::new(0))
    }

    struct Increment;
    impl<Inner: FangProc> Fang<Inner> for Increment {
        type Proc = IncrementProc<Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            IncrementProc(inner)
        }
    }
    struct IncrementProc<Inner: FangProc>(Inner);
    impl<Inner: FangProc> FangProc for IncrementProc<Inner> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send {
            #[cfg(feature="DEBUG")]
            println!("Called `Increment`");

            *N().lock().unwrap() += 1;
            self.0.bite(req)
        }
    }

    async fn h() -> &'static str {"h"}


    /*===== with no nests =====*/
    *N().lock().unwrap() = 0;

    let t = Ohkami::with((Increment,), (
        "/a"  .GET(h),
        "/a/b".GET(h),
    )).test();

    let req = Request::GET("/a");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 1);

    let req = Request::GET("/a");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 2);

    let req = Request::GET("/a/b");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 3);

    
    /*===== with nests =====*/
    *N().lock().unwrap() = 0;

    let t = Ohkami::with((Increment,), (
        "/a"  .GET(h),
        "/a/b".GET(h),
        "/a/b/c".By(Ohkami::with((), (
            "/d"  .GET(h),
            "/d/e".GET(h),
        )))
    )).test();

    let req = Request::GET("/a");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 1);

    let req = Request::GET("/a/b");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 2);
    let req = Request::GET("/a/b/c/d");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 3);

    let req = Request::GET("/a/b/c/d/e");
    t.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 4);
}

#[__rt__::test] async fn test_fangs_nesting() {
    use std::sync::{Mutex, OnceLock};
    use crate::{Fang, FangProc, Ohkami};

    #[allow(non_snake_case)]
    fn MESSAGES() -> &'static Mutex<Vec<String>> {
        static MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
        MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
    }

 
    struct HelloFang {
        name: &'static str
    }
    impl<I: FangProc> Fang<I> for HelloFang {
        type Proc = HelloFangProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            HelloFangProc { inner, name: self.name }
        }
    }
    struct HelloFangProc<I: FangProc> {
        name:  &'static str,
        inner: I,
    }
    impl<I: FangProc> FangProc for HelloFangProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            {
                let mut lock = MESSAGES().lock().unwrap();
                lock.push(format!("Hello, {}!", self.name));
            }
            let res = self.inner.bite(req).await;
            {
                let mut lock = MESSAGES().lock().unwrap();
                lock.push(format!("Bye, {}!", self.name));
            }
            res
        }
    }

    async fn h() -> &'static str {"handler"}


    let t = Ohkami::with((
        HelloFang { name: "Amelia" },
    ), (
        "/abc".GET(h),
        "/def".By(Ohkami::with((
            HelloFang { name: "Brooks" },
            HelloFang { name: "Carter" },
        ), (
            "/".GET(h),
            "/jkl".By(Ohkami::with((
                HelloFang { name: "Daniel" },
            ), (
                "/mno".GET(h),
            )))
        ))),
        "/pqr".By(Ohkami::with((
            HelloFang { name: "Evelyn" },
        ), (
            "/stu".GET(h),
        ))),
    )).test();

    {MESSAGES().lock().unwrap().clear();
        let req = Request::GET("/abc");
        let res = t.oneshot(req).await;

        assert_eq!(res.status(), Status::OK);
        assert_eq!(&*MESSAGES().lock().unwrap(), &[
            "Hello, Amelia!",
            "Bye, Amelia!",
        ]);
    }

    {MESSAGES().lock().unwrap().clear();
        let req = Request::GET("/def");
        let res = t.oneshot(req).await;

        assert_eq!(res.status(), Status::OK);
        assert_eq!(&*MESSAGES().lock().unwrap(), &[
            "Hello, Amelia!",
            "Hello, Brooks!",
            "Hello, Carter!",
            "Bye, Carter!",
            "Bye, Brooks!",
            "Bye, Amelia!",
        ]);
    }

    {MESSAGES().lock().unwrap().clear();
        let req = Request::GET("/def/jklmno");
        let res = t.oneshot(req).await;

        assert_eq!(res.status(), Status::NotFound);
        assert_eq!(&*MESSAGES().lock().unwrap(), &[
            "Hello, Amelia!",
            "Hello, Brooks!",
            "Hello, Carter!",
            "Bye, Carter!",
            "Bye, Brooks!",
            "Bye, Amelia!",
        ]);
    }

    {MESSAGES().lock().unwrap().clear();
        let req = Request::GET("/def/jkl/mno");
        let res = t.oneshot(req).await;

        assert_eq!(res.status(), Status::OK);
        assert_eq!(&*MESSAGES().lock().unwrap(), &[
            "Hello, Amelia!",
            "Hello, Brooks!",
            "Hello, Carter!",
            "Hello, Daniel!",
            "Bye, Daniel!",
            "Bye, Carter!",
            "Bye, Brooks!",
            "Bye, Amelia!",
        ]);
    }
}

#[__rt__::test] async fn test_pararell_registering() {
    async fn hello_help() -> &'static str {
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    }

    async fn hello(name: std::borrow::Cow<'_, str>) -> String {
        format!("Hello, {name}!")
    }


    /* register static pattern in ahead */

    let t = Ohkami::new((
        "/hello/help" .GET(hello_help),
        "/hello/:name".GET(hello),
    )).test();

    let req = Request::GET("/hello/help");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some(
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    ));

    let req = Request::GET("/hello/Mr.%20wolf");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some("Hello, Mr. wolf!"));


    /* register param pattern in ahead */

    let t = Ohkami::new((
        "/hello/:name".GET(hello),
        "/hello/help" .GET(hello_help),
    )).test();

    let req = Request::GET("/hello/help");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some(
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    ));

    let req = Request::GET("/hello/Mr.%20wolf");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some("Hello, Mr. wolf!"));
}

#[test]
#[should_panic(expected = "Duplicate routes registration: `/abc`")]
fn duplcate_routes_registration() {
    Ohkami::new((
        "/abc".GET(|| async {"GET"}),
        "/abc".PUT(|| async {"PUT"}),
    ));
}

#[__rt__::test]
async fn prefixy_routes() {
    let t = Ohkami::new((
        "/abcd".GET(|| async {"This is abcd"}),
        "/abc".GET(|| async {"This is abc"}),
    )).test(); {
        let req = Request::GET("/abc");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(), Some("This is abc"));
    } {
        let req = Request::GET("/ab");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    } {
        let req = Request::GET("/abc2");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    } {
        let req = Request::GET("/abcd");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(), Some("This is abcd"));
    } {
        let req = Request::GET("/abcde");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    }

    /* reversed; MUST have the same behavior */
    
    let t = Ohkami::new((
        "/abc".GET(|| async {"This is abc"}),
        "/abcd".GET(|| async {"This is abcd"}),
    )).test(); {
        let req = Request::GET("/abc");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(), Some("This is abc"));
    } {
        let req = Request::GET("/ab");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    } {
        let req = Request::GET("/abc2");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    } {
        let req = Request::GET("/abcd");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(), Some("This is abcd"));
    } {
        let req = Request::GET("/abcde");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    }
}
