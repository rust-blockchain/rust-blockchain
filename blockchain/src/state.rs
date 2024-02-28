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
    type Error;

    /// Get the value at particular block id.
    fn get(
        &self,
        key: &Self::Key,
        block_id: &<FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<Option<Self::Value>, Self::Error>;

    /// Apply a changeset to a particular block id.
    fn apply<I: Iterator<Item = (Self::Key, Option<Self::Value>)>>(
        &mut self,
        changeset: I,
        block_id: &<FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<(), Self::Error>;
}
