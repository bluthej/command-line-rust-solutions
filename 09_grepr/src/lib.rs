use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader},
};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Search pattern")]
    pattern: Regex,

    #[arg(help = "Input file(s)", value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(short, long, help = "Count occurrences")]
    count: bool,

    #[arg(short, long, help = "Case-insensitive")]
    insensitive: bool,

    #[arg(short = 'v', long = "invert-match", help = "Invert match")]
    invert_match: bool,

    #[arg(short, long, help = "Recursive search")]
    recursive: bool,
}

pub fn get_args() -> MyResult<Cli> {
    let mut cli = Cli::try_parse()?;
    cli.pattern = RegexBuilder::new(cli.pattern.as_str())
        .case_insensitive(cli.insensitive)
        .build()?;
    Ok(cli)
}

pub fn run(cli: Cli) -> MyResult<()> {
    let entries = find_files(&cli.files, cli.recursive);
    let n_entrie = entries.len();
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let matches = find_lines(file, &cli.pattern, cli.invert_match)?;
                    let mut prefix = String::new();
                    if n_entrie > 1 {
                        prefix.push_str(&format!("{filename}:"))
                    }
                    if cli.count {
                        println!("{}{}", prefix, matches.len());
                        continue;
                    }
                    for m in matches {
                        print!("{}{}", prefix, m);
                    }
                }
            },
        }
    }

    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut res = Vec::new();
    for path in paths {
        if path == "-" {
            res.push(Ok(path.to_string()));
            continue;
        }
        match fs::metadata(path) {
            Ok(attr) if !recursive && attr.is_dir() => {
                res.push(Err(format!("{} is a directory", path).into()));
                continue;
            }
            Err(e) => {
                res.push(Err(format!("{}: {}", path, e).into()));
                continue;
            }
            _ => (),
        }
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) if entry.path().is_file() => {
                    res.push(Ok(entry.path().to_string_lossy().to_string()))
                }
                Err(e) => res.push(Err(format!("{}: {}", path, e).into())),
                _ => (),
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");
        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }
        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut matches = Vec::new();
    let mut line = String::new();
    while file.read_line(&mut line)? > 0 {
        if (invert_match && !pattern.is_match(&line)) || (!invert_match && pattern.is_match(&line))
        {
            matches.push(line.clone());
        }
        line.clear();
    }
    Ok(matches)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
