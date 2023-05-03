use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader, BufRead, Write, Read},
    fs::{self, Metadata},
    path::Path,
    ffi::OsStr,
    sync::Arc,
    time::{SystemTime, Instant},
    str::FromStr,
    env,
};
use website::thread::ThreadPool;
use website::apis::ApiRegister;
use website::types::{
    ContentType, RequestType,
    HTTPRequestLine, Response,
    HTTPError, turn_system_time_to_http_date,
    HTTPType, POSTRequest, Request,
    GETRequest, ImageType,
};
use lettre::{transport::smtp::authentication::Credentials, Message, message::Mailbox, Transport};
use lettre::SmtpTransport;

// creds should be filled like:
// example@example.com
// password
const CREDS: &str = include_str!("../secrets");

fn test_api(_: Request) -> Response {
    println!("Test Api!");
    let data = String::from("Test api!").into_bytes();
    Response {
        code: 200,
        content_type: ContentType::PlainText,
        current_time: Some(SystemTime::now()),
        modified_date: None,
        allowed: None,
        data
    }
}

// takes ~1.6 seconds to send both emails and send a response
// ~675ms per email so might async or do something to speed this up
// maybe multithread each email (this is a joke)
fn mail_api(request: Request, mailer: Arc<SmtpTransport>) -> Response {
    println!("{:?}", request);
    let request = match request {
        Request::GetRequest(_) => {
            let res = Response::new_405_error("POST");
            return res;
        }, //405 error
        Request::POSTRequest(r) => r,
    };

    let mut data = BufReader::new(request.get_data());
    let mut email_len = [0_u8; 1];

    match data.read_exact(&mut email_len) {
        Err(_) => {
            return Response::new(
                400,
                ContentType::PlainText,
                None,
                None,
                String::from("Email Length Not Found").into_bytes()
            );
        }
        Ok(_) => {},
    }

    let mut email = vec![0_u8; email_len[0] as usize];

    match data.read_exact(&mut email) {
        Err(_) => {
            return Response::new(
                400,
                ContentType::PlainText,
                None,
                None,
                String::from("Email Not Found").into_bytes()
            );
        }
        Ok(_) => {},
    }

    let mut message_len = [0_u8; 2];

    match data.read_exact(&mut message_len) {
        Err(_) => {
            return Response::new(
                400,
                ContentType::PlainText,
                None,
                None,
                String::from("Message Length Not Found").into_bytes()
            );
        }
        Ok(_) => {},
    }

    let message_len = u16::from_le_bytes(message_len) as usize;
    let mut message = vec![0_u8; message_len];
    match data.read_exact(&mut message) {
        Err(_) => {
            return Response::new(
                400,
                ContentType::PlainText,
                None,
                None,
                String::from("Message Not Found").into_bytes()
            );
        }
        Ok(_) => {},
    }

    // send the email!
    let user_email = String::from_utf8_lossy(&email);
    let user_message = String::from_utf8_lossy(&message); 
    let message_to_self = format!("contacter email: {user_email},\n\n{user_message}");
    let send_to = CREDS.lines().next().unwrap();
    let self_mailbox: Mailbox = format!("Charles Crabtree <{send_to}>").parse().unwrap();
    let email_to_self = Message::builder()
        .from("x <example@example.com>".parse().unwrap())
        .to(self_mailbox)
        .body(message_to_self)
        .unwrap();
    
    let time = Instant::now();
    match mailer.send(&email_to_self) {
        Ok(_) => println!("email send succesfully"),
        Err(e) => println!("Could not send email: {e:?}"),
    }
    println!("{}", time.elapsed().as_millis());

    let self_mailbox: Mailbox = format!("Charles Crabtree <{send_to}>").parse().unwrap();
    let email_to_client = Message::builder()
        .from(self_mailbox)
        .to(format!("person <{user_email}>").parse().unwrap())
        .body("thanks for reaching out I will try to be in contact with you shortly".to_string())
        .unwrap();

    match mailer.send(&email_to_client) {
        Ok(_) => println!("email send succesfully"),
        Err(e) => println!("Could not send email: {e:?}"),
    }

    Response::empty_ok()
}

fn main() {
    let mut secrets = include_str!("../secrets").lines();
    let username = secrets.next().unwrap();
    let password = secrets.next().unwrap();
    drop(secrets);

    let creds = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();
    let mailer = Arc::new(mailer);
    // seething at this implementation of an api with a mailer
    let email_api = move |r: Request| -> Response {
        let clone = mailer.clone();
        mail_api(r, clone)
    };

    let port = env::var("PORT").expect("Need PORT env var");
    let addr = String::from("0.0.0.0:") + &port;
    let listener = TcpListener::bind(addr).unwrap();

    let pool = ThreadPool::new(8);
    let mut apis = ApiRegister::new();
    apis.register_api("/api/test", Box::new(test_api));
    apis.register_api("/api/mail", Box::new(email_api));
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

    let request = match Request::new(buf_reader) {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}, occured at: {:?}", e, turn_system_time_to_http_date(SystemTime::now()));
            let respoonse = Response::new_400_error(e).into_bytes();
            stream.write_all(&respoonse).unwrap();
            return;
        }
    };

    match request {
        Request::GetRequest(_) => process_get_request(request, apis, &mut stream),
        Request::POSTRequest(_) => process_post_request(request, apis, &mut stream)
    }
}

fn process_get_request(request: Request, apis: Arc<ApiRegister>, stream: &mut TcpStream) {
    let path = request.get_path();
    println!("request_line: {:?}, {}", request, path == "/");
    let path = Path::new(path);
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
        RequestType::Html => html_request(path, stream),
        RequestType::OtherFile => file_request(path, stream),
        RequestType::Api => api_request(apis , stream, request),
    }
}

fn process_post_request(request: Request, apis: Arc<ApiRegister>, stream: &mut TcpStream) {
    println!("post!, {:?}", request);
    // should therortically just be an API request
    match Path::new(request.get_path()).parent().and_then(Path::to_str) {
        Some("/api") => {},
        Some(_) => {
            // honeslty not sure what error code belongs here
            let mut response = Response::empty_404().into_bytes();
            stream.write_all(&mut response).unwrap();
            return;
        }
        None => {
            // errors
            let mut response = Response::empty_404().into_bytes();
            stream.write_all(&mut response).unwrap();
            return;
        }
    };
    let api = match apis.get_api(request.get_path()) {
        Some(a) => a,
        None => {
            let mut response = Response::empty_404().into_bytes();
            stream.write_all(&mut response).unwrap();
            return;
        }
    };

    let res = api(request);
    stream.write_all(&mut res.into_bytes()).unwrap();
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
            allowed: None,
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
                    allowed: None,
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
                    allowed: None,
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
        Some("png") => ContentType::Image(ImageType::Png),
        Some("svg") => ContentType::Image(ImageType::Svg),
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
                allowed: None,
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

fn api_request(apis: Arc<ApiRegister>, stream: &mut TcpStream, request: Request) {
    let path = request.get_path();

    let api = apis.get_api(path);
    let response = match api {
        None => Response::empty_404(),
        Some(api) => api(request),
    };

    stream.write_all(&response.into_bytes()).unwrap();
}

// made to use and_then on results for reading meta data to avoid unsessicary unwrap
fn into_modified(metadata: Metadata) -> Result<SystemTime, std::io::Error> {
    metadata.modified()
}