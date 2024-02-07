use crate::{Fang, Method::{self, *}};


/// Represents "can be used as a `Fang`".
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct Log;
/// impl IntoFang for Log {
///     fn into_fang(self) -> Fang {
///         Fang::back(|res: &Response| {
///             println!("{res:?}");
///         })
///     }
/// }
/// ```
pub trait IntoFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];
    fn into_fang(self) -> Fang;
}

pub trait Fangs {
    fn collect(self) -> Vec<(&'static [Method], Fang)>;
} macro_rules! impl_for_tuple {
    ( $( $f:ident ),* ) => {
        impl<$( $f: IntoFang ),*> Fangs for ( $( $f,)* ) {
            #[allow(non_snake_case)]
            fn collect(self) -> Vec<(&'static [Method], Fang)> {
                #[allow(unused_mut)]
                let mut fangs = Vec::new();
                let ( $( $f, )* ) = self;

                $(
                    fangs.push(($f::METHODS, $f.into_fang()));
                )*

                fangs
            }
        }
    };
} const _: () = {
    impl_for_tuple!();
    impl_for_tuple!(F1);
    impl_for_tuple!(F1, F2);
    impl_for_tuple!(F1, F2, F3);
    impl_for_tuple!(F1, F2, F3, F4);
    impl_for_tuple!(F1, F2, F3, F4, F5);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6, F7);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6, F7, F8);
}; impl<F: IntoFang> Fangs for F {
    fn collect(self) -> Vec<(&'static [Method], Fang)> {
        vec![(Self::METHODS, self.into_fang())]
    }
}