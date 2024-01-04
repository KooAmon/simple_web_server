use std::{env, fmt::Debug, str::FromStr};

//  Parameters
const ROOTPARAMETER: &str = "--root";
const IPPARAMETER: &str = "--ip";
const PORTPARAMETER: &str = "--port";
const THREADPOOLSIZEPARAMETER: &str = "--threadpoolsize";
const DEFAULTIP: &str = "127.0.0.1";
const DEFAULTPORT: u16 = 4221;
const DEFAULTTHREADPOOLSIZE: usize = 4;

//  The help text to display when --help is given
static HELP: &'static str = "\
Simple web server.\n\
--root\t\t\troot directory to serve files from\n\
--ip\t\t\tip address to listen on\n\
--port\t\t\tport to listen on\n\
--threadpoolsize\tsize of the threadpool\n\
--help\t\t\tdisplay this help and exit";

//  Checks for the --help argument and displays the help text if found
pub fn check_for_help_arg() {
    if env::args().any(|x| x == "--help") {
        println!("{}", &HELP);
        std::process::exit(0);
    }
}

//  Gets the --root argument and returns the value if found
pub fn get_root_arg() -> Result<String, String> {
    if env::args().any(|x| x == ROOTPARAMETER) {
        return get_parameter_variable_from_args::<String>("--root", "Root parameter given but not a string");
    }

    return Err("Root parameter not found".to_string());
}

//  Gets the --ip argument and returns the value if found
//  If the --ip argument is not found then the default ip is returned
pub fn get_ip_from_args() -> Result<String, String> {
    if env::args().any(|x| x == IPPARAMETER) {
        return get_parameter_variable_from_args::<String>("--ip", "Ip parameter given but not a string");
    }

    return Ok(DEFAULTIP.to_string());
}

//  Gets the --port argument and returns the value if found
//  If the --port argument is not found then the default port is returned
pub fn get_port_from_args() -> Result<u16, String> {
    if env::args().any(|x| x == PORTPARAMETER) {
        return get_parameter_variable_from_args::<u16>("--port", "Port parameter given but not an int");
    }

    return  Ok(DEFAULTPORT);
}

pub fn get_threadpoolsize_from_args() -> Result<usize, String> {
    if env::args().any(|x| x == THREADPOOLSIZEPARAMETER) {
        return get_parameter_variable_from_args::<usize>("--threadpoolsize", "Threadpoolsize parameter given but not an usize");
    }

    return Ok(DEFAULTTHREADPOOLSIZE);
}

//  Gets the value of a parameter from the command line arguments
//  splits the arguments into a vector and then finds the index of the parameter
//  if the parameter is found then the next value is returned
fn get_parameter_variable_from_args<T: std::str::FromStr>(parameter: &str, errormessage: &str) -> Result<T, String> where <T as FromStr>::Err: Debug {
    let args = env::args().collect::<Vec<String>>();
    let index = args.iter().position(|x| x == parameter);

    if index.is_none() || index.unwrap() + 1 >= env::args().count() {
        return Err(format!("Parameter value not found {}\r\n{}", &parameter, &HELP));
    }


    let parametervalue = args.get(index.unwrap() + 1);
    match parametervalue {
        Some(x) => match x.parse::<T>(){
            Ok(x) => return Ok(x),
            Err(_) => return Err(format!("{}\r\n{}", errormessage.to_string(), &HELP)),
        },
        None => return Err(format!("Parameter value not found {}\r\n{}", &parameter, &HELP)),

    }
}
