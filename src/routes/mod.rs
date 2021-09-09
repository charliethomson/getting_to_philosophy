pub mod api;
pub use api::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct OkMessage<T: Serialize> {
    ok: bool,
    message: T,
}
impl<T: Serialize> OkMessage<T> {
    pub fn ok(message: T) -> Self {
        Self { ok: true, message }
    }
    pub fn err(message: T) -> Self {
        Self { ok: false, message }
    }
}
