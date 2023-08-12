use std::str::FromStr;

use pest::Parser;
use rand::Rng;
use thiserror::Error;

pub trait AmountValue:
    FromStr
    + PartialEq
    + Eq
    + rand::distributions::uniform::SampleUniform
    + Copy
    + std::cmp::PartialOrd
    + std::cmp::Ord
    + std::fmt::Display
{
}

#[derive(Debug, Error)]
pub enum AmountError {
    #[error("Found malformed input. Expected a single value, or two values, seperated by '-'")]
    MalformedInput,

    #[error("Could not parse value from the provided string")]
    ParseError,

    #[error("Could not parse value from the provided string")]
    PestError(#[from] pest::error::Error<super::grammar::Rule>),

    #[error("Missing inputs")]
    MissingInput,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Amount<T: AmountValue> {
    Range { start: T, finish: T },
    Single(T),
}

impl<T: AmountValue> FromStr for Amount<T> {
    type Err = AmountError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use super::grammar::{CommandsParser, Rule};

        let result = CommandsParser::parse(Rule::amount, s)?;

        let mut range = dbg!(result)
            .next()
            .ok_or(AmountError::MissingInput)?
            .into_inner();

        let start = range.next().ok_or(AmountError::MissingInput)?;

        if let Some(end) = range.next() {
            let start = start
                .as_str()
                .trim()
                .parse()
                .map_err(|_| AmountError::ParseError)?;
            let finish = dbg!(end.as_str().trim())
                .parse()
                .map_err(|_| AmountError::ParseError)?;

            Ok(Amount::Range { start, finish })
        } else {
            Ok(Amount::Single(
                dbg!(start.as_str().trim())
                    .parse()
                    .map_err(|_| AmountError::ParseError)?,
            ))
        }
    }
}

impl<T: AmountValue> Amount<T> {
    pub fn get_value(&self) -> T {
        match self {
            Self::Range { start, finish } => {
                let range = (*start)..(*finish);

                let mut rng = rand::thread_rng();

                rng.gen_range(range)
            }
            Self::Single(number) => *number,
        }
    }
}

impl<T: AmountValue> std::fmt::Display for Amount<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.get_value();

        write!(f, "{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl AmountValue for u8 {}

    /// This was created to assure that `split_once` returned [`None`] when I thought it would
    #[test]
    fn test_split_once_option_semantics() {
        let range = "1-2";

        assert!(range.split_once('-').is_some());

        let not_range = "12";

        assert!(not_range.split_once('-').is_none());
    }

    #[test]
    fn test_range_parse() {
        let range = "1-12";

        let parsed_range: Amount<u8> = range.parse().unwrap();

        assert_eq!(
            parsed_range,
            Amount::Range {
                start: 1,
                finish: 12,
            }
        );

        let range_whitespace = "1 - 12";

        let parsed_range: Amount<u8> = range_whitespace.parse().unwrap();

        assert_eq!(
            parsed_range,
            Amount::Range {
                start: 1,
                finish: 12,
            }
        );

        let single = "12";

        let parsed_single: Amount<u8> = single.parse().unwrap();

        assert_eq!(parsed_single, Amount::Single(12));
    }
}
