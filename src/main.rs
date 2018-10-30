extern crate aufindlib;
extern crate clap;
#[macro_use] extern crate text_io;

use clap::{Arg, App, SubCommand};
use std::io::Write;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
        print!("?> ");
        std::io::stdout().flush().expect("cannot flush stdout, this is time to panic!");
        let pattern: String = read!("{}\n");
        let target = ".";
        aufindlib::search(&pattern, target);
    }
}
