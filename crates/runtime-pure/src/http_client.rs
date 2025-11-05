// Rust HTTP client for Pure Ruchy runtime
// This module is imported by lib.ruchy to avoid parser limitations

use std::io::{self, Read, Write};
use std::net::TcpStream;

/// Make HTTP GET request and return (request_id, body)
pub fn http_get(endpoint: &str, path: &str) -> Result<(String, String), String> {
    let mut stream = TcpStream::connect(endpoint)
        .map_err(|e| format!("Connection failed: {}", e))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, endpoint
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Write failed: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Flush failed: {}", e))?;

    let mut buffer = Vec::new();
    stream
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Read failed: {}", e))?;

    let response = String::from_utf8_lossy(&buffer).to_string();
    parse_response(&response)
}

/// Make HTTP POST request
pub fn http_post(endpoint: &str, path: &str, body: &str) -> Result<(), String> {
    let mut stream = TcpStream::connect(endpoint)
        .map_err(|e| format!("Connection failed: {}", e))?;

    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, endpoint, body.len(), body
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Write failed: {}", e))?;

    stream
        .flush()
        .map_err(|e| format!("Flush failed: {}", e))?;

    let mut buffer = vec![0u8; 1024];
    let n = stream
        .read(&mut buffer)
        .map_err(|e| format!("Read failed: {}", e))?;

    let response = String::from_utf8_lossy(&buffer[..n]).to_string();

    if response.contains("HTTP/1.1 2") {
        Ok(())
    } else {
        Err(format!("POST failed: {}", response.lines().next().unwrap_or("unknown")))
    }
}

/// Parse HTTP response to extract request_id header and body
fn parse_response(response: &str) -> Result<(String, String), String> {
    let mut request_id = String::new();
    let mut body_start = 0;

    let lines: Vec<&str> = response.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("Lambda-Runtime-Aws-Request-Id:") {
            if let Some(id) = line.split(':').nth(1) {
                request_id = id.trim().to_string();
            }
        }

        if line.is_empty() {
            body_start = i + 1;
            break;
        }
    }

    if request_id.is_empty() {
        request_id = String::from("unknown-request-id");
    }

    let body = if body_start < lines.len() {
        lines[body_start..].join("\n")
    } else {
        String::from("{}")
    };

    Ok((request_id, body))
}
