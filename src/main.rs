use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, BufRead, Write},
    fs,
    thread::{self, JoinHandle}, 
    sync::{mpsc::{Sender, self, Receiver}, Arc, Mutex},
    str::FromStr, fmt::Display,
};

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
    let path = String::from("files") + &request_line.path;
    println!("path: {}", path);
    let (contents, status_line) = if path == "files/" {
        (fs::read("files/index.html").unwrap(), "HTTP/1.1 200 OK")
    } else {
        match fs::read(path) {
            Ok(s) => (s, "HTTP/1.1 200 OK"),
            Err(e) => {
                println!("{}", e);
                (fs::read("files/404.html").unwrap(), "HTTP/1.1 404 NOT FOUND")
            }
        }
    };

    let length = contents.len();

    let response_header =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");

    let response = [response_header.as_bytes(), &contents].concat();

    stream.write_all(&response).unwrap();
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        (0..size).for_each(|id| workers.push(Worker::new(id, Arc::clone(&receiver))));

        ThreadPool {
            workers,
            sender,
        }
    }

    fn execute<F>(&self, function: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(function);

        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker {
            id, 
            thread,
        }
    }
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

#[derive(Debug)]
enum ContentType {
    Image,
    Css,
    Html,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image => write!(f, "image/png"),
            Self::Css => write!(f, "text/css"),
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