// Zero-Copy Deserialization Benchmark
// Target: 40-60% allocation reduction (Section 3.3.1)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct BenchEvent<'a> {
    #[serde(borrow)]
    request_id: &'a str,
    #[serde(borrow)]
    body: &'a str,
}

fn benchmark_zero_copy_json(c: &mut Criterion) {
    let small_json = r#"{"request_id":"test-123","body":"small"}"#;
    let medium_json =
        r#"{"request_id":"test-456","body":"This is a medium-sized payload with some data"}"#;

    let mut group = c.benchmark_group("zero_copy_deserialization");

    group.bench_with_input(BenchmarkId::new("json", "small"), &small_json, |b, json| {
        b.iter(|| {
            let _event: BenchEvent = serde_json::from_str(json).unwrap();
        });
    });

    group.bench_with_input(
        BenchmarkId::new("json", "medium"),
        &medium_json,
        |b, json| {
            b.iter(|| {
                let _event: BenchEvent = serde_json::from_str(json).unwrap();
            });
        },
    );

    group.finish();
}

criterion_group!(benches, benchmark_zero_copy_json);
criterion_main!(benches);
