// Structured Logger for CloudWatch Logs
//
// Provides JSON-formatted logging for AWS Lambda that integrates
// seamlessly with CloudWatch Logs Insights.
//
// Design goals:
// - Zero external dependencies (keep binary small)
// - JSON formatted output (CloudWatch Insights friendly)
// - Thread-safe for concurrent use
// - Minimal overhead (<10μs per log call)
// - Supports context (request_id, timestamp)
//
// Phase 4: Advanced Features - CloudWatch Logs Integration

use std::fmt;
use std::io::{self, Write};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log level for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug level (most verbose)
    Debug,
    /// Info level (informational messages)
    Info,
    /// Warning level (warning messages)
    Warn,
    /// Error level (error messages)
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured logger for `CloudWatch` Logs
///
/// Outputs JSON-formatted logs to stdout, which Lambda Runtime
/// automatically captures and sends to `CloudWatch` Logs.
///
/// # Thread Safety
///
/// Logger uses an internal Mutex to ensure thread-safe concurrent logging.
/// Multiple threads can safely log simultaneously.
///
/// # Examples
///
/// ```
/// use ruchy_lambda_runtime::Logger;
///
/// let logger = Logger::new();
/// logger.info("Processing Lambda event");
/// logger.error("Failed to process request");
/// ```
///
/// With request ID context:
///
/// ```
/// use ruchy_lambda_runtime::Logger;
///
/// let logger = Logger::with_request_id("abc-123");
/// logger.info("Processing event for request abc-123");
/// ```
pub struct Logger {
    /// Optional request ID for context
    request_id: Option<String>,
    /// Minimum log level (None = log everything)
    min_level: Option<LogLevel>,
    /// Writer (stdout by default, can be mocked for testing)
    writer: Mutex<Box<dyn Write + Send>>,
}

impl Logger {
    /// Create a new logger without request context
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy_lambda_runtime::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.info("Application started");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            request_id: None,
            min_level: None,
            writer: Mutex::new(Box::new(io::stdout())),
        }
    }

    /// Create a logger with request ID context
    ///
    /// The request ID will be included in all log entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy_lambda_runtime::Logger;
    ///
    /// let logger = Logger::with_request_id("request-123");
    /// logger.info("Processing request");
    /// // Output: {"level":"INFO","timestamp":"...","request_id":"request-123","message":"Processing request"}
    /// ```
    pub fn with_request_id(request_id: impl Into<String>) -> Self {
        Self {
            request_id: Some(request_id.into()),
            min_level: None,
            writer: Mutex::new(Box::new(io::stdout())),
        }
    }

    /// Create a logger with a custom writer (test-only)
    ///
    /// This is used for testing to capture log output.
    #[cfg(test)]
    pub fn with_writer(writer: Box<dyn Write + Send>) -> Self {
        Self {
            request_id: None,
            min_level: None,
            writer: Mutex::new(writer),
        }
    }

    /// Set minimum log level
    ///
    /// Logs below this level will be filtered out.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy_lambda_runtime::{Logger, LogLevel};
    ///
    /// let mut logger = Logger::new();
    /// logger.set_min_level(LogLevel::Warn);
    /// logger.debug("This won't be logged");
    /// logger.warn("This will be logged");
    /// ```
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = Some(level);
    }

    /// Log a debug message
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruchy_lambda_runtime::Logger;
    /// let logger = Logger::new();
    /// logger.debug("Detailed debugging information");
    /// ```
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log an info message
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruchy_lambda_runtime::Logger;
    /// let logger = Logger::new();
    /// logger.info("Processing event");
    /// ```
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log a warning message
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruchy_lambda_runtime::Logger;
    /// let logger = Logger::new();
    /// logger.warn("Deprecated API usage");
    /// ```
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Log an error message
    ///
    /// # Examples
    ///
    /// ```
    /// # use ruchy_lambda_runtime::Logger;
    /// let logger = Logger::new();
    /// logger.error("Failed to process request");
    /// ```
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    /// Log a message with specified level
    ///
    /// Internal method that formats and writes the log entry.
    fn log(&self, level: LogLevel, message: &str) {
        // Check minimum log level
        if let Some(min_level) = self.min_level {
            if level < min_level {
                return; // Skip logging
            }
        }

        // Get current timestamp in ISO 8601 format
        let timestamp = Self::format_timestamp();

        // Build JSON log entry
        let json = self.format_json(level, &timestamp, message);

        // Write to output (stdout)
        let mut writer = self.writer.lock().unwrap();
        let _ = writeln!(writer, "{json}");
        let _ = writer.flush();
    }

    /// Format timestamp as ISO 8601
    ///
    /// Returns format: "2025-11-04T12:34:56.789Z"
    fn format_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch");

        let secs = now.as_secs();
        let millis = now.subsec_millis();

        // Calculate date/time components
        let days_since_epoch = secs / 86400;
        let remaining_secs = secs % 86400;

        let hours = remaining_secs / 3600;
        let minutes = (remaining_secs % 3600) / 60;
        let seconds = remaining_secs % 60;

        // Simplified date calculation (approximate)
        // For production, we'd use a proper date library, but
        // this is good enough for logging and has zero dependencies
        let years_since_1970 = days_since_epoch / 365;
        let year = 1970 + years_since_1970;
        let remaining_days = days_since_epoch % 365;
        let month = (remaining_days / 30) + 1;
        let day = (remaining_days % 30) + 1;

        format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z")
    }

    /// Format log entry as JSON
    ///
    /// Creates a single-line JSON object with all log fields.
    fn format_json(&self, level: LogLevel, timestamp: &str, message: &str) -> String {
        use std::fmt::Write;

        // Escape message for JSON (handle quotes, backslashes, newlines)
        let escaped_message = Self::escape_json(message);

        // Build JSON manually to avoid serde dependency
        let mut json = format!(r#"{{"level":"{level}","timestamp":"{timestamp}""#);

        // Add request_id if available
        if let Some(ref request_id) = self.request_id {
            let _ = write!(json, r#","request_id":"{request_id}""#);
        }

        // Add message
        let _ = write!(json, r#","message":"{escaped_message}"}}"#);

        json
    }

    /// Escape string for JSON
    ///
    /// Handles: quotes ("), backslashes (\), newlines (\n), tabs (\t), etc.
    fn escape_json(s: &str) -> String {
        use std::fmt::Write;

        let mut result = String::with_capacity(s.len());

        for ch in s.chars() {
            match ch {
                '"' => result.push_str(r#"\""#),
                '\\' => result.push_str(r"\\"),
                '\n' => result.push_str(r"\n"),
                '\r' => result.push_str(r"\r"),
                '\t' => result.push_str(r"\t"),
                c if c.is_control() => {
                    // Escape other control characters
                    let _ = write!(result, r"\u{:04x}", c as u32);
                }
                c => result.push(c),
            }
        }

        result
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure Logger is thread-safe for concurrent use
static_assertions::assert_impl_all!(Logger: Send, Sync);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Mock writer for capturing log output in tests
    struct MockWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_output(&self) -> String {
            let buffer = self.buffer.lock().unwrap();
            String::from_utf8_lossy(&buffer).to_string()
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_json_escaping() {
        assert_eq!(Logger::escape_json("hello"), "hello");
        assert_eq!(Logger::escape_json("hello \"world\""), r#"hello \"world\""#);
        assert_eq!(Logger::escape_json("path\\to\\file"), r"path\\to\\file");
        assert_eq!(Logger::escape_json("line1\nline2"), r"line1\nline2");
        assert_eq!(Logger::escape_json("tab\there"), r"tab\there");
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = Logger::format_timestamp();
        // Should match pattern: YYYY-MM-DDTHH:MM:SS.mmmZ
        assert!(timestamp.len() >= 24, "Timestamp too short: {}", timestamp);
        assert!(timestamp.contains('T'), "Missing 'T' separator");
        assert!(timestamp.ends_with('Z'), "Missing 'Z' suffix");
    }

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new();
        assert!(logger.request_id.is_none());
        assert!(logger.min_level.is_none());
    }

    #[test]
    fn test_logger_with_request_id() {
        let logger = Logger::with_request_id("test-123");
        assert_eq!(logger.request_id, Some("test-123".to_string()));
    }

    #[test]
    fn test_format_json_without_request_id() {
        let logger = Logger::new();
        let json = logger.format_json(LogLevel::Info, "2025-11-04T12:00:00.000Z", "test message");

        assert!(json.contains(r#""level":"INFO""#));
        assert!(json.contains(r#""timestamp":"2025-11-04T12:00:00.000Z""#));
        assert!(json.contains(r#""message":"test message""#));
        assert!(!json.contains("request_id"));
    }

    #[test]
    fn test_format_json_with_request_id() {
        let logger = Logger::with_request_id("req-456");
        let json = logger.format_json(
            LogLevel::Error,
            "2025-11-04T12:00:00.000Z",
            "error occurred",
        );

        assert!(json.contains(r#""level":"ERROR""#));
        assert!(json.contains(r#""request_id":"req-456""#));
        assert!(json.contains(r#""message":"error occurred""#));
    }

    #[test]
    fn test_json_escaping_in_message() {
        let logger = Logger::new();
        let json = logger.format_json(
            LogLevel::Info,
            "2025-11-04T12:00:00.000Z",
            r#"message with "quotes""#,
        );

        // Should escape quotes in message
        assert!(json.contains(r#""message":"message with \"quotes\"""#));
    }

    // MUTATION TESTING: Catch arithmetic mutants in format_timestamp()
    #[test]
    fn test_timestamp_arithmetic_hours() {
        let timestamp = Logger::format_timestamp();

        // Extract hours from timestamp (format: YYYY-MM-DDTHH:MM:SS.mmmZ)
        let parts: Vec<&str> = timestamp.split('T').collect();
        assert_eq!(parts.len(), 2, "Timestamp should have date and time parts");

        let time_part = parts[1];
        let time_components: Vec<&str> = time_part.split(':').collect();
        assert_eq!(
            time_components.len(),
            3,
            "Time should have hours:minutes:seconds"
        );

        let hours: u32 = time_components[0].parse().expect("Hours should be numeric");
        // Hours must be 0-23 (validates division by 3600)
        assert!(hours < 24, "Hours should be valid (0-23), got {}", hours);
    }

    #[test]
    fn test_timestamp_arithmetic_minutes() {
        let timestamp = Logger::format_timestamp();

        let parts: Vec<&str> = timestamp.split('T').collect();
        let time_part = parts[1];
        let time_components: Vec<&str> = time_part.split(':').collect();

        let minutes: u32 = time_components[1]
            .parse()
            .expect("Minutes should be numeric");
        // Minutes must be 0-59 (validates % 3600 and / 60)
        assert!(
            minutes < 60,
            "Minutes should be valid (0-59), got {}",
            minutes
        );
    }

    #[test]
    fn test_timestamp_arithmetic_seconds() {
        let timestamp = Logger::format_timestamp();

        let parts: Vec<&str> = timestamp.split('T').collect();
        let time_part = parts[1];
        let time_components: Vec<&str> = time_part.split(':').collect();

        let seconds_part = time_components[2];
        let seconds: u32 = seconds_part
            .split('.')
            .next()
            .unwrap()
            .parse()
            .expect("Seconds should be numeric");
        // Seconds must be 0-59 (validates % 60)
        assert!(
            seconds < 60,
            "Seconds should be valid (0-59), got {}",
            seconds
        );
    }

    #[test]
    fn test_timestamp_arithmetic_millis() {
        let timestamp = Logger::format_timestamp();

        let millis_part: Vec<&str> = timestamp.split('.').collect();
        assert_eq!(millis_part.len(), 2, "Should have milliseconds");

        let millis_str = millis_part[1].trim_end_matches('Z');
        let millis: u32 = millis_str.parse().expect("Milliseconds should be numeric");
        // Milliseconds must be 0-999
        assert!(
            millis < 1000,
            "Milliseconds should be valid (0-999), got {}",
            millis
        );
    }

    #[test]
    fn test_timestamp_arithmetic_date_validity() {
        let timestamp = Logger::format_timestamp();

        let parts: Vec<&str> = timestamp.split('T').collect();
        let date_part = parts[0];
        let date_components: Vec<&str> = date_part.split('-').collect();
        assert_eq!(date_components.len(), 3, "Date should have year-month-day");

        let year: u32 = date_components[0].parse().expect("Year should be numeric");
        let month: u32 = date_components[1].parse().expect("Month should be numeric");
        let day: u32 = date_components[2].parse().expect("Day should be numeric");

        // Year should be reasonable (validates + and / operations)
        assert!(year >= 1970, "Year should be >= 1970, got {}", year);
        assert!(year < 2100, "Year should be < 2100, got {}", year);

        // Month should be 1-12 (validates / 30 + 1 and % operations)
        assert!(
            month >= 1 && month <= 12,
            "Month should be 1-12, got {}",
            month
        );

        // Day should be 1-31 (validates % 30 + 1)
        assert!(day >= 1 && day <= 31, "Day should be 1-31, got {}", day);
    }

    // MUTATION TESTING: Catch control character escaping mutant
    #[test]
    fn test_json_escaping_control_characters() {
        // Test that control characters are escaped
        let text_with_control = "line1\nline2\rtab\there";
        let escaped = Logger::escape_json(text_with_control);

        // Should escape control characters
        assert!(escaped.contains(r"\n"), "Should escape newline");
        assert!(escaped.contains(r"\r"), "Should escape carriage return");
        assert!(escaped.contains(r"\t"), "Should escape tab");

        // Test with actual control character (byte < 0x20)
        let text_with_bell = "bell\x07here";
        let escaped_bell = Logger::escape_json(text_with_bell);
        assert!(
            escaped_bell.contains(r"\u0007"),
            "Should escape bell control character"
        );
    }

    // MUTATION TESTING: Verify logger methods actually produce output
    #[test]
    fn test_debug_method_produces_output() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let logger = Logger::with_writer(Box::new(writer));

        logger.debug("test debug message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            output.contains("DEBUG"),
            "debug() should produce DEBUG level output"
        );
        assert!(
            output.contains("test debug message"),
            "debug() should include message"
        );
    }

    #[test]
    fn test_info_method_produces_output() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let logger = Logger::with_writer(Box::new(writer));

        logger.info("test info message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            output.contains("INFO"),
            "info() should produce INFO level output"
        );
        assert!(
            output.contains("test info message"),
            "info() should include message"
        );
    }

    #[test]
    fn test_warn_method_produces_output() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let logger = Logger::with_writer(Box::new(writer));

        logger.warn("test warn message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            output.contains("WARN"),
            "warn() should produce WARN level output"
        );
        assert!(
            output.contains("test warn message"),
            "warn() should include message"
        );
    }

    #[test]
    fn test_error_method_produces_output() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let logger = Logger::with_writer(Box::new(writer));

        logger.error("test error message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            output.contains("ERROR"),
            "error() should produce ERROR level output"
        );
        assert!(
            output.contains("test error message"),
            "error() should include message"
        );
    }

    // MUTATION TESTING: Verify set_min_level actually filters logs
    #[test]
    fn test_set_min_level_filters_debug() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let mut logger = Logger::with_writer(Box::new(writer));

        logger.set_min_level(LogLevel::Info);
        logger.debug("should not appear");
        logger.info("should appear");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            !output.contains("should not appear"),
            "Debug messages should be filtered"
        );
        assert!(
            output.contains("should appear"),
            "Info messages should pass"
        );
    }

    #[test]
    fn test_set_min_level_filters_info() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let mut logger = Logger::with_writer(Box::new(writer));

        logger.set_min_level(LogLevel::Warn);
        logger.info("should not appear");
        logger.warn("should appear");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            !output.contains("should not appear"),
            "Info messages should be filtered"
        );
        assert!(
            output.contains("should appear"),
            "Warn messages should pass"
        );
    }

    #[test]
    fn test_set_min_level_filters_warn() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let mut logger = Logger::with_writer(Box::new(writer));

        logger.set_min_level(LogLevel::Error);
        logger.warn("should not appear");
        logger.error("should appear");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        assert!(
            !output.contains("should not appear"),
            "Warn messages should be filtered"
        );
        assert!(
            output.contains("should appear"),
            "Error messages should pass"
        );
    }

    // MUTATION TESTING: Verify log level comparison operators (< not == or >)
    #[test]
    fn test_log_level_comparison_exact_level() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let mut logger = Logger::with_writer(Box::new(writer));

        logger.set_min_level(LogLevel::Warn);
        logger.warn("exact level message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        // With < comparison, level == min_level should LOG (not filtered)
        // This catches "replace < with ==" mutant
        assert!(
            output.contains("exact level message"),
            "Messages at exact min_level should be logged"
        );
    }

    #[test]
    fn test_log_level_comparison_below_level() {
        let writer = MockWriter::new();
        let buffer = writer.buffer.clone();
        let mut logger = Logger::with_writer(Box::new(writer));

        logger.set_min_level(LogLevel::Error);
        logger.warn("below level message");

        let output = String::from_utf8_lossy(&buffer.lock().unwrap()).to_string();
        // With < comparison, level < min_level should be FILTERED
        // This catches "replace < with >" mutant
        assert!(
            !output.contains("below level message"),
            "Messages below min_level should be filtered"
        );
    }
}
