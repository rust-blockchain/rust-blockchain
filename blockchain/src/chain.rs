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
        id: &<Self::Block as Identified>::Identifier,
    ) -> Result<Self::Block, Self::QueryError>;

    /// Get a block depth by its id.
    fn block_depth(
        &self,
        id: &<Self::Block as Identified>::Identifier,
    ) -> Result<usize, Self::QueryError>;

    /// Find an ancestor block at given depth.
    ///
    /// If ancestor depth equals the provided block's depth, return the provided block ID.
    fn ancestor_id_at_depth(
        &self,
        id: &<Self::Block as Identified>::Identifier,
        ancestor_depth: usize,
    ) -> Result<<Self::Block as Identified>::Identifier, Self::QueryError>;

    /// Whether is ancestor.
    ///
    /// If ancestor depth equals the provided block's depth, return true.
    fn is_ancestor(
        &self,
        id: &<Self::Block as Identified>::Identifier,
        ancestor_id: &<Self::Block as Identified>::Identifier,
    ) -> Result<bool, Self::QueryError> {
        let ancestor_depth = self.block_depth(ancestor_id)?;

        Ok(self.ancestor_id_at_depth(id, ancestor_depth)? == *ancestor_id)
    }
}

/// A structure representing a chain with possible forks.
pub trait ForkTreeMut: ForkTree {
    /// Insert error type.
    type InsertError;

    /// Insert a new block.
    fn insert(&mut self, block: Self::Block) -> Result<(), Self::InsertError>;
}

/// Transactional fork tree.
pub trait ForkTreeTransactional: ForkTree {
    /// Transaction type.
    type Transaction;
    /// Insert error type.
    type InsertError;

    /// Insert a new block.
    fn insert(
        &self,
        transaction: &mut Self::Transaction,
        block: Self::Block,
    ) -> Result<(), Self::InsertError>;
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

/// Block builder.
pub trait BlockBuilder<'chain>: Sized {
    /// Type of the chain.
    type Chain;
    /// Type of the block.
    type Block: Identified;
    /// Type of extrinsic.
    type Extrinsic;
    /// Type of error.
    type Error;
    /// Type of pre-log.
    type PreLog;
    /// Type of post-log.
    type PostLog;

    /// Initialize a new block.
    fn initialize(
        chain: &'chain Self::Chain,
        parent_id: <Self::Block as Identified>::Identifier,
        pre_log: Self::PreLog,
    ) -> Result<Self, Self::Error>;
    /// Apply a new extrinsic.
    fn apply_extrinsic(&mut self, extrinsic: Self::Extrinsic) -> Result<(), Self::Error>;
    /// Finalize the current block.
    fn finalize(self, post_log: Self::PostLog) -> Result<Self::Block, Self::Error>;
}
