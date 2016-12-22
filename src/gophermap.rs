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
extern crate std;

pub enum GopherType {
    Informational,
    Gif,
    Directory,
    File,
    BinaryFile,
}

impl GopherType {
    pub fn to_type_string(&self) -> std::string::String {
        match *self {
            GopherType::Informational => "i".to_string(),
            GopherType::Gif => "g".to_string(),
            GopherType::Directory => "1".to_string(),
            GopherType::File => "0".to_string(),
            GopherType::BinaryFile => "9".to_string(),
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

//pub fn from_str() -> Result<Vec<
