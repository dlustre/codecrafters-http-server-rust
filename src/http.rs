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
    // pub headers: HashMap<String, String>,
}

pub fn parse_http(req: String) -> Request {
    // let req_str = String::from_utf8_lossy(req);
    let mut lines = req.lines();

    let start_line = lines.next().expect("Request was empty");
    let parts: Vec<&str> = start_line.split_whitespace().collect();
    if parts.len() != 3 {
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
        // headers,
    }
}

pub struct Response {
    pub status: Status,
    pub content_type: Option<ContentType>,
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

        match (&self.content_type, &self.body) {
            (Some(content_type), Some(body)) => {
                write!(f, "Content-Type: {}\r\n", content_type)?;
                write!(f, "Content-Length: {}\r\n\r\n{body}", body.len())?;
            }
            _ => {
                write!(f, "\r\n")?;
            }
        }

        Ok(())
    }
}
