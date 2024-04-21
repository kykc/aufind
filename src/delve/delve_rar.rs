extern crate regex;
extern crate unrar;

use self::regex::Regex;
use self::unrar::Archive;
use super::DelveError;

pub fn handle_file(file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, highlight: bool) -> Result<(), DelveError> {
    let rar_regex = Regex::new(&("^".to_owned() + &self::regex::escape(file_path))).ok();
    let err_handler = |_| DelveError::new(file_path);

    for entry_res in Archive::new(file_path).open_for_listing().map_err(err_handler)? {
        let entry = entry_res.map_err(err_handler)?;

        if entry.is_file() {
            let entry_str = format!("{}{}{}", file_path, std::path::MAIN_SEPARATOR_STR, entry.filename.display());
            super::super::handle_entry(regex, callback, &entry_str, highlight, rar_regex.as_ref());
        }
    }
    return Ok(());
}

pub struct RarDelveHandler;

impl super::DelveHandler for RarDelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &super::super::SearchArgs) -> Result<(), DelveError> {
        return handle_file(file_path, callback, regex, args.highlight);
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".rar");
    }
}