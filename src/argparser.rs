use std::env;

//  Parameters
const ROOTPARAMETER: &'static str = "--root";
const IPPARAMETER: &'static str = "--ip";
const PORTPARAMETER: &'static str = "--port";
const DEFAULTIP: &'static str = "127.0.0.1";
const DEFAULTPORT: &'static str = "4221";

//  The help text to display when --help is given
static HELP: &'static str = "\
Simple web server.\n\
--root\t\troot directory to serve files from\n\
--ip\t\tip address to listen on\n\
--port\t\tport to listen on\n\
--help\t\tdisplay this help and exit";

//  Checks for the --help argument and displays the help text if found
pub fn check_for_help_arg() {
    if env::args().any(|x| x == "--help") {
        println!("{}", &HELP);
        std::process::exit(0);
    }
}

//  Gets the --root argument and returns the value if found
pub fn get_root_arg() -> String {
    if env::args().any(|x| x == ROOTPARAMETER) {
        let result = get_parameter_variable_from_args("--root");
        match result {
            Some(x) => return x,
            None => throw_error("Root parameter given but no value"),
        }
    }

    throw_error("Root parameter not given");

    //  This is here to satisfy the compiler. It will never be reached
    //  TODO: Find a better way to do this
    return "".to_string();
}

//  Gets the --ip argument and returns the value if found
//  If the --ip argument is not found then the default ip is returned
pub fn get_ip_from_args() -> String {
    if env::args().any(|x| x == IPPARAMETER) {
        let result = get_parameter_variable_from_args("--ip");
        match result {
            Some(x) => return x,
            None => throw_error("IP parameter given but no value"),
        }
    }

    return DEFAULTIP.to_string();
}

//  Gets the --port argument and returns the value if found
//  If the --port argument is not found then the default port is returned
pub fn get_port_from_args() -> String {
    if env::args().any(|x| x == PORTPARAMETER) {
        let result = get_parameter_variable_from_args("--port");
        match result {
            Some(x) => return x,
            None => throw_error("Port parameter given but no value"),
        }
    }

    return DEFAULTPORT.to_string();
}

//  Gets the value of a parameter from the command line arguments
//  splits the arguments into a vector and then finds the index of the parameter
//  if the parameter is found then the next value is returned
fn get_parameter_variable_from_args(parameter: &str) -> Option<String> {
    let args = env::args().collect::<Vec<String>>();
    let index = args.iter().position(|x| x == parameter);
    if index.is_none() || index.unwrap() + 1 >= env::args().count() { throw_error(format!("Value for Parameter {} not found", parameter).as_str()); }
    return args.get(index.unwrap() + 1).cloned();
}

//  Throws an error and exits the program
//  Used for invalid arguments
fn throw_error(e: &str) {
    println!("Error: {}", e);
    println!("{}", &HELP);
    std::process::exit(-1);
}
