#![cfg(feature="openapi")]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Visibility, Ident, LitInt, LitStr, Meta, MetaNameValue, Expr, ExprLit, Lit, token, Token};

pub(super) fn derive_schema(input: TokenStream) -> syn::Result<TokenStream> {
    todo!()
}

pub(super) fn operation(meta: TokenStream, handler: TokenStream) -> syn::Result<TokenStream> {
    #[allow(non_snake_case)]
    struct OperationMeta {
        operationId: Option<String>,
        descriptions: Vec<DescriptionOverride>,
    }

    struct DescriptionOverride {
        key:   DescriptionTarget,
        value: String,
    }
    enum DescriptionTarget {
        Summary,
        RequestBody,
        DefaultResponse,
        Response { status: u16 },
        Param { name: String },
    }

    mod override_keyword {
        syn::custom_keyword!(summary);
        syn::custom_keyword!(requestBody);
    }

    impl syn::parse::Parse for DescriptionOverride {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let key = if false {
            } else if input.peek(override_keyword::summary) {
                input.parse::<override_keyword::summary>()?;
                DescriptionTarget::Summary

            } else if input.peek(override_keyword::requestBody) {
                input.parse::<override_keyword::requestBody>()?;
                DescriptionTarget::RequestBody

            } else if input.peek(Token![default]) {
                input.parse::<Token![default]>()?;
                DescriptionTarget::DefaultResponse

            } else if input.peek(LitInt) {
                let status = input.parse::<LitInt>()?.base10_parse()?;
                DescriptionTarget::Response { status }
                
            } else if input.peek(Ident) {
                let name = input.parse::<Ident>()?.to_string();
                DescriptionTarget::Param { name }

            } else {
                return Err(syn::Error::new(input.span(), format!("\
                    Unepected description key: `{}`. Expected one of\n\
                    - summary       (.summary)\n\
                    - requestBody   (.requestBody.description)\n\
                    - default       (.responses.default.description)\n\
                    - <status:int>  (.responses.<status>.description)\n\
                    - <param:ident> (.parameters.<param>.description)\n\
                ",
                    input.parse2::<TokenStream>()?
                )))
            };

            input.parse::<Token![:]>()?;

            let value = input.parse::<LitStr>()?.value();

            Ok(Self { key, value })
        }
    }

    impl syn::parse::Parse for OperationMeta {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let operationId = input.peek(Ident)
                .then(|| input.parse())
                .transpose()?;

            let descriptions = input.peek(token::Brace)
                .then(|| {
                    let descriptions; syn::braced!(descriptions in input);
                    descriptions
                        .parse_terminated(DescriptionOverride::parse, Token![,])
                        .map(|iter| iter.collect::<Vec<_>>())
                })
                .transpose()?
                .unwrap_or_default();


            Ok(Self { operationId, descriptions })
        }
    }

    //////////////////////////////////////////////////////////////

    let meta = syn::parse2::<OperationMeta>(meta)?;

    let handler = syn::parse2::<ItemFn>(handler)?;
    let handler_vis  = handler.vis;
    let handler_name = handler.ident;

    let description = handler.attrs.into_iter()
        .flat_map(|a| match a.meta {
            Meta::NameValue(MetaNameValue { path,
                eq_token:_,
                value: Expr::Lit(ExprLit { lit: Lit::Str(value), .. })
            }) if path.get_ident().is_some_and(|i| i == "doc")
            => Some(value.value()),
            _ => None
        })
        .fold(String::new(), |mut description, doc| {
            let mut unescaped_doc = String::with_capacity(doc.len());
            {
                let mut chars = doc.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
                        /* do nothing to unescape the next charactor */
                    } else {
                        unescaped_doc.push(ch);
                    }
                }
            }

            description + &unescaped_doc
        });

    let handler = {
        let mut handler = handler.clone();
        handler.vis = Visibility::Public(Token![pub]);
        handler
    };

    let modify_op = {
        let mut modify_op = TokenStream::new();

        modify_op
    };

    Ok(quote! {
        #[allow(non_camelcase_types)]
        #handler_vis struct #handler_name;
        const _: () = {
            mod operation {
                use super::*;
                #handler
            }

            impl ::ohkami::handler::IntoHandler<#handler_name> for #handler_name {
                fn into_handler(self) -> ::ohkami::handler::Handler {
                    ::ohkami::handler::IntoHandler::into_handler(operation::#handler_name)
                        .map_openapi_operation(|op| { #modify_op })
                }
            }
        };
    })
}
