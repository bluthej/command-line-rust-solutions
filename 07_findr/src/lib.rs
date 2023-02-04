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
    let type_filter = |entry: &DirEntry| {
        cli.entry_type.is_empty()
            || cli.entry_type.iter().any(|entry_type| match entry_type {
                EntryType::F => entry.path().is_file() && !entry.path_is_symlink(),
                EntryType::D => entry.path().is_dir(),
                EntryType::L => entry.path_is_symlink(),
            })
    };
    let name_filter = |entry: &DirEntry| {
        cli.name.is_empty()
            || cli
                .name
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };
    for path in &cli.path {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            })
            .filter(type_filter)
            .filter(name_filter)
            .for_each(|entry| println!("{}", entry.path().display()));
    }
    Ok(())
}
