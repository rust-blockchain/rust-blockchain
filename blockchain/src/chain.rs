use crate::Identified;

/// Fork tree.
///
/// A fork tree tracks blocks of forks. However, it does not track the best
/// block. Instead, that is supposed to be handled by the `Chain` directly. The
/// reason for this is that, except storing the best block hash, the majority of
/// best block tracking is used for optimizations -- rebalancing the trees,
/// moving non-canon blocks out of cache, pruning nodes, etc. They are not only
/// needed in a fork tree, but also in states, as well as other related structs.
pub trait ForkTree {
    /// The type of the identified. It can be a block or a header.
    type Block: Identified;
    /// Query error type.
    type QueryError;

    /// Get a block by its id.
    fn block(
        &self,
        id: <Self::Block as Identified>::Identifier,
    ) -> Result<Self::Block, Self::QueryError>;

    /// Get a block depth by its id.
    fn block_depth(
        &self,
        id: <Self::Block as Identified>::Identifier,
    ) -> Result<usize, Self::QueryError>;

    /// Find an ancestor block at given depth.
    fn ancestor_id_at_depth(
        &self,
        id: <Self::Block as Identified>::Identifier,
        ancestor_depth: usize,
    ) -> Result<<Self::Block as Identified>::Identifier, Self::QueryError>;
}

/// A structure representing a chain with possible forks.
pub trait ForkTreeMut: ForkTree {
    /// Insert error type.
    type InsertError;

    /// Insert a new block.
    fn insert(&mut self, block: Self::Block) -> Result<(), Self::InsertError>;
}

/// A chain that can import external blocks.
pub trait ImportBlock {
    /// Type of the block.
    type Block;
    /// Error type.
    type Error;

    /// Import a new block, given a fork tree.
    fn import(&mut self, block: Self::Block) -> Result<(), Self::Error>;
}

/// A chain that can build new blocks.
pub trait BuildBlock {
    /// Type of the block.
    type Block: Identified;
    /// Type of the builder.
    type Builder<'builder>: BlockBuilder<Block = Self::Block>;
    /// Error type.
    type Error;

    /// Initialize building a new block, with the given fork tree, on top of the
    /// given block hash.
    fn build<'a, 'builder: 'a>(
        &'a mut self,
        parent_id: <Self::Block as Identified>::Identifier,
    ) -> Result<Self::Builder<'builder>, Self::Error>;
}

/// Block builder.
pub trait BlockBuilder {
    /// Type of the block.
    type Block;
    /// Type of extrinsic.
    type Extrinsic;
    /// Type of error.
    type Error;

    /// Apply a new extrinsic.
    fn apply_extrinsic(&mut self, extrinsic: Self::Extrinsic) -> Result<(), Self::Error>;
    /// Finalize and import the current block.
    fn finalize_and_import(self) -> Result<Self::Block, Self::Error>;
}
