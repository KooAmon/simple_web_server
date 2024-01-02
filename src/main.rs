use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                //let mut buffer: [u8] =
                //let request = _stream.read(&mut buffer);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                _stream.write(response.as_bytes()).unwrap();
                _stream.flush().unwrap();
            },
            Err(e) => println!("error: {}", e),
        }
    }
}
