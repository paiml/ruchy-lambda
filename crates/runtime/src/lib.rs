// Ruchy Lambda Runtime
// Minimal stub implementation for Extreme TDD (RED phase)
//
// Performance Target: <1ms initialization, <100μs per invocation
// Quality Standard: TDG ≥A, Cyclomatic ≤15, Cognitive ≤20

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::multiple_crate_versions)]

//! Ruchy Lambda Runtime
//!
//! A high-performance AWS Lambda custom runtime built on Ruchy's transpilation
//! to Rust, targeting <8ms cold start times.
//!
//! # Architecture
//!
//! - **Zero-copy deserialization**: 40-60% allocation reduction (Section 3.3.1)
//! - **Minimal initialization**: <1ms startup time (Section 3.2)
//! - **Low invocation overhead**: <100μs per request (Section 3.3)
//!
//! # Examples
//!
//! ```no_run
//! use ruchy_lambda_runtime::Runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = Runtime::new()?;
//! // Future: runtime.run() will start the event loop
//! # Ok(())
//! # }
//! ```

use once_cell::sync::OnceCell;
use std::env;
use std::error::Error as StdError;
use std::fmt;

mod event;
mod http_client;
mod logger;

pub use event::{LambdaEvent, RequestContext};
use http_client::HttpClient;
pub use logger::{LogLevel, Logger};

/// Runtime error type
#[derive(Debug)]
pub enum Error {
    /// Initialization failed
    InitializationFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {msg}"),
        }
    }
}

impl StdError for Error {}

/// Result type for runtime operations
pub type Result<T> = std::result::Result<T, Error>;

/// Ruchy Lambda Runtime
///
/// The main runtime struct that handles Lambda function execution.
///
/// Uses lazy initialization to achieve <1ms startup time.
/// HTTP client is created on first API call, not during `Runtime::new()`.
///
/// # Performance Requirements
///
/// - Initialization: <1ms (Section 3.2) ✅
/// - Invocation overhead: <100μs (Section 3.3)
/// - Memory footprint: <64MB (Section 12.2)
#[derive(Clone)]
pub struct Runtime {
    /// Lambda Runtime API endpoint (e.g., "127.0.0.1:9001")
    api_endpoint: String,

    /// Lazy HTTP client for Lambda Runtime API calls
    /// Created on first use to minimize initialization overhead
    /// Uses `OnceCell` for thread-safe lazy initialization
    /// Minimal HTTP client (no reqwest) for smaller binary size
    client: std::sync::Arc<OnceCell<HttpClient>>,
}

impl fmt::Debug for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Runtime")
            .field("api_endpoint", &self.api_endpoint)
            .field("client", &"OnceCell<HttpClient>")
            .finish()
    }
}

impl Runtime {
    /// Create a new runtime instance
    ///
    /// Reads the `AWS_LAMBDA_RUNTIME_API` environment variable to determine
    /// the Lambda Runtime API endpoint.
    ///
    /// **Lazy Initialization**: HTTP client is NOT created here. It will be
    /// created on the first API call (`next_event()` or `post_response()`).
    /// This achieves <1ms initialization time.
    ///
    /// # Errors
    ///
    /// Returns `Error::InitializationFailed` if runtime setup fails.
    /// Note: HTTP client creation errors are deferred to first use.
    ///
    /// # Performance
    ///
    /// This function completes in <1ms via lazy initialization (Section 3.2).
    /// The HTTP client (~5ms overhead) is created on first invocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy_lambda_runtime::Runtime;
    /// use std::env;
    ///
    /// env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");
    /// let runtime = Runtime::new().expect("Failed to initialize runtime");
    /// ```
    pub fn new() -> Result<Self> {
        // Read AWS Lambda Runtime API endpoint (fast: just env var read)
        // This is provided by Lambda: http://${AWS_LAMBDA_RUNTIME_API}
        let api_endpoint =
            env::var("AWS_LAMBDA_RUNTIME_API").unwrap_or_else(|_| "127.0.0.1:9001".to_string());

        // LAZY INITIALIZATION: Don't create HTTP client yet
        // Client will be created on first API call (next_event/post_response)
        // This reduces initialization time from ~5ms to <1ms
        Ok(Self {
            api_endpoint,
            client: std::sync::Arc::new(OnceCell::new()),
        })
    }

    /// Get or create the HTTP client (lazy initialization)
    ///
    /// This function is called by `next_event()` and `post_response()`.
    /// On first call, it creates the minimal HTTP client (~instant).
    /// Subsequent calls return the cached client (fast).
    fn get_client(&self) -> Result<&HttpClient> {
        self.client
            .get_or_try_init(|| {
                // Create minimal HTTP client (no reqwest overhead)
                Ok::<HttpClient, Error>(HttpClient::new(self.api_endpoint.clone()))
            })
            .map_err(|e| Error::InitializationFailed(format!("HTTP client creation failed: {e}")))
    }

    /// Get the next Lambda event from the Runtime API
    ///
    /// **Phase 3**: Converted to blocking I/O (removed async/await)
    /// **Phase 5**: Extract `request_id` from Lambda-Runtime-Aws-Request-Id header
    ///
    /// Makes a GET request to `/2018-06-01/runtime/invocation/next`
    /// This is a long-polling request that blocks until an event is available.
    ///
    /// # Returns
    ///
    /// Returns `(request_id, event_body)` tuple where:
    /// - `request_id` is extracted from `Lambda-Runtime-Aws-Request-Id` response header
    /// - `event_body` is the raw user event payload (not wrapped in requestContext)
    ///
    /// # Errors
    ///
    /// Returns `Error::InitializationFailed` if the API request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ruchy_lambda_runtime::Runtime;
    /// # use std::env;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");
    /// let runtime = Runtime::new()?;
    /// let (request_id, event_body) = runtime.next_event()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn next_event(&self) -> Result<(String, String)> {
        let path = "/2018-06-01/runtime/invocation/next";

        // Lazy initialization: creates client on first call
        let client = self.get_client()?;

        client
            .get(path)
            .map_err(|e| Error::InitializationFailed(format!("Failed to get next event: {e}")))
    }

    /// Post a response to the Lambda Runtime API
    ///
    /// **Phase 3**: Converted to blocking I/O (removed async/await)
    ///
    /// Makes a POST request to `/2018-06-01/runtime/invocation/{request_id}/response`
    ///
    /// # Errors
    ///
    /// Returns `Error::InitializationFailed` if the API request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ruchy_lambda_runtime::Runtime;
    /// # use std::env;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001");
    /// let runtime = Runtime::new()?;
    /// runtime.post_response("req-123", r#"{"status": "ok"}"#)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn post_response(&self, request_id: &str, response_body: &str) -> Result<()> {
        let path = format!("/2018-06-01/runtime/invocation/{request_id}/response");

        // Lazy initialization: creates client on first call
        let client = self.get_client()?;

        client
            .post(&path, response_body)
            .map_err(|e| Error::InitializationFailed(format!("Failed to post response: {e}")))?;

        Ok(())
    }
}

// Ensure Runtime is thread-safe (required for tokio)
// This is enforced by the test in initialization_tests.rs
static_assertions::assert_impl_all!(Runtime: Send, Sync);

#[cfg(test)]
use serial_test::serial;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_runtime_creation() {
        let result = Runtime::new();
        assert!(result.is_ok());
    }

    // NEW TESTS: Increase coverage from 26.53% to ~80%+

    #[test]
    #[serial]
    fn test_error_display() {
        let error = Error::InitializationFailed("test failure".to_string());
        let msg = format!("{error}");
        assert!(msg.contains("Initialization failed"));
        assert!(msg.contains("test failure"));
    }

    #[test]
    #[serial]
    fn test_error_trait() {
        let error = Error::InitializationFailed("test".to_string());
        let _: &dyn StdError = &error;
    }

    #[test]
    #[serial]
    fn test_runtime_debug() {
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:8888");
        let runtime = Runtime::new().unwrap();
        let debug_str = format!("{runtime:?}");
        assert!(debug_str.contains("Runtime"));
        assert!(debug_str.contains("127.0.0.1:8888"));
        assert!(debug_str.contains("OnceCell<HttpClient>"));
        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_runtime_clone() {
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:7777");
        let runtime = Runtime::new().unwrap();
        let cloned = runtime.clone();
        assert_eq!(runtime.api_endpoint, cloned.api_endpoint);
        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_runtime_default_endpoint() {
        env::remove_var("AWS_LAMBDA_RUNTIME_API");
        let runtime = Runtime::new().unwrap();
        assert_eq!(runtime.api_endpoint, "127.0.0.1:9001");
    }

    #[test]
    #[serial]
    fn test_runtime_custom_endpoint() {
        env::set_var("AWS_LAMBDA_RUNTIME_API", "custom-host:3000");
        let runtime = Runtime::new().unwrap();
        assert_eq!(runtime.api_endpoint, "custom-host:3000");
        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_runtime_lazy_client_not_initialized() {
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9999");
        let runtime = Runtime::new().unwrap();
        // Client should NOT be initialized yet
        assert!(runtime.client.get().is_none());
        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_get_client_initializes_once() {
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:5555");
        let runtime = Runtime::new().unwrap();

        // First call initializes
        let client1 = runtime.get_client();
        assert!(client1.is_ok());
        assert!(runtime.client.get().is_some());

        // Second call returns same instance
        let client2 = runtime.get_client();
        assert!(client2.is_ok());

        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_next_event_error_connection_refused() {
        // Use non-existent endpoint to trigger connection error
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:19999");
        let runtime = Runtime::new().unwrap();

        let result = runtime.next_event();
        assert!(result.is_err());

        if let Err(Error::InitializationFailed(msg)) = result {
            assert!(msg.contains("Failed to get next event"));
        } else {
            panic!("Expected InitializationFailed error");
        }

        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_post_response_error_connection_refused() {
        // Use non-existent endpoint to trigger connection error
        env::set_var("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:19998");
        let runtime = Runtime::new().unwrap();

        let result = runtime.post_response("test-id", r#"{"status":"ok"}"#);
        assert!(result.is_err());

        if let Err(Error::InitializationFailed(msg)) = result {
            assert!(msg.contains("Failed to post response"));
        } else {
            panic!("Expected InitializationFailed error");
        }

        env::remove_var("AWS_LAMBDA_RUNTIME_API");
    }

    #[test]
    #[serial]
    fn test_runtime_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<Runtime>();
        is_sync::<Runtime>();
    }
}
