use rust_server::{ADDRESS, handle_connection};
use std::net::TcpListener;

fn main() {
    let tcp_listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Attempting to bind a listener at: {ADDRESS}");
    let connection_attempts = tcp_listener.incoming();

    for connection_attempt in connection_attempts {
        let mut connection = connection_attempt.unwrap();
        let _ = handle_connection(&mut connection).unwrap();
    }
}
