//! General block framework.

#![warn(missing_docs)]

mod block;
mod chain;

pub use crate::block::{Headered, Identified, Keyed};
