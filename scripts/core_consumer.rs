//! Standalone packaged-core consumer. No Python or workspace source path.
#[cfg(test)]
mod tests {
    use normalizer_tr::{Hint, HintKind, NORMALIZER_ID, NormalizeOptions, Normalizer, SourceRange};
    #[test]
    fn actual_readings_and_identity() {
        let n = Normalizer::new().unwrap();
        assert_eq!(n.normalizer_id(), NORMALIZER_ID);
        for (input, expected) in [
            ("25 TL'den", "yirmi beş Türk lirasından"),
            ("1.'nin", "birincinin"),
            ("10-15 kişi", "on ila on beş kişi"),
            ("info@ornek.com", "info et ornek nokta kom"),
            ("tren trafik", "tren trafik"),
        ] {
            let r = n.normalize(input, &NormalizeOptions::default()).unwrap();
            assert!(r.complete());
            assert_eq!(r.normalized_text(), expected);
        }
    }
    #[test]
    fn hint_and_partial_are_current_api() {
        let n = Normalizer::new().unwrap();
        let r = n
            .normalize(
                "IV",
                &NormalizeOptions {
                    hints: vec![Hint::new(SourceRange::new(0, 2), HintKind::Roman)],
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(r.normalized_text(), "dört");
        let r = n
            .normalize("AB12 25 TL", &NormalizeOptions::default())
            .unwrap();
        assert!(!r.complete());
        assert_eq!(r.normalized_text(), "AB12 yirmi beş Türk lirası");
        assert_eq!(r.issues()[0].range(), SourceRange::new(0, 4));
    }
    #[test]
    fn financial_fixture_full_result() {
        let n = Normalizer::new().unwrap();
        let input = "14.03.2026'da saat 09:30'da imzalanan sözleşmede %37,5'lik sapma ve KDV dahil 1.847.293,50 TL fark vardı.";
        let expected = "on dört Mart iki bin yirmi altıda saat dokuz otuzda imzalanan sözleşmede yüzde otuz yedi virgül beşlik sapma ve katma değer vergisi dahil bir milyon sekiz yüz kırk yedi bin iki yüz doksan üç Türk lirası elli kuruş fark vardı.";
        assert_eq!(input.len(), 108);
        let r = n.normalize(input, &NormalizeOptions::default()).unwrap();
        assert!(r.complete());
        assert!(r.issues().is_empty());
        assert_eq!(r.normalized_text(), expected);
        assert_eq!(
            r.segments()
                .iter()
                .filter(|s| s.kind() != normalizer_tr::SegmentKind::Verbatim)
                .map(|s| (s.range().start(), s.range().end()))
                .collect::<Vec<_>>(),
            [(0, 13), (19, 27), (51, 60), (70, 73), (80, 95)]
        );
    }
}
