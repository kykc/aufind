extern crate aufindlib;
extern crate clap;
extern crate dirs;
extern crate rustyline;

use clap::{Arg, App, SubCommand};
use rustyline::Editor;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HISTORY_FILE: &'static str = ".aufind_history";

fn query_worker(target: &str) {
    let mut rl = Editor::<()>::new();
    let history_file = Path::new(&dirs::home_dir().expect("Cannot get location of home dir")).join(HISTORY_FILE);

    if rl.load_history(&history_file).is_err() {
        println!("Note: no previous history found");
    }
    let readline = rl.readline("?> ");

    match readline {
        Ok(line) => {
            aufindlib::search(&line, target);
            rl.add_history_entry(line.as_ref());
            rl.save_history(&history_file).expect("Failed to store history");
        },
        Err(_) => {
            println!("Cancelled, exiting");
        }
    }
}

fn main() {

    let matches = App::new("aufind")
        .version(VERSION)
        .author("Automatl <ya@tomatl.org")
        .about("Simple file search utility")
        .subcommand(SubCommand::with_name("find")
                    .about("parse from CLI")
                    .arg(Arg::with_name("PATTERN")
                         .help("Pattern to match")
                         .required(false)
                         .takes_value(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("query")
                    .about("read from stdout"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("find") {
        let pattern = matches.value_of("PATTERN").unwrap_or(".*");
        let target = ".";
        aufindlib::search(pattern, target);
    } else if let Some(_) = matches.subcommand_matches("query") {
        let target = ".";
        query_worker(target);
    }
}
