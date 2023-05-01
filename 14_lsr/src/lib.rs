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
    if cli.long {
        let long_out = format_output(&paths)?;
        println!("{}", long_out);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut res = Vec::new();
    for path in paths.iter().map(PathBuf::from) {
        if path.is_file() {
            res.push(path);
        } else if path.is_dir() {
            let dir = path.read_dir();
            let Ok(dir) = dir else {
                eprintln!("{}: {}", path.display(), dir.unwrap_err());
                continue;
            };
            for item in dir {
                let Ok(entry) = item else {
                    eprintln!("{}: {}", path.display(), item.unwrap_err());
                    continue;
                };
                let path = entry.path();
                if show_hidden || !entry.file_name().to_string_lossy().starts_with('.') {
                    res.push(path);
                }
            }
        } else {
            eprintln!("{}: {}", path.display(), path.metadata().unwrap_err());
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
        let permissions = format_mode(metadata.mode());
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

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
///
/// Example:
/// ```
/// let fmt = format_mode(0o755);
/// assert_eq!(fmt, "rwxr-xr-x");
/// ```
fn format_mode(mode: u32) -> String {
    let rights = "rwxrwxrwx";
    let masks = [
        0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
    ];
    rights
        .chars()
        .zip(masks.into_iter())
        .map(|(c, m)| if (mode & m) == m { c } else { '-' })
        .collect()
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{find_files, format_mode, format_output};

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

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);
        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);
        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }
        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);
        let res = format_output(&[bustle]);
        assert!(res.is_ok());
        let out = res.unwrap();
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);
        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }
}
