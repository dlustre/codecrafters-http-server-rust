use core::{fmt, panic};

pub enum Status {
    Ok,
    NotFound,
}

impl Status {
    fn code(&self) -> usize {
        match self {
            Self::Ok => 200,
            Self::NotFound => 404,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::NotFound => "Not Found",
        }
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
    pub body: String,
}

pub fn parse_http(req: &[u8]) -> Request {
    let req_str = String::from_utf8_lossy(req);
    let mut lines = req_str.lines();

    let first_line = lines.next().expect("Request was empty");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
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
        version: parts[2].to_string(),
        body: lines.collect::<Vec<&str>>().join("\r\n"),
    }
}

pub struct Response {
    pub status: Status,
    pub version: String,
    pub body: Option<String>,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\r\n",
            self.version,
            self.status.code(),
            self.status.message()
        )?;

        if let Some(body) = &self.body {
            write!(f, "Content-Type: text/plain\r\n")?;
            write!(f, "Content-Length: {}\r\n", body.len())?;
            write!(f, "\r\n")?;
            write!(f, "{body}")?;
        }

        Ok(())
    }
}
