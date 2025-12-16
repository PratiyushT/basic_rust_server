use std::fmt::format;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// Reads an HTTP request from a TCP connection and returns the request
/// line plus header lines as a `Vec<String>`.
///
/// What this reads:
/// - The request line (example: `GET / HTTP/1.1`)
/// - All header lines (example: `Host: ...`, `User-Agent: ...`)
///
/// Where it stops:
/// - Stops reading when it encounters the first empty line (`""`), which in HTTP
///   marks the end of the header section.
///
/// What this does NOT read:
/// - It does not read the request body (if any). For POST/PUT requests, the body
///   would still be unread in the stream.
///
/// Notes:
/// - This uses `BufReader` so reading line-by-line is efficient on a TCP stream.
/// - This implementation uses `unwrap()`, so an I/O error (client disconnect, etc.)
///   will panic. Production code should return `io::Result<Vec<String>>` instead.
/// # Arguments
/// - `stream`: A reference to the `TcpStream` representing the client connection.
///
/// # Returns
/// - A `Vec<String>` containing the request line followed by each HTTP header line,
///   in the order received. The terminating blank line is not included, and no body
///   bytes are included.
pub fn get_request(stream: &TcpStream) -> Vec<String> {
    let request_buf = BufReader::new(stream);

    let request = request_buf
        .lines()
        .map(|request_line| request_line.unwrap())
        .take_while(|request| !request.is_empty())
        .collect::<Vec<String>>();

    request
}

/// Sends a basic HTTP `200 OK` response for a file by writing the status line,
/// a `Content-Length` header, a blank line, and then the file contents as the body.
///
/// What this does:
/// 1. Opens the file specified by `file_name`.
/// 2. Uses file metadata to compute the body size in bytes and writes it as
///    the `Content-Length` header.
/// 3. Reads the entire file into a `String` (UTF-8 text) and appends it as the body.
/// 4. Writes the full HTTP response to the TCP stream.
///
/// Important details / limitations:
/// - This assumes the file is valid UTF-8 text because it uses `read_to_string`.
///   It will fail for binary files (png, jpg, pdf, etc.) or non-UTF-8 content.
/// - This loads the entire file into memory before sending it.
/// - No `Content-Type` header is sent, so the browser must guess how to interpret
///   the body.
/// - Always returns `200 OK` even if the file is not the right resource for the
///   request; routing and error statuses (404/500/etc.) not handled here.
///
/// # Arguments
/// - `stream`: Mutable reference to the client `TcpStream` to write the response to.
/// - `file_name`: Path to the file to serve as the HTTP response body.
///
/// # Returns
/// - `Ok(())` if the response was fully written.
/// - `Err(io::Error)` if the file cannot be opened/read or the stream cannot be written to.
pub fn send_response_header(stream: &mut TcpStream, file_name: String) -> io::Result<()> {
    let response_status = String::from("HTTP/1.1 200 OK");

    let file = File::open(file_name)?;
    let content_length = format!("Content-Length: {}", file.metadata()?.len());
    let content_type = String::from("text/html");

    let mut content = String::new();
    let mut file_buf = BufReader::new(file);
    file_buf.read_to_string(&mut content)?;

    let response = format!("{response_status}\r\n{content_length}\r\n{content_type}\r\n\r\n{content}");
    stream.write_all(response.as_bytes())?;
    Ok(())
}
