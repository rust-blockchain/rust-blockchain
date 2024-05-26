//! # Tinyjam
//!
//! A tiny implementation of the Polkadot JAM protocol.
//!
//! ## Consensus logic
//!
//! The consensus logic of `tinyjam` consists of three parts:
//!
//! * Block production and finality: to make relay chain blocks.
//! * Map-reduce: the logic of refine, accumulate, and on transfer.
//! * In-core sealing: guaranteeing, availability, auditing and judging.
//!
//! The code is structured so that the parts are swappable. You can replace
//! the block production algorithm to simple Aura. Or you can disable
//! map-reduce, so that the chain notes raw blobs without any functionality.

pub mod core_seal;

pub struct State<Consensus> {
    /// State of the consensus, such as Safrole.
    pub consensus: Consensus,
}
