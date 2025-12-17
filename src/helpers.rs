use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use crate::{Request, RequestError};


/// The address that the server will bind to.
pub static ADDRESS: &str = "127.0.0.1:7878";

///The base directory where the HTML files live.
pub static BASE_DIR: &str = "pages";

/// Parses an HTTP request from the given TCP stream and sends an HTTP response.
///
/// If the requested HTML file exists, a `200 OK` response is sent with the file
/// contents. Otherwise, a `404 NOT FOUND` response is sent using `{BASE_DIR}/error404.html`.
///
/// # Arguments
/// - `tcp_stream`: Mutable reference to the client [`TcpStream`].
///
/// # Returns
/// - `Ok(())`: if the request is handled successfully.
/// - `Err(RequestError)`: if request parsing, file access, or response writing fails.
pub fn handle_connection(tcp_stream: &mut TcpStream) -> Result<(), RequestError> {
    let request = Request::new(tcp_stream)?;

    let file: File;
    let mut status = String::from("HTTP/1.1 ");

    if let Some(url) = request.path_exists() {
        file = File::open(url)?;
        status.push_str("200 Ok");
    } else {
        let url = "error404";
        let mut full_url = if BASE_DIR.is_empty() {
            PathBuf::from(url)
        } else {
            Path::new(BASE_DIR).join(url)
        };
        full_url.set_extension("html");
        file = File::open(full_url)?;
        status.push_str("404 NOT FOUND");
    }
    let mut buf_writer = BufWriter::new(tcp_stream);
    write!(buf_writer, "{status}\r\n")?;
    write!(buf_writer, "Content-Length: {}\r\n", file.metadata()?.len())?;
    write!(buf_writer, "Content-Type: text/html; charset=utf-8\r\n")?;
    write!(buf_writer, "\r\n")?;

    let mut buf_reader = BufReader::new(&file);
    std::io::copy(&mut buf_reader, &mut buf_writer)?;
    buf_writer.flush()?;

    Ok(())
}
