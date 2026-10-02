use std::borrow::Cow;

use unicode_normalization::{UnicodeNormalization, is_nfc};
use unicode_segmentation::UnicodeSegmentation;

use crate::{Hint, NormalizeError, SourceRange, WorkControl};

enum Mapping {
    Identity(Vec<usize>),
    Normalized(Vec<(usize, usize)>),
}

pub(crate) struct SourceMap<'a> {
    recognition: Cow<'a, str>,
    mapping: Mapping,
}

impl<'a> SourceMap<'a> {
    pub(crate) fn new(input: &'a str, control: &WorkControl) -> Result<Self, NormalizeError> {
        control.check()?;
        if is_nfc(input) {
            let mut boundaries = Vec::with_capacity(input.len() / 2 + 1);
            boundaries.push(0);
            for (offset, grapheme) in input.grapheme_indices(true) {
                control.check()?;
                boundaries.push(offset + grapheme.len());
            }
            return Ok(Self {
                recognition: Cow::Borrowed(input),
                mapping: Mapping::Identity(boundaries),
            });
        }
        let mut recognition = String::with_capacity(input.len());
        let mut boundaries = vec![(0, 0)];
        for (offset, grapheme) in input.grapheme_indices(true) {
            control.check()?;
            recognition.extend(grapheme.nfc());
            boundaries.push((recognition.len(), offset + grapheme.len()));
        }
        Ok(Self {
            recognition: Cow::Owned(recognition),
            mapping: Mapping::Normalized(boundaries),
        })
    }
    pub(crate) fn text(&self) -> &str {
        &self.recognition
    }
    pub(crate) fn cover(&self, range: SourceRange) -> SourceRange {
        match &self.mapping {
            Mapping::Identity(boundaries) => {
                let start = boundaries.partition_point(|offset| *offset <= range.start) - 1;
                let end = boundaries.partition_point(|offset| *offset < range.end);
                SourceRange::new(boundaries[start], boundaries[end])
            }
            Mapping::Normalized(boundaries) => {
                let start = boundaries.partition_point(|pair| pair.0 <= range.start) - 1;
                let end = boundaries.partition_point(|pair| pair.0 < range.end);
                SourceRange::new(boundaries[start].0, boundaries[end].0)
            }
        }
    }
    pub(crate) fn original(&self, range: SourceRange) -> Result<SourceRange, NormalizeError> {
        Ok(SourceRange::new(
            self.lookup(range.start, false)?,
            self.lookup(range.end, false)?,
        ))
    }
    fn lookup(&self, offset: usize, original: bool) -> Result<usize, NormalizeError> {
        match &self.mapping {
            Mapping::Identity(boundaries) => boundaries.binary_search(&offset).map(|_| offset),
            Mapping::Normalized(boundaries) => boundaries
                .binary_search_by_key(&offset, |pair| if original { pair.1 } else { pair.0 })
                .map(|index| {
                    if original {
                        boundaries[index].0
                    } else {
                        boundaries[index].1
                    }
                }),
        }
        .map_err(|_| NormalizeError::InvalidHint)
    }
    pub(crate) fn hints(&self, hints: &[Hint]) -> Result<Vec<Hint>, NormalizeError> {
        let mut mapped = Vec::with_capacity(hints.len());
        for hint in hints {
            if hint.range.start >= hint.range.end {
                return Err(NormalizeError::InvalidHint);
            }
            mapped.push(Hint::new(
                SourceRange::new(
                    self.lookup(hint.range.start, true)?,
                    self.lookup(hint.range.end, true)?,
                ),
                hint.kind,
            ));
        }
        mapped.sort_by_key(|hint| hint.range.start);
        if mapped
            .windows(2)
            .any(|pair| pair[0].range.end > pair[1].range.start)
        {
            return Err(NormalizeError::InvalidHint);
        }
        Ok(mapped)
    }
}
