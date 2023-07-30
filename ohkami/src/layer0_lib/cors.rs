#![allow(non_snake_case)]

use crate::{Method};


pub(crate) enum AccessControlAllowOrigin {
    Any,
    Only(&'static str),
} impl AccessControlAllowOrigin {
    #[inline(always)] pub(crate) fn from_literal(lit: &'static str) -> Self {
        match lit {
            "*"    => Self::Any,
            origin => Self::Only(origin),
        }
    }

    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Any          => "*",
            Self::Only(origin) => origin,
        }
    }

    #[inline(always)] pub(crate) fn matches(&self, origin: &str) -> bool {
        match self {
            Self::Any     => true,
            Self::Only(o) => *o == origin,
        }
    }
}

pub struct CORS {
    pub(crate) AllowOrigin:      AccessControlAllowOrigin,
    pub(crate) AllowCredentials: bool,
    pub(crate) AllowHeaders:     Option<Vec<&'static str>>,
    pub(crate) AllowMethods:     Option<Vec<Method>>,
    pub(crate) MaxAge:           Option<u32>,
} impl CORS {
    pub(crate) fn new(AllowOrigin: &'static str) -> Self {
        Self {
            AllowOrigin:      AccessControlAllowOrigin::from_literal(AllowOrigin),
            AllowCredentials: false,
            AllowHeaders:     None,
            AllowMethods:     None,
            MaxAge:           None,
        }
    }

    pub(crate) fn to_string(&self) -> String {
        let mut h = format!("Access-Control-Allow-Origin: {}\r\n", self.AllowOrigin.as_str());
        if self.AllowCredentials {
            h.push_str("Access-Control-Allow-Credentials: true\r\n");
        }
        if let Some(seconds) = &self.MaxAge {
            h.push_str(&format!("Access-Control-Max-Age: {seconds}\r\n"));
        }
        if let Some(headers) = &self.AllowHeaders {
            let headers = headers.join(",");
            h.push_str(&format!("Access-Control-Allow-Headers: {headers}\r\n"));
        }
        if let Some(methods) = &self.AllowMethods {
            let methods = methods.into_iter().map(|m| m.to_string()).collect::<Vec<_>>().join(",");
            h.push_str(&format!("Access-Control-Allow-Methods: {methods}\r\n"));
        }
        h
    }
}

impl CORS {
    pub fn AllowCredentials(mut self) -> Self {
        self.AllowCredentials = true;
        self
    }
    pub fn AllowHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.AllowHeaders.replace(headers.to_vec());
        self
    }
    pub fn AllowMethods<const N: usize>(mut self, methods: [crate::Method; N]) -> Self {
        self.AllowMethods.replace(methods.to_vec());
        self
    }
    pub fn MaxAge(mut self, delta_seconds: u32) -> Self {
        self.MaxAge.replace(delta_seconds);
        self
    }
}
