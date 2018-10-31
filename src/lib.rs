extern crate colored;

use colored::*;

#[allow(dead_code)]
const EMPTY_STRING: &'static str = "";
const DEFAULT_PATTERN: &'static str = ".*";
const DEFAULT_TARGET: &'static str = ".";

#[derive(Clone, Debug)]
pub struct SearchArgs<'a> {
    pub highlight: bool,
    pattern: &'a str,
    pub target: &'a str,
    pub case_insensitive: bool
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
        SearchArgs {
            highlight: true,
            pattern: DEFAULT_PATTERN,
            target: DEFAULT_TARGET,
            case_insensitive: false
        }
    }
}

pub fn search<'a>(args: &'a SearchArgs, callback: &mut FnMut(&str)) {
    let re = regex::Regex::new(&args.construct_pattern());
    for entry in walkdir::WalkDir::new(args.target).into_iter().filter_map(|e| e.ok()) {
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
    #[test]
    fn test_on_self() {
        let mut results = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("\\.rs$");
        args.highlight = false;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 2);
        // TODO: will fail on windows
        assert_eq!(results[0], "./src/lib.rs");
        assert_eq!(results[1], "./src/main.rs");
    }
}
