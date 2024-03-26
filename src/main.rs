use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use http::Response;

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || handle_connection(&mut stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: &mut TcpStream) {
    let mut buf_reader = io::BufReader::new(&mut stream);
    let mut request_line = String::new();

    if let Ok(_) = buf_reader.read_line(&mut request_line) {
        let mut headers = HashMap::new();
        let mut header = String::new();

        while buf_reader.read_line(&mut header).unwrap_or(0) > 2 {
            println!("line: `{}`", header);
            let (key, value) = header.trim_end().split_once(": ").unwrap();
            println!("key: `{}` val: `{}`", key, value);
            headers.insert(key.to_owned(), value.to_owned());
            header.clear();
        }

        let request = http::parse_http(request_line);

        let response = match request.method {
            http::Method::GET => match request.path.as_str() {
                "/" => Response {
                    status: http::Status::Ok,
                    version: request.version,
                    body: None,
                },
                "/user-agent" => {
                    let user_agent = headers.get("User-Agent");

                    Response {
                        status: http::Status::Ok,
                        version: request.version,
                        body: user_agent.cloned(),
                    }
                }
                path => {
                    if path.starts_with("/echo/") {
                        Response {
                            status: http::Status::Ok,
                            version: request.version,
                            body: Some(path.strip_prefix("/echo/").unwrap_or_default().to_string()),
                        }
                    } else {
                        Response {
                            status: http::Status::NotFound,
                            version: request.version,
                            body: None,
                        }
                    }
                }
            },
            http::Method::POST => todo!(),
        };

        println!("{}", response.to_string());

        let response_str = format!("{}", response);
        stream.write_all(response_str.as_bytes()).unwrap();
    } else {
        println!("Error reading from stream");
    }
}
