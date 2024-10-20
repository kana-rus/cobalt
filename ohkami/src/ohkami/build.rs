#![allow(non_snake_case, unused_mut)]

use crate::router::{TrieRouter, RouteSections};
use crate::fang::{Handler, IntoHandler};
use crate::Ohkami;


macro_rules! Handlers {
    ($( $method:ident ),*) => {
        pub struct Handlers {
            pub(crate) route: RouteSections,
            $(
                pub(crate) $method: Option<Handler>,
            )*
        }
        
        impl Handlers {
            pub(crate) fn new(route_str: &'static str) -> Self {
                Self {
                    route:   RouteSections::from_literal(route_str),
                    $(
                        $method: None,
                    )*
                }
            }
        }

        impl Handlers {
            $(
                pub fn $method<T>(mut self, handler: impl IntoHandler<T>) -> Self {
                    self.$method.replace(handler.into_handler());
                    self
                }
            )*
        }
    };
} Handlers! { GET, PUT, POST, PATCH, DELETE }

pub struct ByAnother {
    pub(crate) route:  RouteSections,
    pub(crate) ohkami: Ohkami,
}

pub struct Dir {
    pub(crate) route: &'static str,
    pub(crate) files: Vec<(
        Vec<String>,
        std::fs::File,
    )>,

    /*=== config ===*/

    /// File extensions (leading `.` trimmed) that should not be appeared in handling path
    pub(crate) omit_extensions: Option<Box<[&'static str]>>,
} impl Dir {
    fn new(route: &'static str, dir_path: std::path::PathBuf) -> std::io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} is not directory", dir_path.display()))
            )
        }

        let mut files = Vec::new(); {
            fn fetch_entries(
                dir: std::path::PathBuf
            ) -> std::io::Result<Vec<std::path::PathBuf>> {
                dir.read_dir()?
                    .map(|de| de.map(|de| de.path()))
                    .collect()
            }

            let mut entries = fetch_entries(dir_path.clone())?;
            while let Some(entry) = entries.pop() {
                if entry.is_file() {
                    let path_sections = entry.canonicalize()?
                        .components()
                        .skip(dir_path.components().count())
                        .map(|c| c.as_os_str().to_os_string()
                            .into_string()
                            .map_err(|os_string| std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Can't read a path segment `{}`", os_string.as_encoded_bytes().escape_ascii())
                            ))
                        )
                        .collect::<std::io::Result<Vec<_>>>()?;

                    if path_sections.last().unwrap().starts_with('.') {
                        crate::warning!("\
                            =========\n\
                            [WARNING] `Route::Dir`: found `{}` in directory `{}`, \
                            are you sure to serve this file？\n\
                            =========\n",
                            entry.display(),
                            dir_path.display(),
                        )
                    }

                    files.push((
                        path_sections,
                        std::fs::File::open(entry)?
                    ));

                } else if entry.is_dir() {
                    entries.append(&mut fetch_entries(entry)?)

                } else {
                    continue
                }
            }
        }

        Ok(Self {
            route,
            files,

            omit_extensions: None,
        })
    }

    pub fn omit_extensions<const N: usize>(mut self, target_extensions: [&'static str; N]) -> Self {
        self.omit_extensions = Some(Box::new(
            target_extensions.map(|ext| ext.trim_start_matches('.'))
        ));
        self
    }
}


macro_rules! Route {
    ($( $method:ident ),*) => {
        /// Core trait for Ohkami's routing definition.
        /// 
        /// <br>
        /// 
        /// *example.rs*
        /// ```no_run
        /// use ohkami::{Ohkami, Route};
        /// 
        /// async fn index() -> &'static str {
        ///     "ohkami"
        /// }
        /// 
        /// async fn greet() -> &'static str {
        ///     "I'm fine."
        /// }
        /// 
        /// async fn hello() -> String {
        ///     format!("Hello!!!")
        /// }
        /// 
        /// #[tokio::main]
        /// async fn main() {
        ///     Ohkami::new((
        ///         "/"  // <-- `Route` works here...
        ///             .GET(index),
        ///         "/hello"  // <-- `Route` works here...
        ///             .GET(greet)
        ///             .PUT(hello),
        ///     )).howl("localhost:3000").await
        /// }
        /// ```
        pub trait Route {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> Handlers;
            )*

            fn By(self, another: Ohkami) -> ByAnother;

            fn Dir(self, static_files_dir_path: &'static str) -> Dir;
        }

        impl Route for &'static str {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> Handlers {
                    let mut handlers = Handlers::new(self);
                    handlers.$method.replace(handler.into_handler());
                    handlers
                }
            )*

            fn By(self, another: Ohkami) -> ByAnother {
                ByAnother {
                    route:  RouteSections::from_literal(self),
                    ohkami: another,
                }
            }

            fn Dir(self, path: &'static str) -> Dir {
                // Check `self` is valid route
                let _ = RouteSections::from_literal(self);

                match Dir::new(
                    self,
                    path.into()
                ) {
                    Ok(dir) => dir,
                    Err(e) => panic!("{e}")
                }
            }
        }
    };
} Route! { GET, PUT, POST, PATCH, DELETE }


trait RoutingItem {
    fn apply(self, router: &mut TrieRouter);
} const _: () = {
    impl RoutingItem for Handlers {
        fn apply(self, router: &mut TrieRouter) {
            router.register_handlers(self)
        }
    }

    impl RoutingItem for ByAnother {
        fn apply(self, router: &mut TrieRouter) {
            router.merge_another(self)
        }
    }

    impl RoutingItem for Dir {
        fn apply(self, router: &mut TrieRouter) {
            #[derive(Clone)]
            struct StaticFileHandler {
                mime:     &'static str,
                content:  std::sync::Arc<Vec<u8>>,
            }
            const _: () = {
                impl StaticFileHandler {
                    fn new(path_sections: &[String], file: std::fs::File) -> Result<Self, String> {
                        let filename = path_sections.last()
                            .ok_or_else(|| format!("[.Dir] got empty file path"))?;
                        let (_, extension) = filename.rsplit_once('.')
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: In current version, ohkami doesn't support serving files that have no extenstion"))?;
                        let mime = ohkami_lib::mime::get_by_extension(extension)
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: ohkami doesn't know the extension `{extension}`"))?;

                        let mut content = vec![
                            u8::default();
                            file.metadata().unwrap().len() as usize
                        ]; {use std::io::Read;
                            let mut file = file;
                            file.read_exact(&mut content)
                                .map_err(|e| e.to_string())?;
                        }

                        if mime.starts_with("text/")
                        && std::str::from_utf8(&content).is_err() {
                            return Err(format!("[.Dir] got `{filename}`: Ohkami doesn't support non UTF-8 text file"))
                        }

                        Ok(Self { mime, content:std::sync::Arc::new(content) })
                    }
                }
                
                impl IntoHandler<std::fs::File> for StaticFileHandler {
                    fn into_handler(self) -> Handler {
                        let this: &'static StaticFileHandler
                            = Box::leak(Box::new(self));

                        Handler::new(|_, _| Box::pin(async {
                            whttp::Response::OK()
                                .with_payload(this.mime, &*this.content)
                        }))
                    }
                }
            };

            #[cfg(feature="DEBUG")]
            println!{ "[Dir] .files = {:#?}", self.files }

            let mut register = |path: Vec<String>, handler: StaticFileHandler| router.register_handlers(
                Handlers::new(Box::leak({
                    let base_path = self.route.trim_end_matches('/').to_string();
                    match &*path.join("/") {
                        ""   => if !base_path.is_empty() {base_path} else {"/".into()},
                        some => base_path + "/" + some,
                    }
                }.into_boxed_str())).GET(handler)
            );

            for (mut path, file) in self.files {
                let mut handler = match StaticFileHandler::new(&path, file) {
                    Ok(h) => h,
                    Err(msg) => panic!("{msg}")
                };

                if matches!(&**path.last().unwrap(), "index.html") {
                    if !(self.omit_extensions.as_ref().is_some_and(|exts| exts.contains(&"html"))) {
                        register(path.clone(), handler.clone());
                    }

                    path.pop();
                }

                if let Some(exts) = self.omit_extensions.as_ref() {
                    for ext in exts.iter() {
                        if let Some(filename) = path.last().and_then(|p| p.strip_suffix(&format!(".{ext}"))) {
                            let filename_len = filename.len();
                            path.last_mut().unwrap().truncate(filename_len);
                            break
                        }
                    }
                }

                register(path, handler);
            }
        }
    }

    /// This is for better developer experience.
    /// 
    /// If we don't impl `Routes` `&str`, ohkami users
    /// will see following situations：
    /// 
    /// ---
    /// fn my_ohkami() -> Ohkami {
    ///     Ohkami::new((
    ///         "/".|
    /// /*          ↑ cursor */
    ///     ))
    /// }
    /// 
    /// // Here rust-analyzer puts red underlines for all lines of `Ohkami::new(( 〜 ))`
    /// // because the type of argument of `new` is `&str` **AT NOW** and `Routes` trait is
    /// // NOT IMPLEMENTED for this.
    /// // 
    /// // This must be so annoying!!!
    /// ---
    impl RoutingItem for &'static str {
        fn apply(self, _router: &mut TrieRouter) {}
    }
};

pub trait Routes {
    fn apply(self, router: &mut TrieRouter);
}
const _: () = {
    impl Routes for () {
        fn apply(self, _router: &mut TrieRouter) {}
    }
    impl<R: RoutingItem> Routes for R {
        fn apply(self, router: &mut TrieRouter) {
            <R as RoutingItem>::apply(self, router)
        }
    }

    macro_rules! impl_for_tuple {
        ( $( $item:ident ),+ ) => {
            impl<$( $item: RoutingItem ),+> Routes for ( $($item,)+ ) {
                fn apply(self, router: &mut TrieRouter) {
                    let ( $( $item, )+ ) = self;
                    $(
                        <$item as RoutingItem>::apply($item, router);
                    )+
                }
            }
        };
    }
    impl_for_tuple!(R1);
    impl_for_tuple!(R1, R2);
    impl_for_tuple!(R1, R2, R3);
    impl_for_tuple!(R1, R2, R3, R4);
    impl_for_tuple!(R1, R2, R3, R4, R5);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16);
};
