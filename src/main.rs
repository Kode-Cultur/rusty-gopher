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
#![feature(termination_trait_lib)]
#![feature(process_exitcode_placeholder)]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate libc;
extern crate toml;
#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate nom;
extern crate hostname;
extern crate users;

pub mod gophermap;
pub mod gophertype;
pub mod directoryentry;

use docopt::Docopt;
use gophertype::*;
use directoryentry::*;
use hostname::get_hostname;
use std::{default::Default, fs::File, io::{BufRead, Read, Write}, net::TcpListener,
          process::{exit, ExitCode, Termination}, str::FromStr};
use users::{get_current_uid, get_user_by_name};

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

/// The default config file path
const DEFAULT_MASTER_CONFIG: &'static str = "/etc/rusty_gopher.toml";

/// This struct contains all different CLI arguments
#[derive(Serialize, Deserialize)]
struct Args {
    /// Is true when the command serve was used.
    /// Indicates that we should serve our content to the internet.
    cmd_serve: bool,
    /// Is true when the genconfig command was used. Indicates that we should
    /// generate a fresh config.
    cmd_genconfig: bool,
    /// Optional: Path to user supplied config file.
    arg_config: Option<String>,
}

/// Writes a config file with default values to the given path.
///
/// # Arguments
///
/// * `path` - Path of the new configfile.
fn write_default_configfile(path: &String) -> Result<(), std::io::Error> {
    // Create a default config file object
    let conf = Config::default();

    let mut file = File::create(path.as_str())?;

    // write it to a file
    file.write_all(&toml::to_vec(&conf).unwrap())?;
    file.sync_all()?;
    Ok(())
}

/// General section of the config file.
#[derive(Serialize, Deserialize)]
struct General {
    /// The username rusty-gopher will switch to after binding to a port < 1024.
    user: String,
    /// The data root directory.
    rootdir: String,
    /// The listen address.
    listento: String,
}

impl Default for General {
    fn default() -> Self {
        General {
            user: "gopher".to_string(),
            rootdir: "/var/gopher".to_string(),
            listento: "0.0.0.0:70".to_string(),
        }
    }
}

/// Config file struct.
#[derive(Serialize, Deserialize)]
struct Config {
    /// General section.
    general: General,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General::default(),
        }
    }
}

fn main() {
    // Let docopt parse our arguments.
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Either use the default config file path or the user supplied.
    let cfgpath = args.arg_config.unwrap_or(DEFAULT_MASTER_CONFIG.to_string());

    if args.cmd_genconfig {
        write_default_configfile(&cfgpath).unwrap();
    }

    let mut cfgfile =
        File::open(&cfgpath).expect(&format!("Error opening configuration file at: {}", cfgpath));

    let mut cfgstring = String::new();
    cfgfile.read_to_string(&mut cfgstring).unwrap();
    let config: Config = toml::from_str(&cfgstring).unwrap();
    let addr = std::net::SocketAddr::from_str(&config.general.listento)
        .expect("Error reading \"listento\" value.\n");

    use slog::Drain;

    let rtlog_decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let rtlog = slog::Logger::root(
        slog_term::FullFormat::new(rtlog_decorator).build().fuse(),
        o!(env!("CARGO_PKG_NAME") => env!("CARGO_PKG_VERSION")),
    );

    match listen_and_serve(addr, config.general.rootdir, config.general.user, rtlog) {
        Some(_) => exit(ExitCode::FAILURE.report()),
        None => exit(ExitCode::SUCCESS.report()),
    }
}

fn listen_and_serve(
    addr: std::net::SocketAddr,
    root: String,
    user: String,
    rtlog: slog::Logger,
) -> Option<std::io::Error> {
    // Create tcp listener on provided address
    let listener = TcpListener::bind(addr).unwrap();
    let llog = rtlog.new(o!("local address" => format!("{}", listener.local_addr().unwrap())));
    info!(llog, "listening");

    // Setting desired uid
    let desired = get_user_by_name(&user)?;
    if desired.uid() != get_current_uid() {
        users::switch::set_current_uid(desired.uid()).unwrap();
    }

    // Still messy here but its something
    for stream in listener.incoming() {
        match stream {
            Ok(mut st) => {
                let clog = llog.new(o!("peer address" => format!("{}", st.peer_addr().unwrap())));
                info!(clog, "new connection received");

                // Create a bufreader from the stream...
                let mut reader = std::io::BufReader::new(st.try_clone().unwrap());
                let mut buf = String::new();
                // ...read it...
                let input = reader.read_line(&mut buf).unwrap();
                debug!(clog, "got input"; "bytes read" => input);
                let request = parse_input(buf).unwrap();
                // ...and match the request
                match request {
                    GopherMessage::ListDir(selector) => {
                        info!(clog, "got directory list request"; "selector" => &selector);
                        let listing = get_directory_listing(root.clone(), selector).unwrap();
                        for l in listing {
                            st.write_fmt(format_args!(
                                "{}{}\t{}\t{}\t{}\r\n",
                                l.gtype.to_type_string(),
                                l.description,
                                l.selector,
                                l.host,
                                l.port
                            )).unwrap();
                        }
                    }
                    GopherMessage::SearchDir(selector, search_string) => {
                        debug!(clog, "got search request"; "selector" => selector);
                    }
                };
            }
            Err(e) => error!(rtlog, "error handling client information {}", e),
        };
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
    let rd = std::fs::read_dir(root + &request)?;
    let hostname = get_hostname().expect("Failed to get hostname");
    let mut res: Vec<DirectoryEntry> = Vec::new();

    for possible_entry in rd {
        let entry = possible_entry?;
        // Creating desired directory entry
        // Shouldnt matter assigning GopherType::Error as gtype, after diren only
        // gets pushed into res when its a directory or file
        let mut diren = DirectoryEntry {
            gtype: GopherType::Error,
            description: format!("{}", entry.file_name().into_string().unwrap()), //TODO
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

        // If the entry is a directory...
        if entry.file_type()?.is_dir() {
            diren.gtype = GopherType::Directory;
            res.push(diren);
        } else if entry.file_type()?.is_file() {
            diren.gtype = GopherType::BinaryFile;
            res.push(diren);
        }
    }
    Ok(res)
}

fn parse_input(input: String) -> Result<GopherMessage, &'static str> {
    match input.as_str() {
        "\r\n" => Ok(GopherMessage::ListDir('/'.to_string())),
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
