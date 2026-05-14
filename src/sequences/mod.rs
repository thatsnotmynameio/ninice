//! Sequence bounded context (drip / multi-step flows).

pub mod model;
pub mod repository;
pub mod service;

pub use model::{
    Enrollment, EnrollmentId, EnrollmentStatus, Sequence, SequenceBuilder, SequenceId, SequenceStep,
};
pub use repository::{EnrollmentRepository, SequenceRepository};
pub use service::SequenceService;

use crate::recipients::RecipientId;

/// Errors produced by the sequences bounded context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SequenceError {
    /// No sequence exists with the supplied id (within the tenant).
    #[error("sequence not found")]
    NotFound,
    /// No enrollment exists with the supplied id (within the tenant).
    #[error("enrollment not found")]
    EnrollmentNotFound,
    /// Attempted to build a [`Sequence`] without any steps.
    #[error("sequence must have at least one step")]
    EmptySteps,
    /// The recipient does not exist within the tenant.
    #[error("recipient {0} not found")]
    RecipientNotFound(RecipientId),
}
