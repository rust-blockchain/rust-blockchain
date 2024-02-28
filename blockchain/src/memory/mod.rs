//! Memory-only implementations.

mod state;

use itertools::Itertools;
use std::collections::HashMap;

use crate::{ForkTree, ForkTreeMut, Identified};

#[derive(Clone, Debug)]
struct MemoryForkTreeItem<Block: Identified> {
    block: Block,
    depth: usize,
    children: Vec<Block::Identifier>,
    ancestors: Vec<(usize, Block::Identifier)>,
}

/// A fork tree that resides entirely in memory. Useful for testing.
pub struct MemoryForkTree<Block: Identified> {
    blocks: HashMap<Block::Identifier, MemoryForkTreeItem<Block>>,
    depths: HashMap<usize, Vec<Block::Identifier>>,
}

impl<Block: Identified> MemoryForkTree<Block> {
    /// Create a new fork tree.
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            depths: HashMap::new(),
        }
    }
}

/// Query error for memory fork tree.
pub enum MemoryForkTreeQueryError {
    /// Block is unknown.
    UnknownBlock,
    /// Ancestor depth provided is greater than current block depth.
    InvalidAncestorDepth,
}

impl<Block: Identified + Clone> ForkTree for MemoryForkTree<Block> {
    type Block = Block;
    type QueryError = MemoryForkTreeQueryError;

    fn block(&self, id: Block::Identifier) -> Result<Block, Self::QueryError> {
        Ok(self
            .blocks
            .get(&id)
            .ok_or(MemoryForkTreeQueryError::UnknownBlock)?
            .block
            .clone())
    }

    fn block_depth(&self, id: Block::Identifier) -> Result<usize, Self::QueryError> {
        Ok(self
            .blocks
            .get(&id)
            .ok_or(MemoryForkTreeQueryError::UnknownBlock)?
            .depth)
    }

    fn ancestor_id_at_depth(
        &self,
        id: Block::Identifier,
        ancestor_depth: usize,
    ) -> Result<Block::Identifier, Self::QueryError> {
        let mut current_block = self
            .blocks
            .get(&id)
            .ok_or(MemoryForkTreeQueryError::UnknownBlock)?;

        if current_block.depth >= ancestor_depth {
            return Err(MemoryForkTreeQueryError::InvalidAncestorDepth);
        }

        loop {
            if current_block.depth < ancestor_depth {
                return Err(MemoryForkTreeQueryError::InvalidAncestorDepth);
            }

            if current_block.depth == ancestor_depth {
                return Ok(current_block.block.id());
            }

            let parent_id = current_block
                .block
                .parent_id()
                // If the current block depth is 0, then the ancestor depth
                // provided must be invalid.
                .ok_or(MemoryForkTreeQueryError::InvalidAncestorDepth)?;

            let next_ancestor_id = current_block
                .ancestors
                .iter()
                // Find all ancestor depth that is greater than the target
                // ancestor depth.
                .filter(|(d, _)| *d >= ancestor_depth)
                .cloned()
                // Then find the minimum of all the values.
                .reduce(|(min_depth, min_id), (d, id)| {
                    if d < min_depth {
                        (d, id)
                    } else {
                        (min_depth, min_id)
                    }
                })
                .map(|(_, id)| id)
                .unwrap_or(parent_id);

            current_block = self
                .blocks
                .get(&next_ancestor_id)
                .ok_or(MemoryForkTreeQueryError::UnknownBlock)?;
        }
    }
}

/// Insert error for memory fork tree.
pub enum MemoryForkTreeInsertError {
    /// Parent is unknown.
    UnknownParent,
    /// Encounted a query issue in insertion.
    Query(MemoryForkTreeQueryError),
}

impl From<MemoryForkTreeQueryError> for MemoryForkTreeInsertError {
    fn from(query: MemoryForkTreeQueryError) -> MemoryForkTreeInsertError {
        MemoryForkTreeInsertError::Query(query)
    }
}

/// Skip depths for ancestor list.
const SKIP_DEPTHS: [usize; 16] = [
    4usize.pow(1),
    4usize.pow(2),
    4usize.pow(3),
    4usize.pow(4),
    4usize.pow(5),
    4usize.pow(6),
    4usize.pow(7),
    4usize.pow(8),
    4usize.pow(9),
    4usize.pow(10),
    4usize.pow(11),
    4usize.pow(12),
    4usize.pow(13),
    4usize.pow(14),
    4usize.pow(15),
    4usize.pow(16),
];

impl<Block: Identified + Clone> ForkTreeMut for MemoryForkTree<Block> {
    type InsertError = MemoryForkTreeInsertError;

    fn insert(&mut self, block: Block) -> Result<(), Self::InsertError> {
        let block_id = block.id();

        let depth = if let Some(parent_id) = block.parent_id() {
            let parent = self
                .blocks
                .get_mut(&parent_id)
                .ok_or(MemoryForkTreeInsertError::UnknownParent)?;
            parent.children.push(block.id());
            parent.depth + 1
        } else {
            0
        };

        let ancestors = if let Some(parent_id) = block.parent_id() {
            // Build a skip list of ancestors. If the current block depth can be
            // divided by `SKIP_DEPTHS`, then we call `ancestor_id_at_depth` to
            // track back on the ancestor block.
            let ancestor_depths = SKIP_DEPTHS
                .iter()
                .filter(|skip_depth| &depth >= *skip_depth && depth % *skip_depth == 0)
                .map(|skip_depth| depth - skip_depth)
                .unique();

            let mut ancestors = Vec::new();
            for ancestor_depth in ancestor_depths {
                ancestors.push((
                    ancestor_depth,
                    self.ancestor_id_at_depth(parent_id, ancestor_depth)?,
                ));
            }

            ancestors
        } else {
            Vec::new()
        };

        self.depths.entry(depth).or_default().push(block_id);
        self.blocks.insert(
            block_id,
            MemoryForkTreeItem {
                block,
                depth,
                children: Vec::new(),
                ancestors,
            },
        );

        Ok(())
    }
}
