//! Executable library documentation, not a standalone CLI product.
use normalizer_tr::{
    AmbiguityPolicy, Issue, IssueCategory, NormalizeError, NormalizeOptions, NormalizeResult,
    Normalizer, SegmentKind, SourceRange,
};

fn print_issues(issues: &[Issue]) {
    if issues.is_empty() {
        println!("issues: []");
    } else {
        println!("issues:");
        for issue in issues {
            println!(
                "  {:?} [{},{}): {}",
                issue.category(),
                issue.range().start(),
                issue.range().end(),
                issue.explanation()
            );
        }
    }
}

fn print_result(title: &str, input: &str, result: &NormalizeResult) {
    println!("\n== {title} ==");
    println!("input: {input}");
    println!("normalized_text: {}", result.normalized_text());
    println!("complete: {}", result.complete());
    print_issues(result.issues());
}

fn main() -> Result<(), NormalizeError> {
    let normalizer = Normalizer::new()?;
    println!("normalizer: {}", normalizer.normalizer_id());
    let options = NormalizeOptions::default();
    for (title, input, expected) in [
        (
            "Complete TRY amount",
            "1.234,50 TL",
            "bin iki yüz otuz dört Türk lirası elli kuruş",
        ),
        ("Validated ordinal", "3'üncü", "üçüncü"),
        (
            "Validated date",
            "tarih 01.02.2026",
            "tarih bir Şubat iki bin yirmi altı",
        ),
    ] {
        let result = normalizer.normalize(input, &options)?;
        assert_eq!(result.normalized_text(), expected);
        assert!(result.complete());
        assert!(result.issues().is_empty());
        print_result(title, input, &result);
    }

    let partial_input = "25 TL; 1.234";
    let partial = normalizer.normalize(partial_input, &options)?;
    assert_eq!(partial.normalized_text(), "yirmi beş Türk lirası; 1.234");
    assert!(!partial.complete());
    assert_eq!(partial.issues().len(), 1);
    assert_eq!(partial.issues()[0].category(), IssueCategory::Ambiguous);
    assert_eq!(partial.issues()[0].range(), SourceRange::new(7, 12));
    print_result("Partial preservation", partial_input, &partial);

    let financial_input =
        include_str!("../tests/fixtures/financial-input.txt").trim_end_matches(['\r', '\n']);
    let financial_expected =
        include_str!("../tests/fixtures/financial-output.txt").trim_end_matches(['\r', '\n']);
    assert_eq!(financial_input.len(), 108);
    let financial = normalizer.normalize(financial_input, &options)?;
    assert_eq!(financial.normalized_text(), financial_expected);
    assert!(financial.complete());
    assert!(financial.issues().is_empty());
    let transformed: Vec<_> = financial
        .segments()
        .iter()
        .filter(|segment| segment.kind() != SegmentKind::Verbatim)
        .map(|segment| (segment.kind(), segment.range()))
        .collect();
    assert_eq!(
        transformed,
        [
            (SegmentKind::Date, SourceRange::new(0, 13)),
            (SegmentKind::Time, SourceRange::new(19, 27)),
            (SegmentKind::Percent, SourceRange::new(51, 60)),
            (SegmentKind::Abbreviation, SourceRange::new(70, 73)),
            (SegmentKind::Money, SourceRange::new(80, 95)),
        ]
    );
    print_result(
        "Financial golden (108 UTF-8 bytes)",
        financial_input,
        &financial,
    );
    println!("transformed_segments (original UTF-8 byte ranges):");
    for (kind, range) in transformed {
        println!("  {kind:?} [{},{})", range.start(), range.end());
    }

    let strict = normalizer.normalize(
        partial_input,
        &NormalizeOptions {
            ambiguity_policy: AmbiguityPolicy::Reject,
            ..Default::default()
        },
    );
    assert_eq!(
        strict,
        Err(NormalizeError::Unresolved(partial.issues().to_vec()))
    );
    println!("\n== Strict unresolved error ==");
    println!("input: {partial_input}");
    if let Err(error @ NormalizeError::Unresolved(issues)) = &strict {
        println!("error: Unresolved: {error}");
        println!("result: not returned (strict mode errored)");
        print_issues(issues);
    }
    Ok(())
}
