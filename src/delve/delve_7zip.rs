extern crate regex;
extern crate sevenz_rust;

use self::regex::Regex;
use super::DelveError;
use self::sevenz_rust::{Archive, Password};
pub struct SevenZipDelveHandler;

impl super::DelveHandler for SevenZipDelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &super::super::SearchArgs) -> Result<(), DelveError> {
        let sz_regex = Regex::new(&("^".to_owned() + &self::regex::escape(file_path))).ok();

        let mut file = std::fs::File::open(file_path).map_err(|_| DelveError::new(file_path))?;

        let metadata = file.metadata().map_err(|_| DelveError::new(file_path))?;
        let password = Password::empty();
        let archive = Archive::read(&mut file, metadata.len(), password.as_slice()).map_err(|_| DelveError::new(file_path))?;

        for file in archive.files {
            let entry_str = file_path.to_owned() + &std::path::MAIN_SEPARATOR.to_string() + file.name();
            super::super::handle_entry(regex, callback, &entry_str, args.highlight, sz_regex.as_ref());
        }

        Ok(())
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".7z") || file_path.ends_with(".7zip");
    }
}