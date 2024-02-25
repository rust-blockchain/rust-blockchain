/// A block or a header that is identified. That is, it contains a hash, and
/// reference a parent via a parent hash.
///
/// The hash of a block is unique.
pub trait Identified {
    /// Identifier type.
    type Identifier;

    /// Get the block hash.
    fn id(&self) -> Self::Identifier;
    /// Get the parent block hash. None if this block is genesis.
    fn parent_id(&self) -> Option<Self::Identifier>;
}

/// Additional index keys for a block or a header. This can be, for example,
/// transaction hashes or block numbers.
pub trait Keyed<Key> {
    /// Get the block key.
    fn key(&self) -> Key;
}

/// A block where we can derive a header from.
///
/// The block can be thought as with a header and a body. However, in many
/// blockchains, certain tricks are used to make it so that a header and a block
/// can easily get the same hash, with structures unchanged. In those cases, the
/// header can only be "derived" from the block, and there's no clear
/// "boundary" of a header and a body.
pub trait Headered {
    /// Header type.
    type Header: Clone + Eq;

    /// Get the block header.
    fn header(&self) -> Self::Header;
}
