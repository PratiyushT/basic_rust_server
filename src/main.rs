use std::io::{
    BufReader,
    prelude::{BufRead, Write},
};
use std::net::{TcpListener, TcpStream};

static ADDRESS: &str = "127.0.0.1:7878"; // <IP>:<Port>

fn main() {
    // Start a TCP listener at an ADDRESS.
    // We are binding (connecting/ listening) to this port.
    let listener = TcpListener::bind(ADDRESS)
        .expect(format!("Could not start a TCP listener in {}", ADDRESS).as_str());

    // Each stream represents open connection between client and the server.
    for connection_attempt in listener.incoming() {
        if let Ok(mut stream) = connection_attempt {
            handle_connection(&stream);
            write_connection_response(&mut stream);
        }
    }
}

/// Handles a single inbound TCP connection by reading and printing the HTTP
/// request headers sent by the client.
///
/// # What it reads:
/// - The request line (e.g., `GET / HTTP/1.1`)
/// - All header lines (e.g., `Host: ...`, `User-Agent: ...`)
///
/// # Where it stops:
/// - Stops reading at the first empty line (`""`), which in HTTP marks the end
///   of the header section (`\r\n\r\n`).
///
/// # What it does NOT read:
/// - It does not read the request body (if any). If the request includes a body,
///   those bytes remain unread in the stream.
///
/// # Notes:
/// - `BufReader` buffers bytes from the TCP stream so line-based reading is
///   efficient and works even when TCP splits the request across multiple packets.
/// - This implementation uses `unwrap()` on I/O results, so a read error will
///   panic. Production code should handle errors gracefully.
///
/// # Arguments
/// * `stream` - Reference to an open `TcpStream` representing the client connection.
fn handle_connection(stream: &TcpStream) {
    let buf_reader = BufReader::new(stream);
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<String>>();
    println!("Request: {http_request:#?}");
}

/// Writes a minimal HTTP success response to an open TCP connection.
///
/// This sends only the HTTP status line and the required blank line that marks
/// the end of the headers:
/// - `HTTP/1.1 200 OK` is the status line (protocol + status code + reason).
/// - `\r\n\r\n` terminates the header section.
///
/// # What it does NOT send:
/// - No additional headers (e.g., `Content-Length`, `Content-Type`)
/// - No response body
///
/// # Notes:
/// - `write_all` blocks until all bytes are written to the OS socket buffer.
/// - This uses `unwrap()`, so any write error (client disconnected, etc.) will
///   panic. Production code should return/handle `io::Result<()>`.
///
/// # Arguments
/// * `stream` - The `TcpStream` representing the client connection to write to.
fn write_connection_response(stream: &mut TcpStream) {
    let response = "HTTP/1.1 200 OK\r\n\r\n"; // Status line + end-of-headers marker.
    stream.write_all(response.as_bytes()).unwrap();
}
