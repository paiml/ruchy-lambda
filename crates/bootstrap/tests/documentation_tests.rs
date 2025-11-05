// Documentation Validation Tests
// Extreme TDD: Write tests FIRST (RED phase)
// Then write documentation to make tests pass (GREEN phase)
//
// These tests validate documentation quality:
// - README.md completeness
// - ARCHITECTURE.md existence
// - BENCHMARKS.md existence
// - Code examples compile
// - Links are valid
// - No broken references

use std::fs;
use std::path::{Path, PathBuf};

/// Get workspace root directory
fn workspace_root() -> PathBuf {
    // Tests run from crates/bootstrap/, so go up 2 levels to workspace root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Verify README.md exists and is comprehensive
#[test]
fn test_readme_exists_and_comprehensive() {
    let readme_path = workspace_root().join("README.md");
    assert!(
        readme_path.exists(),
        "README.md must exist at repository root"
    );

    let content = fs::read_to_string(&readme_path).expect("Failed to read README.md");

    // README must be substantial (>1000 bytes for comprehensive docs)
    assert!(
        content.len() > 1000,
        "README.md must be comprehensive (current: {} bytes, required: >1000)",
        content.len()
    );

    // README must contain key sections
    let required_sections = vec![
        "# Ruchy Lambda",
        "Features",
        "Quick Start",
        "Performance",
        "Installation",
        "Usage",
        "Architecture",
        "Benchmarks",
        "Examples",
        "Building",
        "Deployment",
        "Testing",
    ];

    for section in required_sections {
        assert!(
            content.contains(section),
            "README.md must contain '{}' section",
            section
        );
    }
}

/// Verify ARCHITECTURE.md exists and is comprehensive
#[test]
#[ignore] // Will pass after we write ARCHITECTURE.md
fn test_architecture_doc_exists() {
    let arch_path = workspace_root().join("ARCHITECTURE.md");
    assert!(
        arch_path.exists(),
        "ARCHITECTURE.md must exist at repository root"
    );

    let content = fs::read_to_string(&arch_path).expect("Failed to read ARCHITECTURE.md");

    assert!(
        content.len() > 2000,
        "ARCHITECTURE.md must be comprehensive (current: {} bytes, required: >2000)",
        content.len()
    );

    // Must document key architectural components
    let required_topics = vec![
        "Runtime Architecture",
        "Transpiler",
        "Bootstrap",
        "Lambda Runtime API",
        "Event Processing",
        "Handler Interface",
    ];

    for topic in required_topics {
        assert!(
            content.contains(topic),
            "ARCHITECTURE.md must document '{}'",
            topic
        );
    }
}

/// Verify BENCHMARKS.md exists with comprehensive performance data
#[test]
#[ignore] // Will pass after we write BENCHMARKS.md
fn test_benchmarks_doc_exists() {
    let bench_path = workspace_root().join("BENCHMARKS.md");
    assert!(
        bench_path.exists(),
        "BENCHMARKS.md must exist at repository root"
    );

    let content = fs::read_to_string(&bench_path).expect("Failed to read BENCHMARKS.md");

    assert!(
        content.len() > 1500,
        "BENCHMARKS.md must be comprehensive (current: {} bytes, required: >1500)",
        content.len()
    );

    // Must document performance metrics
    let required_metrics = vec![
        "Cold Start",
        "Invocation Time",
        "Memory Usage",
        "Binary Size",
        "vs C++",
        "vs Rust",
        "vs Go",
    ];

    for metric in required_metrics {
        assert!(
            content.contains(metric),
            "BENCHMARKS.md must document '{}'",
            metric
        );
    }
}

/// Verify example Ruchy handlers exist and are well-documented
#[test]
fn test_example_handlers_exist() {
    let examples = vec![
        "examples/simple_handler.ruchy",
        "examples/hello_world.ruchy",
        "examples/fibonacci.ruchy",
    ];

    for example in examples {
        let path = workspace_root().join(example);
        assert!(path.exists(), "Example handler '{}' must exist", example);

        let content = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !content.is_empty(),
            "Example handler '{}' must not be empty",
            example
        );

        // Examples should have comments/documentation
        assert!(
            content.contains("//") || content.contains("///"),
            "Example '{}' should have documentation comments",
            example
        );
    }
}

/// Verify production handlers are documented
#[test]
fn test_production_handlers_documented() {
    let handlers = vec![
        "crates/bootstrap/src/handler.ruchy",
        "crates/bootstrap/src/handler_minimal.ruchy",
        "crates/bootstrap/src/handler_fibonacci.ruchy",
    ];

    for handler in handlers {
        let path = workspace_root().join(handler);
        assert!(path.exists(), "Production handler '{}' must exist", handler);

        let content = fs::read_to_string(&path).expect(&format!("Failed to read {}", handler));

        // Production handlers must be well-documented
        assert!(
            content.contains("///"),
            "Production handler '{}' must have doc comments",
            handler
        );

        assert!(
            content.contains("# Arguments") || content.contains("# Returns"),
            "Production handler '{}' must document API",
            handler
        );
    }
}

/// Verify roadmap is up-to-date
#[test]
fn test_roadmap_current() {
    let roadmap_path = workspace_root().join("docs/execution/roadmap.md");
    assert!(roadmap_path.exists(), "Roadmap must exist");

    let content = fs::read_to_string(&roadmap_path).expect("Failed to read roadmap");

    // Phase 5 should be marked complete
    assert!(
        content.contains("Phase 5") && content.contains("COMPLETED"),
        "Roadmap must reflect Phase 5 completion"
    );

    // Should mention current phase (Phase 6)
    assert!(content.contains("Phase 6"), "Roadmap must document Phase 6");
}

/// Verify quality metrics are documented
#[test]
fn test_quality_metrics_documented() {
    let metrics_path = workspace_root().join("docs/RUCHY_QUALITY_METRICS.md");
    assert!(
        metrics_path.exists(),
        "Quality metrics documentation must exist"
    );

    let content = fs::read_to_string(&metrics_path).expect("Failed to read quality metrics");

    // Should document mutation score and test coverage
    assert!(
        content.contains("mutation") || content.contains("Mutation"),
        "Quality metrics must document mutation testing"
    );
}

/// Verify LICENSE file exists (required for open-source release)
#[test]
#[ignore] // Will pass after we add LICENSE
fn test_license_exists() {
    let license_path = workspace_root().join("LICENSE");
    assert!(
        license_path.exists(),
        "LICENSE file must exist for open-source release"
    );

    let content = fs::read_to_string(&license_path).expect("Failed to read LICENSE");
    assert!(
        content.len() > 100,
        "LICENSE must contain actual license text"
    );
}

/// Verify CONTRIBUTING.md exists (best practice for open-source)
#[test]
#[ignore] // Will pass after we add CONTRIBUTING.md
fn test_contributing_guide_exists() {
    let contributing_path = workspace_root().join("CONTRIBUTING.md");
    assert!(
        contributing_path.exists(),
        "CONTRIBUTING.md should exist for community contributions"
    );
}

#[cfg(test)]
mod example_compilation_tests {
    use std::process::Command;

    /// Verify example handlers can be transpiled
    #[test]
    #[ignore] // Requires Ruchy transpiler
    fn test_examples_transpile_successfully() {
        // This test would invoke the Ruchy transpiler on examples
        // and verify they produce valid Rust code
        // Deferred to when we have a standalone transpiler binary
    }
}
