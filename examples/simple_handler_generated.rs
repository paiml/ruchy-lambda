fn handler(request_id: &str) -> &str {
    println!("Processing request: {}", request_id);
    "Hello from Ruchy Lambda!"
}
#[allow(dead_code)]
fn main() {}
