use std::{collections::HashMap, str::FromStr};
use crate::HttpMethod;

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl FromStr for HttpRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let mut lines = s.lines();
        if s.is_empty() {
            return Err("Empty request".to_string());
        }
        let first_line = lines.next().unwrap();
        let (method, path, version) = {
            let mut parts = first_line.split_whitespace();
            (
                parts.next().unwrap().to_string().parse::<HttpMethod>().unwrap(),
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
            )
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();

        for line in lines {
            if let Some((header, value)) = line.split_once(':') {
                headers.insert(
                    header.trim().to_lowercase().to_string(),
                    value.trim().to_string(),
                );
            } else if matches!(method, HttpMethod::POST) && !line.is_empty() {
                body.push_str(line);
            }
        }

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}