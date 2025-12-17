# Minimal HTTP Server in Rust

A minimal, single-threaded HTTP/1.1 server written in Rust from first principles.
This project is meant for my learning of the rust language maily file handling, string parsing, multi-threading and basic HTTP related stuff.

---

## Features

* Manual HTTP request line parsing
* Safe URL → filesystem resolution
* Protection against directory traversal attacks
* Static `.html` file serving
* Zero external web frameworks (Except rust's libraries)
* Clean error modeling using `thiserror`

---


## How It Works (High-Level Flow)

1. A TCP connection is accepted.
2. The request line is read from the stream.
3. The request line is validated:

   * Must be `GET`
   * Must be `HTTP/1.1`
   * Must contain exactly 3 parts
4. The requested path is sanitized:

   * Query strings (`?`) and fragments (`#`) removed
   * `/` maps to `index.html`
   * Directory traversal (`..`) is rejected
5. The path is resolved relative to `BASE_DIR`.
6. If the file exists, it is served.
7. Otherwise, a `404` response is returned.


---


## Running the Server

1. Place your HTML files inside the base directory (Change it in `/src/constants.rs`):

2. Run the server:

```bash
cargo run
```

3. Open your browser:

```
http://localhost:PORT/
```

(Replace `PORT` with the one defined in `main.rs`.)

---

Credits to: [The Rust Programming Language Book](https://doc.rust-lang.org/stable/book/title-page.html)
