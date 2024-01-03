use chrono::prelude::*;
use http::{Response};//, Request};
use uuid::Uuid;
use lazy_static::lazy_static;
use std::{
    env,
    net::{TcpListener, TcpStream},
    io::{Read, Write},
    sync::Mutex,
    thread,
};

enum ResponseType {
    Ok(ContentType),
    NotFound,
}

enum ContentType {
    Html(String),
    Plain(String)
}

const SOCKETADDRESS: &str = "127.0.0.1:4221";

static HELP: &'static str = "\
Simple web server.\n\
--root\t\troot directory to serve files from\n\
--help\t\tdisplay this help and exit";

lazy_static!{
    static ref SERVERID: Mutex<Uuid> = Mutex::new(Uuid::new_v4());
}

fn check_for_help_arg() {
    if env::args().any(|x| x == "--help") {
        println!("{}", &HELP);
        std::process::exit(0);
    }
}

fn check_for_root_arg() -> String {
    if env::args().any(|x| x == "--root") {
        let result = get_parameter_variable_from_args("--root");
        match result {
            Some(x) => return x,
            None => throw_error("Root parameter given but no value"),
        }
    }

    throw_error("Root parameter not given");
    return "".to_string();
}

fn get_parameter_variable_from_args(parameter: &str) -> Option<String> {
    let args = env::args().collect::<Vec<String>>();
    let index = args.iter().position(|x| x == parameter);
    if index.is_none() || index.unwrap() + 1 >= env::args().count() { throw_error(format!("Value for Parameter {} not found", parameter).as_str()); }
    return args.get(index.unwrap() + 1).cloned();
}

fn throw_error(e: &str) {
    println!("Error: {}", e);
    println!("{}", &HELP);
    std::process::exit(-1);
}

fn get_echo_response(sessionid: &Uuid, request: &str) -> Response<String> {
    log!("{},Sending echo response for {}", &sessionid, request);
    return make_response(ResponseType::Ok(ContentType::Plain(request.to_string())));
}

#[allow(dead_code)]
fn get_root_response(sessionid: &Uuid) -> Response<String> {
    log!("{},Sending root response", &sessionid);
    return make_response(ResponseType::Ok(ContentType::Html("".to_string())));
}

fn get_path_response(sessionid: &Uuid, root:&str, request: &str) -> Response<String> {
    log!("{},Sending path response for {}", &sessionid, request);
    let filecontents = read_file(&root, &sessionid, &request);
    return match filecontents {
        Some(content) => make_response(ResponseType::Ok(ContentType::Html(content))),
        None => get_404_response(&sessionid),
    };
}

fn get_404_response(sessionid: &Uuid) -> Response<String> {
    log!("{},Sending 404 response", &sessionid);
    return make_response(ResponseType::NotFound);
}

fn get_user_agent_response(sessionid: &Uuid, request: &str) -> Response<String> {
    log!("{},Sending user agent response for {}", &sessionid, request);
    let user_agent = request.split(":").collect::<Vec<&str>>()[1];
    return make_response(ResponseType::Ok(ContentType::Html(user_agent.to_string())));
}

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
        ResponseType::NotFound => {
            return Response::builder()
                .status(404)
                .body("".to_string())
                .unwrap();
        }
    }
}

fn parse_request(sessionid: &Uuid, root: &str, request: &Vec<&str>) -> Response<String> {
    if let Some(first_line) = request.get(0) {
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap();
        let path = parts.next().unwrap();
        let version = parts.next().unwrap();
        let useragent = request.get(5).unwrap();
        log!("{},{} {} {} {}", &sessionid, method, path, version, useragent);

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

fn serialize_response(stream: &mut TcpStream, response: &Response<String>) {
    let (parts, body) = response.clone().into_parts();
    writeln!(stream, "{:?} {} {}", parts.version, parts.status, parts.status.canonical_reason().unwrap()).unwrap();

    for (key, value) in parts.headers.iter() {
        writeln!(stream, "{}: {}", key, value.to_str().unwrap()).unwrap();
    }

    writeln!(stream).unwrap();
    stream.write_all(&body.as_bytes()).unwrap();
}

fn handle_incoming_connection(sessionid: &Uuid, root: &String, buffer : &[u8]) -> Response<String> {
    let request = String::from_utf8_lossy(buffer);
    let required_lines: Vec<&str> = request.lines().collect();
    return parse_request(&sessionid, &root,&required_lines);
}

fn parse_path(root: &str, path: &str) -> String {
    let mut path = format!("{}{}", &root, &path);

    if path.ends_with("/") { path.push_str("index.html"); }

    return path;
}

fn read_file(root: &str, sessionid: &Uuid, path: &str) -> Option<String> {
    let path = parse_path(&root, &path);

    log!("{},Looking for file:{}",&sessionid, &path);

    return std::fs::read_to_string(&path).ok();
}

fn start_web_server() -> (String, TcpListener) {
    check_for_help_arg();
    let root = check_for_root_arg();
    log!(",Started web server on socket:{} from:{}", &SOCKETADDRESS, &root);

    return (root, TcpListener::bind(&SOCKETADDRESS).unwrap());
}

#[macro_export]
macro_rules! log {
    //  UTC time, server id, session id, message
    ($($arg:tt)*) => ({
        println!("{},{},{}", Utc::now(), SERVERID.lock().unwrap(), format!($($arg)*));
    })
}

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
    let (root, listener) = start_web_server();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                enclose!((root) {
                    thread::spawn(move || {
                        let sessionid = Uuid::new_v4();
                        log!("{},Connection from {}", &sessionid, _stream.peer_addr().unwrap());

                        let mut buffer = [0; 1024];

                        let response = match _stream.read(&mut buffer) {
                            Ok(_) => handle_incoming_connection(&sessionid, &root, &buffer),
                            Err(_) => {
                                log!("{}, An error occurred, terminating connection with {}", &sessionid, _stream.peer_addr().unwrap());
                                Response::builder()
                                    .status(500)
                                    .body("".to_string())
                                    .unwrap()
                            },
                        };

                        serialize_response(&mut _stream, &response);
                        _stream.flush().unwrap();
                    });
                });
            },
            Err(e) => log!(",Error: {}", e),
        }
    }
}
