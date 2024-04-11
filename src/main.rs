extern crate aufindlib;
extern crate clap;
extern crate dirs;
extern crate rustyline;

use clap::{Arg, App, SubCommand};
use rustyline::Editor;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HISTORY_FILE: &'static str = ".aufind_history";

const ARG_CASE_SENSITIVE: &'static str = "CASE_SENSITIVE";
const ARG_PATTERN: &'static str = "PATTERN";
const ARG_INCLUDE_DIRS: &'static str = "INCLUDE_DIRS";
const ARG_EXCLUDE_FILES: &'static str = "EXCLUDE_FILES";
const ARG_HIGHLIGHT_OUTPUT: &'static str = "HIGHLIGHT_OUTPUT";
const ARG_USE_GITIGNORE: &'static str = "USE_GITIGNORE";
const ARG_EXCLUDE_HIDDEN: &'static str = "EXCLUDE_HIDDEN";

fn query_worker(args: &aufindlib::SearchArgs) {
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
            rl.add_history_entry(line.as_str());
            rl.save_history(&history_file).expect("Failed to store history");
        },
        Err(_) => {
            println!("Cancelled, exiting");
        }
    }
}

fn args_from_matches<'a>(matches: &clap::ArgMatches) -> aufindlib::SearchArgs<'a> {
    let mut args = aufindlib::SearchArgs::default();
    args.case_sensitive = matches.is_present(ARG_CASE_SENSITIVE);
    args.include_dirs = matches.is_present(ARG_INCLUDE_DIRS);
    args.include_files = !matches.is_present(ARG_EXCLUDE_FILES);
    args.highlight = matches.is_present(ARG_HIGHLIGHT_OUTPUT);
    args.use_gitignore = matches.is_present(ARG_USE_GITIGNORE);
    args.exclude_hidden = matches.is_present(ARG_EXCLUDE_HIDDEN);

    args
}

fn main() {
    let default_args = aufindlib::SearchArgs::default();

    let arg_case_sensitive = Arg::with_name(ARG_CASE_SENSITIVE)
        .help("Toggle case sensitivity")
        .required(false)
        .takes_value(false)
        .short("c")
        .long("case-sensitive");

    let arg_use_gitignore = Arg::with_name(ARG_USE_GITIGNORE)
        .help("Use .gitignore files for filtering")
        .required(false)
        .takes_value(false)
        .short("g")
        .long("gitignore");

    let arg_exclude_hidden = Arg::with_name(ARG_EXCLUDE_HIDDEN)
        .help("Exclude hidden files and directories")
        .required(false)
        .takes_value(false)
        .short("x")
        .long("exclude-hidden");

    let arg_include_dirs = Arg::with_name(ARG_INCLUDE_DIRS)
        .help("Include directories in search results")
        .required(false)
        .takes_value(false)
        .short("d")
        .long("include-dirs");

    let arg_exclude_files = Arg::with_name(ARG_EXCLUDE_FILES)
        .help("Exclude files from search results")
        .required(false)
        .takes_value(false)
        .short("f")
        .long("exclude-files");

    let arg_highlight_output = Arg::with_name(ARG_HIGHLIGHT_OUTPUT)
        .help("Highlight matches in output")
        .required(false)
        .takes_value(false)
        .short("h")
        .long("highlight-output");

    let matches = App::new("aufind")
        .version(VERSION)
        .author("Alexander Prokopchuk <ya@tomatl.org>")
        .about("Simple file search utility")
        .subcommand(SubCommand::with_name("find")
                    .about("parse from CLI")
                    .arg(arg_case_sensitive.clone())
                    .arg(arg_include_dirs.clone())
                    .arg(arg_exclude_files.clone())
                    .arg(arg_highlight_output.clone())
                    .arg(arg_use_gitignore.clone())
                    .arg(arg_exclude_hidden.clone())
                    .arg(Arg::with_name(ARG_PATTERN)
                         .help("Pattern to match")
                         .required(false)
                         .takes_value(true)
                         .index(1)))
        .subcommand(SubCommand::with_name("query")
                    .about("read from stdout")
                    .arg(arg_case_sensitive.clone())
                    .arg(arg_include_dirs.clone())
                    .arg(arg_exclude_files.clone())
                    .arg(arg_highlight_output.clone())
                    .arg(arg_use_gitignore.clone())
                    .arg(arg_exclude_hidden.clone()))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("find") {
        let default_pattern = default_args.construct_pattern();
        let args = args_from_matches(&matches)
            .with_pattern(matches.value_of(ARG_PATTERN).unwrap_or(&default_pattern));
        aufindlib::search(&args, &mut |x| println!("{}", x));
    } else if let Some(matches) = matches.subcommand_matches("query") {
        let args = args_from_matches(&matches);
        query_worker(&args);
    }
}
