use crate::BASE_DIR;

use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

/// Represents a parsed HTTP request line.
///
/// This struct stores only the essential components of an HTTP request:
/// the request method, the requested URL path, and the HTTP version.
/// It does not include headers or a request body.
pub struct Request {
    url: String,
    method: String,
    version: String,
}

/// Errors that can occur while parsing or validating an incoming HTTP request.
///
/// # Variants
/// - `EmptyRequest`: The connection contained no request line.
/// - `InvalidLength`: The request line did not contain exactly three parts
///   (`METHOD`, `PATH`, `VERSION`).
/// - `Io`: An underlying I/O error occurred while reading from the stream.
/// - `InvalidHeader`: The request line failed validation (unsupported method/version).
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request was empty.")]
    EmptyRequest,
    #[error("Invalid length in header.")]
    InvalidLength,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("The request is not a valid request")]
    InvalidHeader,
}

impl Request {

    /// Parses the HTTP request line from the given TCP stream and constructs a [`Request`].
    ///
    /// Expects a request line in the form: `GET <path> HTTP/1.1`.
    ///
    /// # Note
    /// - Only the first line of the request is parsed. Everything else is irrelevant.
    ///
    /// # Arguments
    /// - `stream`: Reference to the client [`TcpStream`] to read from.
    ///
    /// # Returns
    /// - `Ok(Request)`: If the request line is present and valid.
    /// - `Err(RequestError::EmptyRequest)`: If the connection contains no request line.
    /// - `Err(RequestError::InvalidLength)`: If the request line does not have exactly three parts.
    /// - `Err(RequestError::InvalidHeader)`: If the method or HTTP version is invalid.
    /// - `Err(RequestError::Io(_))`: If an I/O error occurs while reading from the stream.
    pub fn new(stream: &TcpStream) -> Result<Self, RequestError> {
        let request_buf = BufReader::new(stream);
        let request = match request_buf.lines().next() {
            Some(value) => value?,
            None => return Err(RequestError::EmptyRequest),
        };

        let data: Vec<&str> = request
            .split_ascii_whitespace()
            .map(|value| value.trim())
            .collect();

        if data.len() != 3 {
            return Err(RequestError::InvalidLength);
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

    /// Resolves the current request URL to an HTML file path under [`BASE_DIR`] and checks if it exists.
    ///
    /// Maps `/` to `index.html`, strips the leading `/` from the Request's URL, prepends `BASE_DIR`
    /// if present, and forces the `.html` extension.
    ///
    /// # Note:
    /// - This function currently is not made for nested routes. Might work but is untested.
    /// - Only works for `.html` files.
    ///
    /// # Returns
    /// - `Some(`[`PathBuf`]`)` if the resolved path exists and is a file.
    /// - `None` if the file does not exist. Can be used to show the
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

    /// Formats the request as an HTTP request line: `<METHOD> <URL> <VERSION>`.
    /// # Returns
    /// - `Ok(())` if formatting succeeds, otherwise a formatting error.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.url, self.version)
    }
}
