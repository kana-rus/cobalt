use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hasher, BuildHasherDefault},
};


pub struct Store(
    Option<Box<
        HashMap<
            TypeId,
            Box<dyn Any + Send + Sync>,
            BuildHasherDefault<TypeIDHasger>,
        >
    >>
);
#[derive(Default)]
struct TypeIDHasger(u64);
impl Hasher for TypeIDHasger {
    fn write(&mut self, _: &[u8]) {
        unsafe {std::hint::unreachable_unchecked()}
    }

    #[inline] fn write_u64(&mut self, type_id_value: u64) {
        self.0 = type_id_value
    }
    #[inline] fn finish(&self) -> u64 {
        self.0
    }
}

pub struct Memory<'req, Value: Send + Sync + 'static>(&'req Value);
impl<'req, Value: Send + Sync + 'static> super::FromRequest<'req> for Memory<'req, Value> {
    type Error = &'static str;
    #[inline] fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
        req.memorized::<Value>()
            .map(Memory)
            .ok_or_else(|| "Something went wrong")
    }
}
impl<'req, Value: Send + Sync + 'static> std::ops::Deref for Memory<'req, Value> {
    type Target = &'req Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl Store {
    pub(super) fn new() -> Self {
        Self(None)
    }

    pub fn insert<Value: Send + Sync + 'static>(&mut self, value: Value) {
        self.0.get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<Value>(), Box::new(value));
    }

    pub fn get<Value: Send + Sync + 'static>(&self) -> Option<&Value> {
        self.0.as_ref()
            .and_then(|map|   map.get(&TypeId::of::<Value>()))
            .and_then(|boxed| boxed.downcast_ref())
    }
}