use std::sync::Arc;

use crate::{
    NORMALIZER_ID, NormalizeError, NormalizeOptions, NormalizeResult, WorkControl, pipeline,
    resources::Resources,
};

/// Immutable, thread-safe normalizer. Clones share validated built-in resources.
#[derive(Clone)]
pub struct Normalizer {
    resources: Arc<Resources>,
}

impl Normalizer {
    /// Diagnostic identity of the one built-in normalizer.
    pub fn normalizer_id(&self) -> &'static str {
        NORMALIZER_ID
    }
    /// Create the built-in normalizer, validating its immutable resources.
    pub fn new() -> Result<Self, NormalizeError> {
        Ok(Self {
            resources: Resources::shared()?,
        })
    }
    /// Normalize synchronously with default, uncancelled work control.
    pub fn normalize(
        &self,
        input: &str,
        options: &NormalizeOptions,
    ) -> Result<NormalizeResult, NormalizeError> {
        self.normalize_controlled(input, options, &WorkControl::default())
    }
    /// Normalize with cooperative cancellation and an optional deadline.
    pub fn normalize_controlled(
        &self,
        input: &str,
        options: &NormalizeOptions,
        control: &WorkControl,
    ) -> Result<NormalizeResult, NormalizeError> {
        pipeline::run(input, options, control, &self.resources)
    }
}
