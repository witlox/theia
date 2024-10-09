/// Module for Conflict-free Replicated Data Types
pub mod base;

pub mod vector_clock;
pub use vector_clock::VectorClock;

pub mod traits;
pub use traits::{Actor, CvRDT, CmRDT, Reset};

pub mod version;
pub use version::{Version, VersionRange};

mod serde_ext;

pub mod map;
pub use map::Map;

pub mod multi_value;
mod list;
pub use list::List;

mod identifier;
pub use identifier::Identifier;

