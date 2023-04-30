mod month;

use ansi_term::Style;
use chrono::prelude::*;
use clap::error::Result;
use clap::Parser;
use std::error::Error;
use std::str::FromStr;

use crate::month::Month;

type MyResult<T> = Result<T, Box<dyn Error>>;

const LINE_WIDTH: usize = 22;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Month name or number (1-12)
    #[arg(short)]
    month: Option<Month>,

    /// Show whole current year
    #[arg(short = 'y', long = "year", exclusive = true)]
    show_year: bool,

    /// Year (1-9999)
    #[arg(value_name = "YEAR", value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,
}

pub fn get_args() -> MyResult<Cli> {
    Cli::try_parse().map_err(From::from)
}

pub fn run(cli: Cli) -> MyResult<()> {
    let today = Local::now().date_naive();
    let month = cli.month.unwrap_or(
        Month::from_str(today.month().to_string().as_ref()).map_err(|e| format!("{:?}", e))?,
    );
    let year = cli.year.unwrap_or(today.year());
    let fmt_month = format_month(year, &month, true, today);
    for line in fmt_month {
        println!("{}", line);
    }
    Ok(())
}

fn format_month(year: i32, month: &Month, print_year: bool, today: NaiveDate) -> Vec<String> {
    let title = if print_year {
        format!("{:?} {}", month, year)
    } else {
        format!("{:?}", month)
    };
    let first_line = format!("{:^width$}  ", title, width = LINE_WIDTH - 2);
    let second_line = "Su Mo Tu We Th Fr Sa  ".to_string();

    let last_day = last_day_in_month(year, month);
    let month = month.clone() as u32;
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..first_day.weekday().number_from_sunday())
        .map(|_| "  ".to_string())
        .chain(
            first_day
                .iter_days()
                .take_while(|d| d <= &last_day)
                .map(|day| {
                    let fmt = format!("{:>2}", day.day());
                    if day == today {
                        Style::new().reverse().paint(fmt).to_string()
                    } else {
                        fmt
                    }
                }),
        )
        .collect();
    days.resize(6 * 7, "  ".to_string());

    [first_line, second_line]
        .into_iter()
        .chain(
            days.chunks(7)
                .map(|week| format!("{:width$}  ", week.join(" "), width = LINE_WIDTH - 2)),
        )
        .collect()
}

fn last_day_in_month(year: i32, month: &Month) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month.clone() as u32, 1)
        .and_then(|date| date.checked_add_months(chrono::Months::new(1)))
        .and_then(|date| date.pred_opt())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, Month::*};
    use chrono::NaiveDate;

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            Some(last_day_in_month(2020, &January)),
            NaiveDate::from_ymd_opt(2020, 1, 31)
        );
        assert_eq!(
            Some(last_day_in_month(2020, &February)),
            NaiveDate::from_ymd_opt(2020, 2, 29)
        );
        assert_eq!(
            Some(last_day_in_month(2020, &April)),
            NaiveDate::from_ymd_opt(2020, 4, 30)
        );
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, &February, true, today), leap_february);
        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, &May, false, today), may);
        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, &April, true, today), april_hl);
    }
}
