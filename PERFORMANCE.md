# Performance and measurement

The current normalizer met warm release Rust **p95 < 1 ms**, separately for
short (1-256) and medium (257-1024) original UTF-8 byte cohorts on the inspected
host. This is a measured development target, not a universal SLA, a hosted CI
timing gate or an end-to-end speech-generation guarantee.

The public inputs are in `benches/corpus.json` and `benches/intent-corpus.json`.
Do not shrink the corpus, weaken validation or remove slow cases to claim
improvement. Current semantic outcomes are checked independently by the
characterization and class tests.

## Reproduce

See [CONTRIBUTING.md](CONTRIBUTING.md) to provision the isolated Python tools
and advisory checker/database. From a clean committed Windows checkout:

```powershell
.\scripts\verify.ps1 -PythonPath <absolute-build-python> -RustupHome <optional-isolated-rustup> -OutputDirectory <new-absolute-directory>
```

Omit `-RustupHome` when using the normal installed toolchain. The output path
must be new; reports and packages are never overwritten.

The serial command verifies source/tests/docs, clean packages and external
consumers, then emits `manifest.json` and `summary.txt`. The manifest links
source HEAD, current normalizer identity, compiler/build/host context, corpus/
native-binary hashes, stage results, Cargo VCS and exact package checksums.
It does not commit, publish, run models or change global tools/settings.
Dependent Cargo stages are serial to avoid artifact races.

For the native sampler alone, use a new output path:

```powershell
cargo bench --locked --features serde --bench latency -- --output <new-absolute-report.json>
```

## Native methodology

- `Instant` measures 10,000 individual calls per cohort after 2,000 warmup calls.
- A deterministic mix interleaves Preserve/Reject/Forced policy cases; outcomes
  are checked before and after sampling.
- Timings include owned result/error disposal, not just recognition.
- Nearest-rank p50/p95/p99/max and per-class/policy counts are reported.
- Every outlier is retained. Clock overhead is reported, not subtracted.
- Three repetitions are retained by the full verifier; results are not replaced
  by a batched average or confidence interval.

The first process initialization is recorded separately from 100 later cached
same-process constructor calls. Those later calls are **not cold starts**.
Large 4/16/32 KiB inputs, maximum hints, candidate/result errors and long
identifiers are separate scaling diagnostics, not 1 ms guarantees.

## Python, host and limits

Installed Python measurements include public argument validation, native Rust
work, marshaling and result/error disposal. Eight-thread public-call timings
are separate from the Rust target and from model/worker/audio costs.
Native allocation/peak instrumentation is unavailable; no inferred allocation
counts or peak-memory claims are made.

Inspected host: AMD Ryzen AI 7 PRO 350, 8 cores / 16 logical processors,
Windows 11 Enterprise 26200, Rust 1.94 x64 MSVC. Shared-host power/load/noise
context is recorded without changing settings or stopping other work. Timings
on GitHub-hosted runners or other hardware are not assumed equivalent.

Retained mechanisms are borrowed already-NFC grapheme-safe mapping with an NFC
fallback, indexed overlap windows, exact numerals and one immutable pattern
cache. There is no input/result memoization. Optimize only a measured bottleneck
or regression; once the target passes, prioritize clarity rather than more
performance machinery. Historical local reports/packages are not public
repository contents or a selectable comparison mode.
