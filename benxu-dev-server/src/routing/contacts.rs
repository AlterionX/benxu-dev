#[get("/contacts")]
pub fn get() -> &'static str {
    "Hello, world!"
}
