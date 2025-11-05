// Local Fibonacci Benchmark
// Compare local execution time vs AWS Lambda

use std::time::Instant;

fn fibonacci(n: i32) -> i32 {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}

fn main() {
    println!("=== LOCAL FIBONACCI BENCHMARK ===\n");

    // Warm-up
    let _ = fibonacci(10);

    // Benchmark fibonacci(35) - same as AWS Lambda
    let start = Instant::now();
    let result = fibonacci(35);
    let duration = start.elapsed();

    println!("Input: n = 35");
    println!("Result: fibonacci(35) = {}", result);
    println!("Time: {:?}", duration);
    println!("Time (ms): {:.2}ms", duration.as_secs_f64() * 1000.0);

    // Multiple runs for average
    println!("\n=== 5 CONSECUTIVE RUNS ===");
    let mut times = Vec::new();
    for i in 1..=5 {
        let start = Instant::now();
        let result = fibonacci(35);
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        times.push(ms);
        println!("Run {}: {:.2}ms (result: {})", i, ms, result);
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    println!("\nStatistics:");
    println!("  Average: {:.2}ms", avg);
    println!("  Min: {:.2}ms", min);
    println!("  Max: {:.2}ms", max);
}
