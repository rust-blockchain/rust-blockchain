use crate::Identified;

/// A structure representing a chain with possible forks.
pub trait ForkTree {
    /// The type of the identified. It can be a block or a header.
    type Block: Identified;

    /// Best block.
    fn best(&self) -> &Self::Block;
}

/// A chain that can import external blocks.
pub trait ImportBlock<F: ForkTree<Block = Self::Block>> {
    /// Type of the block.
    type Block;
    /// Error type.
    type Error;

    /// Import a new block, given a fork tree.
    fn import(&mut self, tree: &mut F, block: Self::Block) -> Result<(), Self::Error>;
}

/// A chain that can build new blocks.
pub trait BuildBlock<F: ForkTree<Block = Self::Block>> {
    /// Type of the block.
    type Block: Identified;
    /// Type of the builder.
    type Builder<'builder>: BlockBuilder<Block = Self::Block>;
    /// Error type.
    type Error;

    /// Initialize building a new block, with the given fork tree, on top of the
    /// given block hash.
    fn build<'a, 'b, 'builder: 'a + 'b>(
        &'a mut self,
        tree: &'b mut F,
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
