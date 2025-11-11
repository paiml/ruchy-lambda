use ruchy_lambda_runtime_pure::Runtime;
pub fn lambda_handler(_request_id: &str, _body: &str) -> String {
    String::from("{\"statusCode\":200,\"body\":\"Hello from Pure Ruchy!\"}")
}
pub fn main() {
    println!("[BOOTSTRAP] Initializing Pure Ruchy Lambda Runtime...");
    {
        let runtime = Runtime::new();
        {
            println!("[BOOTSTRAP] Runtime initialized");
            loop {
                {
                    println!("[BOOTSTRAP] Waiting for next event...");
                    let (request_id, event_body) = runtime.next_event();
                    println!("[BOOTSTRAP] Processing request: {}", &request_id);
                    {
                        let response = lambda_handler(&request_id, &event_body);
                        {
                            runtime.post_response(&request_id, &response);
                            println!("[BOOTSTRAP] Response sent for request: {}", &request_id)
                        }
                    }
                }
            }
        }
    }
}
