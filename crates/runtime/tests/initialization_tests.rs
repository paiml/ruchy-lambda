// Extreme TDD: Tests written FIRST
// These tests define expected behavior for <1ms initialization (Section 3.2)

use ruchy_lambda_runtime::Runtime;

#[test]
fn test_runtime_initializes_successfully() {
    // RED: This will fail until we implement Runtime
    let result = Runtime::new();
    assert!(result.is_ok(), "Runtime should initialize successfully");
}

#[test]
fn test_initialization_time_under_1ms() {
    // Performance requirement: <1ms initialization (see spec Section 3.2)
    use std::time::Instant;

    let start = Instant::now();
    let _runtime = Runtime::new().expect("Runtime initialization failed");
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 1,
        "Initialization took {}ms, should be <1ms",
        duration.as_millis()
    );
}

#[test]
fn test_runtime_is_send_and_sync() {
    // Runtime must be thread-safe for tokio
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Runtime>();
}

#[test]
fn test_runtime_can_be_created_multiple_times() {
    // Should support multiple instances (though Lambda uses singleton)
    let _runtime1 = Runtime::new().expect("First runtime failed");
    let _runtime2 = Runtime::new().expect("Second runtime failed");
}

#[cfg(test)]
mod initialization_metrics {
    use super::*;

    #[test]
    fn test_minimal_memory_footprint() {
        // Target: <64MB memory usage (Section 12.2)
        // Note: This is a placeholder - actual measurement needs runtime instrumentation
        let runtime = Runtime::new().expect("Runtime initialization failed");

        // Future: Add actual memory measurement
        // For now, just verify it compiles and runs
        drop(runtime);
    }
}
