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
    lines: usize,
    bytes: Option<usize>,
}

pub fn run(config: Config) -> MyResult<()> {
    let n_files = config.files.len();
    for (i, filename) in config.files.iter().enumerate() {
        let mut reader = open(&filename).map_err(|e| format!("{}: {}", filename, e))?;

        let to_print = if let Some(bytes) = config.bytes {
            let mut buffer = vec![0; bytes];
            let n = reader.read(&mut buffer)?;
            buffer.resize(n, 0);
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            let mut buffer = String::new();
            for _ in 0..config.lines {
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

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .author("bluthej <joffrey.bluthe@e.email>")
        .about("Rust head")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("lines")
                .value_name("LINES")
                .short('n')
                .long("lines")
                .help("Number of lines")
                .default_value("10")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("bytes")
                .value_name("BYTES")
                .short('c')
                .long("bytes")
                .help("Number of bytes")
                .conflicts_with("lines")
                .action(ArgAction::Set),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>();
    let lines = matches
        .get_one::<String>("lines")
        .map(|s| parse_positive_int(s))
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();
    let bytes = matches
        .get_one::<String>("bytes")
        .map(|s| parse_positive_int(s))
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files,
        lines,
        bytes,
    })
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any String is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo");

    // A 0 is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0");
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
