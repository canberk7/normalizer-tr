# Current normalizer provenance

One authored normalizer, one diagnostic package/build identity. Typed lexical/
phonological inventory is in domain/lexicon.rs; patterns are validated/cached
once in resources.rs. There is no selectable profile, older table or mode.

Owned Rust code, unit/currency/abbreviation/character readings and finite
source/target/derived-tail metadata are Apache-2.0. No bulk dictionary or
third-party normalizer source was copied. Exact numeral behavior was
cross-checked against Unicode CLDR release48 Turkish RBNF:
https://github.com/unicode-org/cldr/blob/release-48/common/rbnf/tr.xml
(Copyright1991-2025 Unicode; Unicode LicenseV3).
The notice is retained in THIRD_PARTY_NOTICES.md; CLDR is a reference,
not a bundled runtime grammar or whole-sentence oracle.

TDK number/apostrophe/abbreviation writing guidance informed the bounded rules:
https://tdk.gov.tr/icerik/yazim-kurallari/sayilarin-yazilisi/
https://tdk.gov.tr/icerik/yazim-kurallari/kesme-isareti/
https://tdk.gov.tr/icerik/yazim-kurallari/kisaltmalar/

NFC/grapheme behavior comes from the locked unicode-normalization and
unicode-segmentation dependencies under their own terms. No NFKC, global
casing rewrite, learned model, private dictionary, account lookup or network
address fetch is introduced.

Current semantic fixtures are refactor regression tests, not an API promising
older behavior forever. The financial and IBAN examples are selected public
test inputs, not account verification or native-language/voice sign-off.
