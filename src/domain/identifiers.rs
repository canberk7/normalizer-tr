use crate::numerals;

#[derive(Clone, Debug)]
pub(crate) struct Telephone {
    national: String,
    international: bool,
    national_zero: bool,
}

impl Telephone {
    pub(crate) fn parse(text: &str, explicit: bool) -> Option<Self> {
        if !text
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | ' ' | '-' | '(' | ')'))
        {
            return None;
        }
        if text.matches('+').count() > usize::from(text.starts_with('+')) {
            return None;
        }
        let mut depth = 0;
        for ch in text.chars() {
            match ch {
                '(' if depth == 0 => depth = 1,
                ')' if depth == 1 => depth = 0,
                '(' | ')' => return None,
                _ => {}
            }
        }
        if depth != 0
            || text.ends_with(['-', ' ', '('])
            || text.contains("--")
            || text.contains("  ")
        {
            return None;
        }
        let international = text.starts_with("+90");
        let cleaned: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        let (national, national_zero) = if international {
            if cleaned.len() != 12 {
                return None;
            }
            (&cleaned[2..], false)
        } else if cleaned.len() == 11 && cleaned.starts_with('0') {
            (&cleaned[1..], true)
        } else if cleaned.len() == 10 && explicit {
            (cleaned.as_str(), false)
        } else {
            return None;
        };
        let groups: Vec<_> = text
            .split([' ', '-', '(', ')'])
            .filter(|s| !s.is_empty())
            .collect();
        if groups.len() > 1 {
            let sizes: Vec<_> = groups
                .iter()
                .map(|s| s.trim_start_matches('+').len())
                .collect();
            let allowed = if international {
                sizes == [2, 3, 3, 2, 2]
            } else if national_zero {
                sizes == [4, 3, 2, 2] || sizes == [1, 3, 3, 2, 2]
            } else {
                explicit && sizes == [3, 3, 2, 2]
            };
            if !allowed {
                return None;
            }
        }
        Some(Self {
            national: national.to_owned(),
            international,
            national_zero,
        })
    }
    pub(crate) fn render(&self) -> String {
        let mut chunks = Vec::new();
        if self.international {
            chunks.push("artı doksan".to_owned());
        }
        if self.national_zero {
            chunks.push("sıfır".to_owned());
        }
        for range in [0..3, 3..6, 6..8, 8..10] {
            let digits = &self.national[range];
            let output = if digits.starts_with('0') {
                numerals::digits(digits)
            } else {
                // The constructor restricts every group to at most three ASCII digits.
                let value = digits
                    .bytes()
                    .fold(0_u64, |n, b| n * 10 + u64::from(b - b'0'));
                numerals::cardinal(value).into_text()
            };
            chunks.push(output);
        }
        chunks.join(" ")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Iban {
    canonical: String,
}

impl Iban {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let iban = Self::shaped(text)?;
        let canonical = &iban.canonical;
        let check = (canonical.as_bytes()[2] - b'0') * 10 + canonical.as_bytes()[3] - b'0';
        if !(2..=98).contains(&check) {
            return None;
        }
        let mut remainder = 0_u32;
        for byte in canonical.as_bytes()[4..]
            .iter()
            .copied()
            .chain("2927".bytes())
            .chain(canonical.as_bytes()[2..4].iter().copied())
        {
            remainder = (remainder * 10 + u32::from(byte - b'0')) % 97;
        }
        (remainder == 1).then_some(iban)
    }
    /// A Turkish IBAN by its written shape alone; the check digits are not verified.
    pub(crate) fn shaped(text: &str) -> Option<Self> {
        let groups: Vec<_> = text.split(' ').collect();
        if groups.iter().any(|g| g.is_empty()) {
            return None;
        }
        if groups.len() > 1
            && (groups.len() != 7
                || groups[..6].iter().any(|g| g.len() != 4)
                || groups[6].len() != 2)
        {
            return None;
        }
        let canonical = groups.concat();
        if canonical.len() != 26
            || !canonical.starts_with("TR")
            || !canonical[2..].bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        Some(Self { canonical })
    }
    pub(crate) fn render(&self) -> String {
        let mut output = String::from("te re ");
        output.push_str(&numerals::digits(&self.canonical[2..4]));
        for group in self.canonical.as_bytes()[4..].chunks(4) {
            output.push_str(", ");
            let digits: String = group.iter().map(|b| char::from(*b)).collect();
            output.push_str(&numerals::digits(&digits));
        }
        output
    }
}
