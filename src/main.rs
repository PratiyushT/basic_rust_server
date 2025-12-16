use rust_server::{get_request, send_response_header};
use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

/// The address constant that will be used in the program.
static ADDRESS: &str = "127.0.0.1:7878";

fn main() {
    let tcp_listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Attempting to bind a listener at: {ADDRESS}");
    let connection_attempts = tcp_listener.incoming();

    for connection_attempt in connection_attempts {
        if let Ok(mut connection) = connection_attempt {
            let request_full = get_request(&connection);
            for request_line in request_full {
                println!("{request_line}")
            }

            send_response_header(&mut connection, "hello.html".to_string())
                .expect("Response panicked!");
        }
    }
}
