/*
 *    Copyright (C) 2016-2018 Stefan Luecke
 *                  2018 Nicolas Martin
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
 *             Nicolas Martin <penguwingit@gmail.com>
 */
use super::std;
use gophertype::GopherType;
use std::str::FromStr;

#[derive(Debug)]
pub struct DirectoryEntry {
    pub gtype: GopherType,
    pub description: String,
    pub selector: String,
    pub host: String,
    pub port: u16,
}

impl std::fmt::Display for DirectoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}\t{}\t{}\t{}\r\n",
            self.gtype, self.description, self.selector, self.host, self.port
        )
    }
}

impl Default for DirectoryEntry {
    fn default() -> DirectoryEntry {
        DirectoryEntry {
            gtype: GopherType::Error,
            description: "".to_string(),
            selector: "".to_string(),
            host: "".to_string(),
            port: 0,
        }
    }
}

impl DirectoryEntry {
    /// Constructs a new empty 'DirectoryEntry'
    ///
    /// # Examples
    ///
    /// ```
    /// let de = DirectoryEntry::new();
    /// ```
    pub fn new() -> DirectoryEntry {
        DirectoryEntry {
            gtype: GopherType::Error,
            description: "".to_string(),
            selector: "".to_string(),
            host: "".to_string(),
            port: 0,
        }
    }

    pub fn from_dir_entry(
        e: std::fs::DirEntry,
        host: String,
        port: u16,
    ) -> DirectoryEntry {
        let mut ft = GopherType::Error;
        if let Ok(ftype) = e.file_type() {
            if ftype.is_dir() {
                ft = GopherType::Directory;
            } else if ftype.is_file() {
                if let Some(ext) = e.path().extension() {
                    ft = GopherType::from_file_extension(
                        ext.to_str().unwrap_or(""),
                    );
                } else {
                    ft = GopherType::BinaryFile;
                }
            }
        }

        DirectoryEntry {
            gtype: ft,
            description: format!(
                "{}",
                e.file_name().into_string().unwrap_or("".to_string())
            ),
            selector: format!(
                "{}",
                e.path().to_str().unwrap_or("").to_string()
            ),
            host: host,
            port: port,
        }
    }

    pub fn from_string(st: &str) -> Result<DirectoryEntry, String> {
        let parsing_result = gopher_entry(st.as_bytes()).to_result();

        match parsing_result {
            Ok((g, d, s, h, p)) => {
                return Ok(DirectoryEntry {
                    gtype: GopherType::from_str(g),
                    description: d.to_string(),
                    selector: s.to_string(),
                    host: h.to_string(),
                    port: match p {
                        Ok(o) => o,
                        Err(e) => {
                            println!("{}", e);
                            0
                        }
                    },
                })
            }
            Err(e) => Err(e.description().to_string()),
        }
    }
}

use nom::is_digit;

fn is_tab(chr: u8) -> bool {
    chr == '\t' as u8
}

named!(
    gopher_entry<(
        &str,
        &str,
        &str,
        &str,
        Result<u16, std::num::ParseIntError>
    )>,
    do_parse!(
        gtype: map_res!(take!(1), std::str::from_utf8)
            >> descr: map_res!(take_till!(is_tab), std::str::from_utf8)
            >> tag_s!("\t")
            >> selec: map_res!(take_till!(is_tab), std::str::from_utf8)
            >> tag_s!("\t")
            >> host: map_res!(take_till!(is_tab), std::str::from_utf8)
            >> tag_s!("\t")
            >> port: map!(
                map_res!(dbg!(take_while!(is_digit)), std::str::from_utf8),
                u16::from_str
            ) >> (gtype, descr, selec, host, port)
    )
);
