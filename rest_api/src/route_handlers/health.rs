#[get("/health")]
pub fn check() -> String {
    "OK".to_string()
}
