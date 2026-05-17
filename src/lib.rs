extern crate inline_colorization;

use inline_colorization::*;
use std::sync::{mpsc, Arc};

pub mod delve;

#[allow(dead_code)]
const EMPTY_STRING: &'static str = "";
pub const DEFAULT_PATTERN: &'static str = ".*";
pub const DEFAULT_TARGET: &'static str = ".";

#[derive(Clone, Debug)]
pub struct SearchArgs {
    pub highlight: bool,
    pattern: String,
    pub target: String,
    pub case_sensitive: bool,
    pub include_dirs: bool,
    pub include_files: bool,
    pub use_gitignore: bool,
    pub exclude_hidden: bool,
    pub delve: bool,
    pub parallel: bool,
}

pub struct SearchEventHandler<'a> {
    pub handle_item: &'a mut dyn FnMut(&str),
    pub handle_error: &'a mut dyn FnMut(&str),
}

impl SearchArgs {
    pub fn with_pattern(&self, pattern: &str) -> Self {
        let mut result = self.clone();
        result.pattern = pattern.to_owned();
        result
    }

    pub fn construct_pattern(&self) -> String {
        if !self.case_sensitive {
            String::from("(?i)") + &self.pattern
        } else {
            self.pattern.clone()
        }
    }
}

impl Default for SearchArgs {
    fn default() -> Self {
        SearchArgs {
            highlight: true,
            pattern: String::from(DEFAULT_PATTERN),
            target: String::from(DEFAULT_TARGET),
            case_sensitive: false,
            include_dirs: false,
            include_files: true,
            use_gitignore: false,
            exclude_hidden: false,
            delve: false,
            parallel: false,
        }
    }
}

enum SearchEvent {
    Item(String),
    Error(String),
}

fn configure_walker(args: &SearchArgs) -> ignore::WalkBuilder {
    let mut b = ignore::WalkBuilder::new(&args.target);
    b.hidden(args.exclude_hidden)
     .parents(args.use_gitignore)
     .ignore(args.use_gitignore)
     .git_ignore(args.use_gitignore)
     .git_global(args.use_gitignore)
     .git_exclude(args.use_gitignore);
    b
}

fn build_handlers() -> Vec<Box<dyn delve::DelveHandler + Send + Sync>> {
    let mut handlers: Vec<Box<dyn delve::DelveHandler + Send + Sync>> = Vec::new();
    #[cfg(feature = "delve-iso")] handlers.push(Box::new(delve::delve_iso::IsoDelveHandler {}));
    #[cfg(feature = "delve-zip")] handlers.push(Box::new(delve::delve_zip::ZipDelveHandler {}));
    #[cfg(feature = "delve-rar")] handlers.push(Box::new(delve::delve_rar::RarDelveHandler {}));
    #[cfg(feature = "delve-tar")] handlers.push(Box::new(delve::delve_tar::TarGzDelveHandler {}));
    #[cfg(feature = "delve-tar")] handlers.push(Box::new(delve::delve_tar::TarBz2DelveHandler {}));
    #[cfg(feature = "delve-tar")] handlers.push(Box::new(delve::delve_tar::TarDelveHandler {}));
    #[cfg(feature = "delve-7zip")] handlers.push(Box::new(delve::delve_7zip::SevenZipDelveHandler {}));
    handlers
}

fn process_entry(
    entry: &ignore::DirEntry,
    re: &Result<regex::Regex, regex::Error>,
    args: &SearchArgs,
    handlers: &[Box<dyn delve::DelveHandler + Send + Sync>],
    item_cb: &mut dyn FnMut(&str),
    error_cb: &mut dyn FnMut(&str),
) {
    let file_type = entry.file_type();
    let is_dir = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
    let is_file = file_type.map(|ft| ft.is_file()).unwrap_or(false);

    let should_output = match (args.include_dirs, args.include_files) {
        (true, true) => true,
        (true, false) => is_dir,
        (false, true) => is_file,
        (false, false) => false,
    };

    if !should_output { return; }

    let entry_path = match entry.path().to_str() {
        Some(s) => s,
        None => return,
    };

    if args.delve {
        for handler in handlers.iter() {
            if handler.can_handle_file(entry_path) {
                match handler.handle_file(entry_path, item_cb, re, args) {
                    Err(e) => {
                        let msg = if args.highlight {
                            format!("{color_red}ERR: {}{color_reset}", e)
                        } else {
                            format!("ERR: {}", e)
                        };
                        error_cb(&msg);
                    },
                    _ => ()
                }
            }
        }
    }

    handle_entry(re, item_cb, entry_path, args.highlight, None);
}

pub fn search(args: &SearchArgs, search_handler: &mut SearchEventHandler) {
    let re = regex::Regex::new(&args.construct_pattern());
    let handlers = build_handlers();

    if args.parallel {
        let re = Arc::new(re);
        let handlers = Arc::new(handlers);
        let mut wb = configure_walker(args);
        wb.threads(num_cpus::get());
        let walker = wb.build_parallel();
        let (tx, rx) = mpsc::channel::<SearchEvent>();
        let args_owned = args.clone();

        std::thread::spawn(move || {
            walker.run(move || {
                let tx = tx.clone();
                let re = Arc::clone(&re);
                let args = args_owned.clone();
                let handlers = Arc::clone(&handlers);

                Box::new(move |entry: Result<ignore::DirEntry, ignore::Error>| -> ignore::WalkState {
                    if let Ok(e) = entry {
                        let mut item_cb = |s: &str| { let _ = tx.send(SearchEvent::Item(s.to_owned())); };
                        let mut error_cb = |s: &str| { let _ = tx.send(SearchEvent::Error(s.to_owned())); };
                        process_entry(&e, &*re, &args, &handlers, &mut item_cb, &mut error_cb);
                    }
                    ignore::WalkState::Continue
                })
            });
        });

        for event in rx {
            match event {
                SearchEvent::Item(s) => (search_handler.handle_item)(&s),
                SearchEvent::Error(s) => (search_handler.handle_error)(&s),
            }
        }
    } else {
        let walker = configure_walker(args).build();
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            process_entry(&entry, &re, args, &handlers, search_handler.handle_item, search_handler.handle_error);
        }
    }
}

pub fn handle_entry(regex: &Result<regex::Regex, regex::Error>, callback: &mut dyn FnMut(&str), entry_path: &str, highlight: bool, additional_style: Option<&regex::Regex>) {
    let mut result = entry_path.to_string();
    let mut is_match = true;

    if highlight {
        match additional_style {
            Some(style) => {
                result = style.replace_all(&result, format!("{style_underline}$0{style_reset}")).to_string();
            },
            _ => ()
        }
    }

    match regex {
        Ok(reg) => {
            is_match = reg.is_match(entry_path);

            if is_match && highlight {
                result = reg.replace_all(&result, format!("{color_yellow}$0{color_reset}")).to_string();
            }
        },
        _ => ()
    }

    if is_match {
        callback(&result);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::SearchEventHandler;

    #[test]
    fn test_find_file() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("\\.txt$");

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        args.highlight = false;

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("file_one.txt"));
    }

    #[test]
    fn test_gitignore() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("file_two\\.bin$");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.use_gitignore = true;

        {
            let mut search_handler = SearchEventHandler {
                handle_item: &mut |x| results.push(String::from(x)),
                handle_error: &mut |x| errors.push(String::from(x))
            };

            super::search(&args, &mut search_handler);
            assert_eq!(results.len(), 0);
        }

        {
            results.clear();
            errors.clear();

            let mut search_handler = SearchEventHandler {
                handle_item: &mut |x| results.push(String::from(x)),
                handle_error: &mut |x| errors.push(String::from(x))
            };

            args.use_gitignore = false;

            super::search(&args, &mut search_handler);

            assert_eq!(results.len(), 1);
            assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("file_two.bin"));
        }
    }

    #[test]
    fn test_case_sensitivity() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("FILE_TWO\\.bin$");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.case_sensitive = true;

        {
            let mut search_handler = SearchEventHandler {
                handle_item: &mut |x| results.push(String::from(x)),
                handle_error: &mut |x| errors.push(String::from(x))
            };

            super::search(&args, &mut search_handler);
            assert_eq!(results.len(), 0);
        }

        args.case_sensitive = false;
        results.clear();
        errors.clear();

        {
            let mut search_handler = SearchEventHandler {
                handle_item: &mut |x| results.push(String::from(x)),
                handle_error: &mut |x| errors.push(String::from(x))
            };

            super::search(&args, &mut search_handler);

            assert_eq!(results.len(), 1);
            assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("file_two.bin"));
        }
    }

    #[test]
    fn test_delve_iso() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("GPL_3_0");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(errors.len(), 1);

        assert!(errors[0].starts_with("ERR: ") && errors[0].ends_with("invalid.iso"));
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test.iso").join("GPL_3_0.TXT"));
    }

    #[test]
    fn test_delve_rar() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("rar_archive_file");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test-data.rar").join("rar_archive_file.txt"));
    }

    #[test]
    fn test_delve_7z() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("7z_archive_file");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test-data.7z").join("test-data").join("7z_archive_file.txt"));
    }

    #[test]
    fn test_delve_tar() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("tar_archive_file");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test-data.tar").join("test-data").join("tar_archive_file.txt"));
    }

    #[test]
    fn test_delve_targz() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("targz_archive_file");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test-data.tar.gz").join("test-data").join("targz_archive_file.txt"));
    }

    #[test]
    fn test_delve_tarbz() {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("tarbz_archive_file");
        args.highlight = false;
        args.target = String::from("./test-data");
        args.delve = true;

        let mut search_handler = SearchEventHandler {
            handle_item: &mut |x| results.push(String::from(x)),
            handle_error: &mut |x| errors.push(String::from(x))
        };

        super::search(&args, &mut search_handler);

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("test-data").join("delve").join("test-data.tar.bz2").join("test-data").join("tarbz_archive_file.txt"));
    }
}
