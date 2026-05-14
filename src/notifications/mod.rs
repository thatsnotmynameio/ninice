//! Notification bounded context.

pub mod model;
pub mod repository;
pub mod service;

pub use model::{Content, Notification, NotificationId, NotificationStatus};
pub use repository::NotificationRepository;
pub use service::NotificationService;

use crate::channels::ChannelKind;
use crate::recipients::RecipientId;

/// Errors produced by the notifications bounded context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NotificationError {
    /// No notification exists with the supplied id (within the tenant).
    #[error("notification not found")]
    NotFound,
    /// The recipient does not exist within the tenant.
    #[error("recipient {0} not found")]
    RecipientNotFound(RecipientId),
    /// The requested channel is not currently usable.
    #[error("channel {0:?} unavailable")]
    ChannelUnavailable(ChannelKind),
}
