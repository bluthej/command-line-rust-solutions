use clap::{Arg, ArgAction, Command};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(reader) => {
                let mut n = 0;
                for line in reader.lines() {
                    let line = line?;
                    if config.number_lines || config.number_nonblank_lines && !line.is_empty() {
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

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("catr")
        .version("0.1.0")
        .author("bluthej <joffrey.bluthe@e.email>")
        .about("Rust cat")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .num_args(1..)
                .default_value("-"),
        )
        .arg(
            Arg::new("number_lines")
                .short('n')
                .long("number")
                .help("Number lines")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("number_nonblank_lines")
                .short('b')
                .long("number-nonblank")
                .help("Number nonblank lines")
                .conflicts_with("number_lines")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let files: Vec<String> = matches.get_many("files").unwrap().cloned().collect();
    let number_lines = matches.get_flag("number_lines");
    let number_nonblank_lines = matches.get_flag("number_nonblank_lines");

    Ok(Config {
        files,
        number_lines,
        number_nonblank_lines,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
