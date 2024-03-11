mod components;

mod serde;
mod query;
mod payload;
mod from_request;


/// The *perfect* reexport of [serde](https://crates.io/crates/serde)'s `Serialize`.
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     bio:  Option<String>,
/// }
/// ```
#[proc_macro_derive(Serialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Serialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Serialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
/// The *perfect* reexport of [serde](https://crates.io/crates/serde)'s `Deserialize`.
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct CreateUser<'req> {
///     #[serde(rename = "username")]
///     name: &'req str,
///     bio:  Option<&'req str>,
/// }
/// ```
#[proc_macro_derive(Deserialize, attributes(serde))] #[allow(non_snake_case)]
pub fn Deserialize(data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde::Deserialize(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn consume_struct(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}


/// ## Query parameters
/// 
/// - Value types：types that impls `FromParam`, or `Option<_>` of them
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// <br/>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::Query; // <--
/// 
/// #[Query]
/// struct HelloQuery<'q> {
///     name:     &'q str,
///     n_repeat: Option<usize>,
/// }
/// 
/// async fn hello(queries: HelloQuery<'_>) -> String {
///     let HelloQuery { name, n_repeat } = queries;
/// 
///     match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     }
/// }
/// ```
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Query(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::Query(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


/// ## Request payload
/// 
/// - NOT available for tuple struct ( like `struct S(usize, usize);` ) or unit struct ( like `struct X;` ).
/// 
/// ### Valid format :
/// 
/// - `#[Payload(JSON)]` ( for `application/json` )
///   - automatically derives `Deserialize` and `Serialize`
/// - `#[Payload(Form)]` ( for `multipart/form-data` )
/// - `#[Payload(URLEncoded)]` ( for `application/x-www-form-urlencoded` )
/// 
/// <br/>
/// 
/// ### JSON
/// 
/// - Requires that the struct implements `serde::Deserialize`
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::Payload; // <--
/// 
/// #[Payload(JSON)]
/// struct HelloRequest<'req> {
///     name:     &'req str,
///     n_repeat: Option<usize>,
/// }
/// /* expected payload examples :
///     {"name":"your name"}
///     {"name":"you_name","n_repeat":2}
/// */
/// 
/// async fn hello(body: HelloRequest<'_>) -> String {
///     let HelloRequest { name, n_repeat } = queries;
/// 
///     match n_repeat {
///         None    => format!("Hello"),
///         Some(n) => format!("Hello, {name}! ").repeat(n),
///     }
/// }
/// ```
/// 
/// #### Filter automatic serde impls
/// 
/// ```ignore
/// use ohkami::serde::Serialize;
/// 
/// #[Payload(JSON/D)]  // <-- only derive `Deserialize`
/// struct User<'req> {
///     name: &'req str,
///     profile: Option<&'req str>,
/// }
/// 
/// impl<'req> Serialze for User<'req> {
///     // my special `Serialize` impl here
/// }
/// ```
/// 
/// <br/>
/// 
/// ### URLEncoded
/// 
/// - Available value types : types that impl `FromParam`, or `Option<_>` of them.
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::Payload; // <--
/// 
/// #[Payload(URLEncoded)]
/// struct HelloRequest<'req> {
///     name:     &'req str,
///     n_repeat: Option<usize>,
/// }
/// /* expected payload examples :
///     name=yourname
///     name=yourname&n_repeat=2
/// */
/// ```
/// 
/// <br/>
/// 
/// ### Form
/// 
/// **NOTE**：This can't handle reference types like `&str` in current version. Wait for the development!
/// 
/// - Available value types : `String`, `File`, `Vec<File>`.
/// - Form part of kebab-case-name is handled by field of snake_case version of the name ( example: `name="submitter-name"` is handled by field `submitter_name` ).
/// 
/// 
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::typed::{Payload, File}; // <--
/// 
/// #[Payload(Form)]
/// struct ProfileData {
///     submitter_name: String,
///     pics:           Vec<File>,
/// }
/// /* expected form :
///     <form action="http://server.dom/cgi/handle" enctype="multiprt/form-data" method="post">
///         What is your name? <input type="text" name="submitter-name" />
///         What files are you sending? <input="file" name="pics" />
///     </form>
/// */ 
/// ```
#[proc_macro_attribute] #[allow(non_snake_case)]
pub fn Payload(format: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    payload::Payload(format.into(), data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


/// # `#[derive(FromRequest)]`
/// 
/// Automatically impl `FromRequest` for a struct composed of
/// `FromRequest` types
/// 
/// <br>
/// 
/// *example.rs*
/// ```ignore
/// use ohkami::FromRequest;
/// use sqlx::PgPool;
/// 
/// #[derive(FromRequest)]
/// struct MyItems1<'req> {
///     db: ohkami::Memory<'req, PgPool>,
/// }
/// 
/// #[derive(FromRequest)]
/// struct MyItems2(
///     MyItems<'req>,
/// );
/// ```
#[proc_macro_derive(FromRequest)]
pub fn derive_from_request(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_request::derive_from_request(target.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
