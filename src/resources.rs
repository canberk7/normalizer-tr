use crate::NormalizeError;
use regex::Regex;
use std::sync::{Arc, OnceLock};

/// Diagnostic identity of this built-in normalizer, not a selectable behavior profile.
pub const NORMALIZER_ID: &str = concat!("normalizer-tr/", env!("CARGO_PKG_VERSION"));

/// Immutable compiled patterns. All lexical/phonological data is typed in domain::lexicon.
pub(crate) struct Resources {
    pub(crate) tokens: Regex,
    pub(crate) phone_like: Regex,
    pub(crate) quoted_email: Regex,
}

impl Resources {
    pub(crate) fn shared() -> Result<Arc<Self>, NormalizeError> {
        static INSTANCE: OnceLock<Result<Arc<Resources>, NormalizeError>> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load().map(Arc::new)).clone()
    }
    pub(crate) fn load() -> Result<Self, NormalizeError> {
        let compile =
            |pattern| Regex::new(pattern).map_err(|_| NormalizeError::InvalidConfiguration);
        Ok(Self {
            tokens: compile(r"\S+")?,
            phone_like: compile(r"(?:\+?[0-9]{1,3})(?:[ \t]+(?:\([0-9]{2,}\)|[0-9]{2,})){2,}")?,
            quoted_email: compile(r#""[^"\r\n]{0,128}"@[^\s]+"#)?,
        })
    }
}
