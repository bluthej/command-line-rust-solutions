use chrono::{DateTime, Local};
use clap::error::Result;
use clap::Parser;
use std::path::PathBuf;
use std::{error::Error, os::unix::prelude::MetadataExt};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Files and/or directories
    #[arg(default_value = ".", value_name = "PATH")]
    paths: Vec<String>,

    /// Long listing
    #[arg(short, long)]
    long: bool,

    /// Show all files
    #[arg(short, long)]
    all: bool,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from)
}

pub fn run(cli: Cli) -> MyResult<()> {
    let paths = find_files(&cli.paths, cli.all)?;
    let s = format_output(&paths)?;
    println!("{}", s);
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut res = Vec::new();
    for path in paths.iter().map(PathBuf::from) {
        if path.is_file() {
            res.push(path);
        } else if path.is_dir() {
            let Ok(dir) = path.read_dir() else {
                eprintln!("{}: No such file or directory", path.display());
                continue;
            };
            for item in dir {
                let Ok(entry) = item else {
                    eprintln!("{}: No such file or directory", path.display());
                    continue;
                };
                let path = entry.path();
                if show_hidden || !entry.file_name().to_string_lossy().starts_with('.') {
                    res.push(path);
                }
            }
        } else {
            eprintln!("{}: No such file or directory", path.display());
        }
    }
    Ok(res)
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let file_type = if path.is_dir() { "d" } else { "-" };
        let metadata = path.metadata()?;
        let permissions = metadata.mode();
        let n_links = metadata.nlink();
        let user = get_user_by_uid(metadata.uid()).unwrap();
        let group = get_group_by_gid(metadata.gid()).unwrap();
        let size = metadata.len();
        let modif: DateTime<Local> = metadata.modified()?.into();
        let modif = modif.format("%b %e %y %R");
        table.add_row(
            Row::new()
                .with_cell(file_type)
                .with_cell(permissions)
                .with_cell(n_links)
                .with_cell(user.name().to_string_lossy())
                .with_cell(group.name().to_string_lossy())
                .with_cell(size)
                .with_cell(modif)
                .with_cell(path.display()),
        );
    }
    Ok(format!("{}", table))
}

#[cfg(test)]
mod test {
    use super::find_files;

    #[test]
    fn test_find_files() {
        // Find all nonhidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
        // Find all entries in a directory
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);
        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }
}
