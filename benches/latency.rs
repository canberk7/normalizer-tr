//! Dev-only checked per-call latency sampler; never a product CLI.
use std::{collections::BTreeMap, hint::black_box, time::Instant};

use normalizer_tr::{AmbiguityPolicy, Hint, HintKind, NormalizeOptions, Normalizer, SourceRange};
use serde_json::{Value, json};

const POLICIES: [(AmbiguityPolicy, &str); 3] = [
    (AmbiguityPolicy::Preserve, "preserve"),
    (AmbiguityPolicy::Reject, "reject"),
    (AmbiguityPolicy::Forced, "forced"),
];

fn options(case: &Value, policy: AmbiguityPolicy) -> NormalizeOptions {
    let mut options = NormalizeOptions {
        ambiguity_policy: policy,
        ..Default::default()
    };
    if let Some(hint) = case.get("hint") {
        let kind = match hint["kind"].as_str().unwrap() {
            "cardinal" => HintKind::Cardinal,
            "digits" => HintKind::Digits,
            "date" => HintKind::Date,
            "time" => HintKind::Time,
            "ordinal" => HintKind::Ordinal,
            "roman" => HintKind::Roman,
            "range" => HintKind::Range,
            "telephone" => HintKind::Telephone,
            "electronic" => HintKind::Electronic,
            _ => panic!("frozen corpus hint is unsupported"),
        };
        options.hints.push(Hint::new(
            SourceRange::new(
                hint["start"].as_u64().unwrap() as usize,
                hint["end"].as_u64().unwrap() as usize,
            ),
            kind,
        ));
    }
    options
}

fn outcome(result: Result<normalizer_tr::NormalizeResult, normalizer_tr::NormalizeError>) -> Value {
    match result {
        Ok(result) => json!({"result": result}),
        Err(normalizer_tr::NormalizeError::Unresolved(issues)) => {
            json!({"error":"unresolved","issues":issues})
        }
        Err(error) => json!({"error":format!("{error:?}")}),
    }
}

fn quantiles(values: &mut [u64]) -> Value {
    values.sort_unstable();
    let at = |percent: usize| values[(values.len() * percent).div_ceil(100).saturating_sub(1)];
    json!({"count":values.len(),"p50_ns":at(50),"p95_ns":at(95),"p99_ns":at(99),"max_ns":values[values.len()-1]})
}

fn main() {
    // Cargo supplies --bench even when the target uses its own timing harness.
    let args: Vec<_> = std::env::args().filter(|arg| arg != "--bench").collect();
    let output = match args.as_slice() {
        [_, flag, output] if flag == "--output" => output,
        _ => panic!("usage: latency --output <new-report.json>"),
    };
    let mut corpus: Vec<Value> = serde_json::from_str(include_str!("corpus.json")).unwrap();
    corpus.extend(serde_json::from_str::<Vec<Value>>(include_str!("intent-corpus.json")).unwrap());
    let first_start = Instant::now();
    drop(black_box(Normalizer::new().unwrap()));
    let first_init_ns = first_start.elapsed().as_nanos() as u64;
    let mut constructors = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        black_box(Normalizer::new().unwrap());
        constructors.push(start.elapsed().as_nanos() as u64);
    }
    let normalizer = Normalizer::new().unwrap();
    let mut snapshots = BTreeMap::new();
    let mut cohorts = BTreeMap::new();
    let mut classes = BTreeMap::<String, Vec<u64>>::new();
    let mut distributions = Vec::new();
    for cohort in ["short", "medium"] {
        let selected: Vec<_> = corpus
            .iter()
            .filter(|case| case["cohort"] == cohort)
            .collect();
        let prepared: Vec<_> = selected
            .iter()
            .flat_map(|case| {
                POLICIES.map(|(policy, name)| {
                    let text = case["text"].as_str().unwrap();
                    assert!(if cohort == "short" {
                        !text.is_empty() && text.len() <= 256
                    } else {
                        (257..=1024).contains(&text.len())
                    });
                    let options = options(case, policy);
                    let expected = outcome(normalizer.normalize(text, &options));
                    let key = format!("{}:{name}", case["id"].as_str().unwrap());
                    snapshots.insert(key.clone(), expected.clone());
                    distributions.push(
                        json!({"id":key,"bytes":text.len(),"cohort":cohort,"class":case["class"]}),
                    );
                    (
                        text,
                        options,
                        expected,
                        case["class"].as_str().unwrap(),
                        name,
                    )
                })
            })
            .collect();
        for index in 0..2000 {
            let (text, options, _, _, _) = &prepared[index % prepared.len()];
            drop(black_box(
                normalizer.normalize(black_box(text), black_box(options)),
            ));
        }
        let mut samples = Vec::with_capacity(10000);
        for index in 0..10000 {
            let (text, options, expected, class, policy) = &prepared[index % prepared.len()];
            if index < prepared.len() {
                assert_eq!(outcome(normalizer.normalize(text, options)), *expected);
            }
            let start = Instant::now();
            drop(black_box(
                normalizer.normalize(black_box(text), black_box(options)),
            ));
            let elapsed = start.elapsed().as_nanos() as u64;
            samples.push(elapsed);
            classes
                .entry(format!("{cohort}:{class}:{policy}"))
                .or_default()
                .push(elapsed);
        }
        for (text, options, expected, _, _) in &prepared {
            assert_eq!(outcome(normalizer.normalize(text, options)), *expected);
        }
        cohorts.insert(cohort, quantiles(&mut samples));
    }
    let per_class: BTreeMap<_, _> = classes
        .into_iter()
        .map(|(key, mut samples)| (key, quantiles(&mut samples)))
        .collect();
    let mut clock = (0..10000)
        .map(|_| {
            let start = Instant::now();
            start.elapsed().as_nanos() as u64
        })
        .collect::<Vec<_>>();
    let mut large = Vec::new();
    for size in [4096, 16384, 32768] {
        for (class, pattern) in [
            ("verbatim", "sözcük "),
            ("candidates", "1 "),
            ("amplification", "999999999999999999 "),
        ] {
            let mut text = pattern.repeat(size / pattern.len());
            text.push_str(&" ".repeat(size - text.len()));
            let mut times = Vec::new();
            let expected = outcome(normalizer.normalize(&text, &NormalizeOptions::default()));
            for _ in 0..100 {
                let start = Instant::now();
                drop(black_box(
                    normalizer.normalize(black_box(&text), &NormalizeOptions::default()),
                ));
                times.push(start.elapsed().as_nanos() as u64);
            }
            large.push(json!({"bytes":size,"class":class,"timing":quantiles(&mut times),"outcome":expected}));
        }
    }
    let mut controls = Vec::new();
    for (name, text, options) in [
        (
            "maximum-hints",
            "1;".repeat(256),
            NormalizeOptions {
                hints: (0..256)
                    .map(|i| Hint::new(SourceRange::new(i * 2, i * 2 + 1), HintKind::Cardinal))
                    .collect(),
                ..Default::default()
            },
        ),
        (
            "candidate-limit",
            "1 ".repeat(4097),
            NormalizeOptions::default(),
        ),
        (
            "long-identifier",
            format!("https://ornek.com/{}", "a".repeat(16000)),
            NormalizeOptions::default(),
        ),
        (
            "invalid-compounds",
            "TR33 0006 1005 1978 6457 8413 2A; ".repeat(400),
            NormalizeOptions::default(),
        ),
    ] {
        let expected = outcome(normalizer.normalize(&text, &options));
        let mut times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            drop(black_box(
                normalizer.normalize(black_box(&text), black_box(&options)),
            ));
            times.push(start.elapsed().as_nanos() as u64);
        }
        assert_eq!(expected, outcome(normalizer.normalize(&text, &options)));
        controls.push(json!({"name":name,"bytes":text.len(),"hints":options.hints.len(),"timing":quantiles(&mut times),
                    "outcome_complete":expected.get("result").map(|r|r["complete"].clone()),"error":expected.get("error")}));
    }
    let report = json!({
        "schema_version":1,"normalizer_id":normalizer.normalizer_id(),"package_version":env!("CARGO_PKG_VERSION"),
        "method":{"samples_per_cohort":10000,"warmup_per_cohort":2000,
                  "timing":"Instant per call; validation mapping recognition rendering owned result disposal included",
                  "order":"deterministic corpus order alternating preserve/reject/forced","outliers":"all retained; no overhead subtraction"},
        "constructor_first_process_init_ns":first_init_ns,
        "constructors":quantiles(&mut constructors),
        "constructor_method":"first process init separately; distribution is subsequent same-process constructor calls, not cold starts",
        "cohorts":cohorts,"per_class_policy":per_class,
        "distribution":distributions,"clock_overhead":quantiles(&mut clock),"large":large,"limit_diagnostics":controls,
        "snapshots":snapshots,
    });
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
        .unwrap();
    println!("Saved checked per-call measurements to {output}");
}
