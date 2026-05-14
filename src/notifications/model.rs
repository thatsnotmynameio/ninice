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
///
/// Immutable: construct via [`Content::new`], read via [`Content::body`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    body: String,
}

impl Content {
    /// Builds a new `Content` from any string-like value.
    #[must_use]
    pub fn new(body: impl Into<String>) -> Self {
        Self { body: body.into() }
    }

    /// The body of the message.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
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
/// Immutable: state transitions ([`Notification::mark_sent`],
/// [`Notification::mark_failed`]) consume the value and return a new one
/// with the same identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    id: NotificationId,
    tenant_id: TenantId,
    recipient_id: RecipientId,
    channel: ChannelKind,
    content: Content,
    status: NotificationStatus,
    created_at: chrono::DateTime<chrono::Utc>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
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

    /// Stable identifier.
    #[must_use]
    pub fn id(&self) -> NotificationId {
        self.id
    }

    /// Owning tenant.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Target recipient.
    #[must_use]
    pub fn recipient_id(&self) -> RecipientId {
        self.recipient_id
    }

    /// Channel to deliver through.
    #[must_use]
    pub fn channel(&self) -> ChannelKind {
        self.channel
    }

    /// Payload.
    #[must_use]
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Current lifecycle status.
    #[must_use]
    pub fn status(&self) -> &NotificationStatus {
        &self.status
    }

    /// When the notification was created.
    #[must_use]
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// When the notification was last marked as sent, if ever.
    #[must_use]
    pub fn sent_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.sent_at
    }

    /// Returns a new notification with status `Sent` and `sent_at` set to now.
    #[must_use]
    pub fn mark_sent(self) -> Self {
        Self {
            status: NotificationStatus::Sent,
            sent_at: Some(chrono::Utc::now()),
            ..self
        }
    }

    /// Returns a new notification with status `Failed { reason }` and `sent_at` cleared.
    #[must_use]
    pub fn mark_failed(self, reason: impl Into<String>) -> Self {
        Self {
            status: NotificationStatus::Failed {
                reason: reason.into(),
            },
            sent_at: None,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::channels::ChannelKind;
    use crate::recipients::RecipientId;
    use crate::tenants::TenantId;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(NotificationId::generate(), NotificationId::generate());
    }

    #[test]
    fn from_uuid_round_trips_through_display() {
        let raw = uuid::Uuid::new_v4();
        let id = NotificationId::from_uuid(raw);
        assert_eq!(id.to_string(), raw.to_string());
    }

    #[test]
    fn content_new_stores_body() {
        let c = Content::new("hello");
        assert_eq!(c.body(), "hello");
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
        assert_eq!(n.tenant_id(), tenant);
        assert_eq!(n.recipient_id(), recipient);
        assert_eq!(n.channel(), ChannelKind::Webhook);
        assert_eq!(n.content(), &Content::new("hello"));
        assert_eq!(n.status(), &NotificationStatus::Pending);
        assert!(n.sent_at().is_none());
        assert!(n.created_at() >= before && n.created_at() <= after);
    }

    #[test]
    fn mark_sent_returns_new_notification_with_sent_status_and_timestamp() {
        let original = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        );
        let original_id = original.id();
        assert!(original.sent_at().is_none());

        let before = chrono::Utc::now();
        let sent = original.mark_sent();
        let after = chrono::Utc::now();

        assert_eq!(sent.id(), original_id);
        assert_eq!(sent.status(), &NotificationStatus::Sent);
        let ts = sent.sent_at().expect("sent_at must be set after mark_sent");
        assert!(ts >= before && ts <= after);
    }

    #[test]
    fn mark_failed_returns_new_notification_with_reason_and_cleared_timestamp() {
        let n = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        )
        .mark_sent();
        assert!(n.sent_at().is_some());
        let id_before = n.id();

        let failed = n.mark_failed("4xx response");

        assert_eq!(failed.id(), id_before);
        assert_eq!(
            failed.status(),
            &NotificationStatus::Failed {
                reason: "4xx response".into()
            }
        );
        assert!(failed.sent_at().is_none());
    }

    #[test]
    fn mark_sent_does_not_affect_prior_snapshot() {
        let pending = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        );
        let snapshot = pending.clone();

        let _sent = pending.mark_sent();

        assert_eq!(snapshot.status(), &NotificationStatus::Pending);
        assert!(snapshot.sent_at().is_none());
    }

    #[test]
    fn mark_failed_does_not_affect_prior_snapshot() {
        let sent = Notification::new(
            TenantId::generate(),
            RecipientId::generate(),
            ChannelKind::Webhook,
            Content::new("hi"),
        )
        .mark_sent();
        let snapshot = sent.clone();

        let _failed = sent.mark_failed("boom");

        assert_eq!(snapshot.status(), &NotificationStatus::Sent);
        assert!(snapshot.sent_at().is_some());
    }
}
