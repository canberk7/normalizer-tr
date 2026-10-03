import dataclasses

import pytest
from normalizer_tr import (
    NORMALIZER_ID,
    Hint,
    Normalizer,
)


def test_single_identity_current_coverage_and_frozen_records():
    n = Normalizer()
    assert n.normalizer_id == NORMALIZER_ID == "normalizer-tr/0.4.0"
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
        ("2.5.1", "literal", "iki nokta beş nokta bir"),
    ],
)
def test_all_new_intent_literals_and_original_ranges(text, kind, expected):
    source = "ö " + text
    hint = Hint(3, len(source.encode("utf-8")), kind)
    result = Normalizer().normalize(source, hints=(hint,))
    assert result.complete and result.normalized_text == "ö " + expected
    assert result.segments[1].start_byte == 3
    assert result.segments[1].end_byte == hint.end_byte


def test_forced_policy_reads_unresolved_spans_and_keeps_their_issues():
    n = Normalizer()
    text = "25 TL; 1.234; IV; 10-15"
    partial = n.normalize(text)
    forced = n.normalize(text, ambiguity_policy="forced")
    assert (
        forced.normalized_text
        == "yirmi beş lira; bin iki yüz otuz dört; dört; on tire on beş"
    )
    assert not forced.complete
    assert forced.issues == partial.issues
    assert [
        (s.kind, s.rule_id, s.start_byte, s.end_byte)
        for s in forced.segments
        if s.kind != "verbatim"
    ] == [
        ("money", "quantity", 0, 5),
        ("cardinal", "forced.cardinal", 7, 12),
        ("roman", "forced.roman", 14, 16),
        ("range", "forced.range", 18, 23),
    ]
    for source, spoken, kind, rule_id in [
        ("3 + 4 = 7", "üç artı dört eşittir yedi", "literal", "forced.literal"),
        ("09:30'de", "dokuz otuzda", "time", "forced.time"),
        ("0532", "sıfır beş üç iki", "digits", "forced.digits"),
        ("H2O", "He iki O", "literal", "forced.literal"),
        ("5 TL'lik", "beş liralık", "money", "forced.money"),
        ("29.02.1900", "yirmi dokuz Şubat bin dokuz yüz", "date", "forced.date"),
        ("1. Dünya Savaşı", "birinci Dünya Savaşı", "ordinal", "forced.ordinal"),
        ("TK1956", "Te Ke bin dokuz yüz elli altı", "literal", "forced.literal"),
        ("KDV'li", "katma değer vergili", "literal", "forced.literal"),
        ("24/7", "yirmi dört slaş yedi", "literal", "forced.literal"),
        ("ABD'li", "a be deli", "literal", "forced.literal"),
    ]:
        result = n.normalize(source, ambiguity_policy="forced")
        assert result.normalized_text == spoken and not result.complete
        assert (result.segments[0].kind, result.segments[0].rule_id) == (kind, rule_id)
    # The abbreviation lexicon reads the same under every policy.
    address = "Cumhuriyet Cad. No: 12"
    for policy in ("preserve", "forced"):
        result = n.normalize(address, ambiguity_policy=policy)
        assert result.normalized_text == "Cumhuriyet caddesi numara on iki"
        assert result.complete
    unchanged = n.normalize("5 kg; 3'üncü")
    assert n.normalize("5 kg; 3'üncü", ambiguity_policy="forced") == unchanged
    # Forced reads text with bidirectional controls as if they were not there.
    marked = n.normalize("Fiyat‏ 5‏ TL", ambiguity_policy="forced")
    assert marked.normalized_text == "Fiyat beş lira"
    assert (
        marked.segments[1].text == "" and marked.segments[1].rule_id == "forced.spoken"
    )


def test_forced_policy_speaks_in_the_spoken_style_without_new_issues():
    n = Normalizer()
    for source, exact, spoken in [
        ("4,25", "dört virgül iki beş", "dört virgül yirmi beş"),
        ("25 TL'den", "yirmi beş Türk lirasından", "yirmi beş liradan"),
        (
            "Fatura 25 USD + KDV",
            "Fatura yirmi beş dolar + katma değer vergisi",
            "Fatura yirmi beş dolar artı katma değer vergisi",
        ),
        ("Plan B", "Plan B", "Plan Be"),
    ]:
        assert n.normalize(source).normalized_text == exact
        result = n.normalize(source, ambiguity_policy="forced")
        assert result.normalized_text == spoken
        assert result.complete and result.issues == ()
    stray = n.normalize("Plan B", ambiguity_policy="forced").segments[-1]
    assert (stray.kind, stray.rule_id, stray.text) == ("literal", "forced.spoken", "Be")


def test_literal_hint_and_policy_validation():
    n = Normalizer()
    hinted = n.normalize(
        "3 + 4 = 7", hints=(Hint(0, 9, "literal"),), ambiguity_policy="forced"
    )
    assert hinted.complete and hinted.issues == ()
    assert hinted.normalized_text == "üç artı dört eşittir yedi"
    assert (hinted.segments[0].kind, hinted.segments[0].rule_id) == (
        "literal",
        "literal.hint",
    )
    for policy in ("force", "Forced", "", None):
        with pytest.raises(ValueError):
            n.normalize("1", ambiguity_policy=policy)
    with pytest.raises(ValueError):
        Hint(0, 1, "Literal")
