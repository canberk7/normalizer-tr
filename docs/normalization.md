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

The lexicon reads 477 abbreviations, one table in
`src/domain/lexicon/abbreviations.rs`, the same way under every policy:

- titles (`Dr.` doktor, `Doç.` doçent, `Av.` avukat, `Hz.` hazreti, `Sn.`
  sayın), also written together (`Prof.Dr.` profesör doktor, `Arş.Gör.`
  araştırma görevlisi), and military ranks (`Alb.` albay, `Yzb.` yüzbaşı);
- addresses, said in the possessive form a name takes (`Cumhuriyet Cad.` is
  `Cumhuriyet caddesi`; `Sok.` sokağı, `Mah.` mahallesi, `Apt.` apartmanı, and
  bare with no name before it, `Apt. veya Blok` apartman veya blok), with `D:`
  daire and `No:` numara, which is numarası after a noun it makes a compound
  with (`Sipariş No:` is `Sipariş numarası`, but `Cad. No:` and
  `İstiklal Caddesi No:` say numara);
- companies (`Ltd. Şti.` limited şirketi, `A.Ş.` anonim şirketi, `San.`
  sanayi, `Tic.` ticaret) and institutions (`Ankara Üniv.` Ankara
  üniversitesi, `Tıp Fak.` Tıp fakültesi, `Devlet Hst.` Devlet hastanesi);
- references and law (`vb.` ve benzeri, `bkz.` bakınız, `s.` sayfa, `md.`
  madde, `min.` minimum, `T.C.` Türkiye Cumhuriyeti), days and months (`Pzt.`
  pazartesi, `Şub.` Şubat) and languages (`İng.` İngilizce);
- initialisms as people say them: letter by letter with the TDK letter names
  (`ABD` a be de, `THY` te he ye, `KKTC` ke ke te ce, `HDMI` he de me i), with
  the English letter names they are known by (`BBC` bi bi si, `CD` si di), or
  as a word (`NATO`, `ODTÜ` odtü, `TÜBİTAK` tübitak, `PIN` pin, `SIM` sim,
  `SARS-CoV-2` sars kov iki), and the time of day (`9 AM` ey em, `ÖS 3`
  öğleden sonra);
- measures (`GB` gigabayt, `kW` kilovat, `pH` pe he, `IU` ünite, `dL`
  desilitre, `dpi` de pe i, `fps` fe pe se);
- chat abbreviations, as the words they stand for (`slm` selam, `nbr` naber,
  `tmm` tamam, `cvp` cevap, `bn` ben, `s.a` selamünaleyküm), only in that
  lowercase spelling.

No ordinary word is read as an abbreviation. One whose letters are also a word,
or that means two things, is read only where nothing else fits, and stays as
written elsewhere:

| Kind | Read only | Read | Not read |
|---|---|---|---|
| A title whose letters are a word (`Av.`, `Gen.`, `Gör.`) | before a name | `Av. Ayşe` avukat Ayşe | `Bu Av.` |
| A place whose letters are a word (`Bul.`, `Sok.`, `Der.`) | after a name, an abbreviation or a number | `Atatürk Bul.` Atatürk bulvarı, `1234. Sok.` | `Bul. Hemen.`, `Topla. Der.` |
| A single letter, `No`, `D:`, `Tel.` | before a number | `s. 12` sayfa on iki, `Kapı No 5` | `Doğru cevap c.`, `d. Dördüncü madde` |
| A month, `AM`, `PM`, `ÖÖ`, `ÖS` | next to a number | `5 Kas. 2024`, `9 AM` | `Kolunu kas. Kas.`, `I AM OK` |
| A day whose letters are a word, `ör.` | before a lowercase word or a number | `Sal. günü` salı günü | `Bırak onu. Sal.` |
| A unit | anywhere but before a number | `MB cinsinden` megabayt cinsinden | `Çince (GB 18030)` |
| `min.` | anywhere but after a number | `min. 18 yaş` minimum | `30 min.` |

`vs.` is not read, since it is vesaire after a list and versus between two
names; `Nolu`, `PK` and the languages `Ar.`, `Far.` and `Rus.` are not read
either, as `nolu` is also chat for "ne oldu" and `Rus` a people.

Case matters: `md.` is madde and `Md.` müdürlüğü. A dotted abbreviation also
reads in capitals (`DR.` doktor), unless it could be something else: then only
its own spelling reads, so `SOK.`, `BUL.` and `MAH` (no `mAh`) stay as
written. A suffix after an abbreviation must be a case ending written for how
it is said (`ABD'de` is `a be dede`, `THY'ye` is `te he yeye`; `TCK'nın` may
follow the everyday ka); any other suffix keeps the span unresolved. Unknown
capitals are an `UnknownAbbreviation` issue.

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

Hints are Cardinal, Digits, Date, Time, Ordinal, Roman, Range, Telephone,
Electronic and Literal. They use nonoverlapping, grapheme-safe **original UTF-8
byte ranges** over whole expressions. Cardinal intent permits explicit
zero-padding. Digits accepts optional initial plus and the supported
space/dot/slash/hyphen/parenthesis separators. Literal reads the span as
written (see [the literal reading](#the-literal-reading)) and accepts any
content. A hint cannot legalize an invalid date, checksum or malformed grammar,
or cut a detected compound.

Already-NFC source is borrowed with its grapheme boundaries. Other input uses
NFC recognition with a mapping back to original graphemes. Verbatim/unresolved
text remains exact; no global lowercasing or Unicode compatibility rewrite.
Segments partition the original input contiguously and their text concatenates
to `normalized_text`.

Default Preserve returns useful partial work, `complete = false` and structured
issues. Reject returns `NormalizeError::Unresolved` with the same diagnostics
and no normalized result. Forced returns the same diagnostics and reads the
unresolved spans as described under [Forced readings](#forced-readings).
Invalid input/hints/configuration, cancellation, deadlines, resource limits and
internal failures remain real errors in every policy.

| Limit | Bound |
|---|---|
| Original input | 32 KiB UTF-8 |
| Hints | 256 |
| Candidate records | 4096 |
| Logical result allocation | 512 KiB |

The allocation budget includes owned segment and final text plus result/
diagnostic structures. Empty/whitespace-only input, Bidi_Control and unsupported
controls are invalid; only Forced reads text with bidirectional controls (see
[Forced readings](#forced-readings)). Cancellation/deadlines are cooperative between bounded
steps. Outputs contain input text, but the core does not log it; issue
explanations contain no copied input values.

## Forced readings

`AmbiguityPolicy::Forced` (`ambiguity_policy="forced"` in Python) is opt-in;
the default stays Preserve. It is meant for a speech model, which cannot read
digits or symbols: everything is spoken. The sections above describe what the
engine resolves on its own; where they say a shape is not guessed, or give a
reading, that holds under Preserve and Reject.

| Policy | Unresolved spans |
|---|---|
| Preserve (default) | kept as written, reported as issues |
| Reject | error, no output |
| Forced | read with a forced reading, reported as issues |

Forced does three things and nothing else:

1. **Every unresolved span is read** with the first reading of the
   [forced order](#forced-order), or literally when none fits.
2. **Readings are said in everyday style**, in resolved spans too: a decimal
   fraction is a number after its leading zeros, the Turkish lira is `lira`, a
   dash between numbers is `tire`, `/` is `slaş` (a fraction is said as one), a
   minute below ten keeps its zero, and a long plain number is said as the
   identifier it is.
3. **Stray symbols and letters are named**: a word that Preserve keeps verbatim
   is read with [the literal reading](#the-literal-reading) when it holds a
   named symbol, is a single consonant, letters without a vowel (`TK` Te Ke,
   `vs` ve se) or capitals (`SAAT` saat). A face, an arrow, a decoration and the
   mark that starts a Markdown line say nothing (`:D`, `<3`, `->`, `***`,
   `# Başlık`), and so do a lone `*`, `<` or `>` between words
   (`Kategori > Telefon`).

Diagnostics do not change. Every unresolved span still adds its issue with the
same category and explanation, so the issues equal those of Preserve and
`complete` is `false` whenever the engine had to guess; the spoken style and a
stray symbol or letter add no issue. Segments keep the Preserve partition, and
only verbatim text is cut finer: around a stray word, and around a mark right
after an unresolved span that its reading already says (the inch mark of
`6.8"`, the seconds mark of a coordinate, the period that makes `90+4.` an
ordinal), which becomes an empty segment. A dropped mark, a stray `|` or `→`
included, takes a space next to it with it, so no double space is left where it
stood (`lira | Garanti` is `lira Garanti`). Explicit hints, invalid input and
hints, cancellation and deadlines behave exactly as under Preserve, with one
exception: Forced reads text with bidirectional controls (`U+200E`, `U+200F`,
`U+061C`, `U+202A`-`U+202E`, `U+2066`-`U+2069`) as the text without them,
since only the order text is read in is spoken. Each run of controls between
segments or inside verbatim text is an empty segment, a control inside a read
span is part of it, and hints keep original coordinates; a control whose removal
would join two graphemes, as two halves of a flag, is still invalid input.
Forced readings are longer than their source and each stray word is a
candidate record, so the result and candidate limits can be reached sooner.

| Segment | Kind | Rule id |
|---|---|---|
| Forced reading of an unresolved span | the reading's own kind | `forced.<guess>` |
| Stray symbol or letter | Literal | `forced.spoken` |
| Mark its unresolved span's reading says, or bidirectional controls | Literal, empty text | `forced.spoken` |
| Resolved span | unchanged | unchanged |

### Spoken style

| Written | Preserve | Forced |
|---|---|---|
| `4,25` | `dört virgül iki beş` | `dört virgül yirmi beş` |
| `0,18` | `sıfır virgül bir sekiz` | `sıfır virgül on sekiz` |
| `12,05` | `on iki virgül sıfır beş` | `on iki virgül sıfır beş` |
| `2,50 kg` | `iki virgül beş sıfır kilogram` | `iki virgül elli kilogram` |
| `25 TL'den` | `yirmi beş Türk lirasından` | `yirmi beş liradan` |
| `3,99 TL` | `üç Türk lirası doksan dokuz kuruş` | `üç lira doksan dokuz kuruş` |
| `10-15 kişi` | `on ila on beş kişi` | `on tire on beş kişi` |
| `saat 12:05` | `saat on iki beş` | `saat on iki sıfır beş` |
| `www.ornek.com/kampanya` | `çift ve çift ve çift ve nokta ornek nokta kom eğik çizgi kampanya` | `çift ve çift ve çift ve nokta ornek nokta kom slaş kampanya` |
| `https://ornek.com.tr` | `ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta com nokta te re` | `he te te pe se iki nokta slaş slaş ornek nokta kom nokta te re` |
| `#YapayZeka` | `hashtag YapayZeka` | `heşteg Yapay Zeka` |
| `#1 çok satan` | `hashtag bir çok satan` | `bir numara çok satan` |
| `4111 1111 1111 1111` | `dört bin yüz on bir bin yüz on bir bin yüz on bir bin yüz on bir` | `dört bir bir bir bir bir bir bir bir bir bir bir bir bir bir bir` |
| `123456789` | `yüz yirmi üç milyon dört yüz elli altı bin yedi yüz seksen dokuz` | `bir iki üç dört beş altı yedi sekiz dokuz` |

Leading zeros of a fraction are read one by one and the rest as a number. More
than three digits after the zeros are still read one by one in both styles.
Money keeps its exact major and minor units. A dash between numbers is `tire`,
which is right for a range and a score alike. A plain number of seven digits
or more is an identifier, not an amount: ten digits that start with 5 are a
mobile number said in its groups, and any other run is said digit by digit; so
is each group of a card number written in four groups of four. An
address names `http` and `https` with the TDK letter names and says `com` as
kom also before another label. A hashtag is heşteg, its words split where a
capital or a digit begins one, and its numbers said as numbers; a single digit
is a rank (`#1` is `bir numara`).

### Forced order

For each unresolved span the guesses are tried in this order and the first that
reads the span wins. Where a hint kind reads the span as written, the reading
is exactly what an explicit hint of that kind produces on that span. Before
them, a compass letter right after a coordinate's seconds is its direction
(`41°00'49"K` ends in kuzey) and compass points joined by a dash are a wind's
(`K-KD` is `kuzey kuzeydoğu`).

| Order | Rule id | Reads |
|---|---|---|
| 1 | `forced.digits` | a zero-padded digit run (`0532`, `007`); a card number in four groups of four whose network digits and check digit hold (`4111 1111 1111 1111`, not the years `2020 2021 2022 2023`) |
| 2 | `forced.time` | a clock; a dotted one only after `saat`, with a suffix, with a zero before its hour or with whole hours (`saat 20.45`, `9.30'da`, `09.30`, `22.00`), so that a version stays one (`macOS 10.15`); `24:00` as the end of the day; `H:MM:SS` as a clock after `saat`, with a two-digit hour or with zero seconds (`saat 3:45:30`), and otherwise as a duration (`rekor 2:00:35`); a race time with a fraction of a second (`1:23.456`) |
| 3 | `forced.date` | a dotted or ISO date; day/month/year with slashes, with the year first (`2026.05.19`) or in two digits (`01/04/26`); a day the calendar lacks, by its shape alone (`29.02.1900`) |
| 4 | `forced.range` | two clocks or two dates around one dash (`09:00-17:30`); two numbers (`10-15`, `3-1`); said with `tire` |
| 5 | `forced.ratio` | two plain numbers around a colon that are no clock (`1:3`) |
| 6 | `forced.fraction` | a one-digit numerator below a denominator of at most 1000, said with bölü (`1/2` bir bölü iki, `1/1000` bir bölü bin), and a rating out of 5, 10 or 100 (`4,5/5`, `10/10`); not `7/24`, nor a building and flat after an address word (`No: 3/5`) |
| 7 | `forced.telephone` | a Turkish telephone number, also in other groups or with dashes (`+90 532 123 4567`, `0532-123-4567`), but not an internet address (`255.255.255.0`) |
| 8 | `forced.iban` | a Turkish IBAN by its shape, whatever its checksum |
| 9 | `forced.money` | an amount with its currency sign or code, in English digits too (`$1,299.99`, `₺42.000`) |
| 10 | `forced.cardinal` | an integer, unless the span is zero-padded |
| 11 | `forced.roman` | a Roman numeral of the shapes below |
| 12 | `forced.ordinal` | a number and a period before a suffix, a lowercase word or the comma of a list (`1., 2. ve 3.`), or before a capital on the same line after a number below 1000, or before a numbered street of any number (`1234. Sok.`) |
| 13 | `forced.electronic` | a whole address, only under a suffix that was split off |
| 14 | `forced.literal` | every span |

Roman is guessed for a numeral written with `I`, `V` and `X` that has two or
more letters, a period or a suffix (`IV`, `XIV`, `X.`), and for any numeral of
four or more letters (`MMXXIV`). A lone capital (`X`, `C`), an initial (`M.`)
and short abbreviations and sizes (`XL`, `CD`, `CV`) are not turned into
numbers: the lexicon reads `CD` and `CV`, and the literal reading spells the
rest. Nor is `V.` before a name, an initial far more often than a ruler's
number (`V. Öztürk` is `Ve. Öztürk`), while `I.` and `X.` there are numbers
(`I. Dünya Savaşı`). A Roman ordinal at the end of the text or of its line keeps its period,
which closes the sentence too: `Elizabeth II.` is `Elizabeth ikinci.`. A bare
domain is never guessed as an address.

A period makes an ordinal before a suffix or a lowercase word, after any
whitespace: `21. yüzyılda` is `yirmi birinci yüzyılda`. Before a capital on the
same line it makes one after a number below 1000: `1. Dünya Savaşı` is
`birinci Dünya Savaşı`. At the end of the text, before a capital on the next
line, and after a year before a capital (`1923. Sonra`), the period may close a
sentence and is not guessed: `Toplam 25.` is `Toplam yirmi beş.`.

A date is read by its shape when the calendar lacks the day: day 1 to 31, month
1 to 12 and a four-digit year, dotted, ISO or with slashes. `29.02.1900` is
`yirmi dokuz Şubat bin dokuz yüz` and `31.04.2026'da` is
`otuz bir Nisan iki bin yirmi altıda`. The issue stays, and an explicit Date
hint on such a span is still rejected. `32.01.2026` is no date and is read
literally.

If no guess reads the span and it ends in one apostrophe suffix, the suffix is
split off, the rest is read through the same order, and the suffix is attached
to that reading: harmonized again to its last spoken word when it is a known
inflection, otherwise fitted to that word as in
[the literal reading](#the-literal-reading). An explicit hint on the same span
is still rejected; only Forced splits the suffix. Next, brackets, quotes and
punctuation written around the span are set aside and kept around the reading:
`09:00-18:00),` is `dokuz tire on sekiz),`.

A duration says its hours, minutes and seconds without the zero ones:
`rekor 2:00:35` is `rekor iki saat otuz beş saniye`. A literal reading looks at
what follows the span too: a period before a lowercase word makes its last
number an ordinal (`90+4. dakikada` is `doksan artı dördüncü dakikada`), `"`
right after a decimal is inches (`6.8" ekran` is `altı virgül sekiz inç
ekran`), and one point is a decimal comma before a unit or a currency
(`9.58 sn` is `dokuz virgül elli sekiz saniye`).

| Written | Forced | Rule id |
|---|---|---|
| `1.234` | `bin iki yüz otuz dört` | `forced.cardinal` |
| `Toplantı 09:30'da` | `Toplantı dokuz otuzda` | `forced.time` |
| `kapanış 24:00'te` | `kapanış yirmi dörtte` | `forced.time` |
| `01.02.2026 tarihinde` | `bir Şubat iki bin yirmi altı tarihinde` | `forced.date` |
| `02/10/2026` | `iki Ekim iki bin yirmi altı` | `forced.date` |
| `10-15` | `on tire on beş` | `forced.range` |
| `mesai 09:00-17:30 arası` | `mesai dokuz tire on yedi otuz arası` | `forced.range` |
| `1:3` | `bire üç` | `forced.ratio` |
| `532 123 45 67` | `beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi` | `forced.telephone` |
| `0532` | `sıfır beş üç iki` | `forced.digits` |
| `Maç 3-1 bitti` | `Maç üç tire bir bitti` | `forced.range` |
| `IV` / `IV. Murat` | `dört` / `dördüncü Murat` | `forced.roman` |
| `21. yüzyılda` | `yirmi birinci yüzyılda` | `forced.ordinal` |
| `1. Dünya Savaşı` | `birinci Dünya Savaşı` | `forced.ordinal` |
| `09:30'de` / `09:30'dan` | `dokuz otuzda` / `dokuz otuzdan` | `forced.time`, suffix split off |
| `14.03.2026'daki` | `on dört Mart iki bin yirmi altıdaki` | `forced.date`, suffix split off |
| `Louis XIV'ün` | `Louis on dördün` | `forced.roman`, suffix split off |
| `1.'nın` | `birincinin` | `forced.ordinal`, suffix split off |
| `info@ornek.com'a` | `info et ornek nokta koma` | `forced.electronic`, suffix split off |
| `Toplam 25.` | `Toplam yirmi beş.` | `forced.literal` |
| `29.02.1900` | `yirmi dokuz Şubat bin dokuz yüz` | `forced.date` |
| `29.02.1900'de` | `yirmi dokuz Şubat bin dokuz yüzde` | `forced.date`, suffix split off |
| `32.01.2026` | `otuz iki nokta sıfır bir nokta iki bin yirmi altı` | `forced.literal` |
| `3 + 4 = 7` | `üç artı dört eşittir yedi` | `forced.literal` |
| `1,005 TL` | `bir virgül sıfır sıfır beş lira` | `forced.literal` |
| `Fiyat: $1,299.99` | `Fiyat: bin iki yüz doksan dokuz dolar doksan dokuz sent` | `forced.money` |
| `+90 532 123 4567` | `artı doksan beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi` | `forced.telephone` |
| `son başvuru 01/04/26` | `son başvuru bir Nisan iki bin yirmi altı` | `forced.date` |
| `Saat 3:45:30'da` / `rekor 2:00:35` | `Saat üç kırk beş otuzda` / `rekor iki saat otuz beş saniye` | `forced.time` |
| `ve Elizabeth II.` | `ve Elizabeth ikinci.` | `forced.roman` |
| `maç 90+4. dakikada bitti` | `maç doksan artı dördüncü dakikada bitti` | `forced.literal` |
| `6.8" ekran` | `altı virgül sekiz inç ekran` | `forced.literal` |
| `rüzgâr K-KD` | `rüzgâr kuzey kuzeydoğu` | `forced.literal` |
| `1/2 bardak` / `nüfusun 2/3'ü` | `bir bölü iki bardak` / `nüfusun iki bölü üçü` | `forced.fraction` |
| `3x faydalı` / `2x Intel` / `y = 2x` | `üç kat faydalı` / `iki adet Intel` / `ye eşittir iki iks` | `forced.literal` |
| `70 °F` | `yetmiş derece Fahrenhayt` | `forced.literal` |
| `KDV'ler` / `KDV'lerde` | `katma değer vergileri` / `katma değer vergilerinde` | `forced.literal` |
| `Verstappen 1:23.456` | `Verstappen bir dakika yirmi üç virgül dört yüz elli altı saniye` | `forced.time` |
| `Puan: 4,5/5` | `Puan: dört virgül beş bölü beş` | `forced.fraction` |
| `Atatürk Mah. 1234. Sok.` | `Atatürk mahallesi bin iki yüz otuz dördüncü sokak` | `forced.ordinal` |

### The literal reading

Literal reads a span as written, left to right, without inventing meaning. It
is the last guess of the forced order, the reading of a stray word, and a hint
kind any caller can use. With an explicit hint it follows the policy's style:
`1,005 TL` is `… beş Türk lirası` under Preserve and `… beş lira` under Forced.

- **Digit runs** are read as a cardinal. A zero-padded run, one beyond the
  cardinal magnitude limit, or an identifier of seven digits or more is read
  digit by digit; ten digits that start with 5 are a mobile number, said in its
  groups. A number in groups of thousands is one amount: `1.500`, `9.876,54`,
  `1,000,000` and `1 000 000`. In a code, eight characters or more that switch
  between letters and digits at least twice (`1Z999AA10123456784`), digits are
  said one by one, except a round number: `i9-14900K` is
  `i dokuz tire on dört bin dokuz yüz Ke`. After a decimal comma, leading zeros
  are said one by one and up to three digits after them as a number
  (`6,022` is `altı virgül sıfır yirmi iki`). A number that starts a word, with
  a period glued after it before lowercase letters, is an ordinal: the letters
  are its suffix when they are one (`1.si` birincisi, `4.lük` dördüncülük) and
  the next word otherwise (`3.kat` üçüncü kat, `21.yy'ın` yirmi birinci
  yüzyılın). One letter after the period is a label or a version, not a word:
  `3.x` is `üç nokta iks`, `Madde 2.a` is `Madde iki nokta a`.
- **Separators between two parts of the span:** `.` nokta, `,` virgül,
  `:` iki nokta, `/` slaş, `-` tire. A hyphen between a word and a number is
  silent (`COVID-19` is `kovid on dokuz`), and so is a hyphen between two words
  (`E-POSTA` is `E posta`, `T-shirt` is `Te shirt`), but not between two single
  letters, which is a range (`A-Z` is `A tire Ze`), nor where the span has two
  hyphens or more, as a code does (`SKU-48291-BLK-XL` is
  `se ke u tire dört sekiz iki dokuz bir tire Be Le Ke tire İks Le`); a period
  between two single letters is silent too (`Y.Z.` is `Ye Ze.`). A period,
  comma or colon before a space, and a colon or comma written between a word
  and a number, are punctuation (`Fiyat:1.250TL` is
  `Fiyat: bin iki yüz elli lira`), but letters glued to a digit are no word, so
  each colon of `00:1A:2B` is said. One point between two numbers is a decimal
  comma when a unit, a currency or a count is read with them. A hyphen after a
  Greek letter alone is silent too (`β-karoten` is `beta karoten`). Words joined
  by a slash are said with slaş (`Euro/dolar`), except `ve/veya`, said ve veya.
- **A hyphen directly before the first digit** is a minus sign: eksi.
- **Symbols:** `+` artı, `−` eksi, `=` eşittir, `×` iks, `÷` bölü, `*` çarpı,
  `%` yüzde, `&` ve, `@` et, `#` heşteg (numara before a number or a code:
  `#123`, `#TR-2026-1`), `<` küçüktür, `>` büyüktür,
  `°` derece (`°C` and `℃` are derece too), `_` alt çizgi, `½` yarım (buçuk
  right after a digit), `¼` çeyrek, `¾` üç çeyrek, `₺` lira, `$` dolar,
  `€` avro, `£` sterlin, `¥` yen, `₽` ruble, `₹` rupi, `₼` manat and other
  currency signs, said after their number (`¥500` is `beş yüz yen`), `⅓` bir
  bölü üç and the other fractions of one character, `§` paragraf, `№` numara,
  `★` yıldız, `≈` and `~` yaklaşık (`~` only before a number),
  `≠` eşit değildir, `≤` küçük eşittir, `≥` büyük eşittir, `±` artı eksi,
  `∞` sonsuz, `‰` binde, `√` karekök, `π` pi, `⭐` yıldız, `²` kare, `³` küp,
  and `^` üssü between two numbers or letters and şapka elsewhere. `%` after a
  number is said before it (`5%` is `yüzde beş`). `*`, `<` and `>` are said
  only between two numbers or single letters (`3 * 4`, `a*b`, `x > 5`), a
  comparison also before a number (`< 1 dk`); `_` only inside a word (`a_b`).
  Around words they mark emphasis, a footnote or a menu path and say nothing
  (`**Önemli**` is `Önemli`, `**kişi**den` kişiden, `Fiyat*` Fiyat,
  `_vurgu_` vurgu, `Kategori > Telefon` Kategori Telefon). A `*` right after a
  count is its stars, yıldızlı before the word it rates (`5* otel` is
  `beş yıldızlı otel`). A superscript right after a word with a vowel is its
  footnote and says nothing (`Yazar¹`, `kaynak²`); after a number, a single
  letter, the letters of a formula or a unit it is a power (`10²`, `x²`, `mc²`,
  `cm³`). Among symbols alone, `!` is
  ünlem and `?` soru işareti (`!@#` is `ünlem et heşteg`). Two superscript
  digits or more are one power (`10²³` is `on üssü yirmi üç`, `10⁻³` on üssü
  eksi üç). On a phone's keypad, `*` is yıldız and `#` kare (`*123#` is
  `yıldız yüz yirmi üç kare`). A Greek letter alone is said by its Turkish name
  (`α` alfa, `β` beta, `μ` mü).
- **Counts and units glued to a number:** `K` and `M` after up to three digits
  or a fraction are bin and milyon (`45K` kırk beş bin, `1.5K` bir virgül beş
  bin, while `4K` is dört Ke and `14900K` a model), and `W`, `V` and `MP` are
  vat, volt and megapiksel. `v` before a version number is versiyon. An `x`
  glued after a count multiplies the word or number after the span: kere
  before a number (`3x 1.250 TL`), adet before a name (`2x Intel`) and kat
  before another word (`3x faydalı`); after an operator, or with nothing said
  after it, it is a variable and stays iks (`y = 2x`).
- **Marks after a number:** `''`, `″`, or `"` after a decimal, are inches
  (`55''` is `elli beş inç`); after a degree, `'` and `"` are minutes and
  seconds (`41°00'49"` is `kırk bir derece sıfır dakika kırk dokuz saniye`).
  `°C` is the degree alone, and `°F` and `℉` are derece Fahrenhayt. An
  apostrophe after the last number with nothing after it is a minute's mark,
  said with the number (a goal at `12'` is `on iki`), unless a quote opened
  before the number (`'5'` is `'beş'`).
- **A face, an arrow or a decoration** says nothing: `:D`, `:P`, `XD`, `<3`,
  `^_^`, `-_-`, `->`, `=>`, and one symbol written over and over (`***`, `###`,
  `===`). Nor does the mark that starts a Markdown line before a space: a list
  item or a quote (`* madde`, `> alıntı`), and a heading before a capital or a
  number (`# Başlık`), since `# işaretinden sonra` speaks of the sign itself.
- **A single letter** is said by its Turkish (TDK) name: `B` Be, `H` He,
  `K` Ke, `x` iks, `W` Çift ve. A vowel is its own name.
- **Capitals without a vowel** are spelled letter by letter: `TK` Te Ke,
  `XL` İks Le, `TK'yı` Te Ke'yı. So are consonants after one capital (`Hz` He
  ze), chemical symbols written together (`NaCl` Ne a Ce le, `H₂O` He iki O),
  two capitals that are no word (`ID` İ De, `BA` Be A), three capitals that end
  in two consonants no Turkish word ends in, a vowel being its own name (`ADB`
  is `A De Be`), and three capitals that start with two consonants no word
  starts with (`NGO` Ne Ge O). A longer word is a word even so (`DZEKO` is
  dzeko, `ŞNORKEL` şnorkel), and a doubled last letter is no proof (`OFF`). Two
  capitals that make a Turkish word, or an English one a Turkish text quotes,
  are that word: `VE` is ve, `TA` ta, `MI` mı, `NO` no, `IN` in, `MY` my. A
  sound written in consonants is never spelled (`Hmm`, `Pff`, `Şşt`, `Brr`,
  `HMM`).
- **Lowercase consonants** of two letters or more are spelled too (`vs` is
  `ve se`, `tcp` te ce pe), unless they are a unit (`km`), a sound (`hmm`) or an
  English word whose `y` is its vowel (`my`). Those the lexicon knows in
  capitals are that initialism, as one capital and consonants are: `kdv` and
  `Kdv` are `katma değer vergisi`, `Tl` is lira.
- **Other capitals** are a word, said in lowercase: `İSTANBUL'U` is
  `istanbul'u`, `IĞDIR'A` is `ığdır'a`. A word that cannot be Turkish, with
  `W`, `Q` or `X` or starting with two consonants no word starts with, says its
  `I` as i (`WINDOWS` windows, `PFIZER` pfizer). Letters a Roman numeral is
  written with stay as written (`CLI`), and a word with a lowercase letter is
  unchanged.
- **A word the lexicon reads** is read by it: an abbreviation or a currency
  code anywhere (the lowercase `tl` too), a unit label right after a number,
  and a time letter after a unit and a slash (`km/s` is kilometre slaş saniye).
  A domain's `com` after a point is kom, and an abbreviation of two letters or
  more written without its period before a suffix is still read (`Cad'de`
  caddesinde). A unit written after a number that another span read is read as
  a unit (the `mm` of `159,9 x 76,7 x 8,25 mm`). A unit of two
  letters or more may be in other capitals there: `5 KG` is `beş kilogram`,
  `250 GR'lık` is `iki yüz elli gramlık`. A single letter is exact, so `5G` is
  `beş Ge`. A unit of the lexicon is read anywhere but right before a number
  (`GB 2312` is `Ge Be iki bin üç yüz on iki`). A unit with its power, a
  superscript or one digit, is read wherever it stands: `m2` and `m²` are `metrekare`, `km2` is `kilometrekare` and `m³`
  is `metreküp`.
- **Other letters and words** are unchanged, in the NFC form used for
  recognition.
- **Whitespace, and sentence punctuation at the edges of the span,** are
  unchanged.
- **An apostrophe suffix after a spoken part**, in any case (`'DE` is `'de`), is
  harmonized again to that word when it is a known inflection. Any other suffix
  is fitted to the word: its vowels follow the word's harmony, a first `d` or
  `c` follows its last sound, and after a consonant a buffer `y` or a possessive
  `s` falls away; the relative `ki` keeps its vowel. A suffix written for the
  same sounds is unchanged: `90'lar` -> `doksanlar`, while `5 TL'li` ->
  `beş liralı` and `TL'ler` -> `liralar`. A derivation on a currency is built on
  its everyday word: `5 TL'lik` -> `beş liralık`. A compound says its
  possessive once, drops it before a derivation and says it again after a
  plural: `KDV'li` -> `katma değer vergili`, `KDV'si` -> `katma değer vergisi`,
  `KDV'ler` -> `katma değer vergileri`, `KDV'lerde` ->
  `katma değer vergilerinde`.
- **A backtick or an acute accent** typed for an apostrophe is one
  (`Mehmet`in` is unchanged, `5`i` is `beşi`).
- **Any other character** is removed: an unnamed symbol such as `|`, and the
  marks of a list, an arrow, a check mark and the signs of a trademark (`•`,
  `→`, `✓`, `©`, `®`, `™`).

| Written | Literal reading under Forced |
|---|---|
| `32.01.2026` | `otuz iki nokta sıfır bir nokta iki bin yirmi altı` |
| `32.01.2026'de` | `otuz iki nokta sıfır bir nokta iki bin yirmi altıda` |
| `3 + 4 = 7` | `üç artı dört eşittir yedi` |
| `1,005 TL` | `bir virgül sıfır sıfır beş lira` |
| `5 TL'lik` | `beş liralık` |
| `5kg` | `beş kilogram` |
| `-5°C'den` | `eksi beş dereceden` |
| `2.5.1` | `iki nokta beş nokta bir` |
| `H2O` | `He iki O` |
| `TK1956` / `CD'Yİ` | `Te Ke bin dokuz yüz elli altı` / `si diyi` |
| `ADB` / `İSTANBUL'U` / `COVID-19` | `A De Be` / `istanbul'u` / `kovid on dokuz` |
| `5 KG` / `250 GR'lık` | `beş kilogram` / `iki yüz elli gramlık` |
| `m2` / `120 m²'lik` / `5 m³'lük` | `metrekare` / `yüz yirmi metrekarelik` / `beş metreküplük` |
| `KDV'li` / `5 TL'li` / `TBMM'DE` | `katma değer vergili` / `beş liralı` / `te be me mede` |
| `1920x1080` | `bin dokuz yüz yirmi iks bin seksen` |
| `AB-12` | `a be on iki` |
| `½` / `2½` | `yarım` / `iki buçuk` |
| `12GB/512GB` / `200MP` / `45K` | `on iki gigabayt slaş beş yüz on iki gigabayt` / `iki yüz megapiksel` / `kırk beş bin` |
| `00:1A:2B` | `sıfır sıfır iki nokta bir A iki nokta iki Be` |
| `!@#$%^&*` | `ünlem et heşteg dolar yüzde şapka ve çarpı` |
| `⅓` / `¥500` / `§ 5` | `bir bölü üç` / `beş yüz yen` / `paragraf beş` |
| `10²³` / `6,022` / `1.si` | `on üssü yirmi üç` / `altı virgül sıfır yirmi iki` / `birincisi` |
| `β-karoten` / `ID` / `*123#` | `beta karoten` / `İ De` / `yıldız yüz yirmi üç kare` |
| `VE` / `MI` / `Hmm` / `vs` | `ve` / `mı` / `Hmm` / `ve se` |
| `**Önemli:**` / `Yazar¹` / `5* otel` | `Önemli:` / `Yazar` / `beş yıldızlı otel` |
| `E-POSTA` / `A-Z` / `3.x` | `E posta` / `A tire Ze` / `üç nokta iks` |
| `destek@şirket.com.tr` | `destek et şirket nokta kom nokta te re` |
