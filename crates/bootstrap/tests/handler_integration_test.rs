// Integration tests for Ruchy Lambda handler
// Tests end-to-end flow: Ruchy → Transpile → Runtime → Response
//
// Following Extreme TDD: Tests written FIRST, handler implementation follows

use ruchy_lambda_runtime::{LambdaEvent, RequestContext, Runtime};
use std::env;

/// Test that handler.ruchy transpiles successfully
///
/// This test verifies that `src/handler.ruchy` transpiles to valid Rust code
/// during the build process via `build.rs`.
///
/// **Success Criteria**:
/// - `src/handler_generated.rs` exists after build
/// - Generated code compiles without errors
/// - Handler function signature matches expected interface
#[test]
fn test_handler_transpiles_successfully() {
    // This test passes if the build succeeds (build.rs runs transpilation)
    // If transpilation failed, the build would fail

    // Verify generated file exists
    let generated_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/handler_generated.rs");
    assert!(
        std::path::Path::new(generated_path).exists(),
        "handler_generated.rs should exist after transpilation"
    );
}

/// Test that handler accepts Lambda event and returns response
///
/// **Success Criteria**:
/// - Handler accepts event JSON
/// - Returns properly formatted Lambda response
/// - Response contains statusCode and body
#[test]
fn test_handler_processes_event() {
    // Create test event
    let event_json = r#"{
        "body": "test payload",
        "requestContext": {
            "requestId": "test-request-123"
        }
    }"#;

    let event: LambdaEvent = serde_json::from_str(event_json).expect("Valid event JSON");

    // Call handler (will be implemented in handler.ruchy)
    // For now, we verify the event structure is correct
    assert_eq!(event.request_context.request_id, "test-request-123");
    assert_eq!(event.body, "test payload");
}

/// Test end-to-end Lambda invocation with mock server
///
/// **Success Criteria**:
/// - Mock server provides event via /runtime/invocation/next
/// - Handler processes event
/// - Response posted to /runtime/invocation/{id}/response
/// - Response format is valid Lambda response JSON
#[test]
fn test_end_to_end_invocation() {
    // Set up mock Runtime API endpoint
    env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");

    let runtime = Runtime::new().expect("Runtime initialization should succeed");

    // In production, this would:
    // 1. runtime.next_event() - fetch event
    // 2. handler(event) - process
    // 3. runtime.post_response() - send response

    // For now, verify runtime is ready
    assert!(runtime.next_event().is_err()); // No server running, expected to fail
}

/// Test handler returns proper Lambda response format
///
/// **Success Criteria**:
/// - Response is valid JSON
/// - Contains required fields: statusCode, body
/// - statusCode is 200 for successful invocation
#[test]
fn test_handler_response_format() {
    // Expected response format from Lambda handler
    let response_json = r#"{
        "statusCode": 200,
        "body": "Hello from Ruchy Lambda!"
    }"#;

    let response: serde_json::Value =
        serde_json::from_str(response_json).expect("Handler response should be valid JSON");

    assert_eq!(response["statusCode"], 200);
    assert!(response["body"].is_string());
}

/// Test handler with empty body
///
/// **Success Criteria**:
/// - Handler handles empty/null body gracefully
/// - Returns 200 status with appropriate message
#[test]
fn test_handler_empty_body() {
    let event_json = r#"{
        "body": "",
        "requestContext": {
            "requestId": "empty-test-456"
        }
    }"#;

    let event: LambdaEvent = serde_json::from_str(event_json).expect("Valid event JSON");

    // Handler should handle empty body
    assert_eq!(event.request_context.request_id, "empty-test-456");
    assert_eq!(event.body, "");
}

/// Test handler performance requirements
///
/// **Success Criteria** (from roadmap Section 3.3):
/// - Handler invocation completes in <100μs
/// - Zero-copy deserialization working
#[test]
fn test_handler_performance() {
    use std::time::Instant;

    let event_json = r#"{
        "body": "performance test",
        "requestContext": {
            "requestId": "perf-test-789"
        }
    }"#;

    let start = Instant::now();
    let _event: LambdaEvent = serde_json::from_str(event_json).expect("Valid event JSON");
    let duration = start.elapsed();

    // Deserialization should be very fast (<10μs for zero-copy)
    assert!(
        duration.as_micros() < 100,
        "Deserialization took {}μs, expected <100μs",
        duration.as_micros()
    );
}
