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
use super::std;
use std::str::FromStr;

pub enum GopherType {
    Informational,
    Gif,
    Directory,
    File,
    BinaryFile,
    Error,
}

impl GopherType {
    pub fn to_type_string(&self) -> std::string::String {
        match *self {
            GopherType::Informational => "i".to_string(),
            GopherType::Gif => "g".to_string(),
            GopherType::Directory => "1".to_string(),
            GopherType::File => "0".to_string(),
            GopherType::BinaryFile => "9".to_string(),
            GopherType::Error => "3".to_string(),
        }
    }

    pub fn from_str(s: &str) -> GopherType {
        match s {
            "i" => GopherType::Informational,
            "g" => GopherType::Gif,
            "1" => GopherType::Directory,
            "9" => GopherType::BinaryFile,
            "3" => GopherType::Error,
            _ => GopherType::Error,
        }
    }

    pub fn from_file_extension(s: &str) -> GopherType { 
        match s {
            "txt" | "md" => GopherType::File,
            "gif" => GopherType::Gif,
            _ => GopherType::BinaryFile,
        }
    }
}

impl std::fmt::Display for GopherType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GopherType::Informational => write!(f, "i"),
            GopherType::Gif => write!(f, "g"),
            GopherType::Directory => write!(f, "1"),
            GopherType::File => write!(f, "0"),
            GopherType::BinaryFile => write!(f, "9"),
            GopherType::Error =>  write!(f, "3"),
        }
    }
}

pub struct DirectoryEntry {
    pub gType: GopherType,
    pub description: std::string::String,
    pub selector: std::string::String,
    pub host: std::string::String,
    pub port: u16,
}

impl std::fmt::Display for DirectoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}\t{}\t{}\t{}\r\n",
               self.gType,
               self.description,
               self.selector,
               self.host,
               self.port)
    }
}

impl DirectoryEntry {
    pub fn new() -> DirectoryEntry {
        DirectoryEntry{
            gType: GopherType::Error,
            description: "".to_string(),
            selector: "".to_string(),
            host: "".to_string(),
            port: 0,
        }
    }

    pub fn from_dir_entry(e: std::fs::DirEntry,
                          host: std::string::String,
                          port: u16) -> DirectoryEntry {
        let mut ft = GopherType::Error;
        if let Ok(ftype) = e.file_type() {
            if ftype.is_dir() {
                ft = GopherType::Directory;
            } else if ftype.is_file() {
                if let Some(ext) = e.path().extension() {
                    ft = GopherType::from_file_extension(ext.to_str().unwrap_or(""));
                } else {
                    ft = GopherType::BinaryFile;
                }
            }
        }

        DirectoryEntry{
            gType: ft,
            description: format!("{}", e.file_name().into_string().unwrap_or("".to_string())),
            selector: format!("{}", e.path().to_str().unwrap_or("").to_string()),
            host: host, 
            port: port,
        }
    }
}

pub struct Gophermap {
    pub entries: Vec<DirectoryEntry>,
}

impl Gophermap {
    pub fn from_str(st: &str) -> Result<Gophermap, &'static str> {
        Err("not yet implemented")
    }

    pub fn from_string(string: std::string::String) -> Result<Gophermap, &'static str> {
        Err("not yet implemented")
    }

    pub fn from_directory(path: &std::path::Path,
                          host: std::string::String,
                          port: u16) -> Result<Gophermap, std::io::Error> {
        let rd = try!(std::fs::read_dir(path));
        let mut res = Gophermap::new();
        for p_entry in rd {
            if let Ok(entry) = p_entry {
                let gentry = DirectoryEntry::from_dir_entry(entry, host.clone(), port);
                res.entries.push(gentry);
            }
        }
        Ok(res)
    }

    fn parse(input: &str) -> Result<Gophermap, &'static str> {
        Err("not yet implemented")
        //Ok(gopher_entry(input.as_bytes()))
    }

    /// Constructs a new `Gophermap`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let m = Gophermap::new();
    /// ```
    pub fn new() -> Gophermap {
        Gophermap {
            entries: std::vec::Vec::new(),
        }
    }
}

named!(gopher_entry<(&str, &str, &str, &str, Result<u16, std::num::ParseIntError>)>,
       do_parse!(
           gtype: map_res!(take!(1), std::str::from_utf8) >>
           descr: map_res!(take_until_and_consume!("\t"), std::str::from_utf8) >>
           selec: map_res!(take_until_and_consume!("\t"), std::str::from_utf8) >>
           host:  map_res!(take_until_and_consume!("\t"), std::str::from_utf8) >>
           port:  map!(
               map_res!(take_until_and_consume!("\r\n"), std::str::from_utf8), 
               u16::from_str) >>
           (gtype, descr, selec, host, port)
           )
       );

