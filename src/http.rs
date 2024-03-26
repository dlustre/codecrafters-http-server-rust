use core::panic;

pub enum HTTPStatus {
    OK,
    NotFound,
}

pub enum Method {
    GET,
    POST,
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub body: String,
}

pub fn parse_http(req: &[u8]) -> Request {
    let req_str = String::from_utf8_lossy(req);
    let mut lines = req_str.lines();

    let first_line = lines.next().expect("Request was empty");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        panic!("Invalid request line");
    }

    let method = match parts[0] {
        "GET" => Method::GET,
        "POST" => Method::POST,
        _ => panic!("Unsupported HTTP Method"),
    };

    Request {
        method,
        path: parts[1].to_string(),
        body: lines.collect::<Vec<&str>>().join("\r\n"),
    }
}

pub fn response(status: HTTPStatus, body: &str) -> String {
    match status {
        HTTPStatus::OK => format!("HTTP/1.1 200 OK\r\n\r\n{}", body),
        HTTPStatus::NotFound => format!("HTTP/1.1 404 Not Found\r\n\r\n{}", body),
    }
}
