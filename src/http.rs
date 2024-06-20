use core::fmt;
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read},
    net::TcpStream,
};

use flate2::{write::GzEncoder, Compression};

pub enum Status {
    Ok,
    Created,
    InternalServerError,
    NotFound,
}

impl Status {
    fn code(&self) -> usize {
        match self {
            Self::Ok => 200,
            Self::Created => 201,
            Self::InternalServerError => 500,
            Self::NotFound => 404,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::Created => "Created",
            Self::InternalServerError => "Internal Server Error",
            Self::NotFound => "Not Found",
        }
    }
}

pub enum ContentType {
    Text,
    Application,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Text => "text/plain",
            Self::Application => "application/octet-stream",
        };

        write!(f, "{}", display)?;

        Ok(())
    }
}

pub enum Method {
    GET,
    POST,
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

pub fn parse_http(stream: &mut BufReader<&TcpStream>) -> io::Result<Request> {
    let mut request_line = String::new();
    stream.read_line(&mut request_line)?;

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid request line",
        ));
    }

    let method = match parts[0] {
        "GET" => Method::GET,
        "POST" => Method::POST,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported HTTP Method",
            ))
        }
    };

    let mut headers = HashMap::new();
    let mut header = String::new();

    // parse headers
    while stream.read_line(&mut header)? > 0 {
        let trimmed = header.trim();
        if trimmed.is_empty() {
            break; // End of headers
        }
        let (key, value) = trimmed.split_once(": ").ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid header line",
        ))?;
        headers.insert(key.to_string(), value.to_string());
        header.clear();
    }

    // read the body based on `Content-Length`
    let mut body = String::new();
    if let Some(content_length_str) = headers.get("Content-Length") {
        if let Ok(content_length) = content_length_str.parse::<usize>() {
            let mut body_bytes = vec![0; content_length];
            stream.read_exact(&mut body_bytes)?;
            body = String::from_utf8_lossy(&body_bytes).to_string();
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Content-Length header",
            ));
        }
    }

    Ok(Request {
        method,
        path: parts[1].to_string(),
        version: parts[2].to_string(),
        headers,
        body: Some(body),
    })
}

pub enum Encoding {
    Gzip,
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Gzip => "gzip",
        };

        write!(f, "{}", display)?;

        Ok(())
    }
}

pub struct Response {
    pub status: Status,
    pub content_encoding: Option<Encoding>,
    pub content_type: Option<ContentType>,
    pub version: String,
    pub body: Option<String>,
}

use std::io::Write;

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response_str = String::new();
        response_str.push_str(
            format!(
                "{} {} {}\r\n",
                self.version,
                self.status.code(),
                self.status.message()
            )
            .as_str(),
        );

        match (&self.content_type, &self.body, &self.content_encoding) {
            (Some(content_type), Some(body), Some(content_encoding)) => {
                response_str
                    .push_str(format!("Content-Encoding: {}\r\n", content_encoding).as_str());
                response_str.push_str(format!("Content-Type: {}\r\n", content_type).as_str());

                // Gzip the body
                let mut encoder = GzEncoder::new(vec![], Compression::default());
                encoder.write_all(body.as_bytes()).unwrap();
                let compressed = encoder.finish().unwrap();

                // Length of the compressed body
                let len = compressed.len();
                response_str.push_str(format!("Content-Length: {}\r\n\r\n", len).as_str());
                let mut bytes = response_str.into_bytes();
                bytes.extend(compressed);
                return bytes;
            }
            (Some(content_type), Some(body), None) => {
                response_str.push_str(format!("Content-Type: {}\r\n", content_type).as_str());
                response_str
                    .push_str(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_str());
            }
            _ => {
                response_str.push_str(format!("\r\n").as_str());
            }
        }

        response_str.into_bytes()
    }
}
