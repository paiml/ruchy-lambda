#[allow(clippy::all)]
pub fn lambda_handler(request_id: &str, body: &str) -> String {
    ("{\"statusCode\":200,\"body\":\"ok\"}").to_string()
}
#[allow(dead_code)]
fn main() {}
