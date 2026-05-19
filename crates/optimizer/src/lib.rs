pub mod baseline;
pub mod common;
pub mod fixtures;
pub mod greedy;
pub mod report;
pub mod score;
pub mod validate;

pub use score::score;
pub use validate::{validate, Violation, ViolationKind};
