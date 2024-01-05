use uuid::Uuid;
use log::*;
use std::{
    net::{TcpListener, TcpStream},
    io::{Write, BufReader, BufRead}
};

mod argparser;
mod threads;
mod http;

use http::{HttpMethod, HttpRequest, HttpStatusCode, HttpResponse};
use threads::ThreadPool;

fn get_path_response(sessionid: &Uuid, root:&str, request: &str) -> HttpResponse {
    info!("{},Getting path response for {}", sessionid, request);
    let filecontents = read_file(&root, &sessionid, &request);
    return match filecontents {
        Some(content) => create_response(sessionid, HttpStatusCode::Ok, content),
        None => create_response(&sessionid, HttpStatusCode::NotFound, "".to_string()),
    };
}

fn create_response(sessionid: &Uuid, http_status_code: HttpStatusCode, responsebody: String) -> HttpResponse {

    info!("{},Sending {} response. Body:{}", sessionid, http_status_code, responsebody);

    let mut response = HttpResponse::new();
    response.head.status = http_status_code;

    if !responsebody.is_empty() {
        response.head.headers.insert("Content-Type".to_string(), "text/html".to_string());
        response.body = responsebody;
    }

    return response;
}

fn parse_request(sessionid: &Uuid, root: &str, request: &str) -> HttpResponse {

    let httprequest = request.parse::<HttpRequest>();

    if httprequest.is_err() { return create_response(&sessionid, HttpStatusCode::NotAcceptable, "".to_string()); }

    let httprequest = httprequest.unwrap();

    match httprequest.method {
        HttpMethod::GET => {
            info!("{},{} {} {}", sessionid, &httprequest.method, &httprequest.path, &httprequest.version);

            match httprequest.path {
                _ if httprequest.path.starts_with("/echo") => return create_response(&sessionid, HttpStatusCode::Ok, httprequest.path[6..].to_string()),
                _ if !httprequest.path.starts_with("/echo") => return get_path_response(&sessionid, &root, &httprequest.path),
                _ => return create_response(&sessionid, HttpStatusCode::NotFound, "".to_string()),
            };
        },
        _ => return create_response(&sessionid, HttpStatusCode::NotImplemented, "".to_string()),
    }
}

fn serialize_response(stream: &mut TcpStream, response: &HttpResponse) {

    writeln!(stream, "{} {} {}", response.head.version, response.head.status, response.head.status.to_string()).unwrap();

    for (key, value) in response.head.headers.iter() {
        writeln!(stream, "{}: {}", key, value.to_string()).unwrap();
    }

    writeln!(stream).unwrap();
    stream.write_all(&response.body.as_bytes()).unwrap();
}

fn handle_incoming_connection(sessionid: &Uuid, root: &String, mut stream: &mut TcpStream) {
    info!("{},Connection from {}", sessionid, &stream.peer_addr().unwrap());

    let mut buf_reader = BufReader::new(&mut stream);
    let buffer = buf_reader.fill_buf().unwrap();
    let request = String::from_utf8(buffer.to_vec());
    let response = match request {
        Ok(_) => parse_request(&sessionid, &root,&request.unwrap()),
        Err(_) => {
            error!("{},An error occurred, terminating connection with {}", sessionid, &stream.peer_addr().unwrap());
            create_response(&sessionid, HttpStatusCode::InternalServerError, "An error occurred with parsing the request, terminating connection".to_string())
        },
    };

    serialize_response(stream, &response);
    stream.flush().unwrap();
}

fn parse_path(root: &str, path: &str) -> String {
    let mut path = format!("{}{}", &root, &path);

    if path.ends_with("/") { path.push_str("index.html"); }

    return path;
}

fn read_file(root: &str, sessionid: &Uuid, path: &str) -> Option<String> {
    let path = parse_path(&root, &path);

    info!("{},Looking for file:{}", sessionid, &path);

    return std::fs::read_to_string(&path).ok();
}

fn parse_arguments() -> Result<(String, String, u16, usize), String> {
    argparser::check_for_help_arg();

    let loglevel = argparser::get_loglevel_from_args().map_err(|e| e.to_string())?;
    env_logger::builder().filter_level(loglevel).init();

    let root = argparser::get_root_arg().map_err(|e| e.to_string())?;
    let ip = argparser::get_ip_from_args().map_err(|e| e.to_string())?;
    let port = argparser::get_port_from_args().map_err(|e| e.to_string())?;
    let threadpoolsize = argparser::get_threadpoolsize_from_args().map_err(|e| e.to_string())?;

    Ok((root, ip, port, threadpoolsize))
}

fn start_web_server() -> (String, TcpListener, ThreadPool) {
    let (root, ip, port, threadpoolsize) = match parse_arguments() {
        Ok(args) => args,
        Err(e) => {
            throw_fatal_error(&e);
            std::process::exit(-1);
        }
    };

    info!("{},Started web server on ip:{} port:{} root:{}", Uuid::nil(), &ip, &port, &root);

    let socketaddress = format!("{}:{}", &ip, &port);
    let threadpool = ThreadPool::new(threadpoolsize).unwrap_or_else(|e| {
        error!("{},Error: {:?}", Uuid::nil(), e);
        std::process::exit(1);
    });

    (root, TcpListener::bind(&socketaddress).unwrap(), threadpool)
}

//  Throws an error and exits the program
//  Used for invalid arguments
fn throw_fatal_error(e: &str) {
    error!("{},Error: {}", Uuid::nil(), e);
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
            error!("{},Error: {}", Uuid::nil(), stream.err().unwrap());
            continue;
        }

        let sessionid = Uuid::new_v4();

        enclose!((root) {
            threadpool.execute(move || { handle_incoming_connection(&sessionid, &root, &mut stream.unwrap()); });
        });
    }
}