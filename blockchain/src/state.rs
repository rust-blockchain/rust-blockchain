use crate::{ForkTree, Identified};

pub trait FlatState<FT: ForkTree> {
    type Key;
    type Value;
    type Error;

    fn get(
        &self,
        key: &Self::Key,
        block_id: <FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<Option<Self::Value>, Self::Error>;
    fn apply<I: Iterator<Item = (Self::Key, Option<Self::Value>)>>(
        &self,
        changeset: I,
        block_id: <FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<(), Self::Error>;
}
