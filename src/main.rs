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
                                if buf == "\r\n" { 
                                    info!(clog, "got directory list request");
                                    match std::fs::read_dir(root.clone()) {
                                        Ok(rd) => {
                                            for possible_entry in rd {
                                                match possible_entry {
                                                    Ok(entry) => {
                                                        debug!(clog, "Found directory entry"; "entry" => format!("{:?}", entry.path()));
                                                        //TODO: check return value
                                                        //TODO: use libc::_SC_HOST_NAME_MAX
                                                        //TODO: use gethostname
                                                        //TODO: use .collect() and search for
                                                        //.menuinfo entries
                                                        //https://stackoverflow.com/questions/504810/how-do-i-find-the-current-machines-full-hostname-in-c-hostname-and-domain-info
                                                        c.write_fmt(format_args!("0DESCRIPTION\t{:?}\tOURURL\tOURPORT\r\n", entry.path()));
                                                    }
                                                    Err(e) => {
                                                        warn!(clog, "Could not read directory entry"; "entry" => format!("{}", e));
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            crit!(clog, "error opening root directory. Check your config file and access privileges"; "error" => format!("{}", e));
                                            std::process::exit(libc::EXIT_SUCCESS); //TODO
                                        }
                                    }
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

            None
        }
        Err(e) => {
            crit!(rtlog, "error binding to {} failed {}", addr, e);
            Some(e)
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
