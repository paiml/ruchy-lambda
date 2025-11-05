use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

// Fibonacci recursive implementation
// Source: ruchy-book bench-007-fibonacci pattern
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_: LambdaEvent<Value>) -> Result<Value, Error> {
    // Calculate fibonacci(35) - standard Lambda benchmark
    let result = fibonacci(35);

    Ok(json!({
        "statusCode": 200,
        "body": format!("fibonacci(35)={}", result)
    }))
}
