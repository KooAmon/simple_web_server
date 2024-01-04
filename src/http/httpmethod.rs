use std::{
    str::FromStr,
    fmt::{Display, Formatter}
};

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
            Self::DELETE => write!(f, "DELETE"),
            Self::HEAD => write!(f, "HEAD"),
            Self::OPTIONS => write!(f, "OPTIONS"),
            Self::TRACE => write!(f, "TRACE"),
            Self::CONNECT => write!(f, "CONNECT"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "CONNECT" => Ok(Self::CONNECT),
            _ => Err("Invalid HTTP method".to_string()),
        }
    }
}