//! Recipient bounded context.

pub mod model;
pub mod repository;
pub mod service;

pub use model::{Recipient, RecipientId};
pub use repository::RecipientRepository;
pub use service::RecipientService;

/// Errors produced by the recipients bounded context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RecipientError {
    /// No recipient exists with the supplied id (within the supplied tenant).
    #[error("recipient not found")]
    NotFound,
    /// Attempted to construct a [`Recipient`] without any contact points.
    #[error("recipient must have at least one contact point")]
    NoContactPoints,
}
