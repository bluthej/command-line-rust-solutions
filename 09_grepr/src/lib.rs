use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::{error::Error, fs};
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
    println!("pattern \"{}\"", cli.pattern);

    let entries = find_files(&cli.files, cli.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => println!("file \"{}\"", filename),
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
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};
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
}
