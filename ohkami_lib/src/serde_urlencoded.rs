mod ser;
mod de;

#[cfg(test)]
mod _test;


#[inline]
pub fn to_string(value: &impl serde::Serialize) -> Result<String, Error> {
    let mut s = ser::URLEncodedSerializer { output: String::new() };
    value.serialize(&mut s)?;
    Ok(s.output)
}

#[inline(always)]
pub fn from_str<'de, D: serde::Deserialize<'de>>(input: &'de str) -> Result<D, Error> {
    let mut d = de::URLEncodedDeserializer { input };
    let t = D::deserialize(&mut d)?;
    if d.input.is_empty() {
        Ok(t)
    } else {
        Err((||serde::de::Error::custom(format!("Unexpected trailing charactors: {}", d.input)))())
    }
}


#[derive(Debug)]
pub struct Error(String);
const _: () = {
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}

    impl serde::ser::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
    impl serde::de::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
};

pub(crate) enum Infallible {}
const _: () = {
    impl serde::ser::SerializeStructVariant for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(
            &mut self,
            _key: &'static str,
            _value: &T,
        ) -> Result<(), Self::Error>
        where T: serde::Serialize {
            match *self {}
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            match self {}
        }
    }
};
