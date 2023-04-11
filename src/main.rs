use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, BufRead, Write},
    fs::{self, Metadata},
    str::FromStr,
    fmt::Display,
    path::Path,
    ffi::OsStr,
    time::{SystemTime, UNIX_EPOCH},
};
use website::thread::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    let pool = ThreadPool::new(8);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream)
                });
            }
            Err(e) => println!("Error: {}, \n occured at: {}", e, turn_system_time_to_http_date(SystemTime::now())),
        }

    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next();
    let request_line = match request_line {
        None => return,
        Some(result) => {
            match result {
                Ok(string) => string,
                Err(e) => {
                    println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(SystemTime::now()));
                    let response = Response::new_400_error(HTTPError::InvalidRequestLine);
                    stream.write_all(&response).unwrap();
                    return;
                }
            }
        },
    };

    let request_line = match HTTPRequestLine::from_str(&request_line) {
        Ok(line) => line,
        Err(e) => {
            println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(SystemTime::now()));
            let response = Response::new_400_error(e);
            stream.write_all(&response).unwrap();
            return;
        }
    };

    println!("request_line: {:?}, {}", request_line, request_line.path == "/");
    let path = Path::new(&request_line.path);
    let request_type = match path.parent() {
        Some(parent_path) if parent_path == Path::new("/") => {
            if path == Path::new("/favicon.ico") {
                RequestType::OtherFile
            } else {
                RequestType::Html
            }
        },
        None => RequestType::Html, // this is the index.html
        Some(_) => RequestType::OtherFile,
    };
    println!("{:?}, {:?}", path, request_type);


    match request_type {
        RequestType::Html => html_request(Path::new(&request_line.path), &mut stream),
        RequestType::OtherFile => file_request(Path::new(&request_line.path), &mut stream),
        _ => unimplemented!(),
    }
}

fn html_request(path: &Path, stream: &mut TcpStream) {
    let time = SystemTime::now();

    if path.as_os_str() == "/" {
        let index_path = Path::new("files/index.html");
        let contents = fs::read(index_path).unwrap();
        let last_modified = match index_path.metadata().and_then(into_modified) {
            Ok(time) => Some(time),
            Err(_) => None,
        };

        let response = Response {
            code: 200,
            content_type: ContentType::Html,
            modified_date: last_modified,
            current_time: Some(time),
        }.to_bytes(&contents);
        stream.write_all(&response).unwrap();
    } else {
        // I Hate paths dear lord wtf is this garbage
        let path = Path::new("files").join(path.strip_prefix("/").unwrap()).with_extension("html");
        println!("{:?}", path.as_path());

        match fs::read(&path) {
            Ok(contents) => {
                let last_modified = match path.metadata().and_then(into_modified) {
                    Ok(time) => Some(time),
                    Err(_) => None,
                };
                let response = Response {
                    code: 200,
                    content_type: ContentType::Html,
                    modified_date: last_modified,
                    current_time: Some(time),
                }.to_bytes(&contents);
                stream.write_all(&response).unwrap();
            },
            Err(_) => {
                let data = match fs::read("files/404.html") {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(time));
                        let response = Response::empty_500_error();
                        stream.write_all(&response).unwrap();
                        return;
                    }
                };

                let response = Response {
                    code: 404,
                    content_type: ContentType::Html,
                    modified_date: None,
                    current_time: Some(time),
                }.to_bytes(&data);
                stream.write_all(&response).unwrap();
            }
        }
    }
}

fn file_request(path: &Path, stream: &mut TcpStream) {
    let time = SystemTime::now();
    let content_type = match path.extension().and_then(OsStr::to_str) {
        Some("css") => ContentType::Css,
        Some("js") => ContentType::JavaScript,
        Some("png") => ContentType::Image,
        ext => {
            println!("Unsuported extention: {:?}", ext);
            let response = Response::new_400_error(HTTPError::InvalidPath);
            stream.write_all(&response).unwrap();
            return;
        }
    };

    // paths will single handly kill me
    // also we know path stripping wont fail bc we make sure it starts with one
    let path = Path::new("files").join(path.strip_prefix("/").unwrap());
    let last_modified_date = match path.metadata().and_then(into_modified) {
        Ok(time) => Some(time),
        Err(_) => None,
    };

    println!("{:?}", path);

    match fs::read(path) {
        Ok(data) => {
            let response = Response {
                code: 200,
                content_type,
                current_time: Some(time),
                modified_date: last_modified_date,
            }.to_bytes(&data);
            stream.write_all(&response).unwrap();
        },
        Err(_) => {
            let response = Response::EMPTY404.to_bytes("Not Found".as_bytes());
            stream.write_all(&response).unwrap();
        }
    }
}

#[derive(Debug)]
enum RequestType {
    Api,
    OtherFile,
    Html,
}

fn make_code(code: u16) -> String {
    match code {
        200 => String::from("HTTP/1.1 200 OK"),
        400 => String::from("HTTP/1.1 400 BAD REQUEST"),
        404 => String::from("HTTP/1.1 404 NOT FOUND"),
        500 => String::from("HTTP/1.1 500 INTERAL SERVER ERROR"),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
struct Response {
    code: u16,
    content_type: ContentType,
    modified_date: Option<SystemTime>,
    current_time: Option<SystemTime>,
}

impl Response {
    const EMPTY404: Self = Self {
        code: 404,
        content_type: ContentType::PlainText,
        modified_date: None,
        current_time: None,
    };

    fn empty_500_error() -> Vec<u8> {
        let response = Self {
            code: 500,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
        };
        let content = "Internal Server Error";
        response.to_bytes(content.as_bytes())
    }

    fn new_400_error(error: HTTPError) -> Vec<u8> {
        let response = Self {
            code: 404,
            content_type: ContentType::PlainText,
            modified_date: None,
            current_time: Some(SystemTime::now()),
        };
        let content = format!("{}", error);
        response.to_bytes(content.as_bytes())
    }

    fn to_bytes(self, data: &[u8]) -> Vec<u8> {
        let header = format!("{}\r\nContent-type: {}\r\nContent-length: {}\r\n", make_code(self.code), self.content_type, data.len());
        let modified_date = match self.modified_date {
            None => String::new(),
            Some(time) => format!("Last-Modified: {}\r\n", turn_system_time_to_http_date(time)),
        };

        let date = match self.current_time {
            Some(time) => format!("Date: {}\r\n\r\n", turn_system_time_to_http_date(time)),
            None => String::from("\r\n"),
        };

        let line = header + &modified_date + &date;
        println!("{}", line);
        [line.as_bytes(), data].concat()
    }
}

#[derive(Debug)]
enum ContentType {
    Image,
    Css,
    JavaScript,
    Html,
    PlainText,
}

impl Display for ContentType {
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

#[derive(Debug)]
struct HTTPRequestLine {
    kind: HTTPType,
    path: String,
}

impl FromStr for HTTPRequestLine {
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
        if !s.starts_with('/') {
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
enum HTTPType {
    Post,
    Get,
}

#[derive(Clone, Copy, Debug)]
enum HTTPError {
    InvalidPath,
    InvalidRequestType,
    InvalidVersion,
    InvalidRequestLine,
}

impl Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequestLine => writeln!(f, "Request line was invalid"),
            Self::InvalidRequestType => writeln!(f, "Invalid or missing request type"),
            Self::InvalidVersion => writeln!(f, "Invalid or missing HTTP version"),
            Self::InvalidPath => writeln!(f, "Invalid or missing path")
        }
    }
}

// made to use and_then on results for reading meta data to avoid unsessicary unwrap
fn into_modified(metadata: Metadata) -> Result<SystemTime, std::io::Error> {
    metadata.modified()
}

fn turn_system_time_to_http_date(time: SystemTime) -> String {
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