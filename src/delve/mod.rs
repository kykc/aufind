#[cfg(feature = "delve-iso")] pub mod delve_iso;
#[cfg(feature = "delve-zip")] pub mod delve_zip;
#[cfg(feature = "delve-rar")] pub mod delve_rar;
#[cfg(feature = "delve-tar")] pub mod delve_tar;
#[cfg(feature = "delve-7zip")] pub mod delve_7zip;

use std::fmt;
use super::SearchArgs;

#[derive(Debug, Clone)]
pub struct DelveError {
    filename: String
}

impl fmt::Display for DelveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Failed to open or parse file (skipping): {}", self.filename) // TODO: filetype in error message
    }
}

impl DelveError {
    fn new(msg: &str) -> DelveError {
        DelveError{filename: msg.to_string()}
    }
}

pub trait DelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &SearchArgs) -> Result<(), DelveError>;
    fn can_handle_file(&self, file_path: &str) -> bool;
}