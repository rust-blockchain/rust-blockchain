//! Memory-only implementations.

mod chain;
mod state;

pub use self::chain::{MemoryForkTree, MemoryForkTreeInsertError, MemoryForkTreeQueryError};
pub use self::state::MemoryFlatState;

use core::ops::{Deref, DerefMut};

/// A memory transactional.
///
/// The struct `MemoryTransactional` allows memory-only implementations (such as
/// memory fork tree and memory state) to be transactional. We do this by
/// keeping two copies of the state. When an operation happens, we try to apply
/// it to both. If either fails, the operation halts immediately and the other
/// state is copied back.
#[derive(Debug, Clone)]
pub struct MemoryTransactional<Inner: Clone> {
    first: Inner,
    second: Inner,
}

impl<Inner: Clone> MemoryTransactional<Inner> {
    /// Create a new memory transactional.
    pub fn new(inner: Inner) -> Self {
        Self {
            second: inner.clone(),
            first: inner,
        }
    }

    /// Apply some changes.
    pub fn apply<R, E, F: Fn(&mut Inner) -> Result<R, E>>(&mut self, f: F) -> Result<R, E> {
        match f(&mut self.first) {
            Ok(first_ret) => match f(&mut self.second) {
                Ok(_second_ret) => Ok(first_ret),
                Err(_second_err) => {
                    self.second = self.first.clone();
                    Ok(first_ret)
                }
            },
            Err(first_err) => {
                self.first = self.second.clone();
                Err(first_err)
            }
        }
    }
}

impl<Inner: Clone> Deref for MemoryTransactional<Inner> {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.first
    }
}

impl<Inner: Clone> DerefMut for MemoryTransactional<Inner> {
    fn deref_mut(&mut self) -> &mut Inner {
        &mut self.first
    }
}
