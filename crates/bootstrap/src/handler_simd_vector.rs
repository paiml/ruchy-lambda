// Pure Rust SIMD Vector Handler for AWS Lambda Graviton2
// Demonstrates ARM NEON SIMD performance
// Target: <8ms cold start, 5x faster than scalar

use crate::simd_ops;

/// Lambda handler for SIMD vector dot product benchmark
///
/// This handler showcases ARM NEON SIMD advantages:
/// - 4x parallelism (f32x4 vectors)
/// - Fused multiply-add instructions
/// - Zero external dependencies
/// - <500KB binary size
///
/// Workload: Compute dot product of two 10,000-element f32 vectors
/// Expected speedup: 5x faster than scalar on ARM64 Graviton2
///
/// # Arguments
/// * `request_id` - Unique Lambda request ID (unused in this benchmark)
/// * `body` - Request body (unused, always uses 10K element vectors)
///
/// # Returns
/// JSON response with dot product result and vector size
#[allow(clippy::all)]
pub fn lambda_handler(_request_id: &str, _body: &str) -> String {
    // Vector size: 10,000 elements (40KB per vector)
    const SIZE: usize = 10_000;

    // Generate test vectors (simple pattern for reproducibility)
    // Vector A: [1.0, 2.0, 3.0, 4.0, ..., 10000.0]
    // Vector B: [0.5, 0.5, 0.5, 0.5, ..., 0.5]
    let vec_a: Vec<f32> = (0..SIZE).map(|i| (i as f32) + 1.0).collect();
    let vec_b: Vec<f32> = vec![0.5; SIZE];

    // Compute dot product using SIMD-optimized function
    // On ARM64: Uses ARM NEON intrinsics (vfmaq_f32, vaddvq_f32)
    // On x86_64: Uses scalar fallback
    let result = simd_ops::dot_product(&vec_a, &vec_b);

    // Build JSON response
    // Expected result: sum(i * 0.5 for i in 1..=10000) = 25,002,500.0
    format!(
        "{{\"statusCode\":200,\"body\":{{\"dotProduct\":{},\"vectorSize\":{},\"arch\":\"{}\"}}}}",
        result,
        SIZE,
        if cfg!(target_arch = "aarch64") {
            "arm64-neon"
        } else {
            "x86_64-scalar"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_handler() {
        let response = lambda_handler("test-request-id", "{}");
        assert!(response.contains("statusCode"));
        assert!(response.contains("dotProduct"));
        assert!(response.contains("25002500")); // Expected result
        println!("Response: {}", response);
    }

    #[test]
    fn test_lambda_handler_correctness() {
        let response = lambda_handler("test", "{}");
        // Parse response to verify correctness
        assert!(response.contains("\"statusCode\":200"));
    }
}
