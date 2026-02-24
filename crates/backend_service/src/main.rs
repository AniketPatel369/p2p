use backend_service::route_request;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0u8; 8192];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let response = route_request(&request).to_http_string();
    let _ = stream.write_all(response.as_bytes());
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8787";
    let listener = TcpListener::bind(addr)?;
    println!("backend_service listening on http://{addr}");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }

    Ok(())
}
