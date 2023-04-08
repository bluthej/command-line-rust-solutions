use crate::TakeValue::*;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Clone)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Input file(s)", value_name = "FILE", required = true)]
    files: Vec<String>,

    #[arg(short = 'n', long, help = "Number of lines", default_value = "-10", value_parser = parse_num)]
    lines: TakeValue,

    #[arg(short = 'c', long, help = "Number of bytes", value_parser = parse_num, conflicts_with = "lines")]
    bytes: Option<TakeValue>,

    #[arg(short, long, help = "Suppress headers")]
    quiet: bool,
}

fn parse_num(s: &str) -> Result<TakeValue, String> {
    let input = if !s.is_empty() && !s.starts_with('+') && !s.starts_with('-') {
        format!("-{s}")
    } else {
        s.to_string()
    };
    if input == "+0" {
        Ok(PlusZero)
    } else {
        Ok(TakeNum(input.parse::<i64>().map_err(|_| s)?))
    }
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from)
}

#[allow(unused)]
pub fn run(cli: Cli) -> MyResult<()> {
    println!("{:#?}", cli);
    Ok(())
}

#[allow(unused)]
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_num, TakeValue::*};
    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));
        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));
        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);
        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));
        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));
        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "3.14");
        // Any noninteger string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "foo");
    }
}
