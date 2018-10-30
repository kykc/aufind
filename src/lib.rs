
pub fn search(pattern: &str, target: &str) {
    let re = regex::Regex::new(pattern);
    for entry in walkdir::WalkDir::new(target).into_iter().filter_map(|e| e.ok()) {
        match &re {
            Ok(reg) => {
                if reg.is_match(entry.path().to_str().unwrap_or("")) {
                    println!("{}", entry.path().display());
                }
            },
            Err(_x) => { println!("{}", entry.path().display()); }
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn zhopa() {
        super::search("*", ".");
    }
}
