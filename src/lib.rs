extern crate colored;

use colored::*;

pub fn search(highlight: bool, pattern: &str, target: &str, callback: &mut FnMut(&str)) {
    let re = regex::Regex::new(pattern);
    for entry in walkdir::WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        match &re {
            Ok(reg) => {
                if reg.is_match(entry.path().to_str().unwrap_or("")) {
                    if highlight {
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

        super::search(false, "\\.rs$", ".",
                      &mut |x| results.push(String::from(x)));

        assert_eq!(results.len(), 2);
        // TODO: will fail on windows
        assert_eq!(results[0], "./src/lib.rs");
        assert_eq!(results[1], "./src/main.rs");
    }
}
