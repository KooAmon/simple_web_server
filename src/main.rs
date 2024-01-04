use chrono::prelude::*;
use http::Response;
use uuid::Uuid;
use lazy_static::lazy_static;
use std::{
    net::{TcpListener, TcpStream},
    io::{Read, Write},
    sync::Mutex
};

mod argparser;
mod threads;
use threads::ThreadPool;

//  Used to contain the response type and content type
enum ResponseType {
    Ok(ContentType),
    ServerError(u16),
    NotFound,
}

//  Used to contain the content type and content
enum ContentType {
    Html(String),
    Plain(String)
}

lazy_static!{
    //  The server id. This is used to identify the server run instance in the logs
    static ref SERVERID: Mutex<Uuid> = Mutex::new(Uuid::new_v4());
}

//  Gets the echo response for the given request
fn get_echo_response(sessionid: &Uuid, request: &str) -> Response<String> {
    log!("{},Sending echo response for {}", &sessionid, request);
    return make_response(ResponseType::Ok(ContentType::Plain(request.to_string())));
}

//  Gets the path response for the given request
fn get_path_response(sessionid: &Uuid, root:&str, request: &str) -> Response<String> {
    log!("{},Sending path response for {}", &sessionid, request);
    let filecontents = read_file(&root, &sessionid, &request);
    return match filecontents {
        Some(content) => make_response(ResponseType::Ok(ContentType::Html(content))),
        None => get_404_response(&sessionid),
    };
}

//  Gets the 404 response
fn get_404_response(sessionid: &Uuid) -> Response<String> {
    log!("{},Sending 404 response", &sessionid);
    return make_response(ResponseType::NotFound);
}

//  Gets the 500 response
fn get_5xx_response(sessionid: &Uuid, code: u16) -> Response<String> {
    log!("{},Sending {} response", &sessionid, code);
    return make_response(ResponseType::ServerError(code));
}

//  Gets the user agent response for the given request
fn get_user_agent_response(sessionid: &Uuid, request: &str) -> Response<String> {
    log!("{},Sending user agent response for {}", &sessionid, request);
    let user_agent = request.split(":").collect::<Vec<&str>>()[1];
    return make_response(ResponseType::Ok(ContentType::Html(user_agent.to_string())));
}

//  Makes a response based on the response type
fn make_response(responsetype: ResponseType) -> Response<String> {

    match responsetype {
        ResponseType::Ok(contenttype) => {
            match contenttype {
                ContentType::Html(content) => {
                    return Response::builder()
                        .status(200)
                        .header("Content-Type", "text/html")
                        .body(content)
                        .unwrap();
                },
                ContentType::Plain(content) => {
                    return Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .body(content)
                        .unwrap();
                }
            }
        },
        ResponseType::ServerError(code) => {
            return Response::builder()
                .status(code)
                .body("".to_string())
                .unwrap();
        },
        ResponseType::NotFound => {
            return Response::builder()
                .status(404)
                .body("".to_string())
                .unwrap();
        }
    }
}

//  Parses the request and returns the response
fn parse_request(sessionid: &Uuid, root: &str, request: &Vec<&str>) -> Response<String> {
    if let Some(first_line) = request.get(0) {
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap();

        //  Only GET is supported
        if method != "GET" { return get_5xx_response(&sessionid, 501); }

        let path = parts.next().unwrap();
        let version = parts.next().unwrap();
        let useragent = request.get(5).unwrap();
        log!("{},{} {} {} {}", &sessionid, &method, &path, &version, &useragent);

        let response = match path {
            "/user_agent" =>  get_user_agent_response(&sessionid, &useragent),
            _ if path.starts_with("/echo") => get_echo_response(&sessionid, &path[6..]),
            _ if !path.starts_with("/echo") => get_path_response(&sessionid, &root, &path),
            _ => get_404_response(&sessionid),
        };

        return response;
    } else {
        return get_404_response(&sessionid);
    }
}

//  Serializes the response to the stream
fn serialize_response(stream: &mut TcpStream, response: &Response<String>) {
    let (parts, body) = response.clone().into_parts();
    writeln!(stream, "{:?} {} {}", parts.version, parts.status, parts.status.canonical_reason().unwrap()).unwrap();

    for (key, value) in parts.headers.iter() {
        writeln!(stream, "{}: {}", key, value.to_str().unwrap()).unwrap();
    }

    writeln!(stream).unwrap();
    stream.write_all(&body.as_bytes()).unwrap();
}

//  Handles the incoming connection
//  This is spun off into a thread so that multiple connections can be handled at once
fn handle_incoming_connection(root: &String, stream: &mut TcpStream) {

    let sessionid = Uuid::new_v4();
    log!("{},Connection from {}", &sessionid, &stream.peer_addr().unwrap());

    let mut buffer = [0; 1024];

    let response = match &stream.read(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);
            let required_lines: Vec<&str> = request.lines().collect();
            parse_request(&sessionid, &root,&required_lines)
        },
        Err(_) => {
            log!("{}, An error occurred, terminating connection with {}", &sessionid, &stream.peer_addr().unwrap());
            Response::builder()
                .status(500)
                .body("An error occurred, terminating connection".to_string())
                .unwrap()
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

    log!("{},Looking for file:{}",&sessionid, &path);

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

    log!(",Started web server on ip:{} port:{} root:{}", &ip, &port, &root);

    let socketaddress = format!("{}:{}", &ip, &port);
    let threadpool = ThreadPool::new(threadpoolsize);
    if threadpool.is_err() {
        log!(",Error: {:?}", threadpool.err().unwrap());
        std::process::exit(1);
    }

    return (root, TcpListener::bind(&socketaddress).unwrap(), threadpool.unwrap());
}

//  Throws an error and exits the program
//  Used for invalid arguments
fn throw_error(e: &str) {
    log!("Error: {}", e);
    std::process::exit(-1);
}

//  Logs the message to the console
//  TODO: implement better logging
#[macro_export]
macro_rules! log {
    //  UTC time, server id, session id, message
    ($($arg:tt)*) => ({
        println!("{},{},{}", Utc::now(), SERVERID.lock().unwrap(), format!($($arg)*));
    })
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
            log!(",Error: {}", stream.err().unwrap());
            continue;
        }

        enclose!((root) {
            threadpool.execute(move || {
                handle_incoming_connection(&root, &mut stream.unwrap());
            });
        });
    }
}
