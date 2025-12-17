use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use crate::BASE_DIR;

pub struct Request {
    url: String,
    method: String,
    version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Invalid length in header.")]
    InvalidLength,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("The request is not a valid request")]
    InvalidHeader,
}

impl Request {
    pub fn new(stream: &TcpStream) -> Result<Self, RequestError> {
        let request_buf = BufReader::new(stream);
        let request = match request_buf.lines().next() {
            Some(value) => value?,
            None => panic!("Request cannot be empty."),
        };

        let data: Vec<&str> = request
            .split_ascii_whitespace()
            .map(|value| value.trim())
            .collect();

        if data.len() != 3 {
            return (Err(RequestError::InvalidLength));
        }

        if !(data[0] == "GET" && data[2] == "HTTP/1.1") {
            return Err(RequestError::InvalidHeader);
        }

        Ok(Self {
            url: data[1].to_string(),
            method: data[0].to_string(),
            version: data[2].to_string(),
        })
    }

    pub fn path_exists(&self) -> Option<String> {
        let mut url = self.url.clone();
        if self.url == "/" {
            url = "index".to_string();
        };

        let mut full_url = url;
        if !BASE_DIR.is_empty() {
            full_url = format!("{BASE_DIR}/{full_url}")
        }
        full_url.push_str(".html");

        match File::open(&full_url) {
            Ok(_) => Some(full_url),
            Err(_) => None,
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.url, self.version)
    }
}
