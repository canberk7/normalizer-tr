//! The lexicon's abbreviations, read the same way under every policy.
use normalizer_tr::{AmbiguityPolicy, NormalizeOptions, NormalizeResult, Normalizer, SegmentKind};

fn normalize(input: &str, policy: AmbiguityPolicy) -> NormalizeResult {
    Normalizer::new()
        .unwrap()
        .normalize(
            input,
            &NormalizeOptions {
                ambiguity_policy: policy,
                ..Default::default()
            },
        )
        .unwrap()
}

#[test]
fn an_abbreviation_is_said_as_the_lexicon_reads_it_in_every_policy() {
    for (input, spoken) in [
        (
            "Dr. Ayşe ve Prof. Dr. Mehmet",
            "doktor Ayşe ve profesör doktor Mehmet",
        ),
        (
            "Doç. Dr. Ali ve Av. Ayşe",
            "doçent doktor Ali ve avukat Ayşe",
        ),
        ("ABD'de ve AB'nin", "a be dede ve a benin"),
        ("THY ile TRT'ye", "te he ye ile te re teye"),
        (
            "Cumhuriyet Cad. No: 12 D: 5",
            "Cumhuriyet caddesi numara on iki daire beş",
        ),
        // After a noun `No:` ends a compound with it; after a street or alone it does not.
        (
            "Sipariş No: 12, takip no: 7, Kapı No. 3",
            "Sipariş numarası on iki, takip numarası yedi, Kapı numarası üç",
        ),
        (
            "İstiklal Caddesi No: 45 ve No: 7",
            "İstiklal Caddesi numara kırk beş ve numara yedi",
        ),
        ("pH ve min. tutar", "pe he ve minimum tutar"),
        // Chat abbreviations are said as the words they stand for.
        (
            "slm nbr, yrn grşz tmm",
            "selam naber, yarın görüşürüz tamam",
        ),
        // Titles written together, letters of English time and units of measure.
        (
            "Prof.Dr. Ali ve Doç.Dr. Can, Arş.Gör. Ece",
            "profesör doktor Ali ve doçent doktor Can, araştırma görevlisi Ece",
        ),
        (
            "Saat 9 AM ile 5 PM arası",
            "Saat dokuz ey em ile beş pi em arası",
        ),
        (
            "1000 IU, 300 dpi, 60 fps",
            "bin ünite, üç yüz de pe i, altmış fe pe se",
        ),
        ("SARS-CoV-2 ne zmn biter", "sars kov iki ne zaman biter"),
        (
            "Atatürk Bul. Gül Sok. Yıldız Apt.",
            "Atatürk bulvarı Gül sokağı Yıldız apartmanı",
        ),
        (
            "Kaya Ltd. Şti. ve Demir A.Ş.",
            "Kaya limited şirketi ve Demir anonim şirketi",
        ),
        ("T.C. Kimlik", "Türkiye Cumhuriyeti Kimlik"),
        ("s. 12'ye bakın", "sayfa on ikiye bakın"),
        (
            "Hz. Muhammed (s.a.v.)",
            "hazreti Muhammed (sallallahu aleyhi ve sellem)",
        ),
        ("NATO, ODTÜ ve TÜBİTAK", "nato, odtü ve tübitak"),
        ("BBC, CNN ve BMW", "bi bi si, si en en ve be em ve"),
        ("Ar-Ge ve KVKK", "araştırma geliştirme ve ke ve ke ke"),
        (
            "KDV dahil, vb. şeyler",
            "katma değer vergisi dahil, ve benzeri şeyler",
        ),
    ] {
        for policy in [AmbiguityPolicy::Preserve, AmbiguityPolicy::Forced] {
            let result = normalize(input, policy);
            assert_eq!(result.normalized_text(), spoken, "{input} {policy:?}");
            assert!(result.complete(), "{input} {policy:?}");
        }
        let result = normalize(input, AmbiguityPolicy::Preserve);
        assert!(
            result
                .segments()
                .iter()
                .any(|segment| segment.kind() == SegmentKind::Abbreviation),
            "{input}"
        );
    }
}

#[test]
fn a_dotted_abbreviation_reads_in_capitals_unless_it_is_a_common_word() {
    let result = normalize("DR. AHMET ve SN. BAKAN", AmbiguityPolicy::Preserve);
    assert_eq!(result.normalized_text(), "doktor AHMET ve sayın BAKAN");
    // The names are still unknown capitals: Forced says them as words.
    assert_eq!(
        normalize("DR. AHMET ve SN. BAKAN", AmbiguityPolicy::Forced).normalized_text(),
        "doktor ahmet ve sayın bakan"
    );
    // A sentence that ends in a common word keeps its period, in any case.
    for (input, preserved) in [
        ("Fişi prize sok.", "Fişi prize sok."),
        ("Onu bul.", "Onu bul."),
        ("Bu bir no.", "Bu bir numara"),
    ] {
        assert_eq!(
            normalize(input, AmbiguityPolicy::Preserve).normalized_text(),
            preserved,
            "{input}"
        );
    }
    assert_eq!(
        normalize("CEVABI BUL.", AmbiguityPolicy::Forced).normalized_text(),
        "cevabı bul."
    );
}

#[test]
fn a_suffix_preserve_cannot_check_is_still_said_by_forced() {
    // Only a case ending is checked after an abbreviation; a derivation stays an issue.
    let input = "ABD'li yetkililer";
    let preserved = normalize(input, AmbiguityPolicy::Preserve);
    assert_eq!(preserved.normalized_text(), input);
    assert!(!preserved.complete());
    let forced = normalize(input, AmbiguityPolicy::Forced);
    assert_eq!(forced.normalized_text(), "a be deli yetkililer");
    assert_eq!(forced.issues(), preserved.issues());
}
