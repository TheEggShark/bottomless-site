use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};
use std::str::FromStr;
use std::io::{BufReader, BufRead, Write, Read};

#[derive(Debug)]
pub enum RequestType {
    Api,
    OtherFile,
    Html,
}

fn make_code(code: u16) -> String {
    match code {
        200 => String::from("HTTP/1.1 200 OK"),
        400 => String::from("HTTP/1.1 400 BAD REQUEST"),
        404 => String::from("HTTP/1.1 404 NOT FOUND"),
        405 => String::from("HTTP/1.1 405 METHOD NOT ALLOWED"),
        500 => String::from("HTTP/1.1 500 INTERAL SERVER ERROR"),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
pub struct Response {
    pub code: u16,
    pub content_type: ContentType,
    pub modified_date: Option<SystemTime>,
    pub current_time: Option<SystemTime>,
    pub allowed: Option<String>,
    pub data: Vec<u8>,
}

impl Response {
    pub fn empty_404() -> Self {
        let data = String::from("NOT FOUND").into_bytes();
        Self {
            code: 404,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
            allowed: None,
            data,
        }
    }

    pub fn empty_ok() -> Self {
        let data = String::from("OK").into_bytes();
        Self {
            code: 200,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
            allowed: None,
            data,
        }
    }

    pub fn empty_500_error() -> Self {
        let data = String::from("Internal Server Error").into_bytes();
        Self {
            code: 500,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
            allowed: None,
            data,
        }
    }

    pub fn new_400_error(error: HTTPError) -> Self {
        let data = format!("{}", error).into_bytes();
        Self {
            code: 400,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
            allowed: None,
            data,
        }
    }

    pub fn new_405_error(accpected: &str) -> Self {
        let data = String::from("Method Not Allowed").into_bytes();
        Self {
            code: 405,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
            allowed: Some(accpected.into()),
            data,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let header = format!("{}\r\nContent-type: {}\r\nContent-length: {}\r\n", make_code(self.code), self.content_type, self.data.len());
        let modified_date = match self.modified_date {
            None => String::new(),
            Some(time) => format!("Last-Modified: {}\r\n", turn_system_time_to_http_date(time)),
        };

        let accpected = match self.allowed {
            None => String::new(),
            Some(s) => format!("Accpect: {}\r\n", s),
        };

        let date = match self.current_time {
            Some(time) => format!("Date: {}\r\n\r\n", turn_system_time_to_http_date(time)),
            None => String::from("\r\n"),
        };

        let line = header + &modified_date + &accpected + &date;
        println!("{}", line);
        [line.as_bytes(), &self.data].concat()
    }
}

#[derive(Debug)]
pub enum ContentType {
    Image,
    Css,
    JavaScript,
    Html,
    PlainText,
}

impl std::str::FromStr for ContentType {
    type Err = HTTPError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image/png" => Ok(Self::Image),
            "text/css" => Ok(Self::Css),
            "text/javascript" => Ok(Self::JavaScript),
            "text/html" => Ok(Self::Html),
            "text/plain" => Ok(Self::PlainText),
            _ => Err(HTTPError::InvalidContentType),
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "image/png"),
            Self::Css => write!(f, "text/css"),
            Self::JavaScript => write!(f, "text/javascript"),
            Self::Html => write!(f, "text/html"),
            Self::PlainText => write!(f, "text/plain"),
        }
    }
}

// used for API's to take a request either get or post without wierd jank,
#[derive(Debug)]
pub enum Request {
    GetRequest(GETRequest),
    POSTRequest(POSTRequest),
}

impl Request {
    pub fn new(mut buf_reader: BufReader<&mut TcpStream>) -> Result<Self, HTTPError> {
        // should theoretically grab the 'GET path HTTP/1.1\r\n' 
        let mut first_line_buffer = Vec::new();
        let request_line_string = match buf_reader.read_until(b'\n', &mut first_line_buffer) {
            Ok(_) => {
                match String::from_utf8(first_line_buffer) {
                    Ok(string) => string,
                    Err(e) => {
                        println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(SystemTime::now()));
                        return Err(HTTPError::InvalidRequestLine);
                    },
                }
            },
            Err(e) => {
                println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(SystemTime::now()));
                return Err(HTTPError::InvalidRequestLine);
            }
        };

        let request_line = HTTPRequestLine::from_str(&request_line_string)?;

        match request_line.get_kind() {
            HTTPType::Get => Ok(Self::GetRequest(GETRequest{path: request_line.path})),
            HTTPType::Post => Ok(Self::POSTRequest(POSTRequest::new(request_line, buf_reader)?)),
        }
    }

    pub fn get_path(&self) -> &str {
        match self {
            Request::GetRequest(r) => &r.path,
            Request::POSTRequest(r) => &r.path,
        }
    }
}

#[derive(Debug)]
pub struct POSTRequest {
    path: String,
    host: String,
    content_type: ContentType,
    content_length: usize,
    content: Vec<u8>,
}

impl POSTRequest {
    pub fn new(line: HTTPRequestLine, reader: BufReader<&mut TcpStream>) -> Result<Self, HTTPError>{
        let path = line.path;

        let (header, mut reader) = split_post_request(reader)?;

        let mut host = String::new();
        let mut content_type = ContentType::PlainText;
        let mut content_length = 0;

        for line in header.lines() {
            if line.starts_with("Content-Type: ") {
                content_type = ContentType::from_str(&line[14..])?;
            } else if line.starts_with("Host: ") {
                host.push_str(&line[6..]);
            } else if line.starts_with("Content-Length: ") {
                content_length = match line[16..].parse() {
                    Err(_) => return Err(HTTPError::InvalidContentLength),
                    Ok(num) => num,
                };
            }
        }

        // read content length
        let mut content: Vec<u8> = Vec::with_capacity(content_length);
        let mut amount_read = 0;
        while amount_read < content_length {
            const BUFFER_SIZE: usize = 10;
            let mut buffer = [0_u8; BUFFER_SIZE];
            let amount_to_read = BUFFER_SIZE.min(content_length - amount_read);
            match reader.read_exact(&mut buffer[..amount_to_read]) {
                Err(_) => return Err(HTTPError::InvalidContent),
                Ok(_) => {},
            }
            content.extend(&buffer[..amount_to_read]);
            amount_read += BUFFER_SIZE;
        }

        Ok(Self {
            path,
            host,
            content_type,
            content_length,
            content
        })
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_data(&self) -> &[u8] {
        &self.content
    }
}

fn split_post_request(mut reader: BufReader<&mut TcpStream>) -> Result<(String, BufReader<&mut TcpStream>), HTTPError> {
    // some how split the body from the header
    // this will be painfull and horrible
    let mut buf = Vec::new();
    loop {
        match reader.read_until(b'\r', &mut buf) {
            Ok(_) => {},
            Err(_) => return Err(HTTPError::InvalidHeader),
        }
        let mut minor_buf = [0_u8; 3];
        match reader.read_exact(&mut minor_buf) {
            Ok(_) => {},
            Err(_) => return Err(HTTPError::InvalidHeader),
        }
        if minor_buf == [b'\n', b'\r', b'\n'] {
            buf.extend(minor_buf);
            break;
        } else {
            // failed to find the double newline but still need to extend the buffer
            // as its still part from the header
            buf.extend(minor_buf);
            continue;
        }
    }

    let header_string = match String::from_utf8(buf) {
        Ok(string) => string,
        Err(_) => return Err(HTTPError::InvalidHeader),
    };

    Ok((header_string, reader))
}

#[derive(Debug)]
pub struct GETRequest {
    pub path: String,
}

#[derive(Debug)]
pub struct HTTPRequestLine {
    kind: HTTPType,
    pub path: String,
}

impl HTTPRequestLine {
    pub fn get_kind(&self) -> HTTPType {
        self.kind
    }
}

impl std::str::FromStr for HTTPRequestLine {
    type Err = HTTPError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut groups = s.split_whitespace();

        let kind = match groups.next() {
            None => return Err(HTTPError::InvalidRequestType),
            Some(kind) => {
                match kind {
                    "GET" => HTTPType::Get,
                    "POST" => HTTPType::Post,
                    _ => return Err(HTTPError::InvalidRequestType),
                }
            }
        };

        let path = match groups.next() {
            None => return Err(HTTPError::InvalidPath),
            Some(s) => s.to_string()
        };

        // garuntees unwrap wont fail later
        if !path.starts_with('/') {
            return Err(HTTPError::InvalidPath);
        }

        // prevents people from theoretically escaping the website folder
        // preventing them from accsessing any file on my PC!
        if let Some(_) = path.matches("../").next() {
            return Err(HTTPError::InvalidPath);
        }

        match groups.next() {
            None => return Err(HTTPError::InvalidVersion),
            Some(_) => {},
        };

        Ok(Self {
            kind,
            path,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HTTPType {
    Post,
    Get,
}

#[derive(Clone, Copy, Debug)]
pub enum HTTPError {
    InvalidPath,
    InvalidRequestType,
    InvalidVersion,
    InvalidRequestLine,
    InvalidHeader,
    InvalidContentType,
    InvalidContentLength,
    InvalidContent,
}

impl std::fmt::Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequestLine => writeln!(f, "Request line was invalid"),
            Self::InvalidRequestType => writeln!(f, "Invalid or missing request type"),
            Self::InvalidVersion => writeln!(f, "Invalid or missing HTTP version"),
            Self::InvalidPath => writeln!(f, "Invalid or missing path"),
            Self::InvalidHeader => writeln!(f, "Invalid or missing header"),
            Self::InvalidContentType => writeln!(f, "Invalid or missing Content-Type"),
            Self::InvalidContentLength => writeln!(f, "Invalid or missing Content-Length"),
            Self::InvalidContent => writeln!(f, "Content to short for Content-Length or invalid Content"),
        }
    }
}

pub fn turn_system_time_to_http_date(time: SystemTime) -> String {
    let time_since_epoch = time.duration_since(UNIX_EPOCH).expect("Times should be after the epoch");
    let seconds_since_epoch = time_since_epoch.as_secs();
    if seconds_since_epoch >= 253402300800 {
        // year 9999
        panic!("date must be before year 9999");
    }

    const LEAPOCH: i64 = 11017;
    const DAYS_PER_400Y: i64 = 365 * 400 + 97;
    const DAYS_PER_100Y: i64 = 365 * 100 + 24;
    const DAYS_PER_4Y: i64 = 365 * 4 + 1;

    let days = (seconds_since_epoch / 86400) as i64 - LEAPOCH;
    let secs_of_day = seconds_since_epoch % 86400;

    let mut qc_cycles = days / DAYS_PER_400Y;
    let mut remdays = days % DAYS_PER_400Y;

    if remdays < 0 {
        remdays += DAYS_PER_400Y;
        qc_cycles -= 1;
    }

    let mut c_cycles = remdays / DAYS_PER_100Y;
    if c_cycles == 4 {
        c_cycles -= 1;
    }
    remdays -= c_cycles * DAYS_PER_100Y;

    let mut q_cycles = remdays / DAYS_PER_4Y;
    if q_cycles == 25 {
        q_cycles -= 1;
    }
    remdays -= q_cycles * DAYS_PER_4Y;

    let mut remyears = remdays / 365;
    if remyears == 4 {
        remyears -= 1;
    }
    remdays -= remyears * 365;

    let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

    let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
    let mut mon = 0;
    for mon_len in months.iter() {
        mon += 1;
        if remdays < *mon_len {
            break;
        }
        remdays -= *mon_len;
    }
    let mday = remdays + 1;
    let mon = if mon + 2 > 12 {
        year += 1;
        mon - 10
    } else {
        mon + 2
    };

    let mut wday = (3 + days) % 7;
    if wday <= 0 {
        wday += 7
    };

    let sec = secs_of_day % 60;
    let min = (secs_of_day % 3600) / 60;
    let hour = secs_of_day / 3600;

    let wday = match wday {
        1 => b"Mon",
        2 => b"Tue",
        3 => b"Wed",
        4 => b"Thu",
        5 => b"Fri",
        6 => b"Sat",
        7 => b"Sun",
        _ => unreachable!(),
    };

    let mon = match mon {
        1 => b"Jan",
        2 => b"Feb",
        3 => b"Mar",
        4 => b"Apr",
        5 => b"May",
        6 => b"Jun",
        7 => b"Jul",
        8 => b"Aug",
        9 => b"Sep",
        10 => b"Oct",
        11 => b"Nov",
        12 => b"Dec",
        _ => unreachable!(),
    };

    let mut buf: [u8; 29] = *b"   , 00     0000 00:00:00 GMT";
    buf[0] = wday[0];
    buf[1] = wday[1];
    buf[2] = wday[2];
    buf[5] = b'0' + (mday / 10) as u8;
    buf[6] = b'0' + (mday % 10) as u8;
    buf[8] = mon[0];
    buf[9] = mon[1];
    buf[10] = mon[2];
    buf[12] = b'0' + (year / 1000) as u8;
    buf[13] = b'0' + (year / 100 % 10) as u8;
    buf[14] = b'0' + (year / 10 % 10) as u8;
    buf[15] = b'0' + (year % 10) as u8;
    buf[17] = b'0' + (hour / 10) as u8;
    buf[18] = b'0' + (hour % 10) as u8;
    buf[20] = b'0' + (min / 10) as u8;
    buf[21] = b'0' + (min % 10) as u8;
    buf[23] = b'0' + (sec / 10) as u8;
    buf[24] = b'0' + (sec % 10) as u8;

    String::from_utf8_lossy(&buf).to_string()
}