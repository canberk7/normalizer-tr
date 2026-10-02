use crate::{
    SegmentKind,
    model::{Clock, Date, Value},
    morphology::{Inflection, Spoken},
    numerals,
};

const MONTHS: [&str; 12] = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
    "Kasım", "Aralık",
];

pub(crate) fn date_spoken(date: Date) -> Spoken {
    let mut year = numerals::cardinal(u64::from(date.year()));
    let prefix = format!(
        "{} {} ",
        numerals::cardinal(u64::from(date.day())).into_text(),
        MONTHS[usize::from(date.month() - 1)]
    );
    year.prefix(&prefix);
    year
}

pub(crate) fn time_spoken(time: Clock) -> Spoken {
    let hour = numerals::cardinal(u64::from(time.hour()));
    if time.minute() == 0 {
        hour
    } else {
        let mut minute = numerals::cardinal(u64::from(time.minute()));
        minute.prefix(&format!("{} ", hour.into_text()));
        minute
    }
}

pub(crate) fn render(value: &Value) -> (SegmentKind, &'static str, String) {
    match value {
        Value::Numeric(number) => (number.kind(), "number", number.render().into_text()),
        Value::Digits(text) => (SegmentKind::Digits, "digits.hint", numerals::digits(text)),
        Value::Date(date, locative) => {
            let mut spoken = date_spoken(*date);
            if *locative {
                spoken.inflect(Inflection::Locative);
            }
            (SegmentKind::Date, "date.gregorian", spoken.into_text())
        }
        Value::Time(time, locative) => {
            let mut spoken = time_spoken(*time);
            if *locative {
                spoken.inflect(Inflection::Locative);
            }
            (SegmentKind::Time, "time.digital", spoken.into_text())
        }
        Value::Percent(number, case) => {
            let mut spoken = numerals::number(number);
            if let Some(case) = case {
                spoken.inflect(*case);
            }
            spoken.prefix("yüzde ");
            (SegmentKind::Percent, "percent", spoken.into_text())
        }
        Value::Quantity(quantity) => (
            if quantity.is_money() {
                SegmentKind::Money
            } else {
                SegmentKind::Unit
            },
            "quantity",
            quantity.render().into_text(),
        ),
        Value::Lexical(entry, case) => {
            let mut spoken = Spoken::lexical(entry.output, entry.target);
            if let Some(case) = case {
                spoken.inflect(*case);
            }
            (
                SegmentKind::Abbreviation,
                "abbreviation",
                spoken.into_text(),
            )
        }
        Value::Range(range) => (SegmentKind::Range, "range.context", range.render()),
        Value::Telephone(phone) => (SegmentKind::Telephone, "telephone.tr", phone.render()),
        Value::Iban(iban) => (SegmentKind::Iban, "iban.tr.mod97", iban.render()),
        Value::Roman(number) => (
            SegmentKind::Roman,
            "roman.canonical",
            number.render().into_text(),
        ),
        Value::Electronic(address) => (
            SegmentKind::Electronic,
            "electronic.ascii",
            address.render().to_owned(),
        ),
        Value::Symbol(text) => (SegmentKind::Symbol, "symbol.prose", text.clone()),
    }
}
