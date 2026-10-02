"""Immutable typed Python facade; normalization executes the actual Rust core."""

from dataclasses import dataclass
from typing import Literal, Sequence

from ._native import (
    NORMALIZER_ID,
    BUILD_VERSION,
    CancellationToken,
    NativeNormalizationError,
    NativeNormalizer,
)

__all__ = [
    "Normalizer",
    "NormalizationError",
    "Hint",
    "Issue",
    "Segment",
    "NormalizeResult",
    "CancellationToken",
    "NORMALIZER_ID",
    "BUILD_VERSION",
]

HintKind = Literal[
    "cardinal",
    "digits",
    "date",
    "time",
    "ordinal",
    "roman",
    "range",
    "telephone",
    "electronic",
]


def _integer(value: int, name: str, maximum: int = (1 << 63) - 1) -> None:
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{name} must be an integer, not bool")
    if not 0 <= value <= maximum:
        raise ValueError(f"{name} is outside the supported range")


@dataclass(frozen=True, slots=True)
class Hint:
    start_byte: int
    end_byte: int
    kind: HintKind

    def __post_init__(self) -> None:
        _integer(self.start_byte, "start_byte")
        _integer(self.end_byte, "end_byte")
        if self.kind not in (
            "cardinal",
            "digits",
            "date",
            "time",
            "ordinal",
            "roman",
            "range",
            "telephone",
            "electronic",
        ):
            raise ValueError("unsupported hint kind")


@dataclass(frozen=True, slots=True)
class Issue:
    start_byte: int
    end_byte: int
    category: str
    explanation: str


@dataclass(frozen=True, slots=True)
class Segment:
    start_byte: int
    end_byte: int
    kind: str
    text: str
    rule_id: str


@dataclass(frozen=True, slots=True)
class NormalizeResult:
    normalized_text: str
    locale: str
    normalizer_id: str
    complete: bool
    segments: tuple[Segment, ...]
    issues: tuple[Issue, ...]


class NormalizationError(Exception):
    """Mapped Rust failure: strict errors contain issues, never a partial result."""

    def __init__(
        self,
        code: str,
        message: str,
        issues: tuple[Issue, ...] = (),
        limit_kind: str | None = None,
    ) -> None:
        super().__init__(message)
        self._code = code
        self._issues = issues
        self._limit_kind = limit_kind

    @property
    def code(self) -> str:
        return self._code

    @property
    def issues(self) -> tuple[Issue, ...]:
        return self._issues

    @property
    def limit_kind(self) -> str | None:
        return self._limit_kind


def _error(exc: NativeNormalizationError) -> NormalizationError:
    code, message, limit, records = exc.args
    return NormalizationError(code, message, tuple(Issue(*r) for r in records), limit)


class Normalizer:
    __slots__ = ("_inner",)

    @property
    def normalizer_id(self) -> str:
        return self._inner.normalizer_id

    def __init__(self) -> None:
        try:
            self._inner = NativeNormalizer()
        except NativeNormalizationError as exc:
            raise _error(exc) from None

    def normalize(
        self,
        text: str,
        *,
        ambiguity_policy: str = "preserve",
        hints: Sequence[Hint] = (),
        cancellation: CancellationToken | None = None,
        deadline_ms: int | None = None,
    ) -> NormalizeResult:
        if not isinstance(text, str):
            raise TypeError("text must be str")
        try:
            text.encode("utf-8")
        except UnicodeEncodeError:
            raise NormalizationError(
                "invalid_input", "input contains invalid Unicode"
            ) from None
        if ambiguity_policy not in ("preserve", "reject"):
            raise ValueError("ambiguity_policy must be preserve or reject")
        if not isinstance(hints, Sequence):
            raise TypeError("hints must be a sequence of Hint")
        if any(not isinstance(hint, Hint) for hint in hints):
            raise TypeError("each hint must be Hint")
        if cancellation is not None and not isinstance(cancellation, CancellationToken):
            raise TypeError("cancellation must be CancellationToken or None")
        if deadline_ms is not None:
            _integer(deadline_ms, "deadline_ms", 60_000)
            if deadline_ms == 0:
                raise ValueError("deadline_ms must be between 1 and 60000")
        try:
            text, locale, identity, complete, segments, issues = self._inner.normalize(
                text,
                ambiguity_policy,
                [(h.start_byte, h.end_byte, h.kind) for h in hints],
                cancellation,
                deadline_ms,
            )
        except NativeNormalizationError as exc:
            raise _error(exc) from None
        return NormalizeResult(
            text,
            locale,
            identity,
            complete,
            tuple(Segment(*s) for s in segments),
            tuple(Issue(*i) for i in issues),
        )
