#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Notification service port.

use crate::channels::ChannelKind;
use crate::notifications::{Content, NotificationError, NotificationId};
use crate::recipients::RecipientId;
use crate::tenants::TenantId;

/// Application service for one-shot notifications.
pub trait NotificationService: Send + Sync {
    /// Sends a one-shot notification.
    ///
    /// Implementations must verify that `recipient_id` belongs to
    /// `tenant_id` before creating the notification; cross-tenant
    /// access returns [`NotificationError::RecipientNotFound`].
    ///
    /// # Errors
    /// See [`NotificationError`] variants.
    fn send_one_off(
        &self,
        tenant_id: TenantId,
        recipient_id: RecipientId,
        channel: ChannelKind,
        content: Content,
    ) -> Result<NotificationId, NotificationError>;
}

#[cfg(test)]
mod tests {}
