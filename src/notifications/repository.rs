//! Notification repository port.

use crate::notifications::{Notification, NotificationError, NotificationId};
use crate::tenants::TenantId;

/// Persistence port for [`Notification`].
///
/// Lookups are tenant-scoped; cross-tenant access returns `Ok(None)`.
pub trait NotificationRepository: Send + Sync {
    /// Persists `n`.
    ///
    /// # Errors
    /// Returns a [`NotificationError`] when the backing store fails.
    fn save(&self, n: &Notification) -> Result<(), NotificationError>;

    /// Finds a notification scoped to `tenant_id`.
    ///
    /// # Errors
    /// Returns a [`NotificationError`] when the backing store fails.
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &NotificationId,
    ) -> Result<Option<Notification>, NotificationError>;
}

#[cfg(test)]
mod tests {}
