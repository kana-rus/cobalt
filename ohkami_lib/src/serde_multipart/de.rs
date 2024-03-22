use serde::de::IntoDeserializer;

use super::Error;
use super::parse::{Multipart, Part};


pub(crate) struct MultipartDesrializer<'de> {
    parsed: Multipart<'de>
}

impl<'de> MultipartDesrializer<'de> {
    pub(crate) fn new(input: &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            parsed: Multipart::parse(input)?
        })
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut MultipartDesrializer<'de> {
    type Error = Error;

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_map(visitor)
    }
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_map(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool str string char
        unit unit_struct
        tuple tuple_struct
        bytes byte_buf
        option enum seq identifier
        ignored_any
        i8 i16 i32 i64
        u8 u16 u32 u64
        f32 f64
    } fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        self.deserialize_map(visitor)
    }
}

impl<'de> serde::de::MapAccess<'de> for &mut MultipartDesrializer<'de> {
    type Error = Error;

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
        V: serde::de::DeserializeSeed<'de>,
    {
        let next_parts = self.parsed.next();
        if next_parts.is_empty() {
            return Ok(None)
        }

        Ok(Some((
            kseed.deserialize(unsafe {next_parts.first().unwrap_unchecked()}.name.into_deserializer())?,
            vseed.deserialize(content.into_deserializer())?
        )))
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: serde::de::DeserializeSeed<'de> {
        let Some(Part { name, .. }) = self.parsed.peek() else {
            return Ok(None)
        };
        seed.deserialize(name.into_deserializer()).map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: serde::de::DeserializeSeed<'de> {
        let Part { content, .. } = self.parsed.next().unwrap();
        seed.deserialize(content.into_deserializer())
    }
}
