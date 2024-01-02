use std::net::{TcpListener};
use std::io::{BufReader, Read, Write};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut _reader = BufReader::new(&_stream);
                let mut buffer = [0; 1024];

                if let Ok(_) = _reader.read(&mut buffer) {
                    let request = String::from_utf8_lossy(&mut buffer);
                    let required_lines: Vec<&str> = request.lines().collect();

                    if let Some(first_line) = required_lines.get(0) {
                        let mut parts = first_line.split_whitespace();
                        let method = parts.next().unwrap();
                        let path = parts.next().unwrap();
                        let version = parts.next().unwrap();

                        println!("{} {} {}", method, path, version);

                        if path == "/" {
                            let response = "HTTP/1.1 200 OK\r\n\r\n";
                            _stream.write(response.as_bytes()).unwrap();
                            _stream.flush().unwrap();
                        } else {
                            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                            _stream.write(response.as_bytes()).unwrap();
                            _stream.flush().unwrap();
                        }
                    }
                }
            },
            Err(e) => println!("error: {}", e),
        }
    }
}
