//! Checked reusable shared-profile IO; no model, network or user-data ingestion.
use normalizer_tr::{Hint, HintKind, NormalizeOptions, Normalizer, SourceRange};

fn main() -> Result<(), normalizer_tr::NormalizeError> {
    let n = Normalizer::new()?;
    println!("normalizer: {}", n.normalizer_id());
    for (input, expected) in [
        ("%3,25'ten", "yüzde üç virgül iki beşten"),
        ("25 TL'den", "yirmi beş Türk lirasından"),
        ("1.'nin", "birincinin"),
        ("5 kg'dan", "beş kilogramdan"),
        ("2 mg", "iki miligram"),
        ("3 m²", "üç metrekare"),
        ("90 km/sa", "saatte doksan kilometre"),
        ("10-15 kişi", "on ila on beş kişi"),
        ("€14,05", "on dört avro beş sent"),
        (
            "0850 222 33 44",
            "sıfır sekiz yüz elli iki yüz yirmi iki otuz üç kırk dört",
        ),
        (
            "TR330006100519786457841326",
            "te re üç üç, sıfır sıfır sıfır altı, bir sıfır sıfır beş, bir dokuz yedi sekiz, altı dört beş yedi, sekiz dört bir üç, iki altı",
        ),
        ("II. Dünya Savaşı", "ikinci Dünya Savaşı"),
        ("info@ornek.com", "info et ornek nokta kom"),
        (
            "https://ornek.com/a01",
            "ha te te pe es iki nokta eğik çizgi eğik çizgi ornek nokta kom eğik çizgi a sıfır bir",
        ),
        ("#yapayzeka & PTT", "hashtag yapayzeka ve pe te te"),
    ] {
        let r = n.normalize(input, &NormalizeOptions::default())?;
        assert!(r.complete());
        assert!(r.issues().is_empty());
        assert_eq!(r.normalized_text(), expected);
        println!(
            "{input} -> {} (complete={})",
            r.normalized_text(),
            r.complete()
        );
        for segment in r.segments() {
            println!(
                "  {:?} [{},{})",
                segment.kind(),
                segment.range().start(),
                segment.range().end()
            );
        }
    }
    let input = "II. Abdülhamit";
    let r = n.normalize(
        input,
        &NormalizeOptions {
            hints: vec![Hint::new(SourceRange::new(0, 3), HintKind::Roman)],
            ..Default::default()
        },
    )?;
    assert_eq!(r.normalized_text(), "ikinci Abdülhamit");
    assert!(r.complete());
    println!("hinted: {input} -> {}", r.normalized_text());
    Ok(())
}
