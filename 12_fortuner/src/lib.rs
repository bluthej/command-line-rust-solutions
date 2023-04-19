use regex::{Regex, RegexBuilder};
use std::error::Error;

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        help = "Input files or directories",
        value_name = "FILE",
        required = true
    )]
    sources: Vec<String>,

    #[arg(short = 'm', long, help = "Pattern")]
    pattern: Option<Regex>,

    #[arg(short, long, help = "Random seed")]
    seed: Option<u64>,

    #[arg(short, long, help = "Case-insensitive pattern matching")]
    insensitive: bool,
}

pub fn get_args() -> MyResult<Cli> {
    let mut cli = Cli::try_parse()?;
    if let Some(pattern) = cli.pattern {
        cli.pattern = RegexBuilder::new(pattern.as_str())
            .case_insensitive(cli.insensitive)
            .build()
            .ok();
    }
    Ok(cli)
}

pub fn run(cli: Cli) -> MyResult<()> {
    println!("{:#?}", cli);
    Ok(())
}
