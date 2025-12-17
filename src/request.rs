use crate::BASE_DIR;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

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
    /// Creates a [`Request`] by reading and validating the HTTP request line from a TCP stream.
    ///
    /// # What it does
    /// 1. Reads the first line of the incoming TCP stream (the HTTP request line).
    /// 2. Splits that line into 3 whitespace-separated parts: `METHOD`, `URL`, `VERSION`.
    /// 3. Validates that the request is exactly: `GET <path> HTTP/1.1`. (Nested path not supported yet)
    ///
    /// # Arguments
    /// - `stream`: A reference to the client [`TcpStream`]. This function reads from the stream,
    ///   but does not take ownership of it.
    ///
    /// # Returns
    /// - `Ok(Request)`: If the request line exists and passes validation.
    /// - `Err(RequestError::InvalidLength)`: If the request line does not have exactly 3 parts.
    /// - `Err(RequestError::InvalidHeader)`: If the method is not `GET` or the version is not `HTTP/1.1`.
    /// - `Err(RequestError::Io(_))`: If reading from the stream fails (I/O error).

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

    /// Resolves the request URL to an on-disk `.html` file and checks whether it exists.
    ///
    /// # What it does
    /// 1. Takes the request URL (example: `/about` or `/`).
    /// 2. If the URL is `/`, it maps it to `index`.
    /// 3. Prepends `BASE_DIR/` if `BASE_DIR` is not empty.
    /// 4. Appends `.html` to form a filesystem path.
    /// 5. Attempts to open the file to confirm it exists and is readable.
    ///
    /// # Returns
    /// - `Some(String)`: The computed file path (example: `public/about.html`) if the file can be opened.
    /// - `None`: If the file does not exist or cannot be opened. This can be used as a condition to open the 404 not found page.
    ///
    /// # Notes
    /// - This currently uses `File::open` just to check existence. That means you will typically
    ///   open the same file again later when serving it, which is inefficient. A more efficient
    ///   approach is to compute the path here and check `Path::is_file()`, then open only once
    ///   in the response-writing logic.
    /// - This function does not sanitize the URL. Inputs like `/../secret` could potentially escape
    ///   the base directory depending on how `BASE_DIR` is set. Consider blocking `..` and stripping
    ///   query strings (`?x=1`) before building the path.
    pub fn path_exists(&self) -> Option<PathBuf> {
        let url = if self.url == "/" {
            "index"
        } else {
            self.url.as_str()
        };

        let url = url.trim_start_matches("/");
        let mut full_url = if BASE_DIR.is_empty() {
            PathBuf::from(url)
        } else {
            Path::new(BASE_DIR).join(url)
        };
        full_url.set_extension("html");

        if full_url.is_file() {
            Some(full_url)
        } else {
            None
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.url, self.version)
    }
}
