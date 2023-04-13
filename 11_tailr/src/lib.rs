use crate::TakeValue::*;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
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
    let input = if !s.starts_with(['+', '-']) {
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
    for filename in cli.files {
        let file = File::open(&filename).map_err(|e| format!("{}: {}", filename, e))?;
        let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
        match &cli.bytes {
            Some(_) => todo!(),
            None => print_lines(BufReader::new(file), &cli.lines, total_lines),
        };
    }
    Ok(())
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = File::open(filename)?;
    let mut buf = String::new();
    let n_bytes = file.read_to_string(&mut buf)? as i64;
    let n_lines = buf.lines().count() as i64;
    Ok((n_lines, n_bytes))
}

fn print_lines(file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start_line) = get_start_index(num_lines, total_lines) {
        for line in file.lines().skip(start_line as usize) {
            println!("{}", line?);
        }
    }
    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    if total == 0 {
        return None;
    }
    match take_val {
        PlusZero => Some(total as u64 - 1),
        &TakeNum(index) => match index.cmp(&0) {
            std::cmp::Ordering::Less => Some((total as u64).saturating_sub(-index as u64)),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => {
                if index > total {
                    None
                } else {
                    Some((index - 1) as u64)
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};
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

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));
        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);
        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));
        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);
        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);
        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);
        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));
        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));
        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
