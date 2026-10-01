//! Product-independent application interaction contracts.
// Transport, authorization, persistence and presentation are deliberately not implemented here.
mod input;
mod schema;
mod wire;

pub use input::*;
pub use schema::{Issue, ValueSchema};
pub use wire::*;
