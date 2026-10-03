# Contributing

Use minimal synthetic examples when reporting bugs: written input, expected
spoken text, actual completeness/issues, options/hints, revision and environment.
Do not include credentials, real account identifiers or private customer text.

## Set up

The release toolchain is Rust 1.99.0; the declared minimum is 1.94 and is tested
separately. The toolchain file includes rustfmt/Clippy. Python release tests
cover ordinary CPython 3.11–3.14 on the selected Windows/Linux/macOS platforms.
On Windows, install the MSVC C++ build tools.

```powershell
git clone https://github.com/erdemtuna/normalizer-tr.git
Set-Location normalizer-tr
cargo test --locked
```

For Python development:

```powershell
py -3.13 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r .\bindings\python\requirements-dev.txt -c .\bindings\python\constraints.lock
.\.venv\Scripts\python.exe -m pip install .\bindings\python
.\.venv\Scripts\python.exe -m pytest .\bindings\python\tests -q
```

Core/binding development needs no Torch, model files or credentials.

## Code layout

`src/classify` owns scanning, fixed reader priority, whole-span claims and the
forced order; `src/domain` owns validated values and the literal reading;
`src/numerals.rs`, `src/morphology.rs` and `src/verbalize.rs` share exact
rendering and spoken-tail metadata.
`src/source_map.rs` maps NFC recognition to original UTF-8/grapheme coordinates.
`src/pipeline.rs` composes the source partition, diagnostics and budgets.
The Python binding calls this engine; it does not duplicate language rules.

Add positive, negative and mixed-sentence tests. Assert completeness, issue
scope and byte ranges as well as text. Preserve matched-invalid compounds
instead of rewriting their fragments. Reuse exact-number/morphology helpers;
do not introduce unsafe Rust, floating-point money, global lowercasing,
emitted-text reparsing or result caches. Review deliberate reading changes
separately from refactors; do not regenerate fixtures just to pass.

## Checks

```powershell
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps --all-features
.\.venv\Scripts\python.exe -m ruff check bindings\python benches scripts
.\.venv\Scripts\python.exe -m ruff format --check bindings\python benches scripts
.\.venv\Scripts\python.exe -m mypy bindings\python\python --check-untyped-defs
```

Rebuild/reinstall the binding after native/facade changes before running its
tests. CI also checks default/no-default Rust configurations and installed-wheel
tests, without models.

For full local package/consumer/audit/performance verification, provision:

```powershell
cargo install cargo-audit --version 0.22.2 --locked --root .\target\audit-tool
git clone https://github.com/RustSec/advisory-db.git .\target\advisory-db
```

From a clean committed checkout, use a new absolute output directory:

```powershell
$python = (Resolve-Path .\.venv\Scripts\python.exe).Path
$output = Join-Path $env:TEMP 'normalizer-tr-verification-01'
.\scripts\verify.ps1 -PythonPath $python -OutputDirectory $output
```

Optional `-RustupHome <absolute-isolated-path>` selects an existing isolated
toolchain. Keep the output/environment path short on Windows to avoid
path-length limits. Outputs include logs, packages, consumers and a checksum manifest.
See [PERFORMANCE.md](PERFORMANCE.md) for host-specific timing limits.

Owned contributions are Apache-2.0; retain third-party notices. Keep credentials,
external model source/weights, audio, wheels, environments and reports out of
Git. Only the core can publish to crates.io; the internal Rust/Python companion
keeps `publish = false`. Registry publishing is a separate owner-controlled
release action after built-artifact verification, never an untrusted PR action.
