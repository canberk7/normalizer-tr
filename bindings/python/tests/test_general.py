import dataclasses

import pytest
from normalizer_tr import (
    NORMALIZER_ID,
    Hint,
    Normalizer,
)


def test_single_identity_current_coverage_and_frozen_records():
    n = Normalizer()
    assert n.normalizer_id == NORMALIZER_ID == "normalizer-tr/0.3.0"
    for text, output, kind in [
        ("1.'nin", "birincinin", "ordinal"),
        ("25 TL'den", "yirmi beş Türk lirasından", "money"),
        ("5 kg'dan", "beş kilogramdan", "unit"),
        ("10-15 kişi", "on ila on beş kişi", "range"),
        (
            "0850 222 33 44",
            "sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört",
            "telephone",
        ),
        (
            "TR330006100519786457841326",
            "te re üç üç, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı",
            "iban",
        ),
        ("info@ornek.com", "info et ornek nokta kom", "electronic"),
        ("&", "ve", "symbol"),
    ]:
        result = n.normalize(text)
        assert result.complete and result.normalized_text == output
        assert result.normalizer_id == n.normalizer_id
        assert result.segments[0].kind == kind
        with pytest.raises(dataclasses.FrozenInstanceError):
            result.segments[0].kind = "changed"


@pytest.mark.parametrize(
    "text,kind,expected",
    [
        ("25.", "ordinal", "yirmi beşinci"),
        ("IV", "roman", "dört"),
        ("10-15", "range", "on ila on beş"),
        (
            "5321234567",
            "telephone",
            "beş yüz otuz iki yüz yirmi üç kırk beş altmış yedi",
        ),
        ("ornek.com", "electronic", "ornek nokta kom"),
    ],
)
def test_all_new_intent_literals_and_original_ranges(text, kind, expected):
    source = "ö " + text
    hint = Hint(3, len(source.encode("utf-8")), kind)
    result = Normalizer().normalize(source, hints=(hint,))
    assert result.complete and result.normalized_text == "ö " + expected
    assert result.segments[1].start_byte == 3
    assert result.segments[1].end_byte == hint.end_byte
