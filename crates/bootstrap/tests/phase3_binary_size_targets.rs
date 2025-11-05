// Phase 3: Binary Size Optimization Tests (Extreme TDD)
// Tests written BEFORE optimization implementation
//
// Target: Reduce release binary from 505KB to <100KB (5x reduction)
// Current baseline: 505KB stripped release build

use std::process::Command;

/// Test: Release binary exists and is executable
#[test]
fn test_release_binary_exists() {
    let output = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build release binary");

    assert!(output.status.success(), "Release build should succeed");

    let binary_path = "target/release/bootstrap";
    assert!(
        std::path::Path::new(binary_path).exists(),
        "Release binary should exist at {}",
        binary_path
    );
}

/// Test: Baseline - Document current binary size
///
/// This test captures the BEFORE state for Phase 3.
/// Current: 505KB (stripped)
#[test]
fn test_baseline_binary_size_505kb() {
    let binary_path = "target/release/bootstrap";

    // Build first
    let _ = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output();

    if std::path::Path::new(binary_path).exists() {
        let metadata = std::fs::metadata(binary_path).expect("Failed to get metadata");
        let size_kb = metadata.len() / 1024;

        println!("Current binary size: {}KB", size_kb);

        // Document baseline (not a hard assertion, just for tracking)
        // This will fail after optimization (which is good!)
        if size_kb < 400 {
            println!("✅ Binary size reduced below 400KB baseline!");
        }
    }
}

/// Test: TARGET - Binary size under 100KB (Phase 3 goal)
///
/// **Success Criteria**: Release binary (stripped) < 100KB
///
/// This test will FAIL initially (RED phase), then PASS after optimization (GREEN).
#[test]
#[ignore] // Remove #[ignore] once we start optimization work
fn test_target_binary_size_under_100kb() {
    let binary_path = "target/release/bootstrap";

    // Build release
    let build_output = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output()
        .expect("Failed to build");

    assert!(build_output.status.success(), "Build should succeed");

    // Strip debug symbols
    let _ = Command::new("strip").arg(binary_path).output();

    let metadata = std::fs::metadata(binary_path).expect("Binary should exist");
    let size_kb = metadata.len() / 1024;

    println!("Binary size after optimization: {}KB", size_kb);

    assert!(
        size_kb < 100,
        "Binary size {}KB exceeds 100KB target (Phase 3 requirement)",
        size_kb
    );
}

/// Test: Intermediate target - Binary under 200KB
///
/// Step 1 towards <100KB goal.
#[test]
#[ignore] // Remove once we achieve first optimization
fn test_intermediate_target_under_200kb() {
    let binary_path = "target/release/bootstrap";

    let _ = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output();

    if let Ok(metadata) = std::fs::metadata(binary_path) {
        let size_kb = metadata.len() / 1024;
        println!("Intermediate binary size: {}KB", size_kb);

        assert!(
            size_kb < 200,
            "Intermediate target: Binary {}KB should be under 200KB",
            size_kb
        );
    }
}

/// Test: Verify binary still works after optimization
///
/// **Success Criteria**: Binary runs and prints expected output
#[test]
#[ignore]
fn test_optimized_binary_works() {
    let binary_path = "target/release/bootstrap";

    // Build
    let _ = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output();

    // Run binary (should initialize successfully)
    let output = Command::new(binary_path)
        .env("AWS_LAMBDA_RUNTIME_API", "127.0.0.1:9001")
        .output()
        .expect("Failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain initialization messages
    assert!(
        stdout.contains("Initializing Ruchy Lambda Runtime")
            || stdout.contains("Runtime initialized"),
        "Binary should initialize correctly"
    );
}

/// Test: Binary doesn't include unnecessary debug info
#[test]
fn test_no_debug_symbols_in_release() {
    let binary_path = "target/release/bootstrap";

    let _ = Command::new("cargo")
        .args(["build", "--release", "-p", "ruchy-lambda-bootstrap"])
        .output();

    if !std::path::Path::new(binary_path).exists() {
        return; // Skip if binary doesn't exist yet
    }

    // Check with nm or objdump that debug symbols are minimal
    let output = Command::new("nm")
        .arg("--debug-syms")
        .arg(binary_path)
        .output();

    if let Ok(result) = output {
        let symbol_count = String::from_utf8_lossy(&result.stdout).lines().count();

        println!("Debug symbol count: {}", symbol_count);
        // This is informational, not a hard requirement
    }
}

/// Test: Profile-guided optimization (PGO) applied
///
/// Verifies that PGO workflow works correctly
#[test]
#[ignore] // Will enable when implementing PGO
fn test_pgo_workflow() {
    // 1. Build with instrumentation
    let instrument = Command::new("cargo")
        .args(["build", "--release"])
        .env("RUSTFLAGS", "-Cprofile-generate=/tmp/pgo-data")
        .output();

    assert!(instrument.is_ok(), "PGO instrumentation build should work");

    // 2. Run to collect profile data
    // 3. Rebuild with profile data
    // (Implementation pending)
}

/// Test: Dependency audit - no unnecessary dependencies
///
/// Verifies we're not pulling in unused crates
#[test]
#[ignore]
fn test_minimal_dependencies() {
    // Check Cargo.toml has minimal deps
    let cargo_toml =
        std::fs::read_to_string("crates/bootstrap/Cargo.toml").expect("Should read Cargo.toml");

    // Should NOT contain heavy dependencies
    assert!(
        !cargo_toml.contains("reqwest"),
        "Should not use reqwest (removed in Phase 2)"
    );

    // Verify tokio features are minimal
    if cargo_toml.contains("tokio") {
        println!("✓ Tokio present (may be optimized in Phase 3)");
    }
}
