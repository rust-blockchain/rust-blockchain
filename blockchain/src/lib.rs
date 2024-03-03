//! General block framework.

#![warn(missing_docs)]

mod block;
mod chain;
pub mod memory;
mod state;

pub use crate::block::{Headered, Identified, Keyed};
pub use crate::chain::{BlockBuilder, ForkTree, ForkTreeMut, ForkTreeTransactional, ImportBlock};
pub use crate::state::{FlatState, FlatStateMut, FlatStateTransactional, OverlayedFlatState};
