//! Memory-only implementations.

use std::collections::HashMap;

use crate::{ForkTree, ForkTreeMut, Identified};

#[derive(Clone, Debug)]
struct MemoryForkTreeItem<Block: Identified> {
    block: Block,
    depth: usize,
    children: Vec<Block::Identifier>,
}

/// A fork tree that resides entirely in memory. Useful for testing.
pub struct MemoryForkTree<Block: Identified> {
    blocks: HashMap<Block::Identifier, MemoryForkTreeItem<Block>>,
    depths: HashMap<usize, Vec<Block::Identifier>>,
    best: Block::Identifier,
}

impl<Block: Identified> MemoryForkTree<Block> {
    /// Create a new fork tree with a genesis block.
    pub fn new(genesis: Block) -> Self {
        let genesis_id = genesis.id();
        let mut blocks = HashMap::new();
        let mut depths = HashMap::new();

        depths.insert(0, vec![genesis_id]);
        blocks.insert(
            genesis_id,
            MemoryForkTreeItem {
                block: genesis,
                depth: 0,
                children: Vec::new(),
            },
        );

        Self {
            blocks,
            depths,
            best: genesis_id,
        }
    }
}

/// Query error for memory fork tree.
pub enum MemoryForkTreeQueryError {
    /// Block is unknown.
    UnknownBlock,
}

impl<Block: Identified + Clone> ForkTree for MemoryForkTree<Block> {
    type Block = Block;
    type QueryError = MemoryForkTreeQueryError;

    fn block_by_id(&self, id: Block::Identifier) -> Result<Block, Self::QueryError> {
        Ok(self
            .blocks
            .get(&id)
            .ok_or(MemoryForkTreeQueryError::UnknownBlock)?
            .block
            .clone())
    }

    fn best(&self) -> Block {
        self.blocks
            .get(&self.best)
            .expect("best block exists; qed")
            .block
            .clone()
    }
}

/// Insert error for memory fork tree.
pub enum MemoryForkTreeInsertError {
    /// Parent is unknown.
    UnknownParent,
}

impl<Block: Identified + Clone> ForkTreeMut for MemoryForkTree<Block> {
    type InsertError = MemoryForkTreeInsertError;

    fn insert(&mut self, block: Block, is_new_best: bool) -> Result<(), Self::InsertError> {
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

        self.depths.entry(depth).or_default().push(block_id);
        self.blocks.insert(
            block_id,
            MemoryForkTreeItem {
                block,
                depth,
                children: Vec::new(),
            },
        );

        if is_new_best {
            self.best = block_id;
        }

        Ok(())
    }
}
