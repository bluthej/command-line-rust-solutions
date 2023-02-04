use clap::{Parser, ValueEnum};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Search paths", default_value = ".")]
    path: Vec<String>,

    #[arg(short, long, help = "Name", num_args(0..))]
    name: Vec<Regex>,

    #[arg(short = 't', long = "type", help = "Entry type", value_name = "TYPE", num_args(0..))]
    entry_type: Vec<EntryType>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EntryType {
    F,
    D,
    L,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(|e| e.into())
}

pub fn run(cli: Cli) -> MyResult<()> {
    for path in &cli.path {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) if filter(&entry, &cli.name, &cli.entry_type) => {
                    println!("{}", entry.path().display())
                }
                Err(e) => eprintln!("{}", e),
                _ => (),
            }
        }
    }
    Ok(())
}

fn filter(entry: &DirEntry, names: &Vec<Regex>, entry_types: &Vec<EntryType>) -> bool {
    let entry_type = if entry.path().is_symlink() {
        EntryType::L
    } else if entry.path().is_file() {
        EntryType::F
    } else if entry.path().is_dir() {
        EntryType::D
    } else {
        unreachable!()
    };
    (names.is_empty()
        || names
            .iter()
            .any(|re| re.is_match(&entry.file_name().to_string_lossy())))
        && (entry_types.is_empty() || entry_types.contains(&entry_type))
}
