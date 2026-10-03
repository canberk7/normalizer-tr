use crate::{morphology::Style, numerals};

#[derive(Clone, Debug)]
pub(crate) struct Electronic {
    spoken: String,
}

fn tld(text: &str) -> Option<&'static str> {
    match text.to_ascii_lowercase().as_str() {
        "com" => Some("kom"),
        "net" => Some("net"),
        "org" => Some("org"),
        "tr" => Some("te re"),
        "gov" => Some("gov"),
        "edu" => Some("edu"),
        "app" => Some("app"),
        _ => None,
    }
}

fn host(text: &str) -> bool {
    if text.len() > 253 || text.is_empty() || !text.is_ascii() {
        return false;
    }
    let labels: Vec<_> = text.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && !label
                    .get(..4)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("xn--"))
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        && tld(labels[labels.len() - 1]).is_some()
}

fn characters(text: &str, domain: bool) -> String {
    let mut words = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        let b = text.as_bytes()[offset];
        if b.is_ascii_alphabetic() {
            let start = offset;
            while offset < text.len() && text.as_bytes()[offset].is_ascii_alphabetic() {
                offset += 1;
            }
            let chunk = &text[start..offset];
            words.push(if domain {
                tld(chunk).unwrap_or(chunk).to_owned()
            } else {
                chunk.to_owned()
            });
        } else if b.is_ascii_digit() {
            let start = offset;
            while offset < text.len() && text.as_bytes()[offset].is_ascii_digit() {
                offset += 1;
            }
            words.push(numerals::digits(&text[start..offset]));
        } else {
            let word = match b {
                b'.' => "nokta",
                b'@' => "et",
                b'/' => "eğik çizgi",
                b':' => "iki nokta",
                b'?' => "soru işareti",
                b'=' => "eşittir",
                b'&' => "ve",
                b'#' => "kare",
                b'%' => "yüzde",
                b'-' => "tire",
                b'_' => "alt çizgi",
                b'+' => "artı",
                b'(' => "aç parantez",
                b')' => "kapat parantez",
                b'!' => "ünlem",
                b'$' => "dolar işareti",
                b'\'' => "kesme",
                b'*' => "yıldız",
                b',' => "virgül",
                b';' => "noktalı virgül",
                b'~' => "tilde",
                _ => "",
            };
            if !word.is_empty() {
                words.push(word.to_owned());
            }
            offset += 1;
        }
    }
    words.join(" ")
}

impl Electronic {
    pub(crate) fn parse(text: &str, bare: bool) -> Option<Self> {
        if !text.is_ascii() || text.bytes().any(|b| b.is_ascii_control()) {
            return None;
        }
        if text.contains('@') && !text.contains("://") && !text.starts_with("www.") {
            if text.contains("://") || text.matches('@').count() != 1 {
                return None;
            }
            let (local, domain) = text.split_once('@')?;
            if local.is_empty()
                || local.len() > 64
                || local.starts_with('.')
                || local.ends_with('.')
                || local.contains("..")
                || !local
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-'))
                || !host(domain)
            {
                return None;
            }
            return Some(Self {
                spoken: format!("{} et {}", characters(local, false), domain_words(domain)),
            });
        }
        let (prefix, body) = if text
            .get(..8)
            .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
        {
            (
                "ha te te pe es iki nokta eğik çizgi eğik çizgi ",
                &text[8..],
            )
        } else if text
            .get(..7)
            .is_some_and(|s| s.eq_ignore_ascii_case("http://"))
        {
            ("ha te te pe iki nokta eğik çizgi eğik çizgi ", &text[7..])
        } else if text.starts_with("www.") || bare {
            ("", text)
        } else {
            return None;
        };
        if body.is_empty() {
            return None;
        }
        let split = body.find(['/', '?', '#']).unwrap_or(body.len());
        let authority = &body[..split];
        let rest = &body[split..];
        let domain = if let Some((domain, port)) = authority.split_once(':') {
            if port.is_empty() || port.len() > 5 || !port.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let port_number = port.parse::<u32>().ok()?;
            if !(1..=65535).contains(&port_number) {
                return None;
            }
            domain
        } else {
            authority
        };
        if !host(domain) || authority.contains('@') {
            return None;
        }
        let mut depth = 0_i32;
        let mut i = 0;
        while i < rest.len() {
            let b = rest.as_bytes()[i];
            if b == b'%' {
                if i + 2 >= rest.len()
                    || !rest.as_bytes()[i + 1..i + 3]
                        .iter()
                        .all(u8::is_ascii_hexdigit)
                {
                    return None;
                }
                i += 3;
                continue;
            }
            if !(b.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@/?#".contains(&b)) {
                return None;
            }
            if b == b'(' {
                depth += 1;
            }
            if b == b')' {
                depth -= 1;
            }
            if depth < 0 {
                return None;
            }
            i += 1;
        }
        if depth != 0 || rest.matches('#').count() > 1 {
            return None;
        }
        let mut spoken = format!("{prefix}{}", domain_words(domain));
        if let Some((_, port)) = authority.split_once(':') {
            spoken.push_str(" iki nokta ");
            spoken.push_str(&numerals::digits(port));
        }
        if !rest.is_empty() {
            spoken.push(' ');
            spoken.push_str(&characters(rest, false));
        }
        Some(Self { spoken })
    }
    /// The address as it is said; the spoken style names `/` slaş, as people say it, the
    /// letters of `http` and `https` by their TDK names, and `com` kom also before another
    /// label (`.com.tr`).
    pub(crate) fn render(&self, style: Style) -> String {
        match style {
            Style::Exact => self.spoken.clone(),
            Style::Spoken => self
                .spoken
                .replace("eğik çizgi", "slaş")
                .replacen("ha te te pe es ", "he te te pe se ", 1)
                .replacen("ha te te pe ", "he te te pe ", 1)
                .replace(" nokta com nokta ", " nokta kom nokta "),
        }
    }
}

fn domain_words(domain: &str) -> String {
    let parts: Vec<_> = domain.split('.').collect();
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i == parts.len() - 1 {
                tld(part).unwrap_or(part).to_owned()
            } else if i == 0 && *part == "www" {
                "çift ve çift ve çift ve".to_owned()
            } else {
                characters(part, false)
            }
        })
        .collect::<Vec<_>>()
        .join(" nokta ")
}

pub(crate) fn looks_like(text: &str) -> bool {
    text.contains('@') || text.contains("://") || text.starts_with("www.")
}

/// A hashtag as everyday speech says it: heşteg, then its words, split where a lowercase
/// letter meets a capital or a letter meets a digit, with numbers said as numbers:
/// `#2026hedefleri` is heşteg iki bin yirmi altı hedefleri. A single digit is a rank: `#1` is
/// bir numara.
pub(crate) fn spoken_hashtag(body: &str) -> String {
    if let Ok(rank) = body.parse::<u64>()
        && body.len() == 1
    {
        return format!("{} numara", numerals::cardinal(rank).into_text());
    }
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut previous: Option<char> = None;
    for ch in body.chars() {
        let boundary = previous.is_some_and(|previous| {
            ch == '_'
                || previous == '_'
                || (previous.is_lowercase() && ch.is_uppercase())
                || previous.is_ascii_digit() != ch.is_ascii_digit()
        });
        if boundary && !word.is_empty() {
            words.push(std::mem::take(&mut word));
        }
        if ch != '_' {
            word.push(ch);
        }
        previous = Some(ch);
    }
    if !word.is_empty() {
        words.push(word);
    }
    let said: Vec<String> = words
        .into_iter()
        .map(|word| {
            let number = word.bytes().all(|b| b.is_ascii_digit())
                && !(word.len() > 1 && word.starts_with('0'))
                && word.len() < 7;
            match word.parse::<u64>() {
                Ok(value) if number => numerals::cardinal(value).into_text(),
                _ if word.bytes().all(|b| b.is_ascii_digit()) => numerals::digits(&word),
                _ => word,
            }
        })
        .collect();
    format!("heşteg {}", said.join(" "))
}

pub(crate) fn hashtag(text: &str) -> Option<String> {
    let body = text.strip_prefix('#')?;
    if body.is_empty()
        || !body.chars().all(|c| c.is_alphanumeric() || c == '_')
        || body.chars().any(|c| c.is_numeric() && !c.is_ascii_digit())
    {
        return None;
    }
    let mut output = String::from("hashtag ");
    let mut chunk = String::new();
    for ch in body.chars() {
        if ch.is_ascii_digit() {
            if !chunk.is_empty() {
                output.push_str(&chunk);
                output.push(' ');
                chunk.clear();
            }
            output.push_str(&numerals::digits(&ch.to_string()));
            output.push(' ');
        } else {
            chunk.push(ch);
        }
    }
    output.push_str(&chunk);
    Some(output.trim_end().to_owned())
}
