#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Notification aggregate and value objects.

use std::fmt;

use crate::channels::ChannelKind;
use crate::recipients::RecipientId;
use crate::tenants::TenantId;

/// Identifier of a notification within a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(uuid::Uuid);

impl NotificationId {
    /// Generates a new random notification id (UUID v4).
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Wraps an existing UUID into a `NotificationId`.
    #[must_use]
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for NotificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Payload of a notification. Channel-agnostic; semantics depend on the
/// channel (for Webhook, typically a serialized JSON body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    /// The body of the message.
    pub body: String,
}

impl Content {
    /// Builds a new `Content` from any string-like value.
    #[must_use]
    pub fn new(body: impl Into<String>) -> Self {
        Self { body: body.into() }
    }
}

/// Lifecycle status of a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationStatus {
    /// Created but not yet attempted.
    Pending,
    /// Successfully delivered.
    Sent,
    /// Delivery failed; `reason` carries a description.
    Failed {
        /// Free-form description of the failure.
        reason: String,
    },
}

/// A single one-shot notification.
///
/// Constructor and state-transition methods are added in subsequent tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// Stable identifier.
    pub id: NotificationId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// Target recipient (must belong to the same tenant; service-enforced).
    pub recipient_id: RecipientId,
    /// Channel to deliver through.
    pub channel: ChannelKind,
    /// Payload.
    pub content: Content,
    /// Current lifecycle status.
    pub status: NotificationStatus,
    /// When the notification was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the notification was last marked as sent, if ever.
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(NotificationId::generate(), NotificationId::generate());
    }

    #[test]
    fn content_new_stores_body() {
        let c = Content::new("hello");
        assert_eq!(c.body, "hello");
    }
}
