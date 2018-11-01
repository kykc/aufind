extern crate colored;

use colored::*;

#[allow(dead_code)]
const EMPTY_STRING: &'static str = "";
pub const DEFAULT_PATTERN: &'static str = ".*";
pub const DEFAULT_TARGET: &'static str = ".";

#[derive(Clone, Debug)]
pub struct SearchArgs<'a> {
    pub highlight: bool,
    pattern: &'a str,
    pub target: &'a str,
    pub case_insensitive: bool,
    pub include_dirs: bool,
    pub include_files: bool
}

impl<'a> SearchArgs<'a> {
    pub fn with_pattern(&self, pattern: &'a str) -> Self {
        let mut result = self.clone();
        result.pattern = pattern;

        result
    }

    pub fn construct_pattern(&self) -> String {
        if self.case_insensitive {
            String::from("(?i)") + self.pattern
        } else {
            String::from(self.pattern)
        }
    }
}

impl<'a> Default for SearchArgs<'a> {
    fn default() -> Self {
        let highlight_default = if cfg!(windows) { false } else { true };

        SearchArgs {
            highlight: highlight_default,
            pattern: DEFAULT_PATTERN,
            target: DEFAULT_TARGET,
            case_insensitive: false,
            include_dirs: false,
            include_files: true
        }
    }
}

pub fn search<'a>(args: &'a SearchArgs, callback: &mut FnMut(&str)) {
    let re = regex::Regex::new(&args.construct_pattern());

    let filter: Box<Fn(&walkdir::DirEntry) -> bool> = if args.include_dirs && args.include_files {
        Box::new(|_| true)
    } else if args.include_dirs {
        Box::new(|x| x.path().is_dir())
    } else if args.include_files {
        Box::new(|x| x.path().exists() && !x.path().is_dir())
    } else {
        Box::new(|_| false)
    };

    for entry in walkdir::WalkDir::new(args.target).into_iter().filter_map(|e| e.ok()).filter(filter.as_ref()) {
        match &re {
            Ok(reg) => {
                if reg.is_match(entry.path().to_str().unwrap_or("")) {
                    if args.highlight {
                        let after = reg.replace_all(
                            entry.path().to_str().unwrap_or(""), "$0".yellow().to_string().as_str());
                        callback(&after);
                    } else {
                        callback(&entry.path().to_str().unwrap_or(""));
                    }
                }
            },
            Err(_x) => { callback(&entry.path().to_str().unwrap_or("")); }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_on_self() {
        let mut results = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("\\.rs$");
        args.highlight = false;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 2);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("src").join("lib.rs"));
        assert_eq!(Path::new(&results[1]), Path::new(".").join("src").join("main.rs"));
    }
}
