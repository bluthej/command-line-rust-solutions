use std::error::Error;
use std::str::FromStr;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl FromStr for Month {
    type Err = ParseMonthError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input.to_lowercase();
        if s == "j" || s == "ju" || s == "a" || s == "m" || s == "ma" {
            return Err(ParseMonthError {
                msg: format!("Ambiguous abbreviation '{}'", input),
            });
        }
        if "january".starts_with(&s) || s == "1" {
            return Ok(Month::January);
        }
        if "february".starts_with(&s) || s == "2" {
            return Ok(Month::February);
        }
        if "march".starts_with(&s) || s == "3" {
            return Ok(Month::March);
        }
        if "april".starts_with(&s) || s == "4" {
            return Ok(Month::April);
        }
        if "may".starts_with(&s) || s == "5" {
            return Ok(Month::May);
        }
        if "june".starts_with(&s) || s == "6" {
            return Ok(Month::June);
        }
        if "july".starts_with(&s) || s == "7" {
            return Ok(Month::July);
        }
        if "august".starts_with(&s) || s == "8" {
            return Ok(Month::August);
        }
        if "september".starts_with(&s) || s == "9" {
            return Ok(Month::September);
        }
        if "october".starts_with(&s) || s == "10" {
            return Ok(Month::October);
        }
        if "november".starts_with(&s) || s == "11" {
            return Ok(Month::November);
        }
        if "december".starts_with(&s) || s == "12" {
            return Ok(Month::December);
        }
        Err(ParseMonthError {
            msg: format!("Unreckognized string '{}'", input),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct ParseMonthError {
    msg: String,
}

impl From<ParseMonthError> for Box<dyn Error + Send + Sync> {
    fn from(error: ParseMonthError) -> Self {
        error.msg.into()
    }
}

#[cfg(test)]
mod tests {
    use super::Month;
    use std::str::FromStr;

    #[test]
    fn test_month_from_str() {
        // Valid spellings and abbreviations
        let res = Month::from_str("January");
        assert_eq!(res, Ok(Month::January));
        let res = Month::from_str("s");
        assert_eq!(res, Ok(Month::September));
        let res = Month::from_str("3");
        assert_eq!(res, Ok(Month::March));
        let res = Month::from_str("apr");
        assert_eq!(res, Ok(Month::April));
        let res = Month::from_str("jUnE");
        assert_eq!(res, Ok(Month::June));
        // Invalid abbreviation => ambiguous
        let res = Month::from_str("ju");
        assert!(res.is_err());
        // Invalid string
        let res = Month::from_str("blargh");
        assert!(res.is_err());
    }
}
