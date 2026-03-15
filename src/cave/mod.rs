//! Code cave construction and LOAD segment injection.

pub mod binary;
pub mod inject;
pub mod types;

pub use inject::{inject, inject_many};
pub use types::{CaveInfo, CaveOptions, FillByte};
