//! A local, synchronous Turkish text-to-speech normalizer.
//!
//! The default preserves unresolved spans and reports them. Use
//! [`AmbiguityPolicy::Reject`] when partial speech is not acceptable.
//! Original source ranges are UTF-8 byte coordinates, not character indices.
//!
//! ```
//! use normalizer_tr::{Normalizer, NormalizeOptions};
//! let normalizer = Normalizer::new()?;
//! let result = normalizer.normalize("25 TL", &NormalizeOptions::default())?;
//! assert_eq!(result.normalized_text(), "yirmi beş Türk lirası");
//! assert!(result.complete());
//! # Ok::<(), normalizer_tr::NormalizeError>(())
//! ```
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod api;
mod classify;
mod domain;
mod model;
mod morphology;
mod normalizer;
mod numerals;
mod pipeline;
mod resources;
mod source_map;
mod verbalize;

pub use api::{
    AmbiguityPolicy, Hint, HintKind, Issue, IssueCategory, LimitKind, NormalizeError,
    NormalizeOptions, NormalizeResult, Segment, SegmentKind, SourceRange, WorkControl,
};
pub use normalizer::Normalizer;
pub use resources::NORMALIZER_ID;

/// Maximum original UTF-8 input length in bytes.
pub const MAX_INPUT_BYTES: usize = 32 * 1024;
/// Maximum number of caller hints.
pub const MAX_HINTS: usize = 256;
/// Maximum number of semiotic candidate records, including unresolved spans.
pub const MAX_CANDIDATES: usize = 4096;
/// Maximum logical bytes of owned result and diagnostic allocations.
pub const MAX_RESULT_BYTES: usize = 512 * 1024;
