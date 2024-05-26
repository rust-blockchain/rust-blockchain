//! # In-core sealing algorithm.
//!
//! The in-core sealing protocol is a generalization of Polkadot JAM's part
//! of the in-core computation. It ensures the following:
//!
//! * Given a work package, it ensures that it is *authorized* on the current
//!   core.
//! * Participating validators would then *refine* the work package to produce
//!   a work report.
//! * It further ensures availability, and later handle disputes.
//!
//! The types of the in-core sealing algorithm defined here deals with only a
//! single core (in another word, there's no core ID). It is expected that
//! different cores will need their own worker thread anyway.
//!
//! We do not have the concept of a block in this module. The algorithm works
//! through a *handle*, which acts as a state machine, with always-up-to-date
//! information. It's the responsibility of the handle to fetch states and to
//! update states to the relay chain blocks. In practice, work packages
//! usually have their own pins to specific blocks. No block building or
//! transaction creation is done in this module. In all other cases, it operates
//! over the current best blocks.
//!
//! Related to the specification, this module only handles in-core. This means
//! `authorize` and `refine`, but not later stages of `accumulate` and
//! `on_transfer`.
//!
//! ## Cycle of the worker
//!
//! The worker thread accepts a stream receiving work packages. Upon checking
//! that the work package is authorized, it takes ownership of it, refines it to
//! get the work report, and then attest it to publish on the relay chain.
//!
//! Another stream will receive a tuple of work packages and work reports
//! already generated, and attest them.

use std::future::Future;

/// Handle for the in-core sealing.
///
/// This works like a state machine. Work packages usually have their own pins,
/// and if not specified, it works against the best block / the most updated
/// network.
pub trait CoreSealHandle {
    /// Error type for the handle.
    type Error;

    /// A work package, pre-refine.
    type WorkPackage;
    /// A work report from a work package, post-refine.
    type WorkReport;

    /// Whether the work package is authorized on the current core.
    fn is_authorized(&self, work: &Self::WorkPackage) -> bool;
    /// Refine from a work package into a work report.
    fn refine(
        &self,
        work: Self::WorkPackage,
    ) -> impl Future<Output = Result<Self::WorkReport, Self::Error>> + Send;

    /// Attest to a work report and submit it.
    fn attest(
        &mut self,
        report: Self::WorkReport,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Dispute a work report and submit it.
    fn dispute(
        &mut self,
        own: Self::WorkReport,
        other: Self::WorkReport,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
