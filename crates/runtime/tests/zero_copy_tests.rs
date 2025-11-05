// Extreme TDD: Zero-Copy Deserialization Tests (Section 3.3.1)
// Target: 40-60% reduction in allocation overhead

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestEvent<'a> {
    #[serde(borrow)]
    request_id: &'a str,
    #[serde(borrow)]
    body: &'a str,
}

#[test]
fn test_zero_copy_json_deserialization() {
    // Tier 1: JSON zero-copy with borrowed references
    let json = r#"{"request_id":"test-123","body":"hello world"}"#;

    let event: TestEvent = serde_json::from_str(json).expect("Failed to deserialize event");

    assert_eq!(event.request_id, "test-123");
    assert_eq!(event.body, "hello world");

    // Verify zero-copy: pointers should reference original string
    // JSON: {"request_id":"test-123","body":"hello world"}
    // "test-123" is at positions 15..23
    let json_bytes = json.as_bytes();
    let request_id_in_json = &json[15..23];

    assert_eq!(
        request_id_in_json, "test-123",
        "JSON substring extraction is correct"
    );

    // Check if the deserialized string points into the original JSON
    let event_ptr = event.request_id.as_ptr() as usize;
    let json_start = json_bytes.as_ptr() as usize;
    let json_end = json_start + json_bytes.len();

    assert!(
        event_ptr >= json_start && event_ptr < json_end,
        "request_id should be borrowed from original JSON (zero-copy). \
         event_ptr: {}, json range: {}..{}",
        event_ptr,
        json_start,
        json_end
    );
}

#[test]
fn test_json_deserialization_performance() {
    // Performance requirement: 20-30% faster for <10KB payloads (Section 3.3.1)
    use std::time::Instant;

    let json = r#"{"request_id":"test-123","body":"small payload"}"#;
    let iterations = 10000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _event: TestEvent = serde_json::from_str(json).unwrap();
    }
    let duration = start.elapsed();

    // Baseline: Should complete 10K deserializations quickly
    assert!(
        duration.as_millis() < 100,
        "Deserialization took {}ms for {}K iterations",
        duration.as_millis(),
        iterations / 1000
    );
}

#[cfg(test)]
mod allocation_tests {
    use super::*;

    #[test]
    fn test_borrowed_references_no_allocation() {
        // Zero-copy means no heap allocation for borrowed strings
        let json = r#"{"request_id":"borrowed","body":"also borrowed"}"#;

        // This should NOT allocate new strings
        let event: TestEvent = serde_json::from_str(json).unwrap();

        // Verify the strings point into the original JSON
        let json_ptr_range = json.as_ptr() as usize..json.as_ptr() as usize + json.len();
        let request_id_ptr = event.request_id.as_ptr() as usize;
        let body_ptr = event.body.as_ptr() as usize;

        assert!(
            json_ptr_range.contains(&request_id_ptr),
            "request_id should be borrowed from original JSON"
        );
        assert!(
            json_ptr_range.contains(&body_ptr),
            "body should be borrowed from original JSON"
        );
    }
}
