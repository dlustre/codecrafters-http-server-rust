use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use http::Response;
use itertools::Itertools;

mod http;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let directory = args
        .as_slice()
        .into_iter()
        .collect_tuple::<(&String, &String)>()
        .and_then(|(flag, directory_str)| {
            if flag == "--directory" {
                Some(PathBuf::from(directory_str))
            } else {
                None
            }
        });

    if let Some(dir) = &directory {
        println!("directory: {}", dir.display());
    } else {
        println!("No directory specified, using default configuration.");
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Some(dir) = directory.clone() {
                    thread::spawn(move || handle_connection(&mut stream, Some(dir)));
                } else {
                    thread::spawn(move || handle_connection(&mut stream, None));
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: &mut TcpStream, directory: Option<PathBuf>) {
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
                    content_type: None,
                    version: request.version,
                    body: None,
                },
                "/user-agent" => {
                    let user_agent = headers.get("User-Agent");

                    Response {
                        status: http::Status::Ok,
                        content_type: Some(http::ContentType::Text),
                        version: request.version,
                        body: user_agent.cloned(),
                    }
                }
                file_req if file_req.starts_with("/file/") => {
                    let file_path = directory
                        .expect("no directory provided")
                        .join(file_req.strip_prefix("/file/").unwrap_or_default());

                    match read_file(&file_path) {
                        Ok(contents) => Response {
                            status: http::Status::Ok,
                            content_type: Some(http::ContentType::Application),
                            version: request.version,
                            body: Some(contents),
                        },
                        Err(_) => Response {
                            status: http::Status::NotFound,
                            content_type: None,
                            version: request.version,
                            body: None,
                        },
                    }
                }
                echo_req if echo_req.starts_with("/echo/") => Response {
                    status: http::Status::Ok,
                    content_type: Some(http::ContentType::Text),
                    version: request.version,
                    body: Some(
                        echo_req
                            .strip_prefix("/echo/")
                            .unwrap_or_default()
                            .to_string(),
                    ),
                },
                _ => Response {
                    status: http::Status::NotFound,
                    content_type: None,
                    version: request.version,
                    body: None,
                },
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

fn read_file(file_path: &Path) -> io::Result<String> {
    if file_path.exists() {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }
}
