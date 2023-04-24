use std::error::Error;

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, help = "Month name or number (1-12)")]
    month: Option<u32>,

    #[arg(short = 'y', long = "year", help = "Show whole current year")]
    show_year: bool,

    #[arg(value_name = "YEAR", help = "Year (1-9999)", value_parser = check_year_range)]
    year: u32,
}

fn check_year_range(s: &str) -> Result<u32, String> {
    s.parse().map_err(|e| format!("{e}")).and_then(|year| {
        if !(1..=9999).contains(&year) {
            Err("the accepted range is 1-9999".to_string())
        } else {
            Ok(year)
        }
    })
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from)
}

pub fn run(cli: Cli) -> MyResult<()> {
    println!("{:#?}", cli);
    Ok(())
}
