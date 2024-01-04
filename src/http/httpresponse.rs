use std::collections::HashMap;

use crate::HttpStatusCode;
use crate::http::HttpVersion;

pub struct HttpResponse {
    pub head: Parts,
    pub body: String,
}

impl HttpResponse {
    #[inline]
    pub fn new() -> HttpResponse {
        HttpResponse {
            head: Parts::new(),
            body: String::new(),
        }
    }
}

pub struct Parts {
    pub status: HttpStatusCode,
    pub version: HttpVersion,
    pub headers: HashMap<String, String>,
}

impl Parts {
    pub fn new() -> Parts {
        Parts {
            status: HttpStatusCode::Ok,
            version: HttpVersion::Http11,
            headers: HashMap::new(),
        }
    }
}