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

    #[arg(short, long, help = "Show line count")]
    lines: bool,

    #[arg(short, long, help = "Show word count")]
    words: bool,

    #[arg(short = 'c', long, help = "Show byte count", conflicts_with = "chars")]
    bytes: bool,

    #[arg(short = 'm', long, help = "Show character count")]
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

const JUSTIF: usize = 8;

pub fn get_args() -> MyResult<Cli> {
    let mut cli = Cli::parse();
    if !cli.lines && !cli.words && !cli.bytes && !cli.chars {
        (cli.lines, cli.words, cli.bytes) = (true, true, true);
    }
    Ok(cli)
}

pub fn run(cli: Cli) -> MyResult<()> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    for filename in &cli.file {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let info = count(file)?;
                if cli.lines {
                    num_lines += info.num_lines;
                    print!("{:JUSTIF$}", info.num_lines);
                }
                if cli.words {
                    num_words += info.num_words;
                    print!("{:JUSTIF$}", info.num_words);
                }
                if cli.bytes {
                    num_bytes += info.num_bytes;
                    print!("{:JUSTIF$}", info.num_bytes);
                }
                if cli.chars {
                    num_chars += info.num_chars;
                    print!("{:JUSTIF$}", info.num_chars);
                }
                if filename != "-" {
                    print!(" {}", filename);
                }
                println!("");
            }
        }
    }

    if cli.file.len() > 1 {
        if cli.lines {
            print!("{:JUSTIF$}", num_lines);
        }
        if cli.words {
            print!("{:JUSTIF$}", num_words);
        }
        if cli.bytes {
            print!("{:JUSTIF$}", num_bytes);
        }
        if cli.chars {
            print!("{:JUSTIF$}", num_chars);
        }
        println!(" total");
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();
    while file.read_line(&mut line)? > 0 {
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        num_bytes += line.bytes().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your haln.\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 47,
            num_bytes: 47,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
