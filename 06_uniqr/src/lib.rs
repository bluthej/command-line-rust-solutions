use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Input file", default_value = "-")]
    in_file: String,

    #[arg(help = "Output file")]
    out_file: Option<String>,

    #[arg(short, long, help = "Show counts")]
    count: bool,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(|e| e.into())
}

pub fn run(cli: Cli) -> MyResult<()> {
    let mut in_file = open_read(&cli.in_file).map_err(|e| format!("{}: {}", cli.in_file, e))?;
    let mut out_file = open_write(cli.out_file.as_deref())
        .map_err(|e| format!("{}: {}", cli.out_file.unwrap(), e))?;

    let mut print = |count: usize, line: &str| -> MyResult<()> {
        if cli.count {
            write!(out_file, "{:4} {}", count, line)?;
        } else {
            write!(out_file, "{}", line)?;
        }
        Ok(())
    };

    let mut line = String::new();
    let mut bytes = in_file.read_line(&mut line)?;
    let mut count = 0;
    let mut next_line = String::new();
    while bytes > 0 {
        let next_bytes = in_file.read_line(&mut next_line)?;
        count += 1;
        if line.trim_end() != next_line.trim_end() {
            print(count, &line)?;
            count = 0;
            line = next_line.clone();
            bytes = next_bytes;
        }
        next_line.clear();
    }
    Ok(())
}

fn open_read(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn open_write(filename: Option<&str>) -> MyResult<Box<dyn Write>> {
    match filename {
        Some(filename) => Ok(Box::new(BufWriter::new(File::create(filename)?))),
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}
