//! This is a simple chain test, with fixed hashes and seal.

use blockchain::memory::{
    MemoryFlatState, MemoryForkTree, MemoryForkTreeInsertError, MemoryForkTreeQueryError,
    MemoryTransactional,
};
use blockchain::{
    BlockBuilder, FlatState, FlatStateMut, ForkTree, ForkTreeMut, Headered, Identified,
    ImportBlock, Keyed, OverlayedFlatState,
};

/// A simple seal.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Seal {
    /// Valid seal.
    ValidSeal,
    /// Invalid seal.
    InvalidSeal,
}

/// Specific block hashes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, core::hash::Hash)]
pub struct BlockId {
    fork: u32,
    number: u32,
}

/// Extrinsic type.
#[derive(Debug, Clone)]
pub enum Extrinsic {
    Set(u32, u32),
}

/// Simple block structure.
#[derive(Debug, Clone)]
pub struct Block {
    pub seal: Seal,
    pub id: BlockId,
    pub parent_id: Option<BlockId>,
    pub number: u32,
    pub extrinsics: Vec<Extrinsic>,
}

/// Header is simply block with extrinsic removed.
#[derive(Debug, Clone)]
pub struct Header {
    pub seal: Seal,
    pub id: BlockId,
    pub parent_id: Option<BlockId>,
    pub number: u32,
}

impl Identified for Block {
    type Identifier = BlockId;

    fn id(&self) -> BlockId {
        self.id
    }

    fn parent_id(&self) -> Option<BlockId> {
        self.parent_id
    }
}

// Block can also be keyed by its block number instead of block id.
impl Keyed<u32> for Block {
    fn key(&self) -> u32 {
        self.number
    }
}

impl Headered for Block {
    type Header = Header;

    fn header(&self) -> Header {
        Header {
            seal: self.seal,
            id: self.id,
            parent_id: self.parent_id,
            number: self.number,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChainData {
    pub fork_tree: MemoryForkTree<Block>,
    pub state: MemoryFlatState<u32, u32, BlockId>,
}

/// Define the chain.
#[derive(Debug, Clone)]
pub struct Chain {
    pub data: MemoryTransactional<ChainData>,
}

#[derive(Debug, Clone)]
pub enum ChainError {
    InvalidSeal,
    CantImportGenesis,
    ForkTreeInsert(MemoryForkTreeInsertError),
    ForkTreeQuery(MemoryForkTreeQueryError),
}

impl From<MemoryForkTreeInsertError> for ChainError {
    fn from(err: MemoryForkTreeInsertError) -> Self {
        Self::ForkTreeInsert(err)
    }
}

impl From<MemoryForkTreeQueryError> for ChainError {
    fn from(err: MemoryForkTreeQueryError) -> Self {
        Self::ForkTreeQuery(err)
    }
}

impl ImportBlock for Chain {
    type Block = Block;
    type Error = ChainError;

    fn import(&mut self, block: Block) -> Result<(), Self::Error> {
        // Verify the seal is valid.
        if block.seal != Seal::ValidSeal {
            return Err(ChainError::InvalidSeal);
        }

        self.data.apply(|data| {
            let parent_id = block.parent_id().ok_or(ChainError::CantImportGenesis)?;

            data.fork_tree.insert(block.clone())?;

            let mut overlay = data.state.overlayed(parent_id, &data.fork_tree);
            for extrinsic in &block.extrinsics {
                match extrinsic {
                    Extrinsic::Set(key, value) => {
                        overlay.insert(*key, *value);
                    }
                }
            }

            let changeset = overlay.into_changeset();
            data.state.apply(changeset, block.id(), &data.fork_tree)?;

            Ok(())
        })
    }
}

/// Chain builder.
pub struct ChainBlockBuilder<'chain> {
    pub chain: &'chain Chain,
    pub block: Block,
    pub overlay: OverlayedFlatState<
        'chain,
        'chain,
        MemoryFlatState<u32, u32, BlockId>,
        MemoryForkTree<Block>,
    >,
}

impl<'chain> BlockBuilder<'chain> for ChainBlockBuilder<'chain> {
    type Block = Block;
    type Extrinsic = Extrinsic;
    type Error = ChainError;
    type PreLog = ();
    type PostLog = Seal;
    type Chain = Chain;

    fn initialize(
        chain: &'chain Chain,
        parent_id: <Self::Block as Identified>::Identifier,
        _pre_log: (),
    ) -> Result<Self, Self::Error> {
        let _parent_block = chain.data.fork_tree.block(&parent_id)?;

        let block = Block {
            seal: Seal::InvalidSeal,
            parent_id: Some(parent_id),
            id: BlockId {
                fork: parent_id.fork,
                number: parent_id.number + 1,
            },
            number: parent_id.number + 1,
            extrinsics: Vec::new(),
        };

        Ok(ChainBlockBuilder {
            block,
            chain,
            overlay: chain.data.state.overlayed(parent_id, &chain.data.fork_tree),
        })
    }

    fn apply_extrinsic(&mut self, extrinsic: Extrinsic) -> Result<(), Self::Error> {
        self.block.extrinsics.push(extrinsic);
        Ok(())
    }

    fn finalize(mut self, post_log: Seal) -> Result<Block, Self::Error> {
        self.block.seal = post_log;
        Ok(self.block)
    }
}

#[test]
fn basic_build_and_import() -> Result<(), ChainError> {
    // Define a genesis block.
    let genesis_block = Block {
        seal: Seal::InvalidSeal, // Genesis block does not need to be verified.
        id: BlockId {
            fork: 0,
            number: 0,
        },
        parent_id: None,
        number: 0,
        extrinsics: Vec::new(),
    };

    // Define a genesis state.
    let genesis_state = vec![
        (100, Some(100)),
        (200, Some(200)),
    ];

    // Create a new chain.
    let mut chain = Chain {
        data: MemoryTransactional::new(
            ChainData {
                fork_tree: MemoryForkTree::new(),
                state: MemoryFlatState::new(),
            },
        )
    };

    // Import the genesis into fork tree.
    chain.data.apply(|data| {
        data.fork_tree.insert(genesis_block.clone())?;
        
        // It's possible to handle extrinsics in a genesis, but it's a rare thing,
        // and here we just assert that it's empty.
        assert!(genesis_block.extrinsics.is_empty());

        data.state.apply(genesis_state.clone().into_iter(), genesis_block.id(), &data.fork_tree)?;

        Ok::<_, ChainError>(())
    })?;

    Ok(())
}