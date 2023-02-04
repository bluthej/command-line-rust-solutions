use clap::{Parser, ValueEnum};
use regex::Regex;
use std::error::Error;

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
    println!("{:?}", cli);
    Ok(())
}
