// Minimal HTTP Client for Lambda Runtime API
// Extreme TDD: Tests written FIRST (see tests/http_client_tests.rs)
//
// Phase 3: Converted to BLOCKING I/O (removed tokio)
// Goal: Reduce binary size by ~77KB (tokio removal)
//
// This client ONLY supports the Lambda Runtime API:
// - GET /2018-06-01/runtime/invocation/next
// - POST /2018-06-01/runtime/invocation/{id}/response
//
// NOT supported (not needed for Lambda):
// - HTTPS/TLS (Lambda Runtime API uses plain HTTP internally)
// - Redirects, cookies, compression, etc.
// - Connection pooling (single-threaded Lambda execution)
// - Async/await (Lambda processes one event at a time)

use std::io::{self, Read, Write};
use std::net::TcpStream;

/// Minimal HTTP client error
#[derive(Debug)]
pub enum HttpError {
    /// I/O error
    Io(io::Error),
    /// Invalid response
    InvalidResponse(String),
}

impl From<io::Error> for HttpError {
    fn from(err: io::Error) -> Self {
        HttpError::Io(err)
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Io(e) => write!(f, "HTTP I/O error: {e}"),
            HttpError::InvalidResponse(msg) => write!(f, "Invalid HTTP response: {msg}"),
        }
    }
}

impl std::error::Error for HttpError {}

/// Minimal HTTP client for Lambda Runtime API
///
/// This is a lightweight HTTP/1.1 client that ONLY supports:
/// - GET requests (for `next_event`)
/// - POST requests (for `post_response`)
/// - Plain HTTP (no TLS - Lambda Runtime API is internal)
///
/// Binary size impact: ~10-20KB vs reqwest's ~180KB
pub struct HttpClient {
    /// Lambda Runtime API endpoint (e.g., "127.0.0.1:9001")
    endpoint: String,
}

impl HttpClient {
    /// Create a new HTTP client for the given endpoint
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    /// Make a GET request and return the `request_id` header and response body
    ///
    /// **Phase 3**: Converted to blocking I/O (no async/await)
    /// **Phase 5**: Extract Lambda-Runtime-Aws-Request-Id header for event processing
    ///
    /// # Returns
    ///
    /// Returns `(request_id, body)` tuple where:
    /// - `request_id` is extracted from `Lambda-Runtime-Aws-Request-Id` header
    /// - `body` is the raw response body (user's event payload)
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or response is invalid
    pub fn get(&self, path: &str) -> Result<(String, String), HttpError> {
        // Connect to endpoint (blocking)
        let mut stream = TcpStream::connect(&self.endpoint)?;

        // Build HTTP GET request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, self.endpoint
        );

        // Send request (blocking)
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        // Read response (blocking)
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;

        // Parse response with headers
        Self::parse_response_with_headers(&buffer)
    }

    /// Make a POST request with a body and return the response status
    ///
    /// **Phase 3**: Converted to blocking I/O (no async/await)
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if the request fails or response is invalid
    pub fn post(&self, path: &str, body: &str) -> Result<(), HttpError> {
        // Connect to endpoint (blocking)
        let mut stream = TcpStream::connect(&self.endpoint)?;

        // Build HTTP POST request
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            path, self.endpoint, body.len(), body
        );

        // Send request (blocking)
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        // Read response (we don't need the body, just verify it succeeded)
        let mut buffer = vec![0u8; 1024];
        let n = stream.read(&mut buffer)?;

        // Check for 2xx status code
        let response = String::from_utf8_lossy(&buffer[..n]);
        if !response.contains("HTTP/1.1 2") {
            return Err(HttpError::InvalidResponse(format!(
                "POST request failed: {}",
                response.lines().next().unwrap_or("unknown")
            )));
        }

        Ok(())
    }

    /// Parse HTTP response and extract Lambda `request_id` header + body
    ///
    /// **Phase 5**: Extract Lambda-Runtime-Aws-Request-Id from response headers
    ///
    /// Returns `(request_id, body)` tuple
    fn parse_response_with_headers(data: &[u8]) -> Result<(String, String), HttpError> {
        let response = String::from_utf8_lossy(data);

        // Find HTTP status line
        let status_line = response
            .lines()
            .next()
            .ok_or_else(|| HttpError::InvalidResponse("Empty response".to_string()))?;

        // Check for 2xx status code
        if !status_line.contains("HTTP/1.1 2") {
            return Err(HttpError::InvalidResponse(format!(
                "Non-2xx status: {status_line}"
            )));
        }

        // Find headers section (between first line and \r\n\r\n)
        let headers_start = response.find("\r\n").unwrap_or(0) + 2;
        let body_start = response
            .find("\r\n\r\n")
            .ok_or_else(|| HttpError::InvalidResponse("No body separator found".to_string()))?
            + 4;

        let headers_section = &response[headers_start..body_start - 4];

        // Extract Lambda-Runtime-Aws-Request-Id header
        let request_id = headers_section
            .lines()
            .find(|line| {
                line.to_lowercase()
                    .starts_with("lambda-runtime-aws-request-id:")
            })
            .and_then(|line| line.split(':').nth(1))
            .map_or_else(|| "unknown".to_string(), |id| id.trim().to_string());

        let body = response[body_start..].to_string();

        Ok((request_id, body))
    }

    /// Parse HTTP response and extract body
    ///
    /// Note: Currently unused. Kept for potential future use cases.
    #[allow(dead_code)]
    fn parse_response(data: &[u8]) -> Result<String, HttpError> {
        let response = String::from_utf8_lossy(data);

        // Find HTTP status line
        let status_line = response
            .lines()
            .next()
            .ok_or_else(|| HttpError::InvalidResponse("Empty response".to_string()))?;

        // Check for 2xx status code
        if !status_line.contains("HTTP/1.1 2") {
            return Err(HttpError::InvalidResponse(format!(
                "Non-2xx status: {status_line}"
            )));
        }

        // Find body (after \r\n\r\n)
        let body_start = response
            .find("\r\n\r\n")
            .ok_or_else(|| HttpError::InvalidResponse("No body separator found".to_string()))?
            + 4;

        Ok(response[body_start..].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_valid() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n{\"test\":true}";
        let body = HttpClient::parse_response(response).unwrap();
        assert_eq!(body, "{\"test\":true}");
    }

    #[test]
    fn test_parse_response_202() {
        let response = b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n";
        let body = HttpClient::parse_response(response).unwrap();
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_response_404() {
        let response = b"HTTP/1.1 404 Not Found\r\n\r\nNot found";
        let result = HttpClient::parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_empty() {
        let response = b"";
        let result = HttpClient::parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_error_display() {
        let error = HttpError::InvalidResponse("test error".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("Invalid HTTP response"));
        assert!(msg.contains("test error"));
    }

    // NEW TESTS: Increase coverage from 41.96% to ~80%+

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new("127.0.0.1:9001".to_string());
        assert_eq!(client.endpoint, "127.0.0.1:9001");
    }

    #[test]
    fn test_parse_response_with_headers_valid() {
        let response = b"HTTP/1.1 200 OK\r\nLambda-Runtime-Aws-Request-Id: test-req-123\r\nContent-Length: 13\r\n\r\n{\"test\":true}";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "test-req-123");
        assert_eq!(body, "{\"test\":true}");
    }

    #[test]
    fn test_parse_response_with_headers_no_request_id() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n{\"test\":true}";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "unknown");
        assert_eq!(body, "{\"test\":true}");
    }

    #[test]
    fn test_parse_response_with_headers_empty_response() {
        let response = b"";
        let result = HttpClient::parse_response_with_headers(response);
        assert!(result.is_err());
        if let Err(HttpError::InvalidResponse(msg)) = result {
            assert!(msg.contains("Empty response"));
        } else {
            panic!("Expected InvalidResponse error");
        }
    }

    #[test]
    fn test_parse_response_with_headers_non_2xx() {
        let response = b"HTTP/1.1 404 Not Found\r\n\r\nNot found";
        let result = HttpClient::parse_response_with_headers(response);
        assert!(result.is_err());
        if let Err(HttpError::InvalidResponse(msg)) = result {
            assert!(msg.contains("Non-2xx status"));
        } else {
            panic!("Expected InvalidResponse error");
        }
    }

    #[test]
    fn test_parse_response_with_headers_no_body_separator() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0";
        let result = HttpClient::parse_response_with_headers(response);
        assert!(result.is_err());
        if let Err(HttpError::InvalidResponse(msg)) = result {
            assert!(msg.contains("No body separator"));
        } else {
            panic!("Expected InvalidResponse error");
        }
    }

    #[test]
    fn test_parse_response_with_headers_case_insensitive() {
        // Lambda header with different casing
        let response =
            b"HTTP/1.1 200 OK\r\nLAMBDA-RUNTIME-AWS-REQUEST-ID: test-456\r\n\r\n{\"data\":true}";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "test-456");
        assert_eq!(body, "{\"data\":true}");
    }

    #[test]
    fn test_parse_response_with_headers_whitespace_in_header() {
        // Header with extra whitespace
        let response =
            b"HTTP/1.1 200 OK\r\nLambda-Runtime-Aws-Request-Id:   test-789  \r\n\r\n{\"ok\":true}";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "test-789");
        assert_eq!(body, "{\"ok\":true}");
    }

    #[test]
    fn test_http_error_io_display() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
        let error = HttpError::Io(io_error);
        let msg = format!("{error}");
        assert!(msg.contains("HTTP I/O error"));
    }

    #[test]
    fn test_http_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Timed out");
        let http_error: HttpError = io_error.into();
        assert!(matches!(http_error, HttpError::Io(_)));
    }

    #[test]
    fn test_parse_response_with_headers_empty_body() {
        let response = b"HTTP/1.1 202 Accepted\r\nLambda-Runtime-Aws-Request-Id: req-empty\r\nContent-Length: 0\r\n\r\n";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "req-empty");
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_response_with_headers_large_body() {
        let large_body = "x".repeat(10000);
        let response = format!(
            "HTTP/1.1 200 OK\r\nLambda-Runtime-Aws-Request-Id: req-large\r\nContent-Length: {}\r\n\r\n{}",
            large_body.len(),
            large_body
        );
        let (request_id, body) =
            HttpClient::parse_response_with_headers(response.as_bytes()).unwrap();
        assert_eq!(request_id, "req-large");
        assert_eq!(body.len(), 10000);
    }

    #[test]
    fn test_parse_response_with_headers_multiple_headers() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nLambda-Runtime-Aws-Request-Id: multi-header\r\nX-Custom: value\r\n\r\n{\"multi\":true}";
        let (request_id, body) = HttpClient::parse_response_with_headers(response).unwrap();
        assert_eq!(request_id, "multi-header");
        assert_eq!(body, "{\"multi\":true}");
    }
}
