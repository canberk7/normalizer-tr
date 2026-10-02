# Release 0.3.0

First registry distribution of the single built-in Turkish normalizer.
The Rust crate and Python distribution are both named `normalizer-tr`;
the import is `normalizer_tr`.

The release uses Rust 1.99.0, with a separately tested minimum Rust 1.94.
Python artifacts target ordinary CPython 3.11–3.14: Windows/Linux x64 and
macOS x64/arm64. Linux's baseline is glibc 2.28; macOS's is 12.0.
Other interpreters, free-threaded builds and architectures are not promised.

Normalization readings, source offsets, partial/strict behavior and resource
limits are unchanged. This is a pre-1.0, bounded-coverage library—not a claim
of universal Turkish pronunciation or end-to-end speech quality.

## Releasing

`release.yml` is manually dispatched. Its default is **build/test only**:
16 interpreter/platform wheels, one tested self-contained source distribution,
core archive/dry-run checks and an exact-checksum release manifest.
No PR or ordinary push publishes packages.

PyPI publication requires the matching `v0.3.0` tag, `publish=true`, the `pypi`
environment and its configured Trusted Publisher. Only that publish job has
OIDC permission. No long-lived PyPI token is stored.

The initial crates.io upload requires owner-configured Cargo authentication;
the internal Rust/Python companion remains `publish = false`. Publish only
the reviewed core from the same clean revision, then verify fresh registry
consumers. Do not paste credentials into chat, command arguments or source.

If an upload returns an uncertain result, inspect the registry before retrying.
Do not reuse a version for different artifacts or claim success for a blocked
registry. All speech integrations/private history/models/audio remain outside
the public source and artifacts.
