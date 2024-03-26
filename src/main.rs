use std::{
    io::{self, BufRead, Write},
    net::{TcpListener, TcpStream},
};

use http::Response;
use itertools::Itertools;

mod http;

fn main() {
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

            let response = match request.method {
                http::Method::GET => {
                    let path_segments = request.path.split("/").collect_vec();

                    match path_segments[..] {
                        ["", ""] => Response {
                            status: http::Status::Ok,
                            version: request.version,
                            body: None,
                        },
                        ["", "echo", ..] => {
                            let echo_content = path_segments[2..].join(" ");
                            Response {
                                status: http::Status::Ok,
                                version: request.version,
                                body: Some(echo_content),
                            }
                        }
                        _ => Response {
                            status: http::Status::NotFound,
                            version: request.version,
                            body: None,
                        },
                    }
                }
                http::Method::POST => todo!(),
            };

            println!("{}", response.to_string());

            write!(stream, "{}", response.to_string()).unwrap()
        }
        Err(e) => println!("Error reading from stream: {}", e),
    }
}
