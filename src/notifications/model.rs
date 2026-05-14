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

impl Notification {
    /// Creates a new pending notification.
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        recipient_id: RecipientId,
        channel: ChannelKind,
        content: Content,
    ) -> Self {
        Self {
            id: NotificationId::generate(),
            tenant_id,
            recipient_id,
            channel,
            content,
            status: NotificationStatus::Pending,
            created_at: chrono::Utc::now(),
            sent_at: None,
        }
    }

    /// Marks this notification as successfully sent and stamps `sent_at`.
    pub fn mark_sent(&mut self) {
        self.status = NotificationStatus::Sent;
        self.sent_at = Some(chrono::Utc::now());
    }

    /// Marks this notification as failed and records `reason`.
    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.status = NotificationStatus::Failed {
            reason: reason.into(),
        };
        self.sent_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::ChannelKind;
    use crate::recipients::RecipientId;
    use crate::tenants::TenantId;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(NotificationId::generate(), NotificationId::generate());
    }

    #[test]
    fn content_new_stores_body() {
        let c = Content::new("hello");
        assert_eq!(c.body, "hello");
    }

    #[test]
    fn new_starts_pending_without_sent_at() {
        let tenant = TenantId::generate();
        let recipient = RecipientId::generate();
        let before = chrono::Utc::now();

        let n = Notification::new(
            tenant,
            recipient,
            ChannelKind::Webhook,
            Content::new("hello"),
        );

        let after = chrono::Utc::now();
        assert_eq!(n.tenant_id, tenant);
        assert_eq!(n.recipient_id, recipient);
        assert_eq!(n.channel, ChannelKind::Webhook);
        assert_eq!(n.content, Content::new("hello"));
        assert_eq!(n.status, NotificationStatus::Pending);
        assert!(n.sent_at.is_none());
        assert!(n.created_at >= before && n.created_at <= after);
    }

    #[test]
    fn mark_sent_transitions_status_and_sets_sent_at() {
        let mut n = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        );
        assert!(n.sent_at.is_none());

        let before = chrono::Utc::now();
        n.mark_sent();
        let after = chrono::Utc::now();

        assert_eq!(n.status, NotificationStatus::Sent);
        let sent_at = n.sent_at.expect("sent_at must be set after mark_sent");
        assert!(sent_at >= before && sent_at <= after);
    }

    #[test]
    fn mark_failed_sets_reason_and_clears_sent_at() {
        let mut n = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        );
        n.mark_sent();
        assert!(n.sent_at.is_some());

        n.mark_failed("4xx response");

        assert_eq!(
            n.status,
            NotificationStatus::Failed {
                reason: "4xx response".into()
            }
        );
        assert!(n.sent_at.is_none());
    }
}
