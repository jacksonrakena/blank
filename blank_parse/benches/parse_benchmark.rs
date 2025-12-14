use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn create_test_doc_string(size: i32) -> String {
    let mut doc = String::new();
    for i in 0..size {
        if i % 2 == 0 {
            doc.push_str(&format!("target{}_exp {{ name \"target{}_exp\" \n url \"https://example.com/target{}\"\n }}\n", i, i, i));
        } else {
            doc.push_str(&format!("target{}_name \"https://example.com/target{}\"\n", i, i));
        }
    }
    doc
}
fn parse_doc(c: &mut Criterion) {
    use kdl::{KdlDocument};
    use miette::NamedSource;
    use blank_parse::parse_doc;

    let mut group = c.benchmark_group("parse_doc");
    for rule_count in [1, 10, 100, 1000, 10_000, 100_000, 1_000_000].iter() {
        let doc_string = create_test_doc_string(*rule_count);
        let src = NamedSource::new("synthesized_source".to_string(), doc_string.clone());
        let doc: KdlDocument = doc_string.parse().expect("Could not parse KDL document");


        group.bench_with_input(BenchmarkId::from_parameter(rule_count), &doc, |b, doc| {
            b.iter(|| {
                black_box(parse_doc(src.clone(), doc.clone()).expect("Failed to parse targets"));
            })
        });
    }
}

criterion_group!(benches, parse_doc);
criterion_main!(benches);