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

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_type_string() {
        assert_eq!(GopherType::to_type_string(&GopherType::Informational), "i");
        assert_eq!(GopherType::to_type_string(&GopherType::Gif), "g");
        assert_eq!(GopherType::to_type_string(&GopherType::Directory), "1");
        assert_eq!(GopherType::to_type_string(&GopherType::File), "0");
        assert_eq!(GopherType::to_type_string(&GopherType::BinaryFile), "9");
        assert_eq!(GopherType::to_type_string(&GopherType::Error), "3");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(GopherType::from_str("i"), GopherType::Informational);
        assert_eq!(GopherType::from_str("g"), GopherType::Gif);
        assert_eq!(GopherType::from_str("1"), GopherType::Directory);
        assert_eq!(GopherType::from_str("9"), GopherType::BinaryFile);
        assert_eq!(GopherType::from_str("3"), GopherType::Error);
        assert_eq!(GopherType::from_str("7"), GopherType::Error);
    }

    #[test]
    fn test_from_file_extension() {
        assert_eq!(GopherType::from_file_extension("txt"), GopherType::File);
        assert_eq!(GopherType::from_file_extension("md"), GopherType::File);
        assert_eq!(GopherType::from_file_extension("gif"), GopherType::Gif);
        assert_eq!(GopherType::from_file_extension("wtf"), GopherType::BinaryFile);
    }
}

