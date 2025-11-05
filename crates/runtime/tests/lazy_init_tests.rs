// Extreme TDD: Lazy Initialization Tests
// Written FIRST before implementation
// Target: <1ms initialization time via lazy HTTP client creation

use ruchy_lambda_runtime::Runtime;
use std::env;
use std::time::Instant;

/// Test: Runtime initialization should be <1ms with lazy HTTP client
#[test]
fn test_lazy_initialization_under_1ms() {
    env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");

    let start = Instant::now();

    // Initialize Runtime - should NOT create HTTP client yet
    let _runtime = Runtime::new().expect("Runtime initialization failed");

    let duration = start.elapsed();

    assert!(
        duration.as_micros() < 1000, // <1ms = <1000μs
        "Lazy initialization took {}μs, should be <1000μs (1ms)",
        duration.as_micros()
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Runtime can be created multiple times quickly
#[test]
fn test_multiple_runtime_creation_fast() {
    env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");

    let start = Instant::now();

    for _ in 0..10 {
        let _runtime = Runtime::new().expect("Runtime initialization failed");
    }

    let duration = start.elapsed();
    let avg_per_init = duration.as_micros() / 10;

    assert!(
        avg_per_init < 1000,
        "Average initialization took {}μs per runtime, should be <1000μs",
        avg_per_init
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Runtime stores endpoint correctly for lazy init
#[test]
fn test_runtime_stores_endpoint() {
    env::set_var("AWS_LAMBDA_RUNTIME_API", "custom-endpoint:8080");

    let runtime = Runtime::new().expect("Runtime should initialize");

    // Runtime should store the endpoint even without creating the client
    // This will be validated when we add a getter method
    drop(runtime);

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Initialization creates minimal overhead
#[test]
fn test_initialization_minimal_overhead() {
    env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");

    // Measure baseline (just environment variable read)
    let baseline_start = Instant::now();
    let _endpoint =
        env::var("AWS_LAMBDA_RUNTIME_API").expect("AWS_LAMBDA_RUNTIME_API should be set");
    let baseline = baseline_start.elapsed();

    // Measure Runtime initialization
    let runtime_start = Instant::now();
    let _runtime = Runtime::new().expect("Runtime initialization failed");
    let runtime_duration = runtime_start.elapsed();

    // Runtime init should be close to baseline (just storing the endpoint)
    // Allow up to 100μs overhead for struct creation
    let overhead = runtime_duration.saturating_sub(baseline);

    assert!(
        overhead.as_micros() < 100,
        "Initialization overhead is {}μs, should be <100μs (baseline: {}μs, total: {}μs)",
        overhead.as_micros(),
        baseline.as_micros(),
        runtime_duration.as_micros()
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Lazy client initialization benchmark
#[test]
#[ignore] // Only run explicitly for benchmarking
fn test_lazy_vs_eager_initialization_benchmark() {
    use std::time::Duration;

    env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");

    // Measure lazy initialization (Phase 2 approach)
    let mut lazy_times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _runtime = Runtime::new().expect("Runtime init failed");
        lazy_times.push(start.elapsed());
    }

    let lazy_avg: Duration = lazy_times.iter().sum::<Duration>() / 100;
    let lazy_p95 = lazy_times[95];

    println!("Lazy initialization:");
    println!("  Average: {}μs", lazy_avg.as_micros());
    println!("  P95: {}μs", lazy_p95.as_micros());
    println!("  Max: {}μs", lazy_times.iter().max().unwrap().as_micros());

    // Assertion: lazy should be <1ms
    assert!(
        lazy_avg.as_micros() < 1000,
        "Lazy average {}μs should be <1000μs",
        lazy_avg.as_micros()
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}
