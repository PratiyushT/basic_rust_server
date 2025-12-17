pub mod request;

pub use request::{Request, RequestError};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use std::net::TcpStream;

pub static BASE_DIR: &str = "pages";

pub fn handle_connection(tcp_stream: &mut TcpStream) -> Result<(), RequestError> {
    let request = Request::new(tcp_stream)?;

    let mut file: File;
    let mut status = String::from("HTTP/1.1 ");

    if let Some(url) = request.path_exists() {
        file = File::open(url)?;
        status.push_str("200 Ok");
    } else {
        let mut url = "error404.html".to_string();
        if !BASE_DIR.is_empty() {
            url = format!("{BASE_DIR}/{url}");
        }
        file = File::open(url)?;
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
