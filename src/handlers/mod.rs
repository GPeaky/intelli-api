pub(crate) mod auth;
pub(crate) mod championships;
pub(crate) mod user;

#[inline(always)]
pub(crate) async fn init() -> &'static str {
    "Hello World"
}
