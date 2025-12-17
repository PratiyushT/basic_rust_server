use rust_server::handle_connection;
use std::net::TcpListener;

/// The address constant that will be used in the program.
static ADDRESS: &str = "127.0.0.1:7878";

fn main() {
    let tcp_listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Attempting to bind a listener at: {ADDRESS}");
    let connection_attempts = tcp_listener.incoming();

    for connection_attempt in connection_attempts {
        let mut connection = connection_attempt.unwrap();
        let _ = handle_connection(&mut connection).unwrap();
    }
}
