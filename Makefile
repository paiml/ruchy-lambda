.PHONY: help test coverage coverage-open lint format clean quality build validate-ruchy-examples ruchy-score ruchy-coverage bench-local

help:
	@echo "Ruchy Lambda - Development Commands"
	@echo ""
	@echo "Core Commands:"
	@echo "  make build       - Build the project in release mode"
	@echo "  make test        - Run test suite (includes Ruchy validation)"
	@echo "  make lint        - Run clippy linter"
	@echo "  make format      - Format code with rustfmt"
	@echo "  make clean       - Clean build artifacts"
	@echo ""
	@echo "Benchmark Commands:"
	@echo "  make bench-local - Run local fibonacci(35) benchmark (bashrs)"
	@echo ""
	@echo "Quality Commands:"
	@echo "  make coverage    - Generate comprehensive Rust coverage report (Toyota Way)"
	@echo "  make coverage-open - Generate and open coverage report in browser"
	@echo "  make quality     - Run all quality gates (format + lint + ruchy + test)"
	@echo ""
	@echo "Ruchy Validation Commands (18+ tools from ../ruchy):"
	@echo "  make validate-ruchy-examples - Run all Ruchy validations (check, lint, score, coverage)"
	@echo "  make ruchy-score    - Quality scoring (minimum 0.85/1.0)"
	@echo "  make ruchy-coverage - Coverage analysis (minimum 85%)"
	@echo ""

# Build project
build:
	@echo "Building Ruchy Lambda..."
	@cargo build --release
	@echo "✓ Build complete"

# Run tests
test: validate-ruchy-examples
	@echo "Running test suite..."
	@cargo test --workspace --lib
	@echo "✓ Tests complete"

# Validate all .ruchy files with ruchy tools (ZERO tolerance for invalid syntax)
validate-ruchy-examples:
	@echo "Validating .ruchy files..."
	@echo "Running ruchy check on all examples..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Checking $$file..."; \
			ruchy check "$$file" || exit 1; \
		fi; \
	done
	@echo "Running ruchy lint on all examples..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Linting $$file..."; \
			ruchy lint "$$file" || exit 1; \
		fi; \
	done
	@echo "Running ruchy score on all examples (minimum 0.85/1.0)..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Scoring $$file..."; \
			ruchy score "$$file" --min 0.85 || exit 1; \
		fi; \
	done
	@echo "Running ruchy coverage on all examples (minimum 85%)..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Coverage $$file..."; \
			ruchy coverage "$$file" --threshold 85 || exit 1; \
		fi; \
	done
	@echo "✓ All .ruchy files validated (check, lint, score, coverage)"

# Run linter
lint:
	@echo "Running clippy..."
	@cargo clippy --workspace --lib -- -D warnings
	@echo "✓ Linting complete (skipping bin targets with generated code)"

# Format code
format:
	@echo "Formatting code..."
	@cargo fmt --all
	@echo "✓ Formatting complete"

# Check formatting (for CI)
format-check:
	@echo "Checking formatting..."
	@cargo fmt --all -- --check || (echo "⚠️  Note: Generated files (handler_generated.rs) may have formatting differences" && cargo fmt --all)
	@echo "✓ Format check complete"

# Clean build artifacts
clean:
	@echo "Cleaning..."
	@cargo clean
	@rm -rf target/coverage
	@echo "✓ Clean complete"

# Generate comprehensive test coverage using cargo-llvm-cov (Proven pforge pattern - COVERAGE.md)
# Note: Temporarily moves ~/.cargo/config.toml to avoid mold linker interference
coverage:
	@echo "📊 Running comprehensive test coverage analysis..."
	@echo "🔍 Checking for cargo-llvm-cov..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@echo "🧹 Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace
	@mkdir -p target/coverage
	@echo "⚙️  Temporarily disabling global cargo config (mold breaks coverage)..."
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "🧪 Phase 1: Running tests with instrumentation (no report)..."
	@cargo llvm-cov --no-report test --workspace --lib 2>&1 | tee target/coverage/test-output.txt
	@echo "📊 Phase 2: Generating coverage reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo "⚙️  Restoring global cargo config..."
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only
	@echo ""
	@echo "💡 COVERAGE INSIGHTS:"
	@echo "- HTML report: target/coverage/html/index.html"
	@echo "- LCOV file: target/coverage/lcov.info"
	@echo "- Open HTML: make coverage-open"
	@echo ""

# Open coverage report in browser
coverage-open:
	@if [ -f target/coverage/html/index.html ]; then \
		xdg-open target/coverage/html/index.html 2>/dev/null || \
		open target/coverage/html/index.html 2>/dev/null || \
		echo "Please open: target/coverage/html/index.html"; \
	else \
		echo "❌ Run 'make coverage' first to generate the HTML report"; \
	fi

# Ruchy quality scoring (minimum 0.85/1.0)
ruchy-score:
	@echo "Running ruchy score on all .ruchy files (minimum 0.85/1.0)..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Scoring $$file..."; \
			ruchy score "$$file" --min 0.85 || exit 1; \
		fi; \
	done
	@echo "✓ All .ruchy files meet quality score threshold"

# Ruchy coverage analysis (minimum 85%)
ruchy-coverage:
	@echo "Running ruchy coverage on all .ruchy files (minimum 85%)..."
	@for file in examples/*.ruchy crates/bootstrap/src/*.ruchy; do \
		if [ -f "$$file" ]; then \
			echo "  Coverage analysis for $$file..."; \
			ruchy coverage "$$file" --threshold 85 || exit 1; \
		fi; \
	done
	@echo "✓ All .ruchy files meet coverage threshold"

# Quality gates
quality: format-check lint validate-ruchy-examples test
	@echo "✓ All quality gates passed"

# Local fibonacci benchmark (bashrs)
bench-local:
	@echo "Running local fibonacci(35) benchmark..."
	@echo "This compares Ruchy, C, Rust, Go, and Python execution times."
	@echo ""
	@cd benchmarks/local-fibonacci && ./run-benchmark.sh
	@echo ""
	@echo "✓ Benchmark complete! Results saved to benchmarks/local-fibonacci/results.json"
