mod parse_payload; pub use parse_payload::*;
mod from_request;  pub use from_request::*;
#[cfg(test)] mod _test_parse_payload;
#[cfg(test)] mod _test_parse;

use std::{pin::Pin};
use byte_reader::{Reader};
use percent_encoding::{percent_decode};
use crate::{
    __rt__::{AsyncReader},
    layer0_lib::{List, Method, ContentType, Slice, CowSlice}
};

pub(crate) const METADATA_SIZE: usize = 1024;
pub(crate) const PAYLOAD_LIMIT: usize = 2 << 32;

pub(crate) const QUERIES_LIMIT: usize = 4;
pub(crate) const HEADERS_LIMIT: usize = 32;


pub struct Request {pub(crate) _metadata: [u8; METADATA_SIZE],
    method:  Method,
    path:    Slice,
    queries: List<(Slice, Slice), QUERIES_LIMIT>,
    headers: List<(Slice, Slice), HEADERS_LIMIT>,
    payload: Option<(ContentType, CowSlice)>,
}

impl Request {
    pub(crate) fn init() -> Self {
        Self {_metadata: [0; METADATA_SIZE],
            method:  Method::GET,
            path:    Slice::null(),
            queries: List::<(Slice, Slice), QUERIES_LIMIT>::new(),
            headers: List::<(Slice, Slice), HEADERS_LIMIT>::new(),
            payload: None,
        }
    }

    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncReader + Unpin),
    ) {
        stream.read(&mut self._metadata).await.unwrap();
        let mut r = Reader::new(&self._metadata);

        let method = Method::from_bytes(r.read_while(|b| b != &b' '));
        r.consume(" ").unwrap();
        
       let path = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'?' && b != &b' '))};

        let mut queries = List::<(Slice, Slice), QUERIES_LIMIT>::new();
        if r.consume_oneof([" ", "?"]).unwrap() == 1 {
            while r.peek().is_some() {
                let key = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'='))};
                r.consume("=").unwrap();
                let val = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'&' && b != &b' '))};

                queries.append((key, val));
                if r.consume_oneof(["&", " "]).unwrap() == 1 {break}
            }
        }

        r.consume("HTTP/1.1\r\n").expect("Ohkami can only handle HTTP/1.1");

        let mut headers = List::<(Slice, Slice), HEADERS_LIMIT>::new();
        let (mut content_type, mut content_length) = (None, 0);
        while r.consume("\r\n").is_none() {
            let _key = r.read_while(|b| b != &b':');
            let _content_flag = if _key.eq_ignore_ascii_case(b"Content-Type") {
                Some(true)
            } else if _key.eq_ignore_ascii_case(b"Content-Length") {
                Some(false)
            } else {None};
            let key = unsafe {Slice::from_bytes(_key)};

            r.consume(": ").unwrap();

            let _val = r.read_while(|b| b != &b'\r');
            match _content_flag {None => (),
                Some(true)  => (|| content_type   = ContentType::from_bytes(_val))(),
                Some(false) => (|| content_length = _val.into_iter().fold(0, |len, d| 10*len + (*d-b'0') as usize))(),
            }
            let val = unsafe {Slice::from_bytes(_val)};
            r.consume("\r\n").unwrap();

            headers.append((key, val));
        }

        let payload = match (content_length > 0).then(|| async {(
            content_type.unwrap_or(ContentType::Text),
            Request::read_payload(stream, &self._metadata, r.index, content_length.min(PAYLOAD_LIMIT)).await
        )}) {
            None    => None,
            Some(f) => Some(f.await),
        };

        self.method  = method;
        self.path    = path;
        self.queries = queries;
        self.headers = headers;
        self.payload = payload;
    }

    async fn read_payload(
        stream:       &mut (impl AsyncReader + Unpin),
        ref_metadata: &[u8],
        starts_at:    usize,
        size:         usize,
    ) -> CowSlice {#[cfg(debug_assertions)] assert!(starts_at < METADATA_SIZE, "ohkami can't handle requests if the total size of status and headers exceeds {METADATA_SIZE} bytes");
        if ref_metadata[starts_at] == 0 {// Some HTTP client requests as this
            let mut bytes = vec![0; size];
            stream.read(&mut bytes).await.unwrap();
            CowSlice::Own(bytes)
        } else if starts_at + size <= METADATA_SIZE {
            CowSlice::Ref(unsafe {Slice::new(ref_metadata.as_ptr().add(starts_at), size)})
        } else {
            (|| async move {
                let mut bytes = vec![0; size];
                let size_of_payload_in_metadata_bytes = METADATA_SIZE - starts_at;
                
                bytes[..size_of_payload_in_metadata_bytes].copy_from_slice(&ref_metadata[starts_at..]);
                stream.read(bytes[size_of_payload_in_metadata_bytes..].as_mut()).await.unwrap();
                
                CowSlice::Own(bytes)
            })().await
        }
    }
}

impl Request {
    #[inline] pub fn method(&self) -> Method {
        self.method
    }
    #[inline] pub fn path(&self) -> &str {
        unsafe {std::mem::transmute(
            &*(percent_decode(self.path_bytes()).decode_utf8_lossy())
        )}
    }
    #[inline] pub fn query<Value: FromParam>(&self, key: &str) -> Option<Result<Value, Value::Error>> {
        for (k, v) in self.queries.iter() {
            if key.eq_ignore_ascii_case(&percent_decode(unsafe {k.as_bytes()}).decode_utf8_lossy()) {
                return (|| Some(Value::from_param(&percent_decode(unsafe {v.as_bytes()}).decode_utf8_lossy())))()
            }
        }
        None
    }
    #[inline] pub fn header(&self, key: &str) -> Option<&str> {
        for (k, v) in self.headers.iter() {
            if key.as_bytes().eq_ignore_ascii_case(unsafe {k.as_bytes()}) {
                return (|| Some(unsafe {std::str::from_utf8_unchecked(v.as_bytes())}))()
            }
        }
        None
    }
    #[inline] pub fn payload(&self) -> Option<(&ContentType, &[u8])> {
        let (content_type, body) = (&self.payload).as_ref()?;
        Some((content_type, unsafe {body.as_bytes()}))
    }
}

impl Request {
    #[inline(always)] pub(crate) unsafe fn path_bytes<'b>(&self) -> &'b [u8] {
        self.path.as_bytes()
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let queires = {
                let List { list, next } = &self.queries;
                list[..*next].into_iter()
                    .map(|cell| {
                        let (k, v) = unsafe {cell.assume_init_ref()};
                        format!("{} = {}",
                            percent_decode(unsafe {k.as_bytes()}).decode_utf8_lossy(),
                            percent_decode(unsafe {v.as_bytes()}).decode_utf8_lossy(),
                        )
                    })
            }.collect::<Vec<_>>();

            let headers = {
                let List { list, next } = &self.headers;
                list[..*next].into_iter()
                    .map(|cell| unsafe {
                        let (k, v) = cell.assume_init_ref();
                        format!("{}: {}",
                            std::str::from_utf8_unchecked(k.as_bytes()),
                            std::str::from_utf8_unchecked(v.as_bytes()),
                        )
                    })
            }.collect::<Vec<_>>();

            if let Some((_, payload)) = self.payload() {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queires)
                    .field("headers", &headers)
                    .field("payload", &String::from_utf8_lossy(payload))
                    .finish()
            } else {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queires)
                    .field("headers", &headers)
                    .finish()
            }
        }
    }
};

#[cfg(test)] const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Self) -> bool {
            fn collect<const CAP: usize>(list: &List<(Slice, Slice), CAP>) -> Vec<(&str, &str)> {
                let mut list = list.iter()
                    .map(|(k, v)| unsafe {(
                        std::str::from_utf8(k.as_bytes()).unwrap(),
                        std::str::from_utf8(v.as_bytes()).unwrap(),
                    )})
                    .collect::<Vec<_>>();
                list.sort_by(|(a, _), (b, _)| (a.to_ascii_lowercase()).cmp(&b.to_ascii_lowercase()));
                list
            }

            let eq_ignore_key_case = |left: Vec<(&str, &str)>, right: Vec<(&str, &str)>| {
                left.len() == right.len() &&
                left.iter().zip(right)
                    .all(|((k1, v1), (k2, v2))| k1.eq_ignore_ascii_case(k2) && v1 == &v2)
            };

            self.method == other.method &&
            unsafe {self.path.as_bytes() == other.path.as_bytes()} &&
            collect(&self.queries) == collect(&other.queries) &&
            eq_ignore_key_case(collect(&self.headers), collect(&other.headers)) &&
            self.payload == other.payload
        }
    }
};
