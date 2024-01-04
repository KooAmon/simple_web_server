use std::fmt::{Display, Formatter, Result};

#[allow(dead_code)]
pub enum HttpVersion {
    Http09,
    Http10,
    Http11,
    H2,
    H3,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Http09 => write!(f, "HTTP/0.9"),
            Self::Http10 => write!(f, "HTTP/1.0"),
            Self::Http11 => write!(f, "HTTP/1.1"),
            Self::H2 => write!(f, "HTTP/2.0"),
            Self::H3 => write!(f, "HTTP/3.0"),
        }
    }
}