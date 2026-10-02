use crate::{
    domain::{
        electronic::Electronic,
        identifiers::{Iban, Telephone},
        numeric::{Numeric, NumericRange, Quantity},
    },
    morphology::Inflection,
    numerals::Number,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Date {
    day: u8,
    month: u8,
    year: u16,
    dotted: bool,
}

impl Date {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let dotted = text.contains('.');
        let parts: Vec<_> = text.split(if dotted { '.' } else { '-' }).collect();
        let [first, second, third] = parts.as_slice() else {
            return None;
        };
        let (day, month, year) = if dotted {
            (*first, *second, *third)
        } else {
            (*third, *second, *first)
        };
        if year.len() != 4
            || !(1..=2).contains(&day.len())
            || !(1..=2).contains(&month.len())
            || (!dotted && (day.len() != 2 || month.len() != 2))
            || ![day, month, year]
                .iter()
                .all(|s| s.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let day: u8 = day.parse().ok()?;
        let month: u8 = month.parse().ok()?;
        let year: u16 = year.parse().ok()?;
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days = match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => return None,
        };
        if year == 0 || day == 0 || day > days {
            return None;
        }
        Some(Self {
            day,
            month,
            year,
            dotted,
        })
    }
    pub(crate) fn day(self) -> u8 {
        self.day
    }
    pub(crate) fn month(self) -> u8 {
        self.month
    }
    pub(crate) fn year(self) -> u16 {
        self.year
    }
    pub(crate) fn dotted(self) -> bool {
        self.dotted
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Clock {
    hour: u8,
    minute: u8,
}

impl Clock {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let (hour, minute) = text.split_once([':', '.'])?;
        if !(1..=2).contains(&hour.len())
            || minute.len() != 2
            || ![hour, minute]
                .iter()
                .all(|s| s.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let hour = hour.parse().ok()?;
        let minute = minute.parse().ok()?;
        if hour > 23 || minute > 59 {
            return None;
        }
        Some(Self { hour, minute })
    }
    pub(crate) fn hour(self) -> u8 {
        self.hour
    }
    pub(crate) fn minute(self) -> u8 {
        self.minute
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Value {
    Numeric(Numeric),
    Digits(String),
    Percent(Number, Option<Inflection>),
    Date(Date, bool),
    Time(Clock, bool),
    Quantity(Quantity),
    Lexical(crate::domain::lexicon::Lexeme, Option<Inflection>),
    Range(NumericRange),
    Telephone(Telephone),
    Iban(Iban),
    Roman(Numeric),
    Electronic(Electronic),
    Symbol(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerals::Amount;

    #[test]
    fn gregorian_boundaries_and_iso_shape_are_validated_before_rendering() {
        for year in 1..=9999_u16 {
            let expected_leap =
                year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
            assert_eq!(
                Date::parse(&format!("29.02.{year:04}")).is_some(),
                expected_leap
            );
        }
        for input in [
            "31.04.2026",
            "00.01.2026",
            "01.00.2026",
            "2026-3-04",
            "2026-03-4",
        ] {
            assert!(Date::parse(input).is_none());
        }
    }

    #[test]
    fn clock_and_amount_private_types_cannot_hold_invalid_values() {
        assert!(Clock::parse("23:59").is_some());
        assert!(Clock::parse("00:00").is_some());
        for input in ["24:00", "12:60", "12:5", "1:2:3"] {
            assert!(Clock::parse(input).is_none());
        }
        assert_eq!(Amount::parse("-0,05").unwrap().minor(), 5);
        assert!(Amount::parse("1,005").is_none());
    }
}
