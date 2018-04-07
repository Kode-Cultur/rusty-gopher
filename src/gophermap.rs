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
use directoryentry::DirectoryEntry;
use std::str::FromStr;

pub struct Gophermap {
    pub entries: Vec<DirectoryEntry>,
}

impl Gophermap {
    /// Constructs a new `Gophermap`.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = Gophermap::new();
    /// ```
    pub fn new() -> Gophermap {
        Gophermap {
            entries: Vec::new(),
        }
    }
    pub fn from_string<S: Into<String>>(st: &S) -> Result<Gophermap, &'static str> {
        Err("not yet implemented")
    }

    pub fn from_directory(
        path: &std::path::Path,
        host: String,
        port: u16,
    ) -> Result<Gophermap, std::io::Error> {
        let rd = std::fs::read_dir(path)?;
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
}

named!(
    gopher_entry<(&str, &str, &str, &str, Result<u16, std::num::ParseIntError>)>,
    do_parse!(
        gtype: map_res!(take!(1), std::str::from_utf8)
            >> descr: map_res!(take_until_and_consume!("\t"), std::str::from_utf8)
            >> selec: map_res!(take_until_and_consume!("\t"), std::str::from_utf8)
            >> host: map_res!(take_until_and_consume!("\t"), std::str::from_utf8)
            >> port: map!(
                map_res!(take_until_and_consume!("\r\n"), std::str::from_utf8),
                u16::from_str
            ) >> (gtype, descr, selec, host, port)
    )
);
