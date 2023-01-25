use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Input file(s)", default_value = "-")]
    file: Vec<String>,

    #[arg(short = 'n', long, help = "Number of lines", default_value = "10")]
    lines: NonZeroUsize,

    #[arg(short = 'c', long, help = "Number of bytes", conflicts_with = "lines")]
    bytes: Option<NonZeroUsize>,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(|e| {
        let err = e.to_string();
        if err.contains("--bytes") && !err.contains("--lines") {
            err.replace("invalid value", "illegal byte count --")
                .replace("'", "")
        } else if err.contains("--lines") && !err.contains("--bytes") {
            err.replace("invalid value", "illegal line count --")
                .replace("'", "")
        } else {
            err
        }
        .into()
    })
}

pub fn run(cli: Cli) -> MyResult<()> {
    let n_files = cli.file.len();
    for (i, filename) in cli.file.iter().enumerate() {
        let mut reader = open(&filename).map_err(|e| format!("{}: {}", filename, e))?;

        let to_print = if let Some(bytes) = cli.bytes {
            let mut buffer = vec![0; bytes.into()];
            let n = reader.read(&mut buffer)?;
            buffer.resize(n, 0);
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            let mut buffer = String::new();
            for _ in 0..cli.lines.into() {
                reader.read_line(&mut buffer)?;
            }
            buffer
        };
        if i > 0 {
            println!();
        }
        if n_files > 1 {
            println!("==> {} <==", filename);
        }
        if !to_print.is_empty() {
            print!("{}", to_print);
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
