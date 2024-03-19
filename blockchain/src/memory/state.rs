use core::hash::Hash;
use core::ops::Bound;
use std::collections::{BTreeMap, HashMap};

use crate::{FlatState, FlatStateMut, ForkTree, Identified};

/// A flat state that is stored in memory.
#[derive(Debug, Clone)]
pub struct MemoryFlatState<K, V, Identifier> {
    state: HashMap<K, BTreeMap<usize, HashMap<Identifier, Option<V>>>>,
}

impl<K, V, Identifier> MemoryFlatState<K, V, Identifier>
where
    K: Eq + PartialEq + Hash,
    V: Clone,
    Identifier: Eq + PartialEq + Hash + Clone,
{
    /// Create a new empty flat state.
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }
}

impl<K, V, Identifier, FT, B> FlatState<FT> for MemoryFlatState<K, V, Identifier>
where
    K: Eq + PartialEq + Hash,
    V: Clone,
    Identifier: Eq + PartialEq + Hash + Clone,
    FT: ForkTree<Block = B>,
    B: Identified<Identifier = Identifier>,
{
    type Key = K;
    type Value = V;
    type QueryError = FT::QueryError;

    fn get(
        &self,
        key: &Self::Key,
        block_id: &<FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<Option<Self::Value>, Self::QueryError> {
        if let Some(depth_to_id_value) = self.state.get(key) {
            let depth = fork_tree.block_depth(block_id)?;
            let search_range = depth_to_id_value
                .range((Bound::Unbounded, Bound::Included(depth)))
                .rev();

            for (_, search_id_to_value) in search_range {
                for (search_id, search_value) in search_id_to_value {
                    if fork_tree.is_ancestor(block_id, search_id)? {
                        return Ok(search_value.clone());
                    }
                }
            }
        }

        Ok(None)
    }
}

impl<K, V, Identifier, FT, B> FlatStateMut<FT> for MemoryFlatState<K, V, Identifier>
where
    K: Eq + PartialEq + Hash,
    V: Clone,
    Identifier: Eq + PartialEq + Hash + Clone,
    FT: ForkTree<Block = B>,
    B: Identified<Identifier = Identifier>,
{
    type ApplyError = FT::QueryError;

    fn apply<I: Iterator<Item = (Self::Key, Option<Self::Value>)>>(
        &mut self,
        changeset: I,
        block_id: <FT::Block as Identified>::Identifier,
        fork_tree: &FT,
    ) -> Result<(), Self::ApplyError> {
        let depth = fork_tree.block_depth(&block_id)?;

        for (key, value) in changeset {
            self.state
                .entry(key)
                .or_default()
                .entry(depth)
                .or_default()
                .insert(block_id.clone(), value);
        }

        Ok(())
    }
}
