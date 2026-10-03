"""Run against an installed wheel; no model or audio dependencies."""

from normalizer_tr import Normalizer, NormalizationError, Hint

n = Normalizer()
for text in (
    "25 TL",
    "3'üncü",
    "25 TL; 1.234",
    "1.'nin",
    "10-15 kişi",
    "0850 222 33 44",
    "info@ornek.com",
):
    result = n.normalize(text)
    print(
        {
            "input": text,
            "normalized_text": result.normalized_text,
            "complete": result.complete,
            "issues": result.issues,
        }
    )
try:
    n.normalize("1.234", ambiguity_policy="reject")
except NormalizationError as error:
    print({"code": error.code, "issues": error.issues})
forced = n.normalize("25 TL; 1.234", ambiguity_policy="forced")
print(
    {
        "forced": forced.normalized_text,
        "complete": forced.complete,
        "issues": forced.issues,
    }
)
assert n.normalize("IV", hints=(Hint(0, 2, "roman"),)).normalized_text == "dört"
print({"normalizer_id": n.normalizer_id})
