extern crate regex;

use self::regex::Regex;
use super::DelveError;

pub fn handle_file(file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, highlight: bool) -> Result<(), DelveError> {
    let archive = std::fs::File::open(file_path).map_err(|_| DelveError::new(file_path))?;
    let mut archive = zip::ZipArchive::new(archive).map_err(|_| DelveError::new(file_path))?;

    let zip_regex = Regex::new(&("^".to_owned() + &self::regex::escape(file_path))).ok();

    for idx in 0..archive.len() {
        let entry = archive.by_index(idx).map_err(|_| DelveError::new(file_path))?;

        match entry.enclosed_name() {
            Some(name) => {
                let entry_str = file_path.to_owned() + &std::path::MAIN_SEPARATOR.to_string() + name.to_str().unwrap_or("FAILED_TO_GET_FILE_NAME");
                super::super::handle_entry(regex, callback, &entry_str, highlight, zip_regex.as_ref());
            },
            None => ()
        }
    }

    return Ok(());
}

pub struct ZipDelveHandler;

impl super::DelveHandler for ZipDelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &super::super::SearchArgs) -> Result<(), DelveError> {
        return handle_file(file_path, callback, regex, args.highlight);
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".zip") || file_path.ends_with(".jar");
    }
}