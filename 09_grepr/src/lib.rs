use clap::Parser;
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Search pattern")]
    pattern: Regex,

    #[arg(help = "Input file(s)", value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long, help = "Count occurrences")]
    count: bool,

    #[arg(short, long, help = "Case-insensitive")]
    insensitive: bool,

    #[arg(short = 'v', long = "invert-match", help = "Invert match")]
    invert_match: bool,

    #[arg(short, long, help = "Recursive search")]
    recursive: bool,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(|e| e.into())
}

pub fn run(cli: Cli) -> MyResult<()> {
    dbg!(&cli);
    Ok(())
}
