extern crate inline_colorization;

use inline_colorization::*;

pub mod delve;

#[allow(dead_code)]
const EMPTY_STRING: &'static str = "";
pub const DEFAULT_PATTERN: &'static str = ".*";
pub const DEFAULT_TARGET: &'static str = ".";

#[derive(Clone, Debug)]
pub struct SearchArgs<'a> {
    pub highlight: bool,
    pattern: &'a str,
    pub target: &'a str,
    pub case_sensitive: bool,
    pub include_dirs: bool,
    pub include_files: bool,
    pub use_gitignore: bool,
    pub exclude_hidden: bool,
    pub delve: bool
}

pub struct SearchEventHandler<'a> {
    pub handle_item: &'a mut dyn FnMut(&str),
    pub handle_error: &'a mut dyn FnMut(&str)
}

impl<'a> SearchArgs<'a> {
    pub fn with_pattern(&self, pattern: &'a str) -> Self {
        let mut result = self.clone();
        result.pattern = pattern;

        result
    }

    pub fn construct_pattern(&self) -> String {
        if !self.case_sensitive {
            String::from("(?i)") + self.pattern
        } else {
            String::from(self.pattern)
        }
    }
}

impl<'a> Default for SearchArgs<'a> {
    fn default() -> Self {
        SearchArgs {
            highlight: true,
            pattern: DEFAULT_PATTERN,
            target: DEFAULT_TARGET,
            case_sensitive: false,
            include_dirs: false,
            include_files: true,
            use_gitignore: false,
            exclude_hidden: false,
            delve: false
        }
    }
}

pub fn search<'a>(args: &'a SearchArgs, search_handler: &'a mut SearchEventHandler) {
    let re = regex::Regex::new(&args.construct_pattern());

    let filter: Box<dyn Fn(&ignore::DirEntry) -> bool> =
        if args.include_dirs && args.include_files {
            Box::new(|_| true)
        } else if args.include_dirs {
            Box::new(|x| x.path().is_dir())
        } else if args.include_files {
            Box::new(|x| x.path().exists() && !x.path().is_dir())
        } else {
            Box::new(|_| false)
        };

    let walker = ignore::WalkBuilder::new(args.target)
        .hidden(args.exclude_hidden)
        .parents(args.use_gitignore)
        .ignore(args.use_gitignore)
        .git_ignore(args.use_gitignore)
        .git_global(args.use_gitignore)
        .git_exclude(args.use_gitignore)
        .build();

    let mut handlers: Vec<&dyn delve::DelveHandler> = Vec::new();

    // As this function is called only once per app launch (with the exception of unit tests) it is OK to construct this state here
    // Well, and this is just a modest amount of references to empty objects with the sole purpose of dynamic dispatch/late runtime binding
    #[cfg(feature = "delve-iso")]
    handlers.push(&delve::delve_iso::IsoDelveHandler{});
    #[cfg(feature = "delve-zip")]
    handlers.push(&delve::delve_zip::ZipDelveHandler{});
    #[cfg(feature = "delve-rar")]
    handlers.push(&delve::delve_rar::RarDelveHandler{});
    #[cfg(feature = "delve-tar")]
    handlers.push(&delve::delve_tar::TarGzDelveHandler{});
    #[cfg(feature = "delve-tar")]
    handlers.push(&delve::delve_tar::TarBz2DelveHandler{});
    #[cfg(feature = "delve-tar")]
    handlers.push(&delve::delve_tar::TarDelveHandler{});
    #[cfg(feature = "delve-7zip")]
    handlers.push(&delve::delve_7zip::SevenZipDelveHandler{});

    for entry in walker.into_iter().filter_map(|e| e.ok()).filter(filter.as_ref()) {
        let entry_path = entry.path().to_str().unwrap_or("");

        // TODO: how to handle file itself? should be handled and treated as a directory and respect include_dirs I guess

        if args.delve {
            for handler in &handlers {
                if handler.can_handle_file(entry_path) {
                    match handler.handle_file(entry_path, search_handler.handle_item, &re, args) {
                        Err(e) => {
                            if args.highlight {
                                (search_handler.handle_error)(&format!("{color_red}ERR: {}{color_reset}", e));
                            } else {
                                (search_handler.handle_error)(&format!("ERR: {}", e));
                            }
                        },
                        _ => ()
                    }
                }
            }
        }

        handle_entry(&re, search_handler.handle_item, entry_path, args.highlight, Option::None);
    }
}

pub fn handle_entry(regex: &Result<regex::Regex, regex::Error>, callback: &mut dyn FnMut(&str), entry_path: &str, highlight: bool, additional_style: Option<&regex::Regex>) {

    let mut result = entry_path.to_string();
    let mut is_match = true; // to output everything if regex parsing failed

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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
        args.target = "./test-data";
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
