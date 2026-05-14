//! `ninice` is a multi-channel, multi-tenant notification management library.

pub mod channels;
pub mod notifications;
pub mod recipients;
pub mod sequences;
pub mod tenants;

pub use channels::{Channel, ChannelError, ChannelKind, ContactPoint, WebhookUrl};
pub use recipients::{
    Recipient, RecipientError, RecipientId, RecipientRepository, RecipientService,
};
pub use tenants::TenantId;
