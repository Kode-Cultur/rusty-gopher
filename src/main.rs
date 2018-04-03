/*
 *    Copyright (C) 2016 Stefan Luecke
 *
 *    This program is free software: you can redistribute it and/or modify
 *    it under the terms of the GNU Affero General Public License as published
 *    by the Free Software Foundation, either version 3 of the License, or
 *    (at your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful,
 *    but WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *    GNU Affero General Public License for more details.
 *
 *    You should have received a copy of the GNU Affero General Public License
 *    along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 *    Authors: Stefan Luecke <glaxx@glaxx.net>
 */

#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate ini;
extern crate libc;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate nom;

pub mod gophermap;
pub mod libcwrapper;

use docopt::Docopt;
use gophermap::*;
use libcwrapper::*;
use std::io::BufRead;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;

const USAGE: &'static str = "
Usage:
    rusty-gopher  serve [<config>]
    rusty-gopher  genconfig [<config>]
    rusty-gopher  -h | --help
    rusty-gopher  --version

Options:
    -h --help   Show this screen.
    --version   Show version.
";

const DEFAULT_MASTER_CONFIG: &'static str = "/etc/rusty_gopher.cfg";

const DEFAULT_ROOT_DIR: &'static str = "/var/gopher";
const DEFAULT_USER: &'static str = "gopher";
const DEFAULT_LISTEN_ADDRESS: &'static str = "0.0.0.0:70";

#[derive(Serialize, Deserialize)]
struct Args {
    cmd_serve: bool,
    cmd_genconfig: bool,
    arg_config: Option<String>,
}

fn write_default_configfile(path: &String) {
    let mut conf = ini::Ini::new();
    conf.with_section(Some("General"))
        .set("rootdir", DEFAULT_ROOT_DIR)
        .set("user", DEFAULT_USER)
        .set("listento", DEFAULT_LISTEN_ADDRESS);

    match conf.write_to_file(path) {
        Ok(_) => {
            println!("Configuration file written.\nPlease check {:?}", path);
            std::process::exit(libc::EXIT_SUCCESS);
        }
        Err(e) => {
            println!(
                "Error writing configuration file to: {:?}\nError: {}",
                path, e
            );
            std::process::exit(libc::EXIT_FAILURE);
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let cfgpath = args.arg_config.unwrap_or(DEFAULT_MASTER_CONFIG.to_string());

    if args.cmd_genconfig {
        write_default_configfile(&cfgpath);
    }

    let config: ini::Ini;
    match ini::Ini::load_from_file(&cfgpath) {
        Ok(f) => config = f,
        Err(e) => {
            println!(
                "Error opening configuration file at: {}\nError: {}",
                cfgpath, e
            );
            std::process::exit(libc::EXIT_FAILURE);
        }
    }
    let generalconfig = config.section(Some("General"));
    let addr: std::net::SocketAddr;
    let mut user = String::new();
    let mut root = String::new();

    match generalconfig {
        Some(g) => {
            match g.get("listento") {
                Some(a) => match std::net::SocketAddr::from_str(a) {
                    Ok(ad) => addr = ad,
                    Err(e) => {
                        println!("Error reading \"listento\" value.\nPlease check your config file\nError: {}", e);
                        std::process::exit(libc::EXIT_FAILURE);
                    }
                },
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
            println!(
                "Error reading configuration values.\nYour config file seems corrupted\n/
            You can generate a new one by typing: {} genconfig",
                env!("CARGO_PKG_NAME")
            );
            std::process::exit(libc::EXIT_FAILURE);
        }
    }

    use slog::Drain;

    let rtlog_decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let rtlog = slog::Logger::root(
        slog_term::FullFormat::new(rtlog_decorator).build().fuse(),
        o!(env!("CARGO_PKG_NAME") => env!("CARGO_PKG_VERSION")),
    );

    match listen_and_serve(addr, root, user, rtlog) {
        Some(_) => std::process::exit(libc::EXIT_FAILURE),
        None => std::process::exit(libc::EXIT_SUCCESS),
    }
}

fn listen_and_serve(
    addr: std::net::SocketAddr,
    root: String,
    user: String,
    rtlog: slog::Logger,
) -> Option<std::io::Error> {
    match std::net::TcpListener::bind(addr) {
        Ok(listener) => {
            let llog =
                rtlog.new(o!("local address" => format!("{}", listener.local_addr().unwrap())));
            info!(llog, "listening");

            match get_uid_by_name(user.clone()) {
                Ok(desired_uid) => {
                    if desired_uid != get_uid() {
                        match switch_to_uid(desired_uid) {
                            Ok(uid) => {
                                info!(llog, "user switch successfull"; "current user" => uid)
                            }
                            Err(e) => {
                                crit!(llog, "Error: {}", e; "desired uid" => desired_uid, "current uid" => get_uid());
                                return Some(std::io::Error::new(std::io::ErrorKind::Other, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    crit!(llog, "Error: {}", e; "desired user" => user);
                    return Some(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            }

            // Sorry for the mess below.
            // I'll fix it someday.
            for client in listener.incoming() {
                match client {
                    Ok(mut c) => {
                        let clog =
                            llog.new(o!("peer address" => format!("{}", c.peer_addr().unwrap())));
                        info!(clog, "new connection received");

                        let mut reader = std::io::BufReader::new(c.try_clone().unwrap());
                        let mut buf = String::new();
                        match reader.read_line(&mut buf) {
                            Ok(input) => {
                                debug!(clog, "got input"; "bytes read" => input);

                                match parse_input(buf) {
                                    Ok(request) => match request {
                                        GopherMessage::ListDir(selector) => {
                                            info!(clog, "got directory list request"; "selector" => &selector);
                                            match get_directory_listing(root.clone(), selector) {
                                                Ok(listing) => {
                                                    for l in listing {
                                                        c.write_fmt(format_args!(
                                                            "{}{}\t{}\t{}\t{}\r\n",
                                                            l.gType.to_type_string(),
                                                            l.description,
                                                            l.selector,
                                                            l.host,
                                                            l.port
                                                        ));
                                                    }
                                                }
                                                Err(e) => error!(clog, "Error: {}", e),
                                            }
                                        }
                                        GopherMessage::SearchDir(selector, search_string) => {
                                            debug!(clog, "got search request"; "selector" => selector);
                                        }
                                    },
                                    Err(e) => error!(clog, "Error: {}", e),
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
    ListDir(String),
    SearchDir(String, String),
}

fn get_directory_listing(
    root: String,
    request: String,
) -> Result<Vec<DirectoryEntry>, std::io::Error> {
    match std::fs::read_dir(root + &request) {
        Ok(rd) => {
            let mut res: Vec<DirectoryEntry> = std::vec::Vec::new();
            let hostname = get_canonical_hostname();
            for possible_entry in rd {
                match possible_entry {
                    Ok(entry) => {
                        if entry.file_type().unwrap().is_dir() {
                            let e = DirectoryEntry {
                                gType: GopherType::Directory,
                                description: format!(
                                    "{}",
                                    entry.file_name().into_string().unwrap()
                                ), //TODO
                                selector: format!(
                                    "{}",
                                    entry
                                        .path()
                                        .to_str()
                                        .expect("selector has to be valid utf8")
                                        .to_string()
                                ),
                                host: hostname.clone(),
                                port: 7070, //TODO
                            };
                            res.push(e);
                        } else if entry.file_type().unwrap().is_file() {
                            let e = DirectoryEntry {
                                gType: GopherType::BinaryFile,
                                description: format!(
                                    "{}",
                                    entry.file_name().into_string().unwrap()
                                ),
                                selector: format!(
                                    "{}",
                                    entry
                                        .path()
                                        .to_str()
                                        .expect("selector has to be valid utf8")
                                        .to_string()
                                ),
                                host: hostname.clone(),
                                port: 7070,
                            };
                            res.push(e);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(res)
        }

        Err(e) => Err(e),
    }
}

fn parse_input(input: String) -> Result<GopherMessage, &'static str> {
    match input.as_str() {
        "\r\n" => Ok(GopherMessage::ListDir("/".to_string())),
        _ => {
            if input.is_empty() {
                return Err("Invalid request");
            }
            let selector_and_search: Vec<&str> = input.split("\t").collect();
            if selector_and_search.len() < 2 {
                return Ok(GopherMessage::ListDir(selector_and_search[0].to_string()));
            }
            let mut selector = String::new();

            //named!(parser,);

            //foo and bar or boo and char -> foo bar and boo char and or
            //TODO: iterate over second half to parse all logical operators
            Ok(GopherMessage::ListDir(selector))
        }
    }
}
