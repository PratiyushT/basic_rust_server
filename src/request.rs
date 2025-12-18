use crate::BASE_DIR;

use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;
use std::path::PathBuf;

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
/// - `InvalidURL`: The URL is not valid.
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
    #[error("The url is not a valid url")]
    InvalidURL,
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

        /* If request is not empty, parse into string. */
        let request = match request_buf.lines().next() {
            Some(value) => value?,
            None => return Err(RequestError::EmptyRequest),
        };

        /* Split the request in to a Vec<String> at whitespaces. */
        let data: Vec<&str> = request
            .split_ascii_whitespace()
            .map(|value| value.trim())
            .collect();

        /* Validate Request */
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

    /// Resolves the request URL into a filesystem path under [`BASE_DIR`] and returns it only if it is safe and exists.
    ///
    /// Routing rules
    /// - Routes are folder-based (directory routing).
    /// - Every incoming path is treated as a route, not a direct file reference.
    /// - The server always serves the `index.html` file inside that route directory.
    ///   Examples:
    ///   - `/` -> `index.html`
    ///   - `/docs` -> `docs/index.html`
    ///   - `/docs/` -> `docs/index.html`
    ///   - `/about/value/something/` -> `about/value/something/index.html`
    /// - Query strings (`?`) and fragments (`#`) are ignored for routing.
    ///   Example: `/docs?x=1#top` routes the same as `/docs`.
    ///
    /// Security model
    /// - The base directory is canonicalized first (absolute path, resolves symlinks).
    /// - The candidate file path is then built under the canonical base directory and canonicalized.
    /// - The canonical candidate must start with the canonical base (`starts_with`).
    ///   This blocks both `..` directory traversal and symlink-based escapes.
    ///
    /// Returns
    /// - `Some(PathBuf)` if the resolved file exists, is a regular file, and remains inside [`BASE_DIR`].
    /// - `None` if the file does not exist or the resolved path is unsafe (outside the base directory).

    pub fn path_exists(&self) -> Option<PathBuf> {
        println!("The request is: {}", &self);
        let base_dir_relative = if BASE_DIR.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(BASE_DIR)
        };

        /* Canonicalize and verify the relative path given by user in the program. */
        let base_dir_canonical = match base_dir_relative.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("BASE_DIR cannot be canonicalized: {error}");
                return None;
            }
        };

        /* Join and canonicalize the normalized url with the base dir */
        /* This helps prevent both .. traversal and symlink escapes*/
        let normalized_relative = Self::normalize_path_string(self.url.as_str())?;
        let path_canonical = base_dir_canonical
            .join(normalized_relative)
            .canonicalize()
            .ok()?;

        /* Check if the new path stays inside base directory.*/
        if !path_canonical.starts_with(base_dir_canonical) {
            return None;
        }

        /* Check if the path is a file or not. */
        if path_canonical.is_file() {
            Some(path_canonical)
        } else {
            None
        }
    }

    /// This function is a private helper function for [`Self::path_exists`].
    /// - It normalizes the string by removing anything after `?` or `#`.
    /// - Leading or trailing "/" are ignored.
    fn normalize_path_string(raw: &str) -> Option<PathBuf> {
        let query_stripped = raw.splitn(2, '?').next().unwrap();
        let fragment_stripped = query_stripped.splitn(2, '#').next().unwrap();

        /* Get the OS specific path from the string */
        let mut normalized_path_string = PathBuf::new();
        for element in fragment_stripped.split('/') {
            if element.is_empty() {
                continue;
            }
            normalized_path_string.push(element)
        }
        normalized_path_string.push("index");
        normalized_path_string.set_extension("html");

        Some(normalized_path_string)
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
