mod month;

use chrono::prelude::*;
use clap::error::Result;
use clap::Parser;
use std::error::Error;

use crate::month::Month;

type MyResult<T> = Result<T, Box<dyn Error>>;

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
    println!("{:#?}", cli);
    Ok(())
}

fn last_day_in_month(year: i32, month: &Month) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month.clone() as u32, 1)
        .unwrap()
        .checked_add_months(chrono::Months::new(1))
        .unwrap()
        .pred_opt()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::{last_day_in_month, Month::*};
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
}
