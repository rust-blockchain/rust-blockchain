use std::collections::HashMap;

use crate::{ForkTree, Identified};

/// Flat state.
///
/// A flat state is a type of key-value database that contains all historical
/// data, yet without using a merkle tree. It is usually implemented by storing
/// the changeset in the database, and fetch them on demand.
pub trait FlatState<FT: ForkTree> {
    /// Key type.
    type Key;
    /// Value type.
    type Value;
    /// Error type.
    type QueryError;

    /// Get the value at particular block id.
    fn get(
        &self,
        key: &Self::Key,
        block_id: &<FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<Option<Self::Value>, Self::QueryError>;

    /// Overlayed state.
    fn overlayed<'fs, 'ft>(
        &'fs self,
        block_id: <FT::Block as Identified>::Identifier,
        fork_tree: &'ft FT,
    ) -> OverlayedFlatState<'fs, 'ft, Self, FT> {
        OverlayedFlatState {
            flat_state: self,
            block_id,
            fork_tree,
            changeset: HashMap::new(),
        }
    }
}

/// Mutable flat state.
pub trait FlatStateMut<FT: ForkTree>: FlatState<FT> {
    /// Apply error type.
    type ApplyError;

    /// Apply a changeset to a particular block id.
    fn apply<I: Iterator<Item = (Self::Key, Option<Self::Value>)>>(
        &mut self,
        changeset: I,
        block_id: <FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<(), Self::ApplyError>;
}

/// Transactional flat state.
pub trait FlatStateTransactional<FT: ForkTree>: FlatState<FT> {
    /// Transaction type.
    type Transaction;
    /// Apply error type.
    type ApplyError;

    /// Apply a changeset to a particular block id.
    fn apply<I: Iterator<Item = (Self::Key, Option<Self::Value>)>>(
        &self,
        transaction: &mut Self::Transaction,
        changeset: I,
        block_id: &<FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<(), Self::ApplyError>;
}

/// Convinence function for building a changeset of a flat state.
pub struct OverlayedFlatState<'fs, 'ft, FS: FlatState<FT> + ?Sized, FT: ForkTree> {
    flat_state: &'fs FS,
    fork_tree: &'ft FT,
    block_id: <FT::Block as Identified>::Identifier,
    changeset: HashMap<FS::Key, Option<FS::Value>>,
}

impl<'fs, 'ft, FS, FT> OverlayedFlatState<'fs, 'ft, FS, FT>
where
    FS: FlatState<FT> + ?Sized,
    FS::Key: Clone + Eq + PartialEq + core::hash::Hash,
    FS::Value: Clone,
    FT: ForkTree,
{
    /// Get a value from the overlay.
    pub fn get(&self, key: &FS::Key) -> Result<Option<FS::Value>, FS::QueryError> {
        if let Some(value) = self.changeset.get(key) {
            Ok(value.clone())
        } else {
            self.flat_state.get(key, &self.block_id, self.fork_tree)
        }
    }

    /// Insert a new value.
    pub fn insert(&mut self, key: FS::Key, value: FS::Value) {
        self.changeset.insert(key, Some(value));
    }

    /// Remove an existing value.
    pub fn remove(&mut self, key: &FS::Key) {
        self.changeset.insert(key.clone(), None);
    }

    /// Into changeset.
    pub fn into_changeset(self) -> impl Iterator<Item = (FS::Key, Option<FS::Value>)> {
        self.changeset.into_iter()
    }
}
