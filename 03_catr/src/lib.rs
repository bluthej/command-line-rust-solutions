use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Input file(s)", default_value = "-")]
    file: Vec<String>,

    #[arg(short, long = "number")]
    number_lines: bool,

    #[arg(short = 'b', long = "number-nonblank", conflicts_with = "number_lines")]
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Cli> {
    Ok(Cli::parse())
}

pub fn run(cli: Cli) -> MyResult<()> {
    for filename in cli.file {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(file) => {
                let mut n = 0;
                for line in file.lines() {
                    let line = line?;
                    if cli.number_lines || cli.number_nonblank_lines && !line.is_empty() {
                        n += 1;
                        println!("{:6}\t{}", n, line);
                        continue;
                    }
                    println!("{}", line);
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
