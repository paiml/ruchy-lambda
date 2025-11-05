// LAMBDA-PROF: Ruchy Lambda Performance Profiler
//
// GREEN PHASE: Real AWS Lambda Measurements (NO simulation)
//
// Status: Implementing real measurement infrastructure
// Measurements: AWS Lambda Runtime API (x-amz-* headers)
// Memory: jemalloc profiling
// Goal: Make Ruchy the fastest Lambda runtime in the world
// Target: <8ms cold start (beat C++ 13ms, Rust 17ms, Go 46ms)
//
// Features:
// - Measure cold start latency (10 invocations, lambda-perf methodology)
// - Profile memory usage (peak RSS, allocations)
// - Track binary size contribution
// - Generate lambda-perf compatible JSON reports
// - Compare against fastest runtimes (C++, Rust, Go, Swift)

pub mod real_measurement;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Ruchy Lambda Performance Profiler
#[derive(Parser)]
#[command(name = "profiler")]
#[command(about = "Profile Ruchy Lambda cold start performance")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run cold start benchmark (10 invocations)
    Benchmark {
        /// Lambda function name
        #[arg(short, long)]
        function: String,

        /// Memory size in MB
        #[arg(short, long, default_value = "128")]
        memory: u64,

        /// Architecture (x86_64 or arm64)
        #[arg(short, long, default_value = "x86_64")]
        arch: String,

        /// Output file (JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compare against fastest runtimes
    Compare {
        /// Benchmark results file
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Generate lambda-perf compatible report
    Report {
        /// Benchmark results file
        #[arg(short, long)]
        input: PathBuf,

        /// Output file (lambda-perf JSON format)
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Profile memory usage
    Memory {
        /// Binary path
        #[arg(short, long)]
        binary: PathBuf,
    },
}

/// Performance metrics from a single cold start
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ColdStartMeasurement {
    /// Init duration (ms)
    init_ms: f64,
    /// Handler duration (ms)
    handler_ms: f64,
    /// Total duration (ms)
    total_ms: f64,
    /// Peak memory (KB)
    memory_kb: u64,
    /// Timestamp
    timestamp: u64,
}

/// Benchmark results (10 invocations)
#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResults {
    /// Runtime name
    runtime: String,
    /// Memory size (MB)
    memory_mb: u64,
    /// Architecture
    arch: String,
    /// All measurements
    measurements: Vec<ColdStartMeasurement>,
    /// Statistics
    stats: Statistics,
    /// Binary info
    binary: BinaryInfo,
}

/// Statistical summary
#[derive(Debug, Serialize, Deserialize)]
struct Statistics {
    /// Average cold start (ms)
    avg_ms: f64,
    /// P50 latency (ms)
    p50_ms: f64,
    /// P99 latency (ms)
    p99_ms: f64,
    /// Min latency (ms)
    min_ms: f64,
    /// Max latency (ms)
    max_ms: f64,
    /// Standard deviation
    stddev_ms: f64,
}

/// Binary information
#[derive(Debug, Serialize, Deserialize)]
struct BinaryInfo {
    /// Binary size (KB)
    size_kb: u64,
    /// Binary path
    path: String,
    /// Stripped
    stripped: bool,
}

/// Lambda-perf format output
#[derive(Debug, Serialize, Deserialize)]
struct LambdaPerfEntry {
    /// Init durations (10 measurements)
    i: Vec<f64>,
    /// Memory size (MB)
    m: u64,
    /// Architecture
    a: String,
    /// Max memory used (MB)
    mu: u64,
    /// Average duration (ms)
    ad: f64,
    /// Average cold start duration (ms)
    acd: f64,
    /// Runtime identifier
    r: String,
    /// Package type (zip)
    p: String,
    /// Display name
    d: String,
}

impl BenchmarkResults {
    fn to_lambda_perf(&self) -> LambdaPerfEntry {
        let init_durations: Vec<f64> = self.measurements.iter().map(|m| m.init_ms).collect();

        LambdaPerfEntry {
            i: init_durations,
            m: self.memory_mb,
            a: self.arch.clone(),
            mu: self
                .measurements
                .first()
                .map(|m| m.memory_kb / 1024)
                .unwrap_or(0),
            ad: self.stats.avg_ms,
            acd: self.stats.avg_ms,
            r: "ruchy_on_provided_al2023".to_string(),
            p: "zip".to_string(),
            d: "ruchy (prov.al2023)".to_string(),
        }
    }
}

fn calculate_statistics(measurements: &[ColdStartMeasurement]) -> Statistics {
    let mut durations: Vec<f64> = measurements.iter().map(|m| m.total_ms).collect();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = durations.len();
    let p50 = durations[len / 2];
    let p99 = durations[((len * 99) / 100).min(len - 1)];
    let min = durations[0];
    let max = durations[len - 1];

    let sum: f64 = durations.iter().sum();
    let avg = sum / len as f64;

    let variance = durations.iter().map(|d| (d - avg).powi(2)).sum::<f64>() / len as f64;
    let stddev = variance.sqrt();

    Statistics {
        avg_ms: avg,
        p50_ms: p50,
        p99_ms: p99,
        min_ms: min,
        max_ms: max,
        stddev_ms: stddev,
    }
}

fn get_binary_info() -> BinaryInfo {
    // Try release-ultra first
    let paths = vec![
        ("target/release-ultra/bootstrap", true),
        ("target/release/bootstrap", true),
        ("target/debug/bootstrap", false),
    ];

    for (path, stripped) in paths {
        if let Ok(metadata) = fs::metadata(path) {
            return BinaryInfo {
                size_kb: metadata.len() / 1024,
                path: path.to_string(),
                stripped,
            };
        }
    }

    // Default if no binary found
    BinaryInfo {
        size_kb: 0,
        path: "not_found".to_string(),
        stripped: false,
    }
}

async fn run_benchmark_real(
    function_name: &str,
    memory_mb: u64,
    arch: &str,
) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    println!("✅ GREEN PHASE: Using REAL AWS Lambda measurements");
    println!("   Function: {}", function_name);
    println!("   Memory: {}MB, Arch: {}", memory_mb, arch);
    println!("Collecting 10 cold start measurements...\\n");

    // Initialize AWS SDK
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_lambda::Client::new(&config);

    // Run 10 real invocations
    let real_metrics = real_measurement::run_ten_invocations_real(&client, function_name).await?;

    // Convert to legacy format
    let measurements: Vec<ColdStartMeasurement> = real_metrics
        .iter()
        .map(|m| ColdStartMeasurement {
            init_ms: m.init_ms,
            handler_ms: m.handler_ms,
            total_ms: m.total_ms,
            memory_kb: m.peak_memory_mb * 1024,
            timestamp: m.timestamp,
        })
        .collect();

    let stats = calculate_statistics(&measurements);
    let binary = get_binary_info();

    println!("\\n=== Benchmark Results (REAL AWS Lambda) ===");
    println!("Average:  {:.2}ms", stats.avg_ms);
    println!("P50:      {:.2}ms", stats.p50_ms);
    println!("P99:      {:.2}ms", stats.p99_ms);
    println!("Min:      {:.2}ms", stats.min_ms);
    println!("Max:      {:.2}ms", stats.max_ms);
    println!("StdDev:   {:.2}ms", stats.stddev_ms);
    println!("Binary:   {}KB ({})", binary.size_kb, binary.path);

    Ok(BenchmarkResults {
        runtime: "ruchy".to_string(),
        memory_mb,
        arch: arch.to_string(),
        measurements,
        stats,
        binary,
    })
}

fn compare_results(results: &BenchmarkResults) {
    // Fastest runtimes from lambda-perf 2024-12-31
    const FASTEST_CPP: f64 = 13.539;
    const FASTEST_RUST: f64 = 16.983;
    const FASTEST_GO: f64 = 45.769;
    const FASTEST_SWIFT: f64 = 86.333;

    println!("\\n=== Performance Comparison ===");
    println!("Ruchy:  {:.2}ms", results.stats.avg_ms);
    println!("C++:    {:.2}ms (current fastest)", FASTEST_CPP);
    println!("Rust:   {:.2}ms", FASTEST_RUST);
    println!("Go:     {:.2}ms", FASTEST_GO);
    println!("Swift:  {:.2}ms", FASTEST_SWIFT);

    println!("\\nRuchy Speedup:");
    println!(
        "  vs C++:   {:.2}x {}",
        FASTEST_CPP / results.stats.avg_ms,
        if results.stats.avg_ms < FASTEST_CPP {
            "✓ FASTER"
        } else {
            "✗ SLOWER"
        }
    );
    println!(
        "  vs Rust:  {:.2}x {}",
        FASTEST_RUST / results.stats.avg_ms,
        if results.stats.avg_ms < FASTEST_RUST {
            "✓ FASTER"
        } else {
            "✗ SLOWER"
        }
    );
    println!(
        "  vs Go:    {:.2}x {}",
        FASTEST_GO / results.stats.avg_ms,
        if results.stats.avg_ms < FASTEST_GO {
            "✓ FASTER"
        } else {
            "✗ SLOWER"
        }
    );

    // Target check
    const TARGET: f64 = 8.0;
    println!(
        "\\nTarget: <{}ms {}",
        TARGET,
        if results.stats.avg_ms < TARGET {
            "✓ MET"
        } else {
            "✗ NOT MET"
        }
    );

    // World record status
    if results.stats.avg_ms < FASTEST_CPP {
        let improvement = ((FASTEST_CPP - results.stats.avg_ms) / FASTEST_CPP) * 100.0;
        println!(
            "\\n🎉 NEW WORLD RECORD! {:.1}% faster than C++",
            improvement
        );
        println!("   ✅ Measured with REAL AWS Lambda invocations");
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Benchmark {
            function,
            memory,
            arch,
            output,
        } => {
            let results = run_benchmark_real(&function, memory, &arch)
                .await
                .expect("Failed to run benchmark");
            compare_results(&results);

            if let Some(path) = output {
                let json = serde_json::to_string_pretty(&results).unwrap();
                fs::write(&path, json).expect("Failed to write output file");
                println!("\\nResults saved to: {}", path.display());
            }
        }

        Commands::Compare { input } => {
            let data = fs::read_to_string(&input).expect("Failed to read input file");
            let results: BenchmarkResults =
                serde_json::from_str(&data).expect("Failed to parse JSON");
            compare_results(&results);
        }

        Commands::Report { input, output } => {
            let data = fs::read_to_string(&input).expect("Failed to read input file");
            let results: BenchmarkResults =
                serde_json::from_str(&data).expect("Failed to parse JSON");

            let lambda_perf = results.to_lambda_perf();
            let json = serde_json::to_string_pretty(&lambda_perf).unwrap();

            fs::write(&output, json).expect("Failed to write output file");
            println!("Lambda-perf report generated: {}", output.display());
        }

        Commands::Memory { binary } => {
            if !binary.exists() {
                eprintln!("Binary not found: {}", binary.display());
                std::process::exit(1);
            }

            let metadata = fs::metadata(&binary).expect("Failed to read binary");
            let size_kb = metadata.len() / 1024;

            println!("Binary: {}", binary.display());
            println!("Size: {} KB ({} bytes)", size_kb, metadata.len());

            if size_kb < 100 {
                println!("✓ Under 100KB target");
            } else {
                println!("✗ Exceeds 100KB target by {} KB", size_kb - 100);
            }
        }
    }
}
