#![feature(plugin)]
#![plugin(docopt_macros)]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate docopt;
extern crate rustc_serialize;
extern crate ini;
extern crate libc;
#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::DrainExt;
use std::io::Write;
use std::str::FromStr;
use std::io::BufRead;

docopt!(Args, "
Usage:
    rusty-gopher  serve [<config>]
    rusty-gopher  genconfig [<path>]
    rusty-gopher  -h | --help
    rusty-gopher  --version

Options:
    -h --help   Show this screen.
    --version   Show version.
");

const DEFAULT_MASTER_CONFIG: &'static str = "/etc/rusty_gopher.cfg";

const DEFAULT_ROOT_DIR: &'static str = "/var/gopher";
const DEFAULT_USER: &'static str = "gopher";
const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0:70";

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    
    if args.flag_version {
        println!("{} version: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        std::process::exit(libc::EXIT_SUCCESS);
    }
    
    if args.cmd_genconfig {
        let mut conf = ini::Ini::new();
        conf.with_section(Some("General")).
            set("rootdir", DEFAULT_ROOT_DIR).
            set("user", DEFAULT_USER).
            set("listento", DEFAULT_LISTEN_ADDRESS);

            if args.arg_path.is_empty() {
                match conf.write_to_file(DEFAULT_MASTER_CONFIG) {
                    Ok(_) => println!("Configuration file written.\nPlease check {}", DEFAULT_MASTER_CONFIG),
                    Err(e) => {
                        println!("Error writing configuration file to: {}\nError: {}", DEFAULT_MASTER_CONFIG, e);
                        std::process::exit(libc::EXIT_FAILURE);
                    }
                }
            } else {
                match conf.write_to_file(&args.arg_path) {
                    Ok(_) => println!("Configuration file written.\nPlease check {}", args.arg_path),
                    Err(e) => {
                        println!("Error writing configuration file to: {}\nError: {}", args.arg_path, e);
                        std::process::exit(libc::EXIT_FAILURE);
                    }
                }
            }
        std::process::exit(libc::EXIT_SUCCESS);
    }

    let config: ini::Ini; 
    if args.arg_config.is_empty() {
        match ini::Ini::load_from_file(DEFAULT_MASTER_CONFIG) { 
            Ok(f) => config = f,
            Err(e) => {
                println!("Error opening configuration file at: {}\nError: {}", DEFAULT_MASTER_CONFIG, e);
                std::process::exit(libc::EXIT_FAILURE);
            }
        }
    } else {
        match ini::Ini::load_from_file(&args.arg_config) { 
            Ok(f) => config = f,
            Err(e) => {
                println!("Error opening configuration file at: {}\nError: {}", args.arg_config, e);
                std::process::exit(libc::EXIT_FAILURE);
            }
        }
    }

    let generalconfig = config.section(Some("General"));
    let addr: std::net::SocketAddr;
    let mut user = std::string::String::new();
    let mut root = std::string::String::new();

    match generalconfig {
        Some(g) => {
            match g.get("listento") {
                Some(a) => {
                    match std::net::SocketAddr::from_str(a) {
                        Ok(ad) => addr = ad,
                        Err(e) => {
                            println!("Error reading \"listento\" value.\nPlease check your config file\nError: {}", e);
                            std::process::exit(libc::EXIT_FAILURE);
                        }
                    }
                }
                None => {
                    println!("Error reading \"listento\" value.\nPlease check your config file.");
                    std::process::exit(libc::EXIT_FAILURE);
                }
            }
            match g.get("user") {
                Some(u) => user.push_str(u),
                None => {
                    println!("Error reading \"user\" value.\nPlease check your config file.");
                    std::process::exit(libc::EXIT_FAILURE);
                }
            }
            match g.get("rootdir") {
                Some(r) => root.push_str(r),
                None => {
                    println!("Error reading \"root\" value.\nPlease check your config file.");
                    std::process::exit(libc::EXIT_FAILURE);
                }
            }
        }
        None => {
            println!("Error reading configuration values.\nYour config file seems corrupted\n/
            You can generate a new one by typing: {} genconfig", env!("CARGO_PKG_NAME"));
            std::process::exit(libc::EXIT_FAILURE);
        }
    }
    
    let rtlog = slog::Logger::root(
        slog_term::streamer().full().build().
        fuse(), o!(env!("CARGO_PKG_NAME") => env!("CARGO_PKG_VERSION")));

    match listen_and_serve(addr, root, user, rtlog) {
        Some(_) => std::process::exit(libc::EXIT_FAILURE),
        None => std::process::exit(libc::EXIT_SUCCESS),
    }

}

fn listen_and_serve(addr: std::net::SocketAddr, root: std::string::String, 
                    user: std::string::String, rtlog: slog::Logger) -> Option<std::io::Error> {
    match std::net::TcpListener::bind(addr) {
        Ok(listener) => {
            let llog = rtlog.new(o!("local address" => format!("{}", listener.local_addr().unwrap())));
            info!(llog, "listening");
            
            match get_uid_by_name(user.clone()) {
                Ok(desired_uid) => {
                    if desired_uid != get_uid() {
                        match switch_to_uid(desired_uid) {
                            Ok(uid) => info!(llog, "user switch successfull"; "current user" => uid),
                            Err(e) => {
                                crit!(llog, e; "desired uid" => desired_uid, "current uid" => get_uid());
                                return Some(std::io::Error::new(std::io::ErrorKind::Other, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    crit!(llog, e; "desired user" => user);
                    return Some(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            }
            
            for client in listener.incoming() {
                match client {
                    Ok(mut c) => {
                        let clog = llog.new(o!("peer address" => format!("{}", c.peer_addr().unwrap())));
                        info!(clog, "new connection received");

                        let mut reader = std::io::BufReader::new(c.try_clone().unwrap());
                        let mut buf = std::string::String::new();
                        match reader.read_line(&mut buf) {
                            Ok(input) => {
                                debug!(clog, "got input"; "bytes read" => input);

                                match parse_input(buf) {
                                    Ok(request) => {
                                        match request {
                                            GopherMessage::ListDir(selector) => {
                                                info!(clog, "got directory list request"; "selector" => selector);
                                                match get_directory_listing(root.clone(), selector) {
                                                    Ok(listing) => {
                                                        for l in listing {
                                                            c.write_fmt(format_args!("{}{}\t{}\t{}\t{}\r\n",
                                                                                 l.gType.to_type_string(),
                                                                                 l.description,
                                                                                 l.selector,
                                                                                 l.host,
                                                                                 l.port));

                                                        }
                                                    }
                                                    Err(e) => error!(clog, e),
                                                }
                                            }
                                            GopherMessage::SearchDir(selector, search_string) => {
                                                debug!(clog, "got search request"; "selector" => selector);

                                            }

                                        }
                                    }
                                    Err(e) => error!(clog, e),
                                }
                            }
                            Err(e) => error!(clog, "error reading input: {}", e),
                        }
                    }
                    Err(e) => {
                        error!(rtlog, "error handling client information {}", e);
                    }
                }
            }
        }
        Err(e) => {
            crit!(rtlog, "error binding to {} failed {}", addr, e);
            return Some(e);
        }
    }
    None
}

enum GopherMessage {
    ListDir(std::string::String),
    SearchDir(std::string::String, Vec<Query>),
}

enum Query {
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
    Not(Box<Query>),
    SearchString(std::string::String),
}

enum GopherType {
    Informational,
    Gif,
    Directory,
    File,
    BinaryFile,
}

impl GopherType {
    fn to_type_string(&self) -> std::string::String {
        match *self {
            GopherType::Informational => "i".to_string(),
            GopherType::Gif => "g".to_string(),
            GopherType::Directory => "1".to_string(),
            GopherType::File => "0".to_string(),
            GopherType::BinaryFile => "9".to_string(),
        }
    }
}

struct DirectoryEntry {
    gType: GopherType,
    description: std::string::String,
    selector: std::string::String,
    host: std::string::String,
    port: u16,
}

fn get_directory_listing(root: std::string::String, 
                         request: std::string::String) -> Result<Vec<DirectoryEntry>, std::io::Error> { 
    match std::fs::read_dir(root + &request){
        Ok(rd) => {
            let mut res: Vec<DirectoryEntry> = std::vec::Vec::new();
            for possible_entry in rd {
                match possible_entry {
                    Ok(entry) => {
                        if entry.file_type().unwrap().is_dir() {
                            let e = DirectoryEntry{gType: GopherType::Directory,
                                description: format!("{}", entry.file_name().into_string().unwrap()), //TODO
                                selector: format!("{}", entry.path().to_str().expect("selector has to be valid utf8").to_string()),
                                host: get_host_name(),
                                port: 7070, //TODO
                            };
                            res.push(e);
                        } else if entry.file_type().unwrap().is_file() {
                            let e = DirectoryEntry{
                                gType: GopherType::BinaryFile,
                                description: format!("{}", entry.file_name().into_string().unwrap()),
                                selector: format!("{}", entry.path().to_str().expect("selector has to be valid utf8").to_string()),
                                host: get_host_name(),
                                port: 7070,
                            };
                            res.push(e);

                        }
                    }
                    Err(e) => {}
                }
            }
            Ok(res)
        }

        Err(e) => {
              Err(e)
        }
    }
}

fn get_host_name() -> std::string::String {
    let hostnamelen: libc::c_long;
    unsafe {
        hostnamelen = libc::sysconf(libc::_SC_HOST_NAME_MAX) + 1; // +1 for the trailing \0
    }
    let mut hostnamevec = vec![0 as u8; hostnamelen as usize];

    unsafe {
        libc::gethostname(hostnamevec.as_mut_ptr() as *mut i8, hostnamelen as usize);
    }

    let mut resultvec: Vec<u8> = std::vec::Vec::new();
    for c in hostnamevec {
        if c == 0 {
            break;
        } else {
            resultvec.push(c);
        }
    }
    let mut hostname = std::ffi::CString::new(resultvec).unwrap();

    unsafe {
        let hints = libc::addrinfo{
            ai_family: libc::AF_UNSPEC,
            ai_socktype: libc::SOCK_STREAM,
            ai_flags: libc::AI_CANONNAME,
            ai_addr: 0 as *mut libc::sockaddr,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_canonname: 0 as *mut i8,
            ai_next: 0 as *mut libc::addrinfo,
        };
        let mut gai_info: *mut libc::addrinfo = 0 as *mut libc::addrinfo;
        let gai_service = std::ffi::CString::new("gopher").unwrap();
        let res = libc::getaddrinfo(hostname.as_ptr(), gai_service.as_ptr(), &hints, &mut gai_info);
        if res != 0 {
            panic!("{}", res);
        }
        if gai_info != 0 as *mut libc::addrinfo {
            // from_raw takes ownership of the pointer
            let temp = std::ffi::CString::from_raw((*gai_info).ai_canonname);

            hostname = temp.clone();

            // into_raw releases the ownership of the pointer as it is managed by C code
            temp.into_raw();
        }
        libc::freeaddrinfo(gai_info);
    }
    hostname.into_string().unwrap()
}

fn parse_input(input: std::string::String) -> Result<GopherMessage, &'static str> {
    match input.as_str() {
        "\r\n" => Ok(GopherMessage::ListDir("/".to_string())),
        _ => {
            if input.is_empty() {
                return Err("Invalid request");
            }
            let selector_and_search: Vec<&str> = input.split("\t").collect();
            if  selector_and_search.len() < 2{
                return Ok(GopherMessage::ListDir(selector_and_search[0].to_string()));
            }
            let mut selector = std::string::String::new();
            let mut search_result = std::vec::Vec::<Query>::new();
            let mut out_queue = std::vec::Vec::<std::string::String>::new();
            let mut op_queue = std::vec::Vec::<Query>::new();

            for s in selector_and_search[1].split(" ").into_iter() {
                match s {
                    "and" => {}
                    "or" => {}
                    "not" => {}
                    &_ => {}
                }


                //search.push(Query::SearchString(s.to_string()));
                //teststr: foo not bar -> foo bar not
                //foo and bar not baz -> foo bar and baz not
                //not baz -> baz not
                //and bar (should fail)
                //foo and bar or boo and char -> foo bar and boo char and or
                //foo and not bar or boo and char -> foo bar not and boo char and or
            }
            //TODO: iterate over second half to parse all logical operators
            return Ok(GopherMessage::SearchDir(selector, search_result));
        }
    }
}

fn switch_to_uid(uid: libc::uid_t) -> Result<libc::uid_t, &'static str> {
    let olduid = get_uid();
    if olduid == uid {
        return Ok(olduid);
    }

    let setuidres;
    unsafe {
        setuidres = libc::setuid(uid);
    }
    if setuidres != 0 {
        Err("Error setting uid")
    } else {
        Ok(get_uid())
    }
}

fn get_uid_by_name(user: std::string::String) -> Result<libc::uid_t, &'static str> {
    let uid;
    unsafe {
        let desired_user_name = std::ffi::CString::new(user.clone()).unwrap();
        let desired_user_name_ptr = desired_user_name.as_ptr();
        let desired_user_passwd = libc::getpwnam(desired_user_name_ptr);

        if desired_user_passwd.is_null() {
            return Err("User not found");
        }
        uid = (*desired_user_passwd).pw_uid;
    }
    Ok(uid)
}

fn get_uid() -> libc::uid_t {
    let uid;
    unsafe {
        uid = libc::getuid();
    }
    uid
}
