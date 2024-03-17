use serde::{Serialize, Deserialize};
use ohkami_lib::serde_utf8;
use crate::typed::{Payload, PayloadType};


pub struct Text;
impl PayloadType for Text {
    const CONTENT_TYPE: &'static str = "text/plain";

    type Error = serde_utf8::Error;

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, Self::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }
}

/// This doesn't check the text is valid HTML.
pub struct HTML;
impl PayloadType for HTML {
    const CONTENT_TYPE: &'static str = "text/html";

    type Error = serde_utf8::Error;

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, Self::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }
}

const _: (/* builtin impls */) = {
    use std::borrow::Cow;

    macro_rules! impl_text_payload_for {
        ($( $t:ty )*) => {
            $(
                impl Payload for $t {
                    type Type = Text;
                }
            )*
        };
    }
    
    impl_text_payload_for! {
        &str
        Option<&str>
        String
        Option<String>
        Cow<'_, str>
        Option<Cow<'_, str>>
    }
};
