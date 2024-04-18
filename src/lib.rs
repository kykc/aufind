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

pub fn search<'a>(args: &'a SearchArgs, callback: &mut dyn FnMut(&str)) {
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

    for entry in walker.into_iter().filter_map(|e| e.ok()).filter(filter.as_ref()) {
        let entry_path = entry.path().to_str().unwrap_or("");

        // TODO: how to handle file itself? should be handled and treated as a directory and respect include_dirs I guess
        #[cfg(feature = "iso-cdfs")]
        if entry_path.ends_with(".iso") && args.delve {
            match self::delve::iso_cdfs::handle_iso_file(entry_path, callback, &re, args.highlight) {
                Err(e) => {
                    let e_str = format!("{color_red}ERR: {}{color_reset}", e);
                    //callback(&e_str.red().to_string()); // TODO: separate error callback?
                    callback(&e_str);
                }
                _ => ()
            }
        }

        handle_entry(&re, callback, entry_path, args.highlight, Option::None);
    }
}

pub fn handle_entry(regex: &Result<regex::Regex, regex::Error>, callback: &mut dyn FnMut(&str), entry_path: &str, highlight: bool, additional_style: Option<&regex::Regex>) {

    let mut result = entry_path.to_string();
    let mut is_match = true; // to output everything if regex parsing failed

    if highlight {
        match additional_style {
            Some(style) => {
                //result = style.replace_all(&result, "$0".underline().to_string().as_str()).to_string();
                result = style.replace_all(&result, format!("{style_underline}$0{style_reset}")).to_string();
            },
            _ => ()
        }
    }

    match regex {
        Ok(reg) => {
            is_match = reg.is_match(entry_path);

            if is_match && highlight {
                //result = reg.replace_all(&result, "$0".yellow().to_string().as_str()).to_string();
                result = reg.replace_all(&result, format!("{color_yellow}$0{color_reset}")).to_string();
            }
        },
        _ => ()
    }

    if is_match {
        callback(&result);
    }
}

// TODO: create separate test-data directory and do not use source itself for this purpose. Add tests for delve (valid iso, invalid iso, etc)
#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_on_self() {
        let mut results = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("\\.rs$");
        args.highlight = false;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 4);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("src").join("delve").join("iso_cdfs.rs"));
        assert_eq!(Path::new(&results[1]), Path::new(".").join("src").join("delve").join("mod.rs"));
    }

    #[test]
    fn test_gitignore() {
        let mut results = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("target$");
        args.highlight = false;
        args.include_dirs = true;
        args.include_files = false;
        args.use_gitignore = true;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 0);

        args.use_gitignore = false;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("target"));
    }

    #[test]
    fn test_case_sensitivity() {
        let mut results = Vec::new();
        let mut args = super::SearchArgs::default().with_pattern("LIB.RS$");
        args.highlight = false;
        args.use_gitignore = true;
        args.case_sensitive = true;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 0);

        args.case_sensitive = false;

        super::search(&args, &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 1);
        assert_eq!(Path::new(&results[0]), Path::new(".").join("src").join("lib.rs"));
    }
}
