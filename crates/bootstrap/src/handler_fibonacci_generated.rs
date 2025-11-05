#[allow(clippy::all)]
pub fn fibonacci(n: i32) -> i32 {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}
#[allow(clippy::all)]
pub fn lambda_handler(request_id: &str, body: &str) -> String {
    {
        let n = 35;
        ({
            let result = fibonacci(n);
            {
                let result_str = result.to_string();
                {
                    let response = format!(
                        "{}{}",
                        String::from("{\"statusCode\":200,\"body\":\"fibonacci(35)=") + &
                        result_str, "\"}"
                    );
                    response
                }
            }
        })
            .to_string()
    }
}
#[allow(dead_code)]
fn main() {}
