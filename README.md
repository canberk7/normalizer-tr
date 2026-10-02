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

**Status:** early-development 0.3.0. Source is available on GitHub; packages are
**not yet published to crates.io or PyPI**. Both Cargo packages retain
`publish = false`. The instructions below install from source. Public API,
native-language and voice-quality review remain release gates.

## Choose your API

| Use case | Component | Dependencies |
|---|---|---|
| Rust applications | `normalizer-tr` / import `normalizer_tr` | Rust only; optional Serde |
| Python applications | `normalizer-tr-python` / import `normalizer_tr` | The compiled Rust binding; no speech model |

`bindings/python` is a separate Cargo workspace member and wheel, not part of
the core `.crate`. The binding depends on the Rust core, never the reverse.

## Rust quickstart

Use Rust 1.94.0 / edition 2024. From your application's `Cargo.toml`:

```toml
[dependencies]
normalizer-tr = { git = "https://github.com/erdemtuna/normalizer-tr", branch = "main" }
```

Commit your application's `Cargo.lock` to pin the resolved revision, or replace
`branch` with a reviewed `rev` for an explicit source pin. To work entirely from
a local checkout on Windows, use `normalizer-tr = { path = '..\normalizer-tr' }`
instead.

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

The Python bridge is already implemented; it calls Rust directly rather than
duplicating the language rules. The currently verified platform is **Windows
x64, CPython 3.13, Rust 1.94.0 MSVC**. Other OS/ABI combinations are not yet
verified, and no prebuilt public wheels are supplied.

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

The source installation builds a native wheel; a Rust compiler is required to
build it, but not to use an already-built compatible wheel. For a local wheel,
use `python -m pip install --no-deps <wheel-path>`.

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
failures are errors in either policy, not successful preservation.

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

## License and boundaries

Owned code and independently authored tables are
[Apache-2.0](https://github.com/erdemtuna/normalizer-tr/blob/main/LICENSE).
[Third-party notices](THIRD_PARTY_NOTICES.md) preserve dependency and Unicode
rights. This repository contains the normalizer and Python binding only: no
speech-engine integration, external model source, weights, credentials or audio.
