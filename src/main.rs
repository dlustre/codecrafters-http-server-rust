use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use http::{Encoding, Response};
use itertools::Itertools;

mod http;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let directory = args[..]
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

fn handle_connection(stream: &mut TcpStream, directory: Option<PathBuf>) {
    let mut buf_reader = io::BufReader::new(&*stream);

    let request = http::parse_http(&mut buf_reader).unwrap();
    let encodings: Vec<Encoding> = match request.headers.get("Accept-Encoding") {
        Some(maybe_encodings) => maybe_encodings
            .split(", ")
            .filter_map(|e| match e {
                "gzip" => Some(Encoding::Gzip),
                _ => None,
            })
            .collect(),
        None => vec![],
    };

    // TODO: just a hack since gzip is the only thing implemented
    let content_encoding = match encodings.iter().find_map(|e| match *e {
        Encoding::Gzip => Some(Encoding::Gzip),
    }) {
        Some(e) => Some(e),
        None => None,
    };

    let response = match request.method {
        http::Method::GET => match request.path.as_str() {
            "/" => Response {
                content_encoding,
                status: http::Status::Ok,
                content_type: None,
                version: request.version,
                body: None,
            },
            "/user-agent" => {
                let user_agent = request.headers.get("User-Agent");

                Response {
                    content_encoding,
                    status: http::Status::Ok,
                    content_type: Some(http::ContentType::Text),
                    version: request.version,
                    body: user_agent.cloned(),
                }
            }
            file_req if file_req.starts_with("/files/") => {
                println!(
                    "getting file `{}`",
                    file_req.strip_prefix("/files/").unwrap()
                );
                let file_path = directory
                    .expect("no directory provided")
                    .join(file_req.strip_prefix("/files/").unwrap_or_default());
                println!("path: {}", file_path.display());
                match read_file(&file_path) {
                    Ok(contents) => Response {
                        content_encoding,
                        status: http::Status::Ok,
                        content_type: Some(http::ContentType::Application),
                        version: request.version,
                        body: Some(contents),
                    },
                    Err(e) => {
                        println!("error getting file: {}", e);
                        Response {
                            content_encoding,
                            status: http::Status::NotFound,
                            content_type: None,
                            version: request.version,
                            body: None,
                        }
                    }
                }
            }
            echo_req if echo_req.starts_with("/echo/") => Response {
                content_encoding,
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
                content_encoding,
                status: http::Status::NotFound,
                content_type: None,
                version: request.version,
                body: None,
            },
        },
        http::Method::POST => match request.path.as_str() {
            file_req if file_req.starts_with("/files/") => {
                println!(
                    "posting file `{}`...",
                    file_req.strip_prefix("/files/").unwrap()
                );
                let file_path = directory
                    .expect("no directory provided")
                    .join(file_req.strip_prefix("/files/").unwrap_or_default());
                match save_file(&file_path, &request.body.unwrap()) {
                    Ok(_) => Response {
                        content_encoding,
                        status: http::Status::Created,
                        content_type: None,
                        version: request.version,
                        body: None,
                    },
                    Err(_) => Response {
                        content_encoding,
                        status: http::Status::InternalServerError,
                        content_type: None,
                        version: request.version,
                        body: None,
                    },
                }
            }
            _ => Response {
                content_encoding,
                status: http::Status::NotFound,
                content_type: None,
                version: request.version,
                body: None,
            },
        },
    };

    stream.write_all(response.as_bytes().as_slice()).unwrap();
}

fn read_file(file_path: &Path) -> io::Result<String> {
    if file_path.exists() {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        println!("reading from file...");
        file.read_to_string(&mut contents)?;
        Ok(contents)
    } else {
        println!("File not found");
        Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }
}

fn save_file(file_path: &Path, buf: &String) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    file.write_all(buf.as_bytes())?;

    Ok(())
}
