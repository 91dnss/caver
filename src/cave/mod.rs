//! Code cave construction and LOAD segment injection.

pub mod binary;
pub mod inject;
pub mod inspection;
pub mod types;

pub use inject::{inject, inject_many};
pub use inspection::{
    ExistingCave, SectionInfo, SegmentInfo, find_caves, list_sections, list_segments,
};
pub use types::{CaveInfo, CaveOptions, FillByte, PatchedElf};
