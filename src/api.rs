use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

/// Original-source half-open UTF-8 byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SourceRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl SourceRange {
    /// Construct coordinates. Validity against text is checked by normalization.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    /// Inclusive start byte offset.
    pub const fn start(self) -> usize {
        self.start
    }
    /// Exclusive end byte offset.
    pub const fn end(self) -> usize {
        self.end
    }
}

/// How unresolved linguistic expressions are handled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AmbiguityPolicy {
    /// Preserve source spans and return structured issues.
    #[default]
    Preserve,
    /// Return an error containing unresolved diagnostics, without partial output.
    Reject,
    /// Speak everything: read every unresolved span with a forced reading and still return
    /// its issue, say readings in everyday style, and name stray symbols and spelled letters.
    Forced,
}

/// Explicit interpretation of a whole original-source span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HintKind {
    /// An exact integer, including valid grouping and explicit zero-padding intent.
    Cardinal,
    /// ASCII digits, optional initial plus, and space/dot/slash/hyphen/parentheses.
    Digits,
    /// A valid numeric Gregorian date.
    Date,
    /// A valid 24-hour clock.
    Time,
    /// Explicit ordinal intent on a numeric period or ordinal suffix expression.
    Ordinal,
    /// Canonical uppercase Roman numeral; period establishes ordinal intent.
    Roman,
    /// Exact bounded numeric endpoints with optional quantity context.
    Range,
    /// Turkish national or +90 telephone reading.
    Telephone,
    /// Supported whole ASCII address or cued/hinted bare domain.
    Electronic,
    /// The span as written, with digit runs, separators, symbols and spelled letters spoken;
    /// it reads any content.
    Literal,
}

/// Caller intent at original grapheme-safe byte coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hint {
    pub(crate) range: SourceRange,
    pub(crate) kind: HintKind,
}

impl Hint {
    /// Create a hint; normalization validates coordinates, overlap, and content.
    pub const fn new(range: SourceRange, kind: HintKind) -> Self {
        Self { range, kind }
    }
    /// Original-source range.
    pub const fn range(self) -> SourceRange {
        self.range
    }
    /// Requested interpretation.
    pub const fn kind(self) -> HintKind {
        self.kind
    }
}

/// Per-call options. Guessing is never implicit: [`AmbiguityPolicy::Forced`] is opt-in.
#[derive(Clone, Debug, Default)]
pub struct NormalizeOptions {
    /// Preserve unresolved spans by default, or reject or force them explicitly.
    pub ambiguity_policy: AmbiguityPolicy,
    /// Non-overlapping, whole-expression hints in original-source coordinates.
    pub hints: Vec<Hint>,
}

/// Semantic kind of a result segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SegmentKind {
    /// Unchanged ordinary words, whitespace, or punctuation.
    Verbatim,
    /// Preserved unresolved linguistic work.
    Unresolved,
    /// Exact integer.
    Cardinal,
    /// Integer ordinal.
    Ordinal,
    /// Exact comma decimal.
    Decimal,
    /// Individual digits.
    Digits,
    /// Prefix percentage.
    Percent,
    /// Exact approved currency major/minor amount (TRY/USD/EUR/GBP).
    Money,
    /// Approved unit quantity or bounded rate.
    Unit,
    /// Gregorian date.
    Date,
    /// 24-hour digital clock.
    Time,
    /// Approved abbreviation.
    Abbreviation,
    /// Contextual, explicitly hinted or forced numerical range.
    Range,
    /// Grouped Turkish telephone expression.
    Telephone,
    /// Full checksum-valid Turkish IBAN.
    Iban,
    /// Canonical uppercase Roman numeral with explicit, contextual or forced intent.
    Roman,
    /// Supported email or web address.
    Electronic,
    /// Approved prose hashtag or ampersand.
    Symbol,
    /// Span read as written, with its digit runs, separators, symbols and spelled letters spoken.
    Literal,
}

/// Machine-readable reason for preserved linguistic work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum IssueCategory {
    /// Multiple or insufficiently cued readings.
    Ambiguous,
    /// Invalid supported grammar, value, or suffix allomorph.
    InvalidExpression,
    /// Structured identifier that must not be rewritten in fragments.
    ProtectedIdentifier,
    /// Expression outside the normalizer's bounded coverage.
    Unsupported,
    /// Unapproved uppercase abbreviation.
    UnknownAbbreviation,
}

/// Non-sensitive diagnostic for an unresolved original-source range.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Issue {
    pub(crate) range: SourceRange,
    pub(crate) category: IssueCategory,
    pub(crate) explanation: &'static str,
}

impl Issue {
    /// Original-source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
    /// Machine-readable category.
    pub const fn category(&self) -> IssueCategory {
        self.category
    }
    /// Static explanation; contains no copied input values.
    pub const fn explanation(&self) -> &'static str {
        self.explanation
    }
}

/// One member of an ordered, contiguous original-source partition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Segment {
    pub(crate) range: SourceRange,
    pub(crate) kind: SegmentKind,
    pub(crate) text: String,
    pub(crate) rule_id: &'static str,
}

impl Segment {
    /// Original-source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }
    /// Semantic kind.
    pub const fn kind(&self) -> SegmentKind {
        self.kind
    }
    /// Emitted text.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Diagnostic identifier of the reader that emitted this segment.
    pub const fn rule_id(&self) -> &'static str {
        self.rule_id
    }
}

/// Owned immutable result. Completeness concerns TN work, not voice quality.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NormalizeResult {
    pub(crate) normalized_text: String,
    pub(crate) locale: &'static str,
    pub(crate) normalizer_id: &'static str,
    pub(crate) complete: bool,
    pub(crate) segments: Vec<Segment>,
    pub(crate) issues: Vec<Issue>,
}

impl NormalizeResult {
    /// Concatenation of all emitted segment text.
    pub fn normalized_text(&self) -> &str {
        &self.normalized_text
    }
    /// Selected locale.
    pub const fn locale(&self) -> &'static str {
        self.locale
    }
    /// Diagnostic identity of the normalizer build.
    pub const fn normalizer_id(&self) -> &'static str {
        self.normalizer_id
    }
    /// Whether there are no unresolved TN issues.
    pub fn complete(&self) -> bool {
        self.complete
    }
    /// Ordered original-source partition.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
    /// Ordered unresolved diagnostics.
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }
}

/// Resource limit which was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitKind {
    /// Original UTF-8 input length.
    Input,
    /// Hint count.
    Hints,
    /// Candidate work records.
    Candidates,
    /// Logical owned result allocation.
    Result,
}

/// Explicit input, policy, resource, control, or engine failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizeError {
    /// Empty/all-whitespace input or unsupported control/Bidi_Control characters.
    InvalidInput,
    /// Invalid hint range, content, overlap, or protected-expression boundary.
    InvalidHint,
    /// Bundled assets or project-owned patterns are inconsistent.
    InvalidConfiguration,
    /// A documented resource limit was exceeded.
    LimitExceeded(LimitKind),
    /// Cooperative cancellation or monotonic deadline.
    Cancelled,
    /// Strict-mode unresolved linguistic diagnostics.
    Unresolved(Vec<Issue>),
    /// Unexpected violated internal invariant.
    Internal,
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidInput => "input is empty, whitespace-only, or contains forbidden controls",
            Self::InvalidHint => "hint is not a valid whole-expression original-source range",
            Self::InvalidConfiguration => "built-in normalizer configuration is invalid",
            Self::LimitExceeded(_) => "normalization resource limit exceeded",
            Self::Cancelled => "normalization was cancelled or its deadline expired",
            Self::Unresolved(_) => "strict normalization contains unresolved linguistic work",
            Self::Internal => "normalization invariant failed",
        };
        f.write_str(message)
    }
}

impl std::error::Error for NormalizeError {}

/// Runtime-neutral cooperative control. Clones share the cancellation signal.
#[derive(Clone, Debug, Default)]
pub struct WorkControl {
    cancelled: Arc<AtomicBool>,
    deadline: Option<Instant>,
}

impl WorkControl {
    /// Construct control with an optional monotonic deadline.
    pub fn new(deadline: Option<Instant>) -> Self {
        Self {
            deadline,
            ..Self::default()
        }
    }
    /// Cancel this control and all its clones.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    pub(crate) fn check(&self) -> Result<(), NormalizeError> {
        if self.cancelled.load(Ordering::Relaxed)
            || self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Err(NormalizeError::Cancelled)
        } else {
            Ok(())
        }
    }
}
