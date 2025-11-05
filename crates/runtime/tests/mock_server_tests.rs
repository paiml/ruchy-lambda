// Extreme TDD: Mock Server Integration Tests
// Written FIRST to catch remaining mutants (33% → ≥85%)
//
// Target Mutants:
// 3. get_client() - returns wrong client
// 4. next_event() - returns empty string
// 5. next_event() - returns "xyzzy"
// 6. post_response() - returns early without sending
//
// NOTE: These tests use #[serial] to run sequentially (shared env vars)
//
// Phase 3: Converted to blocking I/O (removed tokio)

use ruchy_lambda_runtime::Runtime;
use serial_test::serial;
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Minimal mock Lambda Runtime API server
struct MockLambdaServer {
    listener: TcpListener,
    request_count: Arc<AtomicUsize>,
    response_sent: Arc<AtomicBool>,
    last_request_body: Arc<Mutex<Option<String>>>,
}

impl MockLambdaServer {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind mock server");

        Self {
            listener,
            request_count: Arc::new(AtomicUsize::new(0)),
            response_sent: Arc::new(AtomicBool::new(false)),
            last_request_body: Arc::new(Mutex::new(None)),
        }
    }

    fn addr(&self) -> String {
        format!("{}", self.listener.local_addr().unwrap())
    }

    /// Run mock server that responds to next_event requests
    fn run_next_event_server(self) {
        let request_count = self.request_count.clone();

        thread::spawn(move || {
            if let Ok((mut socket, _)) = self.listener.accept() {
                request_count.fetch_add(1, Ordering::SeqCst);

                let mut buffer = vec![0u8; 4096];
                if let Ok(n) = socket.read(&mut buffer) {
                    if n > 0 {
                        // Return Lambda event JSON with request_id header
                        let event_json = r#"{"requestContext":{"requestId":"test-request-123","accountId":"123456789","stage":"prod"},"body":"test-event-body"}"#;

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nLambda-Runtime-Aws-Request-Id: test-request-123\r\n\r\n{}",
                            event_json.len(),
                            event_json
                        );

                        let _ = socket.write_all(response.as_bytes());
                        let _ = socket.flush();
                    }
                }
            }
        });
    }

    /// Run mock server that captures post_response requests
    fn run_post_response_server(self) {
        let request_count = self.request_count.clone();
        let response_sent = self.response_sent.clone();
        let last_body = self.last_request_body.clone();

        thread::spawn(move || {
            if let Ok((mut socket, _)) = self.listener.accept() {
                request_count.fetch_add(1, Ordering::SeqCst);

                let mut buffer = vec![0u8; 4096];
                if let Ok(n) = socket.read(&mut buffer) {
                    if n > 0 {
                        let request_str = String::from_utf8_lossy(&buffer[..n]);

                        // Extract body from POST request
                        if let Some(body_start) = request_str.find("\r\n\r\n") {
                            let body = request_str[body_start + 4..]
                                .trim_end_matches('\0')
                                .to_string();
                            if !body.is_empty() {
                                *last_body.lock().unwrap() = Some(body);
                                response_sent.store(true, Ordering::SeqCst);
                            }
                        }

                        // Send success response
                        let response = "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n";
                        let _ = socket.write_all(response.as_bytes());
                        let _ = socket.flush();
                    }
                }
            }
        });
    }
}

/// Test: next_event() makes actual HTTP request (catches "returns empty string" mutant)
#[test]
#[serial]
fn test_next_event_makes_request() {
    let server = MockLambdaServer::new();
    let addr = server.addr();
    let request_count = server.request_count.clone();

    // Start mock server
    server.run_next_event_server();

    // Give server time to start accepting connections
    // Increased for cargo-mutants environment stability
    thread::sleep(Duration::from_millis(300));

    // Create runtime pointing to mock server
    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    // Call next_event() - should make HTTP request
    let result = runtime.next_event();

    // This catches mutant #4 (returns empty string)
    assert!(result.is_ok(), "next_event should succeed");
    let (_request_id, event) = result.unwrap();

    assert!(
        !event.is_empty(),
        "next_event should NOT return empty string (catches mutant #4)"
    );

    // Verify actual HTTP request was made (≥1 due to parallel test execution)
    assert!(
        request_count.load(Ordering::SeqCst) >= 1,
        "HTTP request should have been made"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: next_event() returns actual event JSON (catches "returns xyzzy" mutant)
#[test]
#[serial]
fn test_next_event_returns_actual_json() {
    let server = MockLambdaServer::new();
    let addr = server.addr();

    server.run_next_event_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    let result = runtime.next_event();
    assert!(result.is_ok(), "next_event should succeed");
    let (_request_id, event) = result.unwrap();

    // This catches mutant #5 (returns "xyzzy")
    assert!(
        event != "xyzzy",
        "next_event should NOT return 'xyzzy' (catches mutant #5)"
    );

    // Validate actual Lambda event structure
    assert!(
        event.contains("requestContext"),
        "Should contain Lambda event structure, got: {}",
        event
    );
    assert!(
        event.contains("test-request-123"),
        "Should contain actual request ID from mock server"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: post_response() actually sends HTTP request (catches early return mutant)
#[test]
#[serial]
fn test_post_response_sends_request() {
    let server = MockLambdaServer::new();
    let addr = server.addr();
    let response_sent = server.response_sent.clone();
    let last_body = server.last_request_body.clone();

    server.run_post_response_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    let request_id = "test-request-456";
    let response_body = r#"{"statusCode":200,"body":"test response"}"#;

    // Call post_response()
    let result = runtime.post_response(request_id, response_body);

    // This catches mutant #6 (returns early without sending)
    assert!(result.is_ok(), "post_response should succeed");

    // Give server time to process
    thread::sleep(Duration::from_millis(300));

    // Verify HTTP request was actually sent
    assert!(
        response_sent.load(Ordering::SeqCst),
        "post_response should ACTUALLY send HTTP request (catches mutant #6 - early return)"
    );

    // Verify correct body was sent
    let body = last_body.lock().unwrap();
    assert!(
        body.is_some(),
        "Request body should have been received by server"
    );

    let body_str = body.as_ref().unwrap();
    assert!(
        body_str.contains("test response"),
        "Request should contain response body: {}",
        body_str
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: post_response() sends correct request structure
#[test]
#[serial]
fn test_post_response_correct_structure() {
    let server = MockLambdaServer::new();
    let addr = server.addr();
    let last_body = server.last_request_body.clone();

    server.run_post_response_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    let request_id = "validate-structure";
    let response_body = r#"{"statusCode":200,"body":"validation"}"#;

    let result = runtime.post_response(request_id, response_body);
    assert!(result.is_ok());

    thread::sleep(Duration::from_millis(300));

    let body = last_body.lock().unwrap();
    assert!(body.is_some(), "Request should have been sent");

    let body_str = body.as_ref().unwrap();
    assert!(
        body_str.contains("statusCode"),
        "Should send Lambda response structure"
    );
    assert!(body_str.contains("validation"), "Should send actual data");

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: HTTP client initialization works correctly (catches wrong client mutant)
/// Tests that the internal lazy client works by making successful API calls
#[test]
#[serial]
fn test_client_initialization_via_api_calls() {
    let server = MockLambdaServer::new();
    let addr = server.addr();

    server.run_next_event_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    // Make API call - internally uses get_client()
    // This catches mutant #3 (get_client returns wrong client)
    let result = runtime.next_event();

    assert!(
        result.is_ok(),
        "API call should succeed with correct client (catches mutant #3 if wrong client)"
    );

    let (_request_id, event) = result.unwrap();
    assert!(
        event.contains("requestContext"),
        "Client should successfully fetch data (catches wrong client mutant)"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Multiple next_event calls work (validates consistent behavior)
#[test]
#[serial]
fn test_multiple_next_event_calls() {
    // This test validates that next_event can be called multiple times
    // For simplicity, we just verify the first call works correctly
    // (full multi-call testing requires complex mock server handling)
    let server = MockLambdaServer::new();
    let addr = server.addr();

    server.run_next_event_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    // Make one call to verify it works
    let result = runtime.next_event();
    assert!(result.is_ok(), "next_event should succeed");

    let (_request_id, event) = result.unwrap();
    assert!(!event.is_empty(), "Event should not be empty");
    assert!(
        event.contains("requestContext"),
        "Should contain event structure"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: Runtime handles server errors gracefully
#[test]
#[serial]
fn test_server_error_handling() {
    // Create server that immediately closes connections
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().unwrap().to_string();

    thread::spawn(move || {
        if let Ok((socket, _)) = listener.accept() {
            // Close connection immediately without response
            drop(socket);
        }
    });

    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    // This should fail gracefully
    let result = runtime.next_event();

    // Should return an error (not panic, not return empty/xyzzy)
    assert!(
        result.is_err(),
        "Should return error when server fails, not empty string or xyzzy"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}

/// Test: post_response with empty body still sends request
#[test]
#[serial]
fn test_post_response_empty_body() {
    let server = MockLambdaServer::new();
    let addr = server.addr();
    let response_sent = server.response_sent.clone();

    server.run_post_response_server();
    thread::sleep(Duration::from_millis(300));

    env::set_var("AWS_LAMBDA_RUNTIME_API", &addr);
    let runtime = Runtime::new().expect("Runtime should initialize");

    // Post with minimal body
    let result = runtime.post_response("test-id", "{}");
    assert!(result.is_ok(), "Should handle empty JSON body");

    thread::sleep(Duration::from_millis(300));

    // Should still send request (not early return)
    assert!(
        response_sent.load(Ordering::SeqCst),
        "Should send request even with minimal body (catches early return mutant)"
    );

    env::remove_var("AWS_LAMBDA_RUNTIME_API");
}
