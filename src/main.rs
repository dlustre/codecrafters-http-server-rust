// uncomment this block to pass the first stage
use std::{
    io::{self, BufRead, Write},
    net::{TcpListener, TcpStream},
};

mod http;

fn main() {
    // you can use print statements as follows for debugging, they'll be visible when running tests.
    println!("logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_request(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: &mut TcpStream) {
    let mut buf_reader = io::BufReader::new(&mut stream);
    let mut request_buffer = vec![];

    match buf_reader.read_until(b'\n', &mut request_buffer) {
        Ok(_) => {
            let request = http::parse_http(&request_buffer);

            let response = {
                if request.path == "/".to_string() {
                    http::response(http::HTTPStatus::OK, "")
                } else {
                    http::response(http::HTTPStatus::NotFound, "")
                }
            };

            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => println!("Error reading from stream: {}", e),
    }
}
