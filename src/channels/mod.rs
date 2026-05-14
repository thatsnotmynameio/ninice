//! Channel taxonomy and the `Channel` port.

pub mod model;
pub mod service;

pub use model::{ChannelKind, ContactPoint, WebhookUrl};
pub use service::Channel;

/// Errors produced by the channels bounded context.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ChannelError {
    /// The provided webhook URL was empty or did not start with `http(s)://`.
    #[error("invalid webhook URL")]
    InvalidWebhookUrl,
}
