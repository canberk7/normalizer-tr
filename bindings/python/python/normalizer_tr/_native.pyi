from typing import Sequence

NORMALIZER_ID: str
BUILD_VERSION: str

class NativeNormalizationError(Exception): ...

class CancellationToken:
    def __init__(self) -> None: ...
    def cancel(self) -> None: ...

class NativeNormalizer:
    @property
    def normalizer_id(self) -> str: ...
    def __init__(self) -> None: ...
    def normalize(
        self,
        text: str,
        policy: str,
        hints: Sequence[tuple[int, int, str]],
        token: CancellationToken | None = None,
        deadline_ms: int | None = None,
    ) -> tuple[
        str,
        str,
        str,
        bool,
        list[tuple[int, int, str, str, str]],
        list[tuple[int, int, str, str]],
    ]: ...
