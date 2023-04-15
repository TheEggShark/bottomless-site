use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, BufRead, Write},
    fs::{self, Metadata},
    path::Path,
    ffi::OsStr,
    sync::Arc,
    time::SystemTime,
    str::FromStr,
    env,
};
use website::thread::ThreadPool;
use website::apis::ApiRegister;
use website::types::{
    ContentType, RequestType,
    HTTPRequestLine, Response,
    HTTPError, turn_system_time_to_http_date
};

fn test_api() -> Response {
    let data = String::from("Test api!").into_bytes();
    Response {
        code: 200,
        content_type: ContentType::PlainText,
        current_time: Some(SystemTime::now()),
        modified_date: None,
        data
    }
}

fn main() {
    let port = env::var("PORT").expect("Need PORT env var");
    let addr = String::from("0.0.0.0:") + &port;
    let listener = TcpListener::bind(addr).unwrap();

    let pool = ThreadPool::new(8);
    let mut apis = ApiRegister::new();
    apis.register_api("/api/test", Box::new(test_api));
    let apis = Arc::new(apis);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let apis = apis.clone();
                pool.execute(move || {
                    handle_connection(stream, apis)
                });
            }
            Err(e) => println!("Error: {}, \n occured at: {}", e, turn_system_time_to_http_date(SystemTime::now())),
        }

    }
}

fn handle_connection(mut stream: TcpStream, apis: Arc<ApiRegister>) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next();
    let request_line = match request_line {
        None => return,
        Some(result) => {
            match result {
                Ok(string) => string,
                Err(e) => {
                    println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(SystemTime::now()));
                    let response = Response::new_400_error(HTTPError::InvalidRequestLine).into_bytes();
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
            let response = Response::new_400_error(e).into_bytes();
            stream.write_all(&response).unwrap();
            return;
        }
    };

    println!("request_line: {:?}, {}", request_line, request_line.path == "/");
    let path = Path::new(&request_line.path);
    let request_type = match path.parent().and_then(Path::to_str) {
        Some("/") => {
            if path == Path::new("/favicon.ico") {
                RequestType::OtherFile
            } else {
                RequestType::Html
            }
        },
        Some("/api") => RequestType::Api,
        None => RequestType::Html, // this is the index.html
        Some(_) => RequestType::OtherFile,
    };
    println!("{:?}, {:?}", path, request_type);


    match request_type {
        RequestType::Html => html_request(path, &mut stream),
        RequestType::OtherFile => file_request(path, &mut stream),
        RequestType::Api => api_request(path, apis ,&mut stream),
    }
}

fn html_request(path: &Path, stream: &mut TcpStream) {
    let time = SystemTime::now();

    if path.as_os_str() == "/" {
        let index_path = Path::new("files/index.html");
        let data = fs::read(index_path).unwrap();
        let last_modified = match index_path.metadata().and_then(into_modified) {
            Ok(time) => Some(time),
            Err(_) => None,
        };

        let response = Response {
            code: 200,
            content_type: ContentType::Html,
            modified_date: last_modified,
            current_time: Some(time),
            data,
        }.into_bytes();
        stream.write_all(&response).unwrap();
    } else {
        // I Hate paths dear lord wtf is this garbage
        let path = Path::new("files").join(path.strip_prefix("/").unwrap()).with_extension("html");
        println!("{:?}", path.as_path());

        match fs::read(&path) {
            Ok(data) => {
                let last_modified = match path.metadata().and_then(into_modified) {
                    Ok(time) => Some(time),
                    Err(_) => None,
                };
                let response = Response {
                    code: 200,
                    content_type: ContentType::Html,
                    modified_date: last_modified,
                    current_time: Some(time),
                    data,
                }.into_bytes();
                stream.write_all(&response).unwrap();
            },
            Err(_) => {
                let data = match fs::read("files/404.html") {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Error: {}\n Occured at: {}", e, turn_system_time_to_http_date(time));
                        let response = Response::empty_500_error().into_bytes();
                        stream.write_all(&response).unwrap();
                        return;
                    }
                };

                let modified_date = match Path::new("files/404.html").metadata().and_then(into_modified) {
                    Ok(m) => Some(m),
                    Err(_) => None,
                };

                let response = Response {
                    code: 404,
                    modified_date,
                    content_type: ContentType::Html,
                    current_time: Some(SystemTime::now()),
                    data,
                }.into_bytes();
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
            let response = Response::new_400_error(HTTPError::InvalidPath).into_bytes();
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
                data,
            }.into_bytes();
            stream.write_all(&response).unwrap();
        },
        Err(_) => {
            let response = Response::empty_404().into_bytes();
            stream.write_all(&response).unwrap();
        }
    }
}

fn api_request(path: &Path, apis: Arc<ApiRegister>, stream: &mut TcpStream) {
    let path = match path.to_str() {
        Some(string) => string,
        None => {
            stream.write_all(&Response::new_400_error(HTTPError::InvalidPath).into_bytes()).unwrap();
            return;
        }
    };
    let api = apis.get_api(path);
    let response = match api {
        None => Response::empty_404(),
        Some(api) => api(),
    };

    stream.write_all(&response.into_bytes()).unwrap();
}

// made to use and_then on results for reading meta data to avoid unsessicary unwrap
fn into_modified(metadata: Metadata) -> Result<SystemTime, std::io::Error> {
    metadata.modified()
}