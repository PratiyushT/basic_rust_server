use std::net::TcpListener;

static ADDRESS: &str = "127.0.0.1:7878"; // <IP>:<Port>

fn main() {
    // Start a TCP listener at an ADDRESS.
    // We are binding (connecting/ listening) to this port.
    let listener = TcpListener::bind(ADDRESS)
        .expect(format!("Could not start a TCP listener in {}", ADDRESS).as_str());

    // Each stream represents open connection between client and the server.
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            println!("{:?}", stream);
        }
    }
}


