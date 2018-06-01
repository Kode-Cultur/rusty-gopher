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

#[derive(Debug)]
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

    /// Generates a Gophermap from a string
    ///
    /// # Examples
    ///
    /// ```
    /// let m = Gophermap::from_string("");
    /// ```
    pub fn from_string(input: &str) -> Result<Gophermap, &'static str> {
        let mut result: Gophermap = Gophermap::new();
        for line in input.lines() {
            match DirectoryEntry::from_string(line) {
                Ok(d) => result.entries.push(d),
                Err(e) => println!("Error parsing line: {}", e), // TODO: Return error
            }
        }
        Ok(result)
    }

    /// Generates a Gophermap out of directory entries.
    ///
    /// # Examples
    ///
    /// ```
    /// let m = Gophermap::from_directory("path", "localhost", 7070);
    /// ```
    pub fn from_directory(
        path: &std::path::Path,
        host: String,
        port: u16,
    ) -> Result<Gophermap, std::io::Error> {
        let rd = std::fs::read_dir(path)?;
        let mut res = Gophermap::new();
        for p_entry in rd {
            if let Ok(entry) = p_entry {
                let gentry =
                    DirectoryEntry::from_dir_entry(entry, host.clone(), port);
                res.entries.push(gentry);
            }
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::black_box;
    use test::Bencher;
    use GopherType;

    #[test]
    fn test_from_str() {
        let teststr = format!("0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t7070\r\n0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70\r\n");

        let mut entry = DirectoryEntry::new();
        entry.gtype = GopherType::from_str("0");
        entry.description = "About internet Gopher".to_string();
        entry.selector = "Stuff:About us".to_string();
        entry.host = "rawBits.micro.umn.edu".to_string();
        entry.port = 7070;

        let mut entry2 = DirectoryEntry::new();
        entry2.gtype = GopherType::from_str("0");
        entry2.description = "About internet Gopher".to_string();
        entry2.selector = "Stuff:About us".to_string();
        entry2.host = "rawBits.micro.umn.edu".to_string();
        entry2.port = 70;

        let parsed_map = Gophermap::from_string(&teststr).unwrap();
        assert_eq!(parsed_map.entries[0].gtype, entry.gtype);
        assert_eq!(parsed_map.entries[0].description, entry.description);
        assert_eq!(parsed_map.entries[0].selector, entry.selector);
        assert_eq!(parsed_map.entries[0].host, entry.host);
        assert_eq!(parsed_map.entries[0].port, entry.port);

        assert_eq!(parsed_map.entries[1].gtype, entry2.gtype);
        assert_eq!(parsed_map.entries[1].description, entry2.description);
        assert_eq!(parsed_map.entries[1].selector, entry2.selector);
        assert_eq!(parsed_map.entries[1].host, entry2.host);
        assert_eq!(parsed_map.entries[1].port, entry2.port);
    }

    #[bench]
    fn bench_from_str(b: &mut Bencher) {
        let teststr = format!("0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t7070\r\n0About internet Gopher\tStuff:About us\trawBits.micro.umn.edu\t70\r\n");
        b.iter(|| black_box(Gophermap::from_string(&teststr).unwrap()));
    }
}
