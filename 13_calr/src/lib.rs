mod month;

use std::error::Error;

use clap::error::Result;
use clap::Parser;

use crate::month::Month;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Month name or number (1-12)
    #[arg(short)]
    month: Option<Month>,

    /// Show whole current year
    #[arg(short = 'y', long = "year", exclusive = true)]
    show_year: bool,

    /// Year (1-9999)
    #[arg(value_name = "YEAR", value_parser = clap::value_parser!(u32).range(..=9999))]
    year: Option<u32>,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from)
}

pub fn run(cli: Cli) -> MyResult<()> {
    println!("{:#?}", cli);
    Ok(())
}
