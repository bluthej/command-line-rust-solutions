use std::{
    cmp::Ordering,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

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

#[allow(unused)]
pub fn run(cli: Cli) -> MyResult<()> {
    let file1 = &cli.file1;
    let file2 = &cli.file2;

    if file1 == "-" && file2 == "-" {
        return Err("Both input files cannot be STDIN (\"-\")".into());
    }

    let del = cli.delimiter.to_string();
    let pref2 = del.repeat(cli.show_col1 as usize);
    let pref3 = del.repeat(cli.show_col1 as usize + cli.show_col2 as usize);

    let choose_case = |line: String| {
        if cli.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    let file1 = open(file1)?;
    let file2 = open(file2)?;
    let mut lines1 = file1.lines().filter_map(|l| l.ok().map(choose_case));
    let mut lines2 = file2.lines().filter_map(|l| l.ok().map(choose_case));
    let mut line1 = lines1.next();
    let mut line2 = lines2.next();
    loop {
        match (&line1, &line2) {
            (Some(l1), Some(l2)) => match l1.cmp(l2) {
                Ordering::Less => {
                    if cli.show_col1 {
                        println!("{}", l1);
                    }
                    line1 = lines1.next();
                }
                Ordering::Greater => {
                    if cli.show_col2 {
                        println!("{}{}", pref2, l2);
                    }
                    line2 = lines2.next();
                }
                Ordering::Equal => {
                    if cli.show_col3 {
                        println!("{}{}", pref3, l1);
                    }
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
            },
            (Some(l1), None) => {
                if cli.show_col1 {
                    println!("{}", l1);
                }
                line1 = lines1.next();
            }
            (None, Some(l2)) => {
                if cli.show_col2 {
                    println!("{}{}", pref2, l2);
                }
                line2 = lines2.next();
            }
            (None, None) => break,
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
