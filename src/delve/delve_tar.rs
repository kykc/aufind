extern crate tar;
extern crate flate2;
extern crate regex;
extern crate bzip2;

use super::DelveError;
use super::super::SearchArgs;
use std::fs::File;
use delve::delve_tar::flate2::read::GzDecoder;
use delve::delve_tar::tar::Archive;
use self::regex::Regex;
use self::bzip2::read::BzDecoder;

// Minding the recent xz-related calamity I decided not to add support for it just now. I perfectly understand that this is
// mostly a superstition, but anyway.

fn handle_tar<T: std::io::Read>(archive: &mut Archive<T>, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &SearchArgs) -> Result<(), DelveError> {
    let tar_regex = Regex::new(&("^".to_owned() + &self::regex::escape(file_path))).ok();

    for file in archive.entries().map_err(|_| DelveError::new(file_path))? {
        let file = file.map_err(|_| DelveError::new(file_path))?;
        let entry_str = format!("{}{}{}", file_path, std::path::MAIN_SEPARATOR_STR, file.header().path().unwrap().display());

        super::super::handle_entry(regex, callback, &entry_str, args.highlight, tar_regex.as_ref());
    }

    return Ok(());
}

pub struct TarGzDelveHandler;

impl super::DelveHandler for TarGzDelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &super::super::SearchArgs) -> Result<(), DelveError> {
        let tar_gz = File::open(file_path).map_err(|_| DelveError::new(file_path))?;
        let tar = GzDecoder::new(tar_gz);

        handle_tar(&mut Archive::new(tar), file_path, callback, regex, args)?;

        Ok(())
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".tar.gz");
    }
}

pub struct TarBz2DelveHandler;

impl super::DelveHandler for TarBz2DelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &SearchArgs) -> Result<(), DelveError> {
        let tar_bz2 = File::open(file_path).map_err(|_| DelveError::new(file_path))?;
        let tar = BzDecoder::new(tar_bz2);

        handle_tar(&mut Archive::new(tar), file_path, callback, regex, args)?;

        Ok(())
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".tar.bz2");
    }
}

pub struct TarDelveHandler;

impl super::DelveHandler for TarDelveHandler {
    fn handle_file(&self, file_path: &str, callback: &mut dyn FnMut(&str), regex: &Result<regex::Regex, regex::Error>, args: &SearchArgs) -> Result<(), DelveError> {
        let tar = File::open(file_path).map_err(|_| DelveError::new(file_path))?;

        handle_tar(&mut Archive::new(tar), file_path, callback, regex, args)?;

        Ok(())
    }

    fn can_handle_file(&self, file_path: &str) -> bool {
        return file_path.ends_with(".tar");
    }
}