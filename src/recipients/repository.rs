//! Recipient repository port.

use crate::recipients::{Recipient, RecipientError, RecipientId};
use crate::tenants::TenantId;

/// Persistence port for [`Recipient`].
///
/// Implementations enforce tenant scoping: looking up an id that
/// belongs to a different tenant returns `Ok(None)` (cross-tenant
/// access is indistinguishable from non-existence).
pub trait RecipientRepository: Send + Sync {
    /// Persists `recipient`.
    ///
    /// # Errors
    /// Returns a [`RecipientError`] when the backing store fails.
    fn save(&self, recipient: &Recipient) -> Result<(), RecipientError>;

    /// Finds a recipient scoped to `tenant_id`.
    ///
    /// # Errors
    /// Returns a [`RecipientError`] when the backing store fails.
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &RecipientId,
    ) -> Result<Option<Recipient>, RecipientError>;
}

#[cfg(test)]
mod tests {}
