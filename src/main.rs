use uuid::Uuid;
use lazy_static::lazy_static;
use std::{
    net::{TcpListener, TcpStream},
    io::{Write, BufReader, BufRead},
    sync::Mutex
};

mod argparser;
mod threads;
mod http;
mod logger;

use http::{HttpMethod, HttpRequest, HttpStatusCode, HttpResponse};
use threads::ThreadPool;
use logger::{log, LogLevel};

lazy_static!{
    //  The server id. This is used to identify the server run instance in the logs
    static ref SERVERID: Mutex<Uuid> = Mutex::new(Uuid::new_v4());
}

//  Gets the path response for the given request
fn get_path_response(sessionid: &Uuid, root:&str, request: &str) -> HttpResponse {
    log(LogLevel::Trace, &SERVERID.lock().unwrap(), sessionid, format!("Getting path response for {}", request).as_str());
    let filecontents = read_file(&root, &sessionid, &request);
    return match filecontents {
        Some(content) => create_response(sessionid, HttpStatusCode::Ok, content),
        None => create_response(&sessionid, HttpStatusCode::NotFound, "".to_string()),
    };
}

//  Makes a response based on the response type
fn create_response(sessionid: &Uuid, http_status_code: HttpStatusCode, responsebody: String) -> HttpResponse {

    log(LogLevel::Trace, &SERVERID.lock().unwrap(), sessionid, format!("Sending {} response. Body:{}", http_status_code, responsebody).as_str());

    let mut response = HttpResponse::new();
    response.head.status = http_status_code;

    if !responsebody.is_empty() {
        response.head.headers.insert("Content-Type".to_string(), "text/html".to_string());
        response.body = responsebody;
    }

    return response;
}

//  Parses the request and returns the response
fn parse_request(sessionid: &Uuid, root: &str, request: &str) -> HttpResponse {

    let httprequest = request.parse::<HttpRequest>();

    if httprequest.is_err() { return create_response(&sessionid, HttpStatusCode::NotAcceptable, "".to_string()); }

    let httprequest = httprequest.unwrap();

    match httprequest.method {
        HttpMethod::GET => {
            log(LogLevel::Trace, &SERVERID.lock().unwrap(), sessionid, format!("{} {} {}", &httprequest.method, &httprequest.path, &httprequest.version).as_str());

            match httprequest.path {
                _ if httprequest.path.starts_with("/echo") => return create_response(&sessionid, HttpStatusCode::Ok, httprequest.path[6..].to_string()),
                _ if !httprequest.path.starts_with("/echo") => return get_path_response(&sessionid, &root, &httprequest.path),
                _ => return create_response(&sessionid, HttpStatusCode::NotFound, "".to_string()),
            };
        },
        _ => return create_response(&sessionid, HttpStatusCode::NotImplemented, "".to_string()),
    }
}

//  Serializes the response to the stream
fn serialize_response(stream: &mut TcpStream, response: &HttpResponse) {

    writeln!(stream, "{} {} {}", response.head.version, response.head.status, response.head.status.to_string()).unwrap();

    for (key, value) in response.head.headers.iter() {
        writeln!(stream, "{}: {}", key, value.to_string()).unwrap();
    }

    writeln!(stream).unwrap();
    stream.write_all(&response.body.as_bytes()).unwrap();
}

//  Handles the incoming connection
//  This is spun off into a thread so that multiple connections can be handled at once
fn handle_incoming_connection(sessionid: &Uuid, root: &String, mut stream: &mut TcpStream) {
    log(LogLevel::Trace, &SERVERID.lock().unwrap(), &sessionid, format!("Connection from {}", &stream.peer_addr().unwrap()).as_str());

    let mut buf_reader = BufReader::new(&mut stream);
    let buffer = buf_reader.fill_buf().unwrap();
    let request = String::from_utf8(buffer.to_vec());
    let response = match request {
        Ok(_) => parse_request(&sessionid, &root,&request.unwrap()),
        Err(_) => {
            log(LogLevel::Error, &SERVERID.lock().unwrap(), &sessionid, format!("An error occurred, terminating connection with {}", &stream.peer_addr().unwrap()).as_str());
            create_response(&sessionid, HttpStatusCode::InternalServerError, "An error occurred with parsing the request, terminating connection".to_string())
        },
    };

    serialize_response(stream, &response);
    stream.flush().unwrap();
}

//  Parses the path and returns the full path
fn parse_path(root: &str, path: &str) -> String {
    let mut path = format!("{}{}", &root, &path);

    if path.ends_with("/") { path.push_str("index.html"); }

    return path;
}

//  Reads the file and returns the contents
fn read_file(root: &str, sessionid: &Uuid, path: &str) -> Option<String> {
    let path = parse_path(&root, &path);

    log(LogLevel::Trace, &SERVERID.lock().unwrap(), sessionid, format!("Looking for file:{}", &path).as_str());

    return std::fs::read_to_string(&path).ok();
}

//  Starts the web server and returns the root and listener
fn start_web_server() -> (String, TcpListener, ThreadPool) {
    argparser::check_for_help_arg();
    let root = argparser::get_root_arg();
    let ip = argparser::get_ip_from_args();
    let port = argparser::get_port_from_args();
    let threadpoolsize = argparser::get_threadpoolsize_from_args();

    if root.is_err() { throw_error(root.as_ref().err().unwrap().as_str()); }
    if ip.is_err() { throw_error(ip.as_ref().err().unwrap().as_str()); }
    if port.is_err() { throw_error(port.as_ref().err().unwrap().as_str()); }
    if threadpoolsize.is_err() { throw_error(threadpoolsize.as_ref().err().unwrap().as_str()); }

    let root = root.unwrap();
    let ip = ip.unwrap();
    let port = port.unwrap();
    let threadpoolsize = threadpoolsize.unwrap();

    log(LogLevel::Info, &SERVERID.lock().unwrap(), &Uuid::nil(), format!("Started web server on ip:{} port:{} root:{}", &ip, &port, &root).as_str());

    let socketaddress = format!("{}:{}", &ip, &port);
    let threadpool = ThreadPool::new(threadpoolsize, SERVERID.lock().unwrap().clone());
    if threadpool.is_err() {
        log(LogLevel::Error, &SERVERID.lock().unwrap(), &Uuid::nil(), format!("Error: {:?}", threadpool.err().unwrap()).as_str());
        std::process::exit(1);
    }

    return (root, TcpListener::bind(&socketaddress).unwrap(), threadpool.unwrap());
}

//  Throws an error and exits the program
//  Used for invalid arguments
fn throw_error(e: &str) {
    log(LogLevel::Error, &SERVERID.lock().unwrap(), &Uuid::nil(), format!("Error: {}", e).as_str());
    std::process::exit(-1);
}

//  Clones the variables and passes them to the closure
//  This is used to pass variables that cannont be copied to a thread
#[macro_export]
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn main() {
    let (root, listener, threadpool) = start_web_server();


    for stream in listener.incoming() {
        if stream.is_err() {
            log(LogLevel::Error, &SERVERID.lock().unwrap(), &Uuid::nil(), format!("Error: {}", stream.err().unwrap()).as_str());
            continue;
        }

        let sessionid = Uuid::new_v4();

        enclose!((root) {
            threadpool.execute(move || { handle_incoming_connection(&sessionid, &root, &mut stream.unwrap()); });
        });
    }
}
