# Ruchy Lambda Runtime Architecture

**Technical Design Document** - v2.0.0 (Updated for v3.212.0)

## Table of Contents

1. [System Overview](#system-overview)
2. [Runtime Architecture](#runtime-architecture)
3. [Transpiler Integration](#transpiler)
4. [Bootstrap Process](#bootstrap)
5. [Lambda Runtime API](#lambda-runtime-api)
6. [Event Processing](#event-processing)
7. [Handler Interface](#handler-interface)
8. [Performance Optimizations](#performance-optimizations)
9. [Quality Assurance](#quality-assurance)

---

## System Overview

Ruchy Lambda is the **world's fastest custom AWS Lambda runtime**, combining the expressiveness of the Ruchy programming language with the performance of Rust. The system achieves industry-leading cold start times (**7.69ms best, 9.48ms average**) and minimal invocation overhead through aggressive binary size optimization.

### Architecture Goals

1. **Performance**: **✅ ACHIEVED** - 9.48ms average cold start (target: <10ms)
2. **Reliability**: **✅ ACHIEVED** - 100% success rate, comprehensive error handling
3. **Simplicity**: **✅ ACHIEVED** - Minimal dependencies, blocking I/O
4. **Quality**: **✅ ACHIEVED** - TDG A+, 85%+ test coverage, zero SATD

### Measured Performance (v3.212.0)

**Deployed on AWS Lambda us-east-1** (x86_64, 128MB memory):
- **Cold Start**: 9.48ms average, **7.69ms best** (5 measurements)
- **Binary Size**: 352KB (6x smaller than opt-level=3)
- **Memory Usage**: 14MB peak
- **Package Size**: 174-175KB (zipped)

**Comparison vs Other Runtimes**:
- **48% faster** than Rust (tokio): 14.90ms
- **73% faster** than C++ (AWS SDK): 28.96ms
- **83% faster** than Go: 56.49ms
- **91% faster** than Python 3.12: 85.73ms

### Key Components

```
┌─────────────────────────────────────────────────────────────┐
│                     AWS Lambda Environment                   │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                    Bootstrap Binary                     │ │
│  │                                                         │ │
│  │  ┌──────────────┐      ┌──────────────────────────┐   │ │
│  │  │ Runtime API  │ ───> │  Event Processing Loop   │   │ │
│  │  │   Client     │ <─── │  (Blocking I/O)          │   │ │
│  │  └──────────────┘      └──────────────────────────┘   │ │
│  │         │                          │                   │ │
│  │         │                          ▼                   │ │
│  │         │               ┌──────────────────────┐       │ │
│  │         │               │  Ruchy Handler       │       │ │
│  │         │               │  (Transpiled Rust)   │       │ │
│  │         │               └──────────────────────┘       │ │
│  │         │                          │                   │ │
│  │         ▼                          ▼                   │ │
│  │  ┌──────────────────────────────────────────────┐     │ │
│  │  │         CloudWatch Logs (stdout)             │     │ │
│  │  └──────────────────────────────────────────────┘     │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │          Lambda Runtime API (AWS-provided)             │ │
│  │  - /runtime/invocation/next  (GET - long poll)        │ │
│  │  - /runtime/invocation/{id}/response (POST)           │ │
│  │  - /runtime/invocation/{id}/error (POST)              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Runtime Architecture

### Crate Structure

The project is organized as a Cargo workspace with specialized crates:

```
ruchy-lambda/
├── crates/
│   ├── runtime/           # Lambda Runtime API client
│   ├── bootstrap/         # Custom runtime entry point
│   ├── profiler/          # Performance profiling tools
│   └── runtime-pure/      # Pure Rust runtime (no Ruchy)
├── examples/              # Example Ruchy handlers
├── scripts/               # Build and deployment scripts
└── docs/                  # Documentation and specifications
```

### Runtime Crate (`ruchy-lambda-runtime`)

**Purpose**: Minimal HTTP client for AWS Lambda Runtime API

**Key Components**:
1. **Runtime Struct**: Main API client
   - Manages endpoint configuration
   - Handles HTTP communication
   - Provides event/response interface

2. **HttpClient**: Minimal HTTP implementation
   - Blocking I/O (no async overhead)
   - TCP socket-based communication
   - Hand-crafted HTTP/1.1 requests

3. **Logger**: Structured CloudWatch logging
   - JSON-formatted output
   - Request ID context
   - Log level filtering
   - Zero external dependencies

**Design Decisions**:
- **Blocking I/O**: Lambda processes one event at a time, async adds overhead
- **No reqwest/hyper**: Minimal HTTP client saves ~180KB binary size
- **std only**: Production build uses only standard library + serde

**Dependencies** (production):
```toml
[dependencies]
serde = "1.0"
serde_json = "1.0"
```

### Bootstrap Crate (`ruchy-lambda-bootstrap`)

**Purpose**: Custom runtime entry point and event loop

**Responsibilities**:
1. Initialize runtime environment
2. Load transpiled Ruchy handler
3. Process events in infinite loop
4. Handle errors and logging

**Binary Size Optimization**:
- Release profile: `opt-level = 'z'` (size optimization)
- LTO: `fat` link-time optimization
- Strip: Remove debug symbols
- Codegen units: 1 (maximum optimization)
- Result: **400KB deployment package**

---

## Transpiler

### Build-Time Transpilation

The Ruchy transpiler converts `.ruchy` source files to Rust during `cargo build`:

**Process Flow**:
```
┌──────────────────┐
│ handler.ruchy    │ (Source)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   build.rs       │ (Runs during cargo build)
│  - Detects .ruchy│
│  - Calls Ruchy   │
│  - Generates .rs │
└────────┬─────────┘
         │
         ▼
┌──────────────────────────┐
│ handler_generated.rs     │ (Generated Rust)
└────────┬─────────────────┘
         │
         ▼
┌──────────────────┐
│   rustc          │ (Compiles to binary)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   bootstrap      │ (Executable)
└──────────────────┘
```

### Handler Selection

The build system supports multiple handlers via compile-time selection:

```bash
./scripts/build-lambda-package.sh minimal     # handler_minimal.ruchy
./scripts/build-lambda-package.sh fibonacci   # handler_fibonacci.ruchy
./scripts/build-lambda-package.sh default     # handler.ruchy
```

**Implementation** (`build.rs`):
1. Transpile all `.ruchy` files to `*_generated.rs`
2. Build script modifies `main.rs` `#[path = "..."]` attribute
3. Cargo compiles with selected handler
4. Result: Single binary with chosen handler

**CPU Compatibility**:
- Build target: `x86-64` (generic, no modern extensions)
- Ensures compatibility with AWS Lambda baseline CPU
- Avoids "illegal instruction" errors

---

## Bootstrap

### Initialization Phase (<1ms target)

**Sequence**:
1. Read environment variables (`AWS_LAMBDA_RUNTIME_API`)
2. Create `Runtime` instance (lazy initialization)
3. Initialize logger (if configured)
4. Enter event processing loop

**Lazy Initialization**:
- HTTP client created on first use
- Environment variables cached
- Result: ~200μs initialization time ✅

### Event Processing Loop

**Main Loop** (`main.rs:59`):
```rust
loop {
    if let Err(e) = process_single_event(&runtime) {
        eprintln!("[ERROR] Event processing failed: {e}");
        // Continue processing (don't exit on errors)
    }
}
```

**Error Handling Strategy**:
- Log errors to stderr
- Continue processing next event
- Prevent single-event failures from crashing runtime
- AWS Lambda monitors and restarts if needed

---

## Lambda Runtime API

### AWS Lambda Runtime API Integration

The runtime implements the [AWS Lambda Custom Runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html):

**Endpoints**:
1. **GET** `/runtime/invocation/next` - Long-polling for events
2. **POST** `/runtime/invocation/{id}/response` - Submit response
3. **POST** `/runtime/invocation/{id}/error` - Report errors

### Next Event Request

**HTTP Request**:
```http
GET /2018-06-01/runtime/invocation/next HTTP/1.1
Host: ${AWS_LAMBDA_RUNTIME_API}
```

**Response Headers** (critical):
```
Lambda-Runtime-Aws-Request-Id: 8476a536-e9f4-11e8-9739-2dfe598c3fcd
Lambda-Runtime-Deadline-Ms: 1542409706888
Lambda-Runtime-Invoked-Function-Arn: arn:aws:lambda:us-east-1:...
Lambda-Runtime-Trace-Id: Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700
```

**Response Body**: User event payload (JSON)

### Post Response Request

**HTTP Request**:
```http
POST /2018-06-01/runtime/invocation/{RequestId}/response HTTP/1.1
Host: ${AWS_LAMBDA_RUNTIME_API}
Content-Type: application/json
Content-Length: {length}

{
  "statusCode": 200,
  "body": "Hello from Ruchy Lambda!"
}
```

### Implementation Details

**Key Insight**: Request ID comes from **response headers**, not body

**Runtime.next_event()** (`runtime/src/lib.rs:140`):
```rust
pub fn next_event(&self) -> Result<(String, String), Error> {
    // 1. GET /runtime/invocation/next (long poll)
    let (headers, body) = self.client.get(&next_event_url)?;

    // 2. Extract request_id from Lambda-Runtime-Aws-Request-Id header
    let request_id = headers
        .iter()
        .find(|(k, _)| k == "lambda-runtime-aws-request-id")
        .map(|(_, v)| v.clone())
        .ok_or(Error::MissingRequestId)?;

    // 3. Return (request_id, event_body)
    Ok((request_id, body))
}
```

---

## Event Processing

### Single Event Flow

**Process** (`bootstrap/src/main.rs:76`):
```rust
fn process_single_event(runtime: &Runtime) -> Result<(), Box<dyn Error>> {
    // 1. Get next event (blocks until available)
    let (request_id, event_body) = runtime.next_event()?;

    // 2. Invoke Ruchy handler (transpiled Rust)
    let response = ruchy_handler(&request_id, &event_body);

    // 3. Post response back to Runtime API
    runtime.post_response(&request_id, &response)?;

    Ok(())
}
```

**Performance Characteristics**:
- **Long-polling**: `next_event()` blocks until event available
- **Zero-copy**: Event body passed as `&str` reference
- **Minimal overhead**: Direct function call to handler
- **Blocking I/O**: No async overhead, optimal for single-event processing

### Handler Invocation

**Handler Signature**:
```rust
fn ruchy_handler(request_id: &str, event_body: &str) -> String
```

**Example Handler** (transpiled from Ruchy):
```rust
pub fn lambda_handler(request_id: &str, body: &str) -> String {
    let message = "Hello from Ruchy Lambda!";
    format!("{{\"statusCode\":200,\"body\":\"{}\"}}", message)
}
```

**Invocation Overhead**: <100μs (measured) ✅

---

## Handler Interface

### Ruchy Handler Contract

**Required Signature**:
```ruchy
pub fun lambda_handler(request_id: &str, body: &str) -> String
```

**Parameters**:
1. `request_id`: Unique AWS request ID (for logging/tracing)
2. `body`: Raw event payload (JSON string)

**Return**: JSON response string with `statusCode` and `body`

### Example Handlers

**Minimal** (pure runtime overhead test):
```ruchy
pub fun lambda_handler(request_id: &str, body: &str) -> String {
    "{\"statusCode\":200,\"body\":\"ok\"}"
}
```

**Fibonacci** (CPU benchmark):
```ruchy
pub fun fibonacci(n: i32) -> i32 {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}

pub fun lambda_handler(request_id: &str, body: &str) -> String {
    let result = fibonacci(35);
    format!("{{\"statusCode\":200,\"body\":\"fibonacci(35)={}\"}}", result)
}
```

**Production** (full-featured):
```ruchy
pub fun lambda_handler(request_id: &str, body: &str) -> String {
    println("Processing request: {}", request_id);

    let message = if body.is_empty() {
        "Hello from Ruchy Lambda! (no body)"
    } else {
        "Hello from Ruchy Lambda!"
    };

    let response_body = String::from(message) + ". Request ID: " + request_id;
    String::from("{\"statusCode\":200,\"body\":\"") + &response_body + "\"}"
}
```

---

## Performance Optimizations

### Binary Size Optimization

**Techniques Applied**:
1. **Removed tokio**: Async runtime → Blocking I/O (-77KB)
2. **Removed reqwest**: HTTP client → Custom minimal client (-180KB)
3. **LTO**: Fat link-time optimization (eliminates dead code)
4. **opt-level = 'z'**: Size optimization over speed
5. **Strip symbols**: Remove debug information
6. **Minimal dependencies**: std + serde + serde_json only

**Results**:
- Original (with tokio + reqwest): ~2MB
- Phase 3 (blocking I/O): 317KB
- Phase 5 (with handlers): **400KB** ✅

### Cold Start Optimization

**Strategies**:
1. **Lazy initialization**: OnceCell for deferred setup
2. **Minimal startup**: No complex initialization logic
3. **Blocking I/O**: No async runtime overhead
4. **Small binary**: Faster loading from disk
5. **Generic CPU target**: No instruction cache misses

**Results**:
- Cold start: **2ms** (target: <8ms) ✅
- Initialization: ~200μs (target: <1ms) ✅

### Invocation Overhead Optimization

**Techniques**:
1. **Zero-copy**: Event body passed by reference
2. **Direct function call**: No dynamic dispatch
3. **Minimal HTTP overhead**: Hand-crafted requests
4. **Blocking I/O**: No async task spawning

**Results**:
- Invocation overhead: **<100μs** (measured) ✅

---

## Quality Assurance

### Testing Strategy

**Test Categories** (100+ tests total):
1. **Unit Tests** (45 tests): Core logic, logger, HTTP client
2. **Integration Tests** (35 tests): Mock Runtime API server
3. **Behavioral Tests** (12 tests): Trait implementations
4. **AWS Validation Tests** (11 tests): Real AWS Lambda deployment

**Test Isolation**:
- `#[serial]` attribute for tests using environment variables
- Prevents parallel execution conflicts
- Ensures consistent test results

### Mutation Testing

**Tool**: `cargo-mutants`

**Process**:
1. Introduce code mutations (change operators, replace methods, etc.)
2. Run test suite
3. Verify tests catch mutations
4. Calculate mutation score: caught / total

**Results**:
- Total mutants: 75
- Caught by tests: 65
- **Mutation score: 86.67%** (exceeds 85% target) ✅

**Uncaught Mutants**: Primarily in logger timestamp arithmetic (non-critical)

### Code Coverage

**Tool**: `cargo-tarpaulin`

**Results**:
- Lines covered: 161/176
- **Coverage: 91.48%** (exceeds 85% target) ✅

**Excluded**:
- Generated code (`*_generated.rs`)
- Test files
- Profiler tools

### Complexity Metrics

**Cyclomatic Complexity**: Max 5 (target: ≤15) ✅
**Cognitive Complexity**: Max 4 (target: ≤20) ✅

**Tools**: PMAT quality gates

### Toyota Way Quality Principles

1. **Kaizen** (Continuous Improvement): PMAT feedback loops
2. **Genchi Genbutsu** (Go and See): Evidence-based decisions via benchmarks
3. **Jidoka** (Automation with Human Touch): Pre-commit hooks, CI/CD
4. **Zero Defects**: TDG A+, zero SATD violations
5. **Andon Cord**: CI/CD fails on quality violations

---

## Deployment Architecture

### AWS Lambda Configuration

**Runtime**: `provided.al2023` (Amazon Linux 2023)
**Handler**: Not used (custom runtime)
**Architecture**: x86_64
**Memory**: 128MB (minimum, uses ~15MB actual)
**Timeout**: 3 seconds (default)

### Deployment Package

**Structure**:
```
lambda-package.zip
└── bootstrap  (executable, 400KB)
```

**Build Process**:
```bash
# 1. Transpile Ruchy → Rust
cargo build --release -p ruchy-lambda-bootstrap

# 2. Strip symbols
strip target/release/bootstrap

# 3. Package for Lambda
zip -j lambda-package.zip target/release/bootstrap
```

### Environment Variables

**Required**:
- `AWS_LAMBDA_RUNTIME_API`: Provided by AWS (e.g., `127.0.0.1:9001`)

**Optional**:
- `RUST_LOG`: Log level (debug, info, warn, error)

---

## Future Enhancements

### Planned Features (Phase 7+)

1. **ARM64 Support**: Graviton2 optimization
2. **Response Streaming**: Large payload support
3. **DataFrame Integration**: Polars for data processing
4. **X-Ray Tracing**: Distributed tracing support
5. **Memory Size Optimization**: Test across 128MB-1024MB configurations

### Research Areas

1. **PGO** (Profile-Guided Optimization): 5-15% performance improvement
2. **LLVM Bolt**: Post-link optimization
3. **Custom Allocator**: jemalloc or mimalloc evaluation
4. **Async I/O**: Evaluate for multi-request scenarios

---

## References

- [AWS Lambda Custom Runtime API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html)
- [Ruchy Language Specification](docs/transpiler-specification.md)
- [Performance Benchmarks](BENCHMARKS.md)
- [Development Roadmap](docs/execution/roadmap.md)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-04
**Status**: Phase 6 - Documentation & Release
