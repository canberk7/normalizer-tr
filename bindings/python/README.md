# Python binding

Distribution `normalizer-tr` 0.4.0; import `normalizer_tr`, native submodule
`normalizer_tr._native`. This is the Python bridge to the same Rust engine, not a
second implementation. Release target: ordinary CPython 3.11–3.14 on
Windows/Linux x64 and macOS x64/arm64. Linux requires glibc 2.28+ and macOS
targets 12.0+. Rust 1.99 builds the extension.
No ABI3, PyPy or free-threaded Python claim. Import/use needs no Torch, hub, model folder, credentials,
network or Python normalization subprocess.

## Install

```text
python -m pip install normalizer-tr
```

Compatible binary wheels need no Rust compiler.

## Build from source

Install Rust and the MSVC C++ build tools before building. From a repository
checkout, in PowerShell:

```powershell
py -3.13 -m venv .venv
.\.venv\Scripts\python.exe -m pip install .\bindings\python
.\.venv\Scripts\python.exe -X utf8 .\bindings\python\examples\showcase.py
```

This builds a wheel locally. To install an existing compatible wheel, use
`python -m pip install --no-deps <wheel-path>`. Only source builds require Rust.

## API

```python
from normalizer_tr import Normalizer, Hint, NormalizationError, NORMALIZER_ID
n = Normalizer()
assert n.normalizer_id == NORMALIZER_ID
r = n.normalize("25 TL; 1.234")
assert r.normalized_text == "yirmi beş Türk lirası; 1.234"
assert not r.complete
assert (r.issues[0].start_byte, r.issues[0].end_byte) == (7, 12)
assert n.normalize("IV", hints=(Hint(0, 2, "roman"),)).normalized_text == "dört"
try:
    n.normalize("1.234", ambiguity_policy="reject")
except NormalizationError as error:
    assert error.code == "unresolved"
    assert error.issues
forced = n.normalize("25 TL; 1.234", ambiguity_policy="forced")
assert forced.normalized_text == "yirmi beş lira; bin iki yüz otuz dört"
assert not forced.complete and forced.issues == r.issues
assert forced.segments[2].rule_id == "forced.cardinal"
literal = n.normalize("2.5.1", hints=(Hint(0, 5, "literal"),))
assert literal.normalized_text == "iki nokta beş nokta bir"
```

`Normalizer()` is the only constructor; there is no selector argument or older
behavior mode. `normalizer_id` is diagnostic package/build metadata, not a
configuration knob. Result properties: normalized_text, locale, normalizer_id,
complete, immutable tuples of frozen segments/issues. Segment fields:
start_byte/end_byte/kind/text/rule_id. Issue fields:
start_byte/end_byte/category/explanation. All offsets are half-open
**original UTF-8 bytes**, not Python character indices.

`normalize(text, *, ambiguity_policy="preserve", hints=(), cancellation=None,
deadline_ms=None)` takes Python str, typed Hint objects, optional
CancellationToken, and an integer deadline1..60000ms. Booleans do not count as
integer coordinates/deadlines; surrogates are invalid input. Call-shape errors
are TypeError/ValueError, never partial success. Hint kinds:
cardinal/digits/date/time/ordinal/roman/range/telephone/electronic/literal.
Whole-span/grapheme/overlap/content checks are authoritative in Rust.

`ambiguity_policy` is `"preserve"` (default: unresolved spans stay as written),
`"reject"` (error, no result) or `"forced"`. Forced speaks everything for a
speech model: it reads every unresolved span with the first reading of a fixed
order, or literally when none fits, and still returns its issue, so `complete`
stays false; it speaks in everyday style (`4,25` is `dört virgül yirmi beş`,
`TL` is `lira`, `10-15` is `on tire on beş`, `24/7` is `yirmi dört slaş yedi`,
`12:05` is `on iki sıfır beş`, `#` is `heşteg`, `1/2` is `bir bölü iki`,
`1.si` is `birincisi`); it names
symbols and spells letters (`+` is `artı`, `B` is `Be`, `TK` is `Te Ke`); and
it reads text with bidirectional controls, which the other policies reject, as
if they were not there. The abbreviation
lexicon (`ABD` is `a be de`, `Cad.` is `caddesi`, `Sipariş No:` is
`Sipariş numarası`) reads the same under every policy. Forced segments keep
their reading's kind and have a `forced.` rule id. The `"literal"` hint reads a
span as written. The order and the literal rules are in the
[normalization reference](../../docs/normalization.md#forced-readings).

`NormalizationError` has code, immutable issues and limit_kind. Codes:
invalid_input, invalid_hint, invalid_configuration, limit_exceeded, cancelled,
unresolved, internal. Strict errors contain issues, no normalized result.
CancellationToken.cancel() signals associated ongoing/later calls. Native
work executes pure Rust detached from the interpreter after copying/validating
arguments; records marshal afterward. Actual concurrency/control tests cover it.

## Development and verification

See the root [contribution guide](../../CONTRIBUTING.md) for pinned development
requirements, source builds and installed-wheel tests. To explicitly build a
release wheel after installing those tools:

```powershell
.\.venv\Scripts\python.exe -m maturin build --release --locked --manifest-path .\bindings\python\Cargo.toml --interpreter .\.venv\Scripts\python.exe --out .\target\wheels
```

Final validation is one root command:

```powershell
.\scripts\verify.ps1 -PythonPath <isolated-build-python> -RustupHome <isolated-rustup> -OutputDirectory <new-absolute-directory>
```

The build interpreter needs pinned `requirements-dev.txt` / `constraints.lock` tools
(maturin, pytest, Ruff, mypy, pip-audit); the existing local Cargo audit tool
is provisioned locally as described in the contribution guide. The verifier builds/packages, inspects
licenses/content, installs into a fresh environment and tests actual public IO
without engine dependencies. It refuses overwrite/failure-shaped success.

The wheel contains facade/stubs/py.typed/native binary/SBOM and complete owned/
upstream notices only. No speech integration, model weights, network code or credentials.
External code/model redistribution rights remain separate and unestablished.
