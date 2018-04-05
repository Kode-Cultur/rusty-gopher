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

pub enum GopherType {
    Informational,
    Gif,
    Directory,
    File,
    BinaryFile,
    Error,
}

impl std::fmt::Display for GopherType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GopherType::Informational => write!(f, "i"),
            GopherType::Gif => write!(f, "g"),
            GopherType::Directory => write!(f, "1"),
            GopherType::File => write!(f, "0"),
            GopherType::BinaryFile => write!(f, "9"),
            GopherType::Error => write!(f, "3"),
        }
    }
}

impl GopherType {
    pub fn to_type_string(&self) -> String {
        match *self {
            GopherType::Informational => "i",
            GopherType::Gif => "g",
            GopherType::Directory => "1",
            GopherType::File => "0",
            GopherType::BinaryFile => "9",
            GopherType::Error => "3",
        }.to_string()
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

