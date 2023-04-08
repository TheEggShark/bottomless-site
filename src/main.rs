use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, BufRead, Write},
    fs::{self, Metadata, FileType},
    str::FromStr, fmt::Display,
};
use website::thread::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(8);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream)
        });
    }
    println!("Hello, world!");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next();
    let request_line = match request_line {
        None => return,
        Some(option) => option.unwrap(),
    };

    let request_line = HTTPRequestLine::from_str(&request_line).unwrap();

    println!("request_line: {:?}, {}", request_line, request_line.path == "/");
    let request_type = if request_line.path.starts_with("/css") || request_line.path.starts_with("/images") {
        RequestType::OtherFile
    } else {
        RequestType::Html // worry about api later
    };

    match request_type {
        RequestType::Html => html_request(&request_line.path, &mut stream),
        RequestType::OtherFile => file_request(&request_line.path, &mut stream),
        _ => unimplemented!(),
    }
    // let (contents, status_line) = if path == "files/" {
    //     (fs::read("files/index.html").unwrap(), "HTTP/1.1 200 OK")
    // } else {
    //     match fs::read(path) {
    //         Ok(s) => (s, "HTTP/1.1 200 OK"),
    //         Err(e) => {
    //             println!("{}", e);
    //             (fs::read("files/404.html").unwrap(), "HTTP/1.1 404 NOT FOUND")
    //         }
    //     }
    // };

    // let length = contents.len();

    // let response_header =
    //     format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");

    // let response = [response_header.as_bytes(), &contents].concat();

    // stream.write_all(&response).unwrap();
}

fn html_request(path: &str, stream: &mut TcpStream) {
    println!("html!");
    if path == "/" {
        let contents = fs::read("files/index.html").unwrap();
        let response = Response {
            code: 200,
            content_type: ContentType::Html,
        }.to_bytes(contents);
        stream.write_all(&response).unwrap();
    } else {
        let path = String::from("files") + path + ".html";
        match fs::read(path) {
            Ok(contents) => {
                let response = Response {
                    code: 200,
                    content_type: ContentType::Html,
                }.to_bytes(contents);
                stream.write_all(&response).unwrap();
            },
            Err(_) => {
                let data = fs::read("files/404.html").unwrap();
                let response = Response {
                    code: 404,
                    content_type: ContentType::Html,
                }.to_bytes(data);
                stream.write_all(&response).unwrap();
            }
        }
    }
}

fn file_request(path: &str, stream: &mut TcpStream) {
    let content_type = if path.ends_with(".css") {
        ContentType::Css
    } else if path.ends_with(".js") {
        ContentType::JavaScript
    } else if path.ends_with(".png") {
        ContentType::Image
    } else {
        unimplemented!()
    };

    let path = String::from("files") + path;
    match fs::read(path) {
        Ok(data) => {
            let response = Response {
                code: 200,
                content_type,
            }.to_bytes(data);
            stream.write_all(&response).unwrap();
        },
        Err(_) => {
            let data = fs::read("files/404.html").unwrap();
            let response = Response {
                code: 404,
                content_type: ContentType::Html,
            }.to_bytes(data);
            stream.write_all(&response).unwrap();
        }
    }
}

enum RequestType {
    Api,
    OtherFile,
    Html,
}

fn make_code(code: u16) -> String {
    match code {
        200 => String::from("HTTP/1.1 200 OK"),
        404 => String::from("HTTP/1.1 404 NOT FOUND"),
        _ => unimplemented!(),
    }
}

#[derive(Debug)]
struct Response {
    code: u16,
    content_type: ContentType,
}

impl Response {
    fn to_bytes(self, data: Vec<u8>) -> Vec<u8> {
        let header = format!("{}\r\nContent-type: {}\r\nContent-length: {}\r\n\r\n", make_code(self.code), self.content_type, data.len());
        println!("{}", header);
        [header.as_bytes(), &data].concat()
    }
}

#[derive(Debug)]
enum ContentType {
    Image,
    Css,
    JavaScript,
    Html,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "image/png"),
            Self::Css => write!(f, "text/css"),
            Self::JavaScript => write!(f, "text/javascript"),
            Self::Html => write!(f, "text/html"),
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
            None => return Err(HTTPError::MissingPath),
            Some(s) => s.to_string()
        };

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
    MissingPath,
    InvalidRequestType,
    InvalidVersion,
    InvalidRequestLine,
}