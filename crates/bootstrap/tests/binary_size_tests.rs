// Extreme TDD: Binary Size Tests
// Written FIRST before implementation
// Target: <100KB binary size (Section 12.2 of specification)

use std::fs;
use std::path::Path;
use std::process::Command;

/// Get the size of the bootstrap binary in bytes
fn get_binary_size(profile: &str) -> Option<u64> {
    let binary_path = if profile == "release-ultra" {
        "../../target/release-ultra/bootstrap"
    } else {
        &format!("../../target/{}/bootstrap", profile)
    };

    let path = Path::new(binary_path);
    if path.exists() {
        fs::metadata(path).ok().map(|m| m.len())
    } else {
        None
    }
}

/// Test: Release-ultra binary should be under 100KB
#[test]
#[ignore] // Run explicitly: cargo test --release -- --ignored
fn test_release_ultra_binary_size_under_100kb() {
    // Build with release-ultra profile
    let output = Command::new("cargo")
        .args([
            "build",
            "--profile",
            "release-ultra",
            "-p",
            "ruchy-lambda-bootstrap",
        ])
        .output()
        .expect("Failed to build release-ultra binary");

    assert!(
        output.status.success(),
        "Build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Measure binary size
    let size_bytes = get_binary_size("release-ultra").expect("Binary not found after build");

    let size_kb = size_bytes / 1024;

    println!("Binary size: {} bytes ({} KB)", size_bytes, size_kb);

    // Assert under 100KB target (Section 12.2)
    assert!(
        size_kb < 100,
        "Binary size {}KB exceeds 100KB target (specification Section 12.2)",
        size_kb
    );
}

/// Test: Binary size regression tracking
#[test]
#[ignore] // Run explicitly for benchmarking
fn test_binary_size_regression_tracking() {
    // This test tracks binary size over time
    // Baseline will be established in Phase 2

    let size_bytes = get_binary_size("release-ultra");

    if let Some(size) = size_bytes {
        let size_kb = size / 1024;
        println!("Current binary size: {} KB", size_kb);

        // Future: Compare against baseline stored in .pmat-gates.toml
        // assert!(size_kb <= baseline_kb, "Binary size regression detected");
    } else {
        println!("Binary not built yet - run cargo build --profile release-ultra first");
    }
}

/// Test: Debug build should compile (no size requirements)
#[test]
fn test_debug_binary_compiles() {
    let output = Command::new("cargo")
        .args(["build", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build debug binary");

    assert!(
        output.status.success(),
        "Debug build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Debug binary exists (no size requirement)
    let debug_exists = get_binary_size("debug").is_some();
    assert!(
        debug_exists,
        "Debug binary not found after successful build"
    );
}

/// Test: Release build should be smaller than debug
#[test]
#[ignore] // Run explicitly
fn test_release_smaller_than_debug() {
    // Build both profiles
    Command::new("cargo")
        .args(["build", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build debug");

    Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build release");

    let debug_size = get_binary_size("debug").expect("Debug binary not found");
    let release_size = get_binary_size("release").expect("Release binary not found");

    println!("Debug size: {} KB", debug_size / 1024);
    println!("Release size: {} KB", release_size / 1024);

    assert!(
        release_size < debug_size,
        "Release binary ({} KB) should be smaller than debug ({} KB)",
        release_size / 1024,
        debug_size / 1024
    );
}

/// Test: Strip reduces binary size significantly
#[test]
#[ignore] // Run explicitly
fn test_strip_reduces_binary_size() {
    // Build release binary
    Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build release");

    let before_strip = get_binary_size("release").expect("Release binary not found");

    // Strip the binary
    let binary_path = "target/release/bootstrap";
    let output = Command::new("strip")
        .arg(binary_path)
        .output()
        .expect("Failed to run strip");

    assert!(output.status.success(), "strip command failed");

    let after_strip = get_binary_size("release").expect("Binary not found after strip");

    println!("Before strip: {} KB", before_strip / 1024);
    println!("After strip: {} KB", after_strip / 1024);

    let reduction_percent = ((before_strip - after_strip) as f64 / before_strip as f64) * 100.0;

    println!("Size reduction: {:.1}%", reduction_percent);

    assert!(
        after_strip < before_strip,
        "Strip should reduce binary size"
    );

    // Stripping typically reduces size by 20-40%
    assert!(
        reduction_percent > 10.0,
        "Strip should reduce size by at least 10% (got {:.1}%)",
        reduction_percent
    );
}

/// Test: Binary size breakdown (dependencies contribution)
#[test]
#[ignore] // Run explicitly with cargo-bloat
fn test_binary_size_breakdown() {
    // This test uses cargo-bloat to analyze size
    // Install: cargo install cargo-bloat

    let output = Command::new("cargo")
        .args([
            "bloat",
            "--release",
            "-p",
            "ruchy-lambda-bootstrap",
            "--crates",
        ])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let analysis = String::from_utf8_lossy(&result.stdout);
            println!("Binary size breakdown:\n{}", analysis);

            // Future: Assert that no single dependency contributes >30% of size
        } else {
            println!("cargo-bloat not installed or failed");
            println!("Install with: cargo install cargo-bloat");
        }
    }
}

/// Test: Optimize binary with UPX compression
#[test]
#[ignore] // Run explicitly, requires upx installed
fn test_upx_compression() {
    // Build release-ultra first
    Command::new("cargo")
        .args([
            "build",
            "--profile",
            "release-ultra",
            "-p",
            "ruchy-lambda-bootstrap",
        ])
        .output()
        .expect("Failed to build");

    let before_upx = get_binary_size("release-ultra").expect("Binary not found");

    // Try to compress with UPX
    let binary_path = "target/release-ultra/bootstrap";

    // Make a copy to test UPX (don't corrupt original)
    let upx_test_path = "target/release-ultra/bootstrap.upx";
    fs::copy(binary_path, upx_test_path).expect("Failed to copy binary");

    let output = Command::new("upx")
        .args(["--best", "--lzma", upx_test_path])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let after_upx = fs::metadata(upx_test_path)
                .expect("UPX binary not found")
                .len();

            println!("Before UPX: {} KB", before_upx / 1024);
            println!("After UPX: {} KB", after_upx / 1024);

            let compression_ratio = (after_upx as f64 / before_upx as f64) * 100.0;
            println!("Compression ratio: {:.1}%", compression_ratio);

            // UPX typically achieves 40-60% compression
            assert!(after_upx < before_upx, "UPX should reduce binary size");

            // Clean up test file
            fs::remove_file(upx_test_path).ok();
        } else {
            println!("UPX not installed or failed");
            println!("Install with: apt-get install upx (Linux) or brew install upx (macOS)");
        }
    }
}
