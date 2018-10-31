extern crate aufindlib;
extern crate clap;
extern crate dirs;
extern crate rustyline;

use clap::{Arg, App, SubCommand};
use rustyline::Editor;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HISTORY_FILE: &'static str = ".aufind_history";

const ARG_CASE_INSENSITIVE: &'static str = "CASE_INSENSITIVE";
const ARG_PATTERN: &'static str = "PATTERN";

fn query_worker(args: &mut aufindlib::SearchArgs) {
    let mut rl = Editor::<()>::new();
    let history_file = Path::new(&dirs::home_dir()
        .expect("Cannot get location of home dir")).join(HISTORY_FILE);

    if rl.load_history(&history_file).is_err() {
        println!("Note: no previous history found");
    }
    let read_line = rl.readline("?> ");

    match read_line {
        Ok(line) => {
            aufindlib::search(&args.with_pattern(&line), &mut |x| println!("{}", x));
            rl.add_history_entry(line.as_ref());
            rl.save_history(&history_file).expect("Failed to store history");
        },
        Err(_) => {
            println!("Cancelled, exiting");
        }
    }
}

fn main() {
    let arg_case_insensitive = Arg::with_name(ARG_CASE_INSENSITIVE)
        .help("Toggle case insensitive")
        .required(false)
        .short("i")
        .long("case-insensitive");

    let matches = App::new("aufind")
        .version(VERSION)
        .author("Automatl <ya@tomatl.org")
        .about("Simple file search utility")
        .subcommand(SubCommand::with_name("find")
                    .about("parse from CLI")
                    .arg(arg_case_insensitive.clone())
                    .arg(Arg::with_name(ARG_PATTERN)
                         .help("Pattern to match")
                         .required(false)
                         .takes_value(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("query")
                    .about("read from stdout")
                    .arg(arg_case_insensitive.clone()))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("find") {
        let mut args = aufindlib::SearchArgs::default()
            .with_pattern(matches.value_of(ARG_PATTERN).unwrap_or(".*"));
        args.case_insensitive = matches.is_present(ARG_CASE_INSENSITIVE);
        aufindlib::search(&args, &mut |x| println!("{}", x));
    } else if let Some(matches) = matches.subcommand_matches("query") {
        let mut args = aufindlib::SearchArgs::default();
        args.case_insensitive = matches.is_present(ARG_CASE_INSENSITIVE);
        query_worker(&mut args);
    }
}
