use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use normalizer_tr::{
    AmbiguityPolicy, Hint, HintKind, Issue, IssueCategory, LimitKind, NormalizeError,
    NormalizeOptions, Normalizer, SegmentKind, SourceRange, WorkControl,
};
use pyo3::{create_exception, exceptions::PyException, prelude::*};

create_exception!(_native, NativeNormalizationError, PyException);

type IssueRecord = (usize, usize, &'static str, &'static str);
type SegmentRecord = (usize, usize, &'static str, String, &'static str);
type ResultRecord = (
    String,
    &'static str,
    &'static str,
    bool,
    Vec<SegmentRecord>,
    Vec<IssueRecord>,
);

fn issues(records: &[Issue]) -> Vec<IssueRecord> {
    records
        .iter()
        .map(|issue| {
            let category = match issue.category() {
                IssueCategory::Ambiguous => "ambiguous",
                IssueCategory::InvalidExpression => "invalid_expression",
                IssueCategory::ProtectedIdentifier => "protected_identifier",
                IssueCategory::Unsupported => "unsupported",
                IssueCategory::UnknownAbbreviation => "unknown_abbreviation",
            };
            (
                issue.range().start(),
                issue.range().end(),
                category,
                issue.explanation(),
            )
        })
        .collect()
}

fn error(error: NormalizeError) -> PyErr {
    let (code, limit, diagnostics) = match &error {
        NormalizeError::InvalidInput => ("invalid_input", None, Vec::new()),
        NormalizeError::InvalidHint => ("invalid_hint", None, Vec::new()),
        NormalizeError::InvalidConfiguration => ("invalid_configuration", None, Vec::new()),
        NormalizeError::Cancelled => ("cancelled", None, Vec::new()),
        NormalizeError::Internal => ("internal", None, Vec::new()),
        NormalizeError::Unresolved(records) => ("unresolved", None, issues(records)),
        NormalizeError::LimitExceeded(kind) => {
            let limit = match kind {
                LimitKind::Input => "input",
                LimitKind::Hints => "hints",
                LimitKind::Candidates => "candidates",
                LimitKind::Result => "result",
            };
            ("limit_exceeded", Some(limit), Vec::new())
        }
    };
    NativeNormalizationError::new_err((code, error.to_string(), limit, diagnostics))
}

fn segment_kind(kind: SegmentKind) -> &'static str {
    match kind {
        SegmentKind::Verbatim => "verbatim",
        SegmentKind::Unresolved => "unresolved",
        SegmentKind::Cardinal => "cardinal",
        SegmentKind::Ordinal => "ordinal",
        SegmentKind::Decimal => "decimal",
        SegmentKind::Digits => "digits",
        SegmentKind::Percent => "percent",
        SegmentKind::Money => "money",
        SegmentKind::Unit => "unit",
        SegmentKind::Date => "date",
        SegmentKind::Time => "time",
        SegmentKind::Abbreviation => "abbreviation",
        SegmentKind::Range => "range",
        SegmentKind::Telephone => "telephone",
        SegmentKind::Iban => "iban",
        SegmentKind::Roman => "roman",
        SegmentKind::Electronic => "electronic",
        SegmentKind::Symbol => "symbol",
        SegmentKind::Literal => "literal",
    }
}

#[derive(Default)]
struct Controls {
    cancelled: bool,
    next: u64,
    active: BTreeMap<u64, WorkControl>,
}

#[pyclass(frozen, skip_from_py_object, module = "normalizer_tr._native")]
#[derive(Clone, Default)]
struct CancellationToken {
    controls: Arc<Mutex<Controls>>,
}

#[pymethods]
impl CancellationToken {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn cancel(&self) -> PyResult<()> {
        let mut controls = self
            .controls
            .lock()
            .map_err(|_| error(NormalizeError::Internal))?;
        controls.cancelled = true;
        for control in controls.active.values() {
            control.cancel();
        }
        Ok(())
    }
}

impl CancellationToken {
    fn register(&self, control: &WorkControl) -> Result<u64, NormalizeError> {
        let mut controls = self.controls.lock().map_err(|_| NormalizeError::Internal)?;
        if controls.cancelled {
            control.cancel();
        }
        controls.next = controls
            .next
            .checked_add(1)
            .ok_or(NormalizeError::Internal)?;
        let id = controls.next;
        controls.active.insert(id, control.clone());
        Ok(id)
    }
    fn unregister(&self, id: u64) -> Result<(), NormalizeError> {
        self.controls
            .lock()
            .map_err(|_| NormalizeError::Internal)?
            .active
            .remove(&id);
        Ok(())
    }
}

#[pyclass(frozen, name = "NativeNormalizer", module = "normalizer_tr._native")]
struct NativeNormalizer {
    inner: Normalizer,
}

#[pymethods]
impl NativeNormalizer {
    #[getter]
    fn normalizer_id(&self) -> &'static str {
        self.inner.normalizer_id()
    }
    #[new]
    fn new() -> PyResult<Self> {
        let inner = Normalizer::new().map_err(error)?;
        Ok(Self { inner })
    }

    #[pyo3(signature = (text, policy, hints, token=None, deadline_ms=None))]
    fn normalize(
        &self,
        py: Python<'_>,
        text: String,
        policy: &str,
        hints: Vec<(usize, usize, String)>,
        token: Option<PyRef<'_, CancellationToken>>,
        deadline_ms: Option<u64>,
    ) -> PyResult<ResultRecord> {
        let policy = match policy {
            "preserve" => AmbiguityPolicy::Preserve,
            "reject" => AmbiguityPolicy::Reject,
            "forced" => AmbiguityPolicy::Forced,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "invalid ambiguity policy",
                ));
            }
        };
        let hints = hints
            .into_iter()
            .map(|(start, end, kind)| {
                let kind = match kind.as_str() {
                    "cardinal" => HintKind::Cardinal,
                    "digits" => HintKind::Digits,
                    "date" => HintKind::Date,
                    "time" => HintKind::Time,
                    "ordinal" => HintKind::Ordinal,
                    "roman" => HintKind::Roman,
                    "range" => HintKind::Range,
                    "telephone" => HintKind::Telephone,
                    "electronic" => HintKind::Electronic,
                    "literal" => HintKind::Literal,
                    _ => return Err(pyo3::exceptions::PyValueError::new_err("invalid hint kind")),
                };
                Ok(Hint::new(SourceRange::new(start, end), kind))
            })
            .collect::<PyResult<Vec<_>>>()?;
        let deadline = match deadline_ms {
            Some(ms @ 1..=60_000) => Some(
                Instant::now()
                    .checked_add(Duration::from_millis(ms))
                    .ok_or_else(|| error(NormalizeError::Internal))?,
            ),
            Some(_) => return Err(pyo3::exceptions::PyValueError::new_err("invalid deadline")),
            None => None,
        };
        let control = WorkControl::new(deadline);
        let token = token.map(|t| t.clone());
        let id = token
            .as_ref()
            .map(|t| t.register(&control))
            .transpose()
            .map_err(error)?;
        let inner = self.inner.clone();
        let options = NormalizeOptions {
            ambiguity_policy: policy,
            hints,
        };
        let result = py.detach(move || inner.normalize_controlled(&text, &options, &control));
        if let (Some(token), Some(id)) = (token, id) {
            token.unregister(id).map_err(error)?;
        }
        let result = result.map_err(error)?;
        let segments = result
            .segments()
            .iter()
            .map(|s| {
                (
                    s.range().start(),
                    s.range().end(),
                    segment_kind(s.kind()),
                    s.text().to_owned(),
                    s.rule_id(),
                )
            })
            .collect();
        Ok((
            result.normalized_text().to_owned(),
            result.locale(),
            result.normalizer_id(),
            result.complete(),
            segments,
            issues(result.issues()),
        ))
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<NativeNormalizer>()?;
    module.add_class::<CancellationToken>()?;
    module.add(
        "NativeNormalizationError",
        module.py().get_type::<NativeNormalizationError>(),
    )?;
    module.add("NORMALIZER_ID", normalizer_tr::NORMALIZER_ID)?;
    module.add("BUILD_VERSION", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
