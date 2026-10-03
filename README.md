# normalizer-tr

[![CI](https://github.com/erdemtuna/normalizer-tr/actions/workflows/ci.yml/badge.svg)](https://github.com/erdemtuna/normalizer-tr/actions/workflows/ci.yml)

**Turkish text normalization for text-to-speech, with a Rust core and a typed
Python binding.** Turn written numbers, measurements, money and other supported
expressions into spoken Turkish without rewriting ordinary prose.

```text
Input:  saat 09:30'da 5 kg malzeme ve %12,5'lik fark
Output: saat dokuz otuzda beş kilogram malzeme ve yüzde on iki virgül beşlik fark
```

The same Rust engine powers both APIs. It is synchronous, works offline and
requires no model, network service, Torch or async runtime. Exact decimal and
money arithmetic uses integers, not floating point.

**Status:** early-development 0.4.0, not a stable 1.0 API or a universal
pronunciation guarantee. The Rust crate and Python distribution share the name
`normalizer-tr`; both use `normalizer_tr` in code. The internal Rust/Python
companion is not a separate crates.io product.

## Choose your API

| Use case | Component | Dependencies |
|---|---|---|
| Rust applications | `normalizer-tr` / import `normalizer_tr` | Rust only; optional Serde |
| Python applications | `normalizer-tr` / import `normalizer_tr` | The compiled Rust binding; no speech model |

`bindings/python` is a separate Cargo workspace member and wheel, not part of
the core `.crate`. The binding depends on the Rust core, never the reverse.

## Rust quickstart

Use Rust 1.94 or newer / edition 2024. Development builds use Rust 1.99.0.
From your application's `Cargo.toml`:

```toml
[dependencies]
normalizer-tr = "0.4"
```

Commit your application's `Cargo.lock` to pin resolved dependencies. For an
unpublished checkout, use a reviewed Git revision or a local path such as
`normalizer-tr = { path = '..\normalizer-tr' }`.

```rust
use normalizer_tr::{Normalizer, NormalizeOptions};

let normalizer = Normalizer::new()?;
let result = normalizer.normalize(
    "saat 09:30'da 5 kg malzeme ve %12,5'lik fark",
    &NormalizeOptions::default(),
)?;

assert_eq!(
    result.normalized_text(),
    "saat dokuz otuzda beş kilogram malzeme ve yüzde on iki virgül beşlik fark",
);
assert!(result.complete());
# Ok::<(), normalizer_tr::NormalizeError>(())
```

Keep and reuse a `Normalizer`: its compiled resources are immutable and shared.
It is cloneable and `Send + Sync`; inputs and results are not cached. Enable the
optional `serde` feature to serialize results, segments and issues.

## Python quickstart

The Python bridge calls Rust directly rather than duplicating language rules.
The release targets ordinary CPython 3.11–3.14 on Windows/Linux x64 and macOS
x64/arm64. Linux wheels require glibc 2.28 or newer; macOS wheels target 12.0 or
newer. No PyPy, free-threaded Python or other architectures are claimed.

```text
python -m pip install normalizer-tr
```

Compatible wheels require no Rust compiler. To build from source instead:

Install [Rust](https://www.rust-lang.org/tools/install) and the MSVC C++ build
tools, then run in PowerShell. On Windows, use a short checkout path (for
example `C:\src\normalizer-tr`) to avoid path-length limits during installation.

```powershell
git clone https://github.com/erdemtuna/normalizer-tr.git
Set-Location normalizer-tr
py -3.13 -m venv .venv
.\.venv\Scripts\python.exe -m pip install .\bindings\python
.\.venv\Scripts\python.exe -X utf8 .\bindings\python\examples\showcase.py
```

The source installation builds a native wheel and requires Rust/C++ build tools.
For a local wheel, use `python -m pip install --no-deps <wheel-path>`.

```python
from normalizer_tr import Normalizer

normalizer = Normalizer()
result = normalizer.normalize("25 TL; 5 kg")
print(result.normalized_text)  # yirmi beş Türk lirası; beş kilogram
assert result.complete
assert result.issues == ()
```

`-X utf8` avoids Turkish stdout encoding errors on Windows shells configured
with a legacy encoding. It does not alter normalization. See the
[Python API and build guide](bindings/python/README.md) for hints, cancellation,
exceptions and installed-wheel testing.

## Ambiguity is explicit

By default, supported spans normalize and unresolved spans remain **exactly as
written**. Check `complete` before passing a result to a speech model:

```rust
use normalizer_tr::{Normalizer, NormalizeOptions};

let result = Normalizer::new()?.normalize("25 TL; 1.234", &NormalizeOptions::default())?;
assert_eq!(result.normalized_text(), "yirmi beş Türk lirası; 1.234");
assert!(!result.complete());
assert_eq!(result.issues().len(), 1);
# Ok::<(), normalizer_tr::NormalizeError>(())
```

Bare `1.234` has insufficient reading intent. Use a whole-span hint if your
application knows how to interpret it, or select strict rejection:

```python
from normalizer_tr import Hint, Normalizer, NormalizationError

normalizer = Normalizer()
assert normalizer.normalize(
    "00042", hints=(Hint(0, 5, "digits"),)
).normalized_text == "sıfır sıfır sıfır dört iki"

try:
    normalizer.normalize("1.234", ambiguity_policy="reject")
except NormalizationError as error:
    assert error.code == "unresolved"
    assert error.issues  # No partial result is returned in strict mode.
```

All ranges are half-open **original UTF-8 byte offsets**, not character
positions, Python indices or UTF-16 offsets. Hints must cover a whole expression,
be grapheme-safe and not overlap. For a prefix in Python, compute its byte length
with `len(prefix.encode("utf-8"))`.

Invalid input/hints, cancellation, deadlines, resource limits and internal
failures are errors in every policy, not successful preservation. The one
exception is a bidirectional control such as `U+200F`, which `Forced` reads past
as if it were not there.

### Forced readings

A speech model cannot read digits or symbols. Opt in to `Forced` to have
everything spoken: every unresolved span is read with the most common reading
for its shape, or literally when no reading fits. The issues are still
reported, so `complete` stays `false` wherever the engine had to guess. The
default stays `Preserve`, whose output is unchanged.

```rust
use normalizer_tr::{AmbiguityPolicy, Normalizer, NormalizeOptions};

let options = NormalizeOptions {
    ambiguity_policy: AmbiguityPolicy::Forced,
    ..Default::default()
};
let result = Normalizer::new()?.normalize("25 TL; 1.234; 3 + 4 = 7", &options)?;
assert_eq!(
    result.normalized_text(),
    "yirmi beş lira; bin iki yüz otuz dört; üç artı dört eşittir yedi",
);
assert!(!result.complete());
assert_eq!(result.issues().len(), 2);
# Ok::<(), normalizer_tr::NormalizeError>(())
```

In Python, pass `ambiguity_policy="forced"`.

| Policy | Unresolved spans |
|---|---|
| `Preserve` (default) | kept as written, reported as issues |
| `Reject` | error, no output |
| `Forced` | read with a forced reading, reported as issues |

`Forced` does three things:

1. **It reads every unresolved span.** The guesses below are tried in one fixed
   order and the first that reads the span wins; where a hint kind reads the
   span as written, the reading is exactly that explicit hint's.
2. **It speaks in everyday style**, also in spans the engine resolves. A decimal
   fraction is read as a number after its leading zeros (`4,25` is
   `dört virgül yirmi beş`, not `iki beş`), `TL` is `lira`, a dash between
   numbers is `tire` (`10-15` is `on tire on beş`, right for a score too), `/`
   is `slaş` unless it writes a fraction, a minute below ten keeps its zero (`12:05` is
   `on iki sıfır beş`), `#` is `heşteg` (`#1` is `bir numara`), and a plain
   number of seven digits or more is said as the identifier it is: a ten-digit
   one that starts with 5 as a mobile number, any other digit by digit, as is a
   card number in four groups of four.
3. **It names stray symbols and letters** that `Preserve` keeps as written
   (`+` is `artı`, `B` is `Be`, `vs` is `ve se`), without adding an issue. A
   face, an arrow, a decoration, a Markdown mark and a lone `*`, `<` or `>`
   between words say nothing (`:D`, `<3`, `->`, `***`, `# Başlık`,
   `Kategori > Telefon`).

The forced order:

1. `Digits`: a zero-padded digit run (`0532`, `007`), or a card number in four
   groups of four whose network and check digits hold (not the years
   `2020 2021 2022 2023`)
2. `Time`: a clock; a dotted one needs `saat` before it, a suffix, a zero before
   its hour or whole hours (`15.30'da`, `09.30`, `22.00`, not `macOS 10.15`), and
   `24:00` is the end of the day; `H:MM:SS` is a clock after `saat`, with a
   two-digit hour or zero seconds, and otherwise a duration (`rekor 2:00:35` is
   `rekor iki saat otuz beş saniye`); a race time with a fraction of a second
   (`1:23.456`)
3. `Date`: dotted, ISO, or day/month/year with slashes, also year first
   (`2026.05.19`) or with a two-digit year (`01/04/26`); a day the calendar
   lacks is still read by the shape of a date (`29.02.1900`)
4. `Range`: two clocks or two dates around a dash, or two numbers, said with
   `tire`
5. a ratio: two numbers around a colon that are no clock (`1:3`)
6. a fraction: a one-digit numerator below a denominator of at most 1000, said
   with `bölü` (`1/2` is `bir bölü iki`, `2/3'ü` is `iki bölü üçü`), and a
   rating out of 5, 10 or 100 (`4,5/5`); `7/24`, `24/7` and `No: 3/5` keep
   `slaş`
7. `Telephone`, also in other groups or with dashes (`+90 532 123 4567`), but
   not an internet address (`255.255.255.0`)
8. an IBAN by its shape, even when its checksum fails
9. money with its currency sign or code, in English digits too (`$1,299.99`)
10. `Cardinal`, unless the span is zero-padded
11. `Roman`: a numeral of `I`, `V`, `X` with two or more letters, a period or a
    suffix (`IV`, `X.`), or any numeral of four or more letters, but not `V.`
    before a name, an initial (`V. Öztürk`); at the end of the text an ordinal
    keeps its period (`Elizabeth II.` is `Elizabeth ikinci.`)
12. `Ordinal`: a number and a period before a suffix, a lowercase word or a
    list comma (`21. yüzyılda`, `1., 2. ve 3.`), or before a capital on the
    same line after a number below 1000 (`1. Dünya Savaşı`), or before a
    numbered street of any number (`1234. Sok.`); at the end of the
    text, before a capital on the next line or after a year, the period may
    close a sentence
13. `Electronic`: a whole address under a suffix that was split off
14. `Literal`, which reads every span

If no guess reads a span that ends in an apostrophe suffix, the suffix is split
off, the rest is read through the same order, and the suffix is harmonized onto
that reading; then brackets and punctuation around the span are set aside. A
compass letter after a coordinate, or compass points joined by a dash
(`K-KD`), are read first. A bare domain is never guessed as an address, and a
lone capital or an abbreviation such as `X`, `M.`, `XL` or `CD` is not turned
into a number; the literal reading spells it instead. A mark right after a span
that its reading says (the inch mark of `6.8"`, the period of `90+4.` before a
lowercase word) is dropped, as an empty segment, and so is each bidirectional
control, which `Forced` reads the text without.

| Written | Forced |
|---|---|
| `1.234` | `bin iki yüz otuz dört` |
| `Toplantı 09:30'dan 11:00'e kadar` | `Toplantı dokuz otuzdan on bire kadar` |
| `mesai 09:00-17:30 arası` | `mesai dokuz tire on yedi otuz arası` |
| `01.02.2026 tarihinde` / `02/10/2026` | `bir Şubat iki bin yirmi altı tarihinde` / `iki Ekim iki bin yirmi altı` |
| `21. yüzyılda` / `1. Dünya Savaşı` / `Toplam 25.` | `yirmi birinci yüzyılda` / `birinci Dünya Savaşı` / `Toplam yirmi beş.` |
| `IV` / `IV. Murat` | `dört` / `dördüncü Murat` |
| `0532` / `Maç 3-1 bitti` / `1:3` | `sıfır beş üç iki` / `Maç üç tire bir bitti` / `bire üç` |
| `24/7` / `12:05` / `5321234567` | `yirmi dört slaş yedi` / `on iki sıfır beş` / `beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi` |
| `5 TL'lik` / `3,99 TL'ye` / `4,25` | `beş liralık` / `üç lira doksan dokuz kuruşa` / `dört virgül yirmi beş` |
| `-5°C'den` | `eksi beş dereceden` |
| `H2O` / `4K` / `C++` | `He iki O` / `dört Ke` / `Ce artı artı` |
| `TK1956` / `XL` / `5 KG` | `Te Ke bin dokuz yüz elli altı` / `İks Le` / `beş kilogram` |
| `ADB` / `IĞDIR'A` / `COVID-19` | `A De Be` / `ığdır'a` / `kovid on dokuz` |
| `120 m2'lik` / `5 m³'lük` | `yüz yirmi metrekarelik` / `beş metreküplük` |
| `KDV'li` / `5 TL'li` / `TBMM'DE` | `katma değer vergili` / `beş liralı` / `te be me mede` |
| `₺1.250,75 + KDV` | `bin iki yüz elli lira yetmiş beş kuruş artı katma değer vergisi` |
| `3 + 4 = 7` | `üç artı dört eşittir yedi` |
| `29.02.1900` / `31.04.2026'da` | `yirmi dokuz Şubat bin dokuz yüz` / `otuz bir Nisan iki bin yirmi altıda` |
| `Fiyat: $1,299.99` | `Fiyat: bin iki yüz doksan dokuz dolar doksan dokuz sent` |
| `6.8" ekran` / `maç 90+4. dakikada bitti` | `altı virgül sekiz inç ekran` / `maç doksan artı dördüncü dakikada bitti` |
| `rüzgâr K-KD` / `200MP` / `i9-14900K` | `rüzgâr kuzey kuzeydoğu` / `iki yüz megapiksel` / `i dokuz tire on dört bin dokuz yüz Ke` |
| `1/2 bardak` / `3x faydalı` / `2x Intel` | `bir bölü iki bardak` / `üç kat faydalı` / `iki adet Intel` |
| `70 °F` / `KDV'ler` | `yetmiş derece Fahrenhayt` / `katma değer vergileri` |
| `1.si` / `21.yy'ın` / `Prof.Dr.` | `birincisi` / `yirmi birinci yüzyılın` / `profesör doktor` |
| `10²³` / `⅓` / `¥500` | `on üssü yirmi üç` / `bir bölü üç` / `beş yüz yen` |
| `β-karoten` / `ID'si` / `*123#` | `beta karoten` / `İ De'si` / `yıldız yüz yirmi üç kare` |

`Literal` reads a span as written and is also a hint kind any caller can use:

- A digit run is a cardinal, or digit by digit when it is zero-padded, beyond
  the magnitude limit, or an identifier of seven digits or more; ten digits that
  start with 5 are a mobile number, said in its groups. Groups of thousands
  are one amount (`1.500`, `1,000,000`); in a code such as
  `1Z999AA10123456784` digits are said one by one, except round numbers.
- A separator between two parts of the span is named: `.` nokta, `,` virgül,
  `:` iki nokta, `/` slaş, `-` tire. A hyphen directly before the first digit
  is `eksi`, one between a word and a number or between two words is silent
  (`COVID-19` is `kovid on dokuz`, `E-POSTA` is `E posta`) unless the span has
  more hyphens or joins two single letters (`A-Z` is `A tire Ze`), and so is a
  period between two single letters (`Y.Z.` is `Ye Ze.`). One letter after a
  number's period is no word: `3.x` is `üç nokta iks`. One point is a decimal comma
  before a unit or a currency (`9.58 sn` is `dokuz virgül elli sekiz saniye`).
- A symbol is named: `+` artı, `−` eksi, `=` eşittir, `×` iks, `÷` bölü,
  `*` çarpı, `%` yüzde, `&` ve, `@` et, `#` heşteg, `<` küçüktür, `>` büyüktür,
  `°` and `°C` derece, `°F` derece Fahrenhayt, `_` alt çizgi, `½` yarım (buçuk
  after a digit), `¼` çeyrek, `¾` üç çeyrek, `₺` lira, `$` dolar, `€` avro,
  `£` sterlin, and math symbols such as `≈` yaklaşık, `√` karekök and `²`
  kare. `5%` is `yüzde beş`, and `K`, `M`, `W`, `V`, `MP` glued to a number are
  bin, milyon, vat, volt and megapiksel (`45K` is `kırk beş bin`; `4K` is
  `dört Ke`). An `x` glued to a count is kere before a number, adet before a
  name and kat before another word; after an operator it stays `iks`
  (`y = 2x`). Currency signs such as `¥`, `₽` and `₹`, the fractions of one
  character (`⅓`), `§`, `№`, superscript powers (`10²³`), a Greek letter alone
  (`β-karoten`) and a phone's keypad (`*123#`) are named; list marks, arrows,
  check marks and trademark signs are dropped. `*`, `<` and `>` are said only
  between two numbers or single letters, or a comparison before a number, and
  `_` only inside a word: around words they mark emphasis, a footnote or a menu
  path and say nothing (`**Önemli**` is `Önemli`, `Fiyat*` is `Fiyat`), and a
  `*` after a count is its stars (`5* otel` is `beş yıldızlı otel`). A
  superscript right after a word is its footnote (`Yazar¹` is `Yazar`); faces
  (`:D`, `<3`) say nothing.
- A single letter is said by its Turkish (TDK) name (`B` Be, `H` He, `x` iks;
  a vowel is its own name), and so is each letter of capitals without a vowel
  (`TK` Te Ke, `XL` İks Le, `Hz` He ze), of lowercase consonants (`vs` ve se),
  of two capitals that are no word (`ID` İ De, but `VE` is ve and `MI` mı) and
  of three capitals that end in two consonants no Turkish word ends in (`ADB`
  A De Be). A sound written in consonants stays as written (`Hmm`, `Pff`).
  Other capitals are a word, said in lowercase (`İSTANBUL'U` is `istanbul'u`,
  `DZEKO` is `dzeko`, `PFIZER` is `pfizer`). The lexicon reads
  what it knows: an abbreviation or a currency code anywhere, a unit right
  after a number, in any capitals (`5kg` and `5 KG` are `beş kilogram`), and a
  unit with its power anywhere (`m2` and `m²` are `metrekare`). Another word is
  unchanged.
- Whitespace, and sentence punctuation at the edges of the span, are unchanged.
- An apostrophe suffix, in any case, is harmonized to the last spoken word when
  it is a known inflection, and any other suffix is fitted to that word's vowels
  and last sound (`90'lar` is `doksanlar`, `5 TL'li` is `beş liralı`). A
  compound drops its possessive before a derivation (`KDV'li` is
  `katma değer vergili`) and says it after a plural (`KDV'ler` is
  `katma değer vergileri`).
- Any other character is removed.

A forced segment keeps its reading's own kind and gets a `forced.` rule id such
as `forced.time` or `forced.literal`; a stray symbol or letter is a `Literal`
segment with `forced.spoken`. See the
[normalization reference](docs/normalization.md#forced-readings) for details.

## Supported expressions

| Written | Spoken |
|---|---|
| `12,05` | `on iki virgül sıfır beş` |
| `%3,25'ten` | `yüzde üç virgül iki beşten` |
| `25 TL'den` | `yirmi beş Türk lirasından` |
| `€14,05` / `$40` / `25 GBP` | `on dört avro beş sent` / `kırk dolar` / `yirmi beş sterlin` |
| `1.'nin` / `4.'ye` | `birincinin` / `dördüncüye` |
| `5 kg'dan` / `2 sa'ten` / `5 m³'e` | `beş kilogramdan` / `iki saatten` / `beş metrekübe` |
| `90 km/sa` / `5 m/s` | `saatte doksan kilometre` / `saniyede beş metre` |
| `10-15 kişi` | `on ila on beş kişi` |
| `tarih 01.02.2026` | `tarih bir Şubat iki bin yirmi altı` |
| `Prof.` / `TBMM` / `KDV` | `profesör` / `te be me me` / `katma değer vergisi` |
| `ABD'de` / `THY` / `BBC` | `a be dede` / `te ha ye` / `bi bi si` |
| `Cumhuriyet Cad. No: 12 D: 5` | `Cumhuriyet caddesi numara on iki daire beş` |
| `Kaya Ltd. Şti.` / `Hz.` / `s. 12` | `Kaya limited şirketi` / `hazreti` / `sayfa on iki` |
| `0850 222 33 44` | `sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört` |
| `II. Dünya Savaşı` | `ikinci Dünya Savaşı` |
| `info@ornek.com` | `info et ornek nokta kom` |

Coverage is deliberately bounded, not a universal Turkish pronunciation
engine. [Normalization reference](docs/normalization.md) documents the exact
grammars, aliases, hints, boundaries and unsupported cases. Unknown or malformed
identifiers/composites are protected as whole spans, not rewritten in fragments.
Ordinary word case is retained; the library does not globally lowercase or
adapt output to a particular voice/model vocabulary.

## Develop and contribute

```powershell
cargo test --locked
cargo run --locked --example normalize
cargo run --locked --example general
cargo doc --locked --no-deps --open
```

Default Cargo operations build the standalone core. `--workspace` also includes
the Python binding. CI exercises the Rust configurations and the actual
installed Python wheel without speech models.

See [CONTRIBUTING.md](CONTRIBUTING.md) for environment setup, architecture,
regression tests and the full `scripts\verify.ps1` command. That serial command
produces clean packages, external-consumer results, latency reports and one
checksum/provenance manifest in a new output directory.

The measured warm release Rust p95 was below 1 ms separately for representative
short and medium cohorts on the inspected Windows host. This is **not** an
all-input, cold-start, Python, concurrent-call or end-to-end audio guarantee.
See [PERFORMANCE.md](PERFORMANCE.md) for the measurement protocol and limits.

## Acknowledgements

Thanks to [@canberk7](https://github.com/canberk7) for practical TTS feedback
and suggestions on making the Python package easier to distribute.

## License and boundaries

Owned code and independently authored tables are
[Apache-2.0](https://github.com/erdemtuna/normalizer-tr/blob/main/LICENSE).
[Third-party notices](THIRD_PARTY_NOTICES.md) preserve dependency and Unicode
rights. This repository contains the normalizer and Python binding only: no
speech-engine integration, external model source, weights, credentials or audio.
