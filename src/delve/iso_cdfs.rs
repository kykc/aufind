extern crate cdfs;
extern crate regex;

#[allow(unused_imports)]
use self::cdfs::{DirectoryEntry, ExtraAttributes, ISO9660Reader, ISODirectory, ISO9660};
use std::fs::File;
use self::regex::Regex;
use std::fmt;
use inline_colorization::*;

#[derive(Debug, Clone)]
pub struct IsoDelveError {
    filename: String
}

impl fmt::Display for IsoDelveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Failed to open or parse ISO file: {}", self.filename)
    }
}

impl IsoDelveError {
    fn new(msg: &str) -> IsoDelveError {
        IsoDelveError{filename: msg.to_string()}
    }
}

fn traverse_tree<T: ISO9660Reader>(iso_path_regex: Option<&Regex>, dir: &ISODirectory<T>, current_path: String, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, highlight: bool) {
    for entry in dir.contents() {
        match entry {
            Ok(DirectoryEntry::Directory(dir)) => {
                if dir.identifier == "." || dir.identifier == ".." {
                    continue;
                }

                traverse_tree(iso_path_regex, &dir, current_path.clone() + dir.identifier.as_str() + &std::path::MAIN_SEPARATOR.to_string(), callback, regex, highlight);
            }
            Ok(DirectoryEntry::File(file)) => {
                let entry_str = make_entry_string(&current_path, &file.identifier, highlight);
                super::super::handle_entry(regex, callback, &entry_str, highlight, iso_path_regex);
            }
            Ok(DirectoryEntry::Symlink(link)) => {
                #[allow(unused_variables)]
                let target = match link.target() {
                    Some(target) if target == "\u{0}" => "..",
                    Some(target) if target == "\u{1}" => ".",
                    Some(target) if target.is_empty() => "‽‽‽",
                    Some(target) => target,
                    None => "‽‽‽",
                };

                let entry_str = make_entry_string(&current_path, &link.identifier, highlight);
                super::super::handle_entry(regex, callback, &entry_str, highlight, iso_path_regex);
            },
            Err(e) => {
                let e_str = format!("{color_red}ERR cdfs: {}{color_reset}", e);
                callback(&e_str); // TODO: separate error callback?
            }
        }
    }
}

fn make_entry_string(current_path: &str, filename: &str, _highlight: bool) -> String {
    return current_path.to_string() + filename;
}

pub fn handle_iso_file(file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, highlight: bool) -> Result<(), IsoDelveError> {
    let file = File::open(file_path).map_err(|_| IsoDelveError::new(file_path))?;
    let iso = ISO9660::new(file).map_err(|_| IsoDelveError::new(file_path))?;

    let iso_regex = Regex::new(&("^".to_owned() + &self::regex::escape(file_path))).ok();
    traverse_tree(iso_regex.as_ref(), iso.root(), file_path.to_owned() + &std::path::MAIN_SEPARATOR.to_string(), callback, regex, highlight);

    Result::Ok(())
}