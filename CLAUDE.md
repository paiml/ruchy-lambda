# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

**Ruchy Lambda** is a research project to build the **world's fastest AWS Lambda runtime**, targeting <8ms cold start times to beat current leaders (C++ at 10-16ms, Rust at 11-17ms). This is achieved by transpiling Ruchy (a high-level language) to Rust, then applying aggressive optimizations.

**Current Status**: **Specification Phase** - No implementation code yet. The comprehensive 1,700+ line specification is in `docs/specification/ruchy-compiled-transpiled-fast-lambda-in-world-spec.md` (v3.0.0, peer-reviewed with 14 scientific papers).

**Key Performance Target**: <8ms cold start (27% faster than current fastest runtime)

---

## Development Workflow

**CRITICAL**: Always work on `main` branch. No branching strategy.

```bash
git pull origin main
# Make changes
git add .
git commit -m "description"
git push origin main
```

### Crates.io Release Policy (MANDATORY - BLOCKING)

**Friday-Only Releases**: ALL crates.io releases MUST occur on Friday ONLY.

**Rationale**:
- Stability: Full week of testing and validation before release
- Recovery time: Weekend buffer if issues discovered
- Quality gates: All quality checks pass by end of week
- Predictability: Users expect stable weekly cadence

**Release Checklist** (Run on Friday ONLY):
```bash
# 1. Verify all quality gates pass
make quality                          # PMAT + TDG checks
cargo test --all                      # All tests passing
cargo clippy -- -D warnings           # Zero warnings

# 2. Verify version numbers updated
grep "^version" crates/*/Cargo.toml   # Check all crate versions
git tag -l                            # Check existing tags

# 3. Create git tag
git tag -a v0.1.0 -m "Release v0.1.0: Description"
git push origin v0.1.0

# 4. Publish to crates.io (in dependency order)
cd crates/profiler && cargo publish   # Publish standalone crates first
cd ../bootstrap && cargo publish      # Then dependent crates

# 5. Verify publication
cargo search ruchy-lambda-profiler
cargo search ruchy-lambda-bootstrap
```

**Enforcement**:
- ❌ NO releases Monday-Thursday
- ✅ Releases ONLY on Friday
- ❌ NO emergency releases (fix in main, wait for Friday)
- ✅ ALL releases MUST have passing quality gates

**Version Management**:
- Workspace version in root `Cargo.toml`
- All crates inherit version via `version.workspace = true`
- Semantic versioning: MAJOR.MINOR.PATCH
- Pre-1.0: Breaking changes increment MINOR

---

## Core Architecture (From Specification)

### Three-Phase Compilation Pipeline

```
Ruchy Source (.ruchy)
    ↓ [PMAT Quality Gate: TDG, complexity, SATD]
Ruchy Transpiler (from ../ruchy v3.182.0+)
    ↓ [Generates idiomatic Rust with optimizations]
Rust Source Code
    ↓ [PMAT Quality Gate: dead code, mutation testing]
rustc (release-ultra profile: LTO, PGO, opt-level='z')
    ↓ [Compile for ARM64 Graviton2]
bootstrap executable (<100KB target)
    ↓ [PMAT Quality Gate: binary size, performance]
AWS Lambda (provided.al2023 runtime)
```

### Key Optimization Strategies

1. **Ruchy Transpiler Optimizations**:
   - Dead code elimination (liveness analysis)
   - Constant folding (compile-time evaluation)
   - Function inlining (complexity-guided, <10 complexity)
   - Escape analysis (stack vs heap allocation)

2. **Rust Compilation** (`release-ultra` profile):
   - `opt-level = 'z'` (size optimization)
   - `lto = "fat"` (link-time optimization)
   - `codegen-units = 1` (maximum optimization)
   - `panic = 'abort'` (no unwinding overhead)
   - PGO (Profile-Guided Optimization) with 100K+ invocation profiling

3. **Zero-Copy Deserialization** (3-tier strategy):
   - Tier 1: `serde_json` with borrowed references (40-60% allocation reduction)
   - Tier 2: Binary formats (FlatBuffers, Cap'n Proto) - 3-5x faster
   - Tier 3: Memory-mapped I/O for large payloads (>1MB)

4. **ARM64 Graviton2 Targeting**:
   - `target-cpu=neoverse-n1` optimization
   - 5-10% faster cold starts vs x86_64
   - Lower performance variance (<50% vs x86)

---

## Quality Standards (ZERO Tolerance)

**Quality Enforcement**: PMAT (Pragmatic AI Labs MCP Agent Toolkit) v2.192.0+

### Required Quality Metrics

| Metric | Requirement | Enforced By |
|--------|-------------|-------------|
| **TDG Grade** | ≥A (85/100), target A+ | `pmat tdg check-quality` |
| **Test Coverage** | ≥85% | `cargo llvm-cov` |
| **Mutation Score** | ≥85% | `pmat mutate --threshold 85` |
| **Cyclomatic Complexity** | ≤15 per function | `pmat analyze complexity` |
| **Cognitive Complexity** | ≤20 per function | `pmat analyze complexity` |
| **SATD** | 0 (TODO/FIXME/HACK) | `pmat analyze satd` |
| **Dead Code** | 0% | `pmat analyze dead-code` |
| **Clippy Warnings** | 0 | `cargo clippy -- -D warnings` |

### Development Commands (When Code Exists)

**Setup** (Phase 0):
```bash
# Install PMAT
cargo install pmat

# Initialize quality infrastructure
pmat hooks install
pmat tdg baseline create --output .pmat/tdg-baseline.json --path src/

# Install Rust toolchain
rustup target add aarch64-unknown-linux-musl
```

**Quality Gates** (run before every commit):
```bash
# All quality checks
make quality                # PMAT quality-gate + TDG regression check

# Individual checks
pmat quality-gate --strict --fail-on-violation
pmat tdg check-regression --baseline .pmat/tdg-baseline.json
pmat mutate --target src/ --threshold 85
pmat validate-readme --targets README.md ARCHITECTURE.md
cargo clippy -- -D warnings
cargo fmt --check
```

**Testing**:
```bash
# Unit tests
cargo test --all

# With coverage
cargo llvm-cov --all --lcov --output-path lcov.info

# Mutation testing (critical code: 95% score)
pmat mutate --target src/bootstrap.rs --threshold 95
```

**Building**:
```bash
# Development build
cargo build

# Optimized build (ARM64)
cargo build --profile release-ultra --target aarch64-unknown-linux-musl

# Binary size check (must be <100KB)
ls -lh target/aarch64-unknown-linux-musl/release-ultra/bootstrap
```

**PERF-002 Features** (Ruchy v3.211.0+):
```bash
# Show profile characteristics before compilation
ruchy compile handler_fibonacci.ruchy --optimize nasa --show-profile-info

# Profile-Guided Optimization (25-50× speedup for CPU-intensive Lambda functions)
ruchy compile handler_fibonacci.ruchy -o bootstrap --pgo
# → Builds profiled binary, prompts for workload, builds optimized binary

# PGO Benefits for Lambda:
# - Optimized for actual usage patterns (not synthetic benchmarks)
# - Native CPU instruction targeting (-C target-cpu=native)
# - Best for compute-heavy functions (fibonacci, image processing, crypto)
```

**Quality Analysis**:
```bash
# Technical debt analysis
pmat analyze tdg src/ --with-git-context --format table

# Complexity analysis
pmat analyze complexity --language rust --path src/ \
  --max-cyclomatic 15 --max-cognitive 20 --fail-on-violation

# Dead code detection
pmat analyze dead-code --path src/ --suggest-removal

# Cost of Quality report
pmat analyze cost-of-quality --period monthly --format report
```

---

## Toyota Way Principles

This project follows **Toyota Production System (TPS)** principles:

1. **Kaizen (改善)**: Continuous improvement
   - Track TDG scores over time
   - Iterative optimization based on benchmarks

2. **Genchi Genbutsu (現地現物)**: Go and see for yourself
   - All performance claims backed by peer-reviewed research
   - Empirical benchmarking via lambda-perf methodology

3. **Jidoka (自働化)**: Built-in quality
   - Automated quality gates (pre-commit hooks)
   - Stop-the-line mentality (fail builds on violations)

4. **Zero Defects**: Proactive quality
   - Zero SATD policy
   - Zero dead code
   - 100% test pass rate

5. **Andon Cord**: Pull to stop the line
   - Pre-commit hooks prevent bad commits
   - CI/CD blocks merges on quality violations

---

## Related Codebases

**Ruchy Ecosystem** (sibling directories):
- **../ruchy**: Main Ruchy compiler (v3.182.0+, 4,031 tests)
  - Use `ruchy transpile` to generate Rust from Ruchy source
- **../ruchy-book**: Documentation (100% working examples, scientific benchmarks)
  - Chapter 21: Scientific benchmarking (Ruchy achieves 82% of C performance)
- **../ruchyruchy**: JIT compiler + debugging tools (v1.26.0, 1,257 tests)
  - Cranelift JIT (10-100x speedup for hot functions)
  - 10+ debugging tools for parser/JIT analysis

**Quality Infrastructure**:
- **../paiml-mcp-agent-toolkit**: PMAT quality enforcement toolkit
  - 19 MCP tools for AI-assisted quality analysis
  - Technical Debt Grading (6 orthogonal metrics)
  - Mutation testing (Rust, Python, TypeScript, Go, C++)
  - Documentation validation via Semantic Entropy (Farquhar et al., Nature 2024)

**Performance Benchmarking**:
- **../lambda-perf**: AWS Lambda cold start benchmarks
  - 40 runtime configurations tested
  - 10 invocations per test (forced cold starts)
  - Baseline: C++ 10-16ms, Rust 11-17ms, Go 38-50ms

---

## Specification Document

**PRIMARY REFERENCE**: `docs/specification/ruchy-compiled-transpiled-fast-lambda-in-world-spec.md`

This 1,700+ line document is the **source of truth** for all architecture decisions. Key sections:

- **Section 2**: Three-phase compilation strategy (transpilation → rustc → binary)
- **Section 3**: Custom runtime bootstrap design (<1ms initialization target)
- **Section 3.3**: Zero-copy deserialization (3-tier strategy, 3x throughput gain)
- **Section 5**: ARM64 Graviton2 optimizations
- **Section 6**: Binary size optimization (<100KB target)
- **Section 7**: PMAT quality enforcement (TDG, mutation testing, CoQ tracking)
- **Section 8**: Benchmark methodology (lambda-perf style, 10 invocations)
- **Section 9**: Implementation roadmap (12-week timeline, Phase 0-6)
- **Appendix A**: 14 peer-reviewed papers validating all claims
- **Appendix B**: Cargo.toml configuration (release-ultra profile)
- **Appendix C**: PMAT configuration (.pmat-gates.toml, pmat-quality.toml, Makefile)

**When making architecture decisions, always reference the specification first.**

---

## Key Performance Claims (Peer-Reviewed)

All claims validated by scientific research (see Appendix A of specification):

1. **<100KB binary → faster cold start**
   - Research: Balasubrahmanya (2023) - <3MB reduces init by 41%

2. **ARM64 advantage**
   - Research: Chen (2023) - 15/18 benchmarks more cost-effective
   - Research: Lambion (2022) - <50% runtime variance vs x86

3. **Zero-copy 3x speedup**
   - Research: Björck et al. (2021) - Zerializer achieves 3x throughput

4. **PGO effectiveness**
   - Expected: 5-10% instruction cache improvement, 3-7% branch prediction

5. **Ruchy transpilation performance**
   - Validated: 82% of C performance (15.12x faster than Python)

---

## Cost of Quality (CoQ) Framework

Track quality investment vs. defect prevention:

| Quality Gate | Time Investment | Defects Prevented | ROI |
|--------------|----------------|-------------------|-----|
| Pre-commit hooks | ~30 sec/commit | SATD, formatting | 20:1 |
| Mutation testing | ~5 min/run | Weak tests, logic errors | 50:1 |
| TDG regression checks | ~1 min/run | Technical debt | 100:1 |
| Documentation validation | ~2 min/run | Stale docs, hallucinations | 200:1 |

**Expected ROI**: For every $1 spent on prevention, save $4-10 in failure costs (Crosby, 1979).

---

## Implementation Timeline (12 Weeks)

**Phase 0** (Week 1): Quality Infrastructure Setup
- Install PMAT, configure hooks, create baseline

**Phase 1** (Week 2-3): Foundation
- Minimal bootstrap with AWS Lambda Runtime API
- "Hello world" Ruchy function

**Phase 2** (Week 4-5): Transpilation Pipeline
- Integrate `ruchy transpile`
- Configure `release-ultra` profile
- Mutation testing ≥85%

**Phase 3** (Week 6-7): Optimization
- PGO workflow (100K+ invocation profiling)
- ARM64 optimizations
- Zero dead code enforcement

**Phase 4** (Week 8-9): Advanced Features
- DataFrame support (Polars integration)
- Response streaming
- X-Ray distributed tracing

**Phase 5** (Week 10-11): Testing & Validation
- Lambda-perf benchmarking (10 invocations per config)
- Test across memory sizes (128MB, 256MB, 512MB, 1024MB)
- Final TDG grade ≥A (target: A+)

**Phase 6** (Week 12): Documentation & Release
- Comprehensive documentation (README, ARCHITECTURE, BENCHMARKS)
- Documentation hallucination validation
- Open-source release

---

## Common Pitfalls to Avoid

1. **DO NOT commit code with TODO/FIXME/HACK comments** (enforced via pre-commit hooks)
2. **DO NOT bypass quality gates** (`--no-verify` is forbidden)
3. **DO NOT create branches** (always work on `main`)
4. **DO NOT skip mutation testing** (85%+ score required, 95% for critical code)
5. **DO NOT exceed complexity limits** (cyclomatic ≤15, cognitive ≤20)
6. **DO NOT commit if TDG grade drops** (regression check enforced)
7. **DO NOT write documentation without code** (zero vaporware policy)

---

## Semantic Entropy for Documentation Validation

PMAT uses **Semantic Entropy** (Farquhar et al., Nature 2024) to detect hallucinations:

- **Low entropy** (<0.5): Claim is consistent and well-supported
- **High entropy** (>2.0): Claim is contradictory (likely hallucination)

Example validation:
```bash
# Generate codebase context (ground truth)
pmat context --output deep_context.md --format llm-optimized

# Validate README (semantic entropy analysis)
pmat validate-readme \
  --targets README.md ARCHITECTURE.md \
  --deep-context deep_context.md \
  --fail-on-contradiction \
  --semantic-threshold 1.5
```

---

## Success Criteria

| Metric | Target | Stretch Goal | Measurement |
|--------|--------|--------------|-------------|
| **Avg Cold Start** | <10ms | <8ms | Mean of 10 invocations |
| **P50 Cold Start** | <9ms | <7ms | Median latency |
| **P99 Cold Start** | <15ms | <12ms | Tail latency |
| **Binary Size** | <100KB | <50KB | `ls -lh bootstrap` |
| **TDG Grade** | ≥A | A+ | PMAT analysis |
| **Test Coverage** | ≥85% | ≥90% | `cargo llvm-cov` |
| **Mutation Score** | ≥85% | ≥90% | `pmat mutate` |

---

## Questions & Context

For questions about architecture or implementation decisions:
1. **First**, consult the specification: `docs/specification/ruchy-compiled-transpiled-fast-lambda-in-world-spec.md`
2. **Second**, check related codebases: `../ruchy`, `../ruchy-book`, `../paiml-mcp-agent-toolkit`
3. **Third**, verify with peer-reviewed research (14 papers in Appendix A)

For questions about Ruchy language syntax or features:
- See `../ruchy-book` (100% working examples, Chapters 1-23)
- Reference `../ruchy` source code (v3.182.0+)

For questions about quality enforcement:
- See `../paiml-mcp-agent-toolkit` documentation
- PMAT book: https://paiml.github.io/pmat-book/
