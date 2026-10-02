# Normalization reference

This document describes the single built-in normalizer, not selectable reading
profiles. `NORMALIZER_ID` / `normalizer_id` is diagnostic package metadata.
`complete` means all recognized normalization work was resolved; it is not a
claim about native-language or speech quality.

## Exact numbers and morphology

Magnitudes are strictly below `1_000_000_000_000_000_000`. Comma decimals have
1-9 fractional digits, read individually: `12,05` -> `on iki virgül sıfır beş`.
Explicit signs, including negative zero, are retained. Dot grouping must use
valid three-digit groups; bare `1.234` remains ambiguous without intent.

Integers, explicit ordinals and approved case/derivational suffixes share exact
number and spoken-tail helpers. Written-form pronunciation, the target spoken
stem and a derived stem are distinct: `25 TL'den` becomes `yirmi beş Türk
lirasından`, while `1.'nin` becomes `birincinin`. Emitted text is never reparsed
to infer suffixes. Arbitrary suffix chains and invalid allomorphs are unresolved.

Percentages use `%` followed by a valid exact number and supported suffix:
`%37,5'lik` -> `yüzde otuz yedi virgül beşlik`;
`%3,25'ten` -> `yüzde üç virgül iki beşten`.

## Currencies, units and ranges

Only these currencies are supported, with 0-2 fractional digits and no rounding,
conversion, exchange-rate lookup or currency catalog inference:

| Currency | Labels | Major / minor words |
|---|---|---|
| TRY | `TL`, `TRY`, `₺` | `Türk lirası` / `kuruş` |
| USD | `USD`, `$` | `dolar` / `sent` |
| EUR | `EUR`, `€` | `avro` / `sent` |
| GBP | `GBP`, `£` | `sterlin` / `peni` |

Zero minor units are omitted. Unknown or malformed currency expressions remain
whole unresolved spans.

Approved unit labels are exactly `kg`, `g`, `km`, `m`, `cm`, `mm`, `L`, `mL`,
`mg`, `µg`, `μg`, `gr`, `ml`, `lt`, `dk`, `sn`, `sa`, `m²`, `cm²`, `km²`, `m³`.
Aliases are explicit: there is no arbitrary case-folding or NFKC expansion.
Rates are only `km/sa`, `km/h` and `m/s`, using `saatte` / `saniyede`.

Ranges keep exact endpoint order and use `-` or `–` plus an approved unit or
`kişi`, `adet`, `gün`, `yaş` context, or a whole Range hint. Bare `10-15`,
mathematical subtraction/equations and date/time/currency ranges are not guessed.

## Dates, clocks and abbreviations

Gregorian dates accept dotted day-month-year with a four-digit nonzero year, or
unsuffixed ISO dates. A `tarih` / `tarihi` cue, including its supported colon
form, or a Date hint establishes intent. Clocks are valid 24-hour values with
a `saat` cue or Time hint.

The bounded date-locative + whitespace + `saat` + whitespace + clock-locative
frame is also recognized. Date locative uses the spoken year; clock locative
uses the minute, or hour when `:00` elides minutes. Suffixed ISO dates,
arbitrary suffix chains and invalid calendar/clock values stay unresolved.
`Toplam 25.` is not silently treated as an ordinal.

The explicit abbreviation inventory includes `Dr.`, `Prof.`, `vb.`, `TBMM`,
`PTT`, `NATO`, `IBAN` and bare `KDV`. Initialisms retain spaced letter readings.
Suffix support is bounded per entry; arbitrary abbreviations and KDV suffixes
are not inferred.

## Phones, IBANs and Romans

Turkish national phones use `0 + 3 + 3 + 2 + 2` grouping. International `+90`
forms say `artı doksan` followed by the ten written national digits; no trunk
zero is invented. Significant leading zeros in groups remain digitwise.
Plain ten digits require Telephone intent or a clear phone cue. Digits hints
remain digitwise, not telephone prosody. Subscriber assignment is not verified.

IBANs require the full uppercase `TR` + two check digits + 22 numeric BBAN
characters, canonical space grouping and modulo-97 validity. The public
`TR330006100519786457841326` fixture is a checksum example, not a verified
account. Fragments and malformed expressions remain protected. Boundary rules
keep following prose, abbreviations and independently valid quantities separate.

Romans must be canonical uppercase values 1-3999. A whole Roman hint or the
bounded ordinal context `Dünya Savaşı` / `yüzyıl` establishes reading intent.
Bare `IV`, `I.` and list markers are not guessed. `II. Abdülhamit` can use an
explicit hint; there is no general proper-name detector.

## Electronic text and symbols

The bounded grammar supports ASCII unquoted email, HTTP/HTTPS (scheme case
insensitive), `www`, and a bare domain with a `web` / `site` cue or Electronic
hint. Final TLDs are `com`, `net`, `org`, `tr`, `gov`, `edu`, `app`.

Alphabetic chunks keep source case. Schemes, ports, paths, query strings,
fragments, escaped bytes and digit order are spoken without fetching, decoding
or dropping components. `@` in a URI path is not authority userinfo. Quoted or
Unicode email, IDN, authority userinfo, unknown schemes/TLDs and bad escapes
remain unresolved as whole expressions. Prose hashtags and ampersands have
role-aware readings.

## Hints, coordinates and errors

Hints are Cardinal, Digits, Date, Time, Ordinal, Roman, Range, Telephone and
Electronic. They use nonoverlapping, grapheme-safe **original UTF-8 byte ranges**
over whole expressions. Cardinal intent permits explicit zero-padding. Digits
accepts optional initial plus and the supported space/dot/slash/hyphen/
parenthesis separators. A hint cannot legalize an invalid date, checksum or
malformed grammar, or cut a detected compound.

Already-NFC source is borrowed with its grapheme boundaries. Other input uses
NFC recognition with a mapping back to original graphemes. Verbatim/unresolved
text remains exact; no global lowercasing or Unicode compatibility rewrite.
Segments partition the original input contiguously and their text concatenates
to `normalized_text`.

Default Preserve returns useful partial work, `complete = false` and structured
issues. Reject returns `NormalizeError::Unresolved` with the same diagnostics
and no normalized result. Invalid input/hints/configuration, cancellation,
deadlines, resource limits and internal failures remain real errors in both.

| Limit | Bound |
|---|---|
| Original input | 32 KiB UTF-8 |
| Hints | 256 |
| Candidate records | 4096 |
| Logical result allocation | 512 KiB |

The allocation budget includes owned segment and final text plus result/
diagnostic structures. Empty/whitespace-only input, Bidi_Control and unsupported
controls are invalid. Cancellation/deadlines are cooperative between bounded
steps. Outputs contain input text, but the core does not log it; issue
explanations contain no copied input values.
