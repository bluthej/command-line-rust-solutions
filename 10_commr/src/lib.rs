use std::error::Error;

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Input file 1")]
    file1: String,

    #[arg(help = "Input file 2")]
    file2: String,

    #[arg(short = '1', help = "Suppress printing of column 1")]
    show_col1: bool,

    #[arg(short = '2', help = "Suppress printing of column 2")]
    show_col2: bool,

    #[arg(short = '3', help = "Suppress printing of column 3")]
    show_col3: bool,

    #[arg(short, help = "Case-insensitive comparison of lines")]
    insensitive: bool,

    #[arg(
        short,
        long = "output-delimiter",
        help = "Output delimiter",
        default_value = "\t"
    )]
    delimiter: char,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from).map(|cli| Cli {
        show_col1: !cli.show_col1,
        show_col2: !cli.show_col2,
        show_col3: !cli.show_col3,
        ..cli
    })
}

pub fn run(cli: Cli) -> MyResult<()> {
    dbg!(&cli);
    Ok(())
}
