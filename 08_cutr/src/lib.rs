use crate::Extract::*;
use clap::{ArgGroup, Parser};
use csv::{ReaderBuilder, StringRecord};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("list")
                .required(true)
                .args(&["bytes", "chars", "fields"]),
        ))]
pub struct Cli {
    #[arg(help = "Input file(s)", value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long = "delim", help = "Field delimiter", default_value = "\t", value_parser = parse_delim)]
    delimiter: u8,

    #[arg(short, long, help = "Selected bytes")]
    bytes: Option<String>,

    #[arg(short, long, help = "Selected characters")]
    chars: Option<String>,

    #[arg(short, long, help = "Selected fields")]
    fields: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let cli = Cli::try_parse().map_err(|e| -> Box<dyn Error> { e.into() })?;
    let files = cli.files;
    let delimiter = cli.delimiter;
    let extract = if let Some(list) = cli.bytes {
        Bytes(parse_pos(&list)?)
    } else if let Some(list) = cli.fields {
        Fields(parse_pos(&list)?)
    } else if let Some(list) = cli.chars {
        Chars(parse_pos(&list)?)
    } else {
        unreachable!();
    };
    Ok(Config {
        files,
        delimiter,
        extract,
    })
}

fn parse_delim(delimiter: &str) -> Result<u8, String> {
    let bytes = delimiter.as_bytes();
    if bytes.len() > 1 {
        return Err(format!("--delim \"{}\" must be a single byte", delimiter).into());
    }
    bytes
        .get(0)
        .ok_or(format!("--delim \"{}\" must be a single byte", delimiter))
        .copied()
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    if range.is_empty() {
        return Err(format!("empty range").into());
    }
    let value_error = |val: &str| format!("illegal list value: \"{}\"", val);
    range
        .split(',')
        .map(|val| {
            let range: Vec<&str> = val.split('-').collect();
            if range.len() > 2 {
                return Err(value_error(val));
            }
            let start = range.get(0).ok_or(value_error(val)).map(|bound| {
                bound
                    .starts_with("+")
                    .then(|| Err(value_error(val)))
                    .unwrap_or_else(|| {
                        bound
                            .parse::<NonZeroUsize>()
                            .map(|n| usize::from(n) - 1)
                            .map_err(|_| value_error(bound))
                    })
            })??;
            let end = range
                .get(1)
                .map(|bound| {
                    bound
                        .starts_with("+")
                        .then(|| Err(value_error(val)))
                        .unwrap_or_else(|| {
                            bound
                                .parse::<NonZeroUsize>()
                                .map(|n| usize::from(n))
                                .map_err(|_| value_error(bound))
                        })
                })
                .unwrap_or_else(|| Ok(start + 1))?;
            if range.len() == 2 && end <= start {
                return Err(format!(
                    "First number in range ({}) must be lower than second number ({})",
                    start, end
                ));
            };
            Ok(start..end)
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

#[cfg(test)]
mod unit_tests {
    use crate::{extract_bytes, extract_chars, parse_pos};
    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());
        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);
        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);
        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);
        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);
        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);
        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());
        let res = parse_pos(",");
        assert!(res.is_err());
        let res = parse_pos("1,");
        assert!(res.is_err());
        let res = parse_pos("1-");
        assert!(res.is_err());
        let res = parse_pos("1-1-1");
        assert!(res.is_err());
        let res = parse_pos("1-1-a");
        assert!(res.is_err());
        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string())
    }
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => match &config.extract {
                Bytes(pos) => {
                    let mut line = String::new();
                    while file.read_line(&mut line)? > 0 {
                        println!("{}", extract_bytes(&line, &pos));
                        line.clear();
                    }
                }
                Chars(pos) => {
                    let mut line = String::new();
                    while file.read_line(&mut line)? > 0 {
                        println!("{}", extract_chars(&line, &pos));
                        line.clear();
                    }
                }
                Fields(pos) => {
                    let delim = String::from_utf8(vec![config.delimiter])?;
                    let mut rdr = ReaderBuilder::new()
                        .delimiter(config.delimiter)
                        .from_reader(file);
                    println!("{}", extract_fields(rdr.headers()?, pos).join(&delim));
                    for record in rdr.records() {
                        println!("{}", extract_fields(&record?, pos).join(&delim));
                    }
                }
            },
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

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<char> = line.chars().collect();
    char_pos
        .iter()
        .filter_map(|range| {
            chars
                .get(range.clone())
                .map(|chars| chars.iter().collect::<String>())
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes: Vec<u8> = line.as_bytes().to_vec();
    byte_pos
        .iter()
        .filter_map(|range| {
            bytes
                .get(range.clone())
                .map(|bytes| String::from_utf8_lossy(bytes))
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let fields: Vec<&str> = record.iter().collect();
    field_pos
        .iter()
        .filter_map(|range| fields.get(range.clone()))
        .flatten()
        .map(|&s| s.to_owned())
        .collect()
}
