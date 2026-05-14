#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Recipient service port.

use crate::channels::ContactPoint;
use crate::recipients::{RecipientError, RecipientId};
use crate::tenants::TenantId;

/// Application service for managing recipients.
pub trait RecipientService: Send + Sync {
    /// Registers a new recipient under `tenant_id`.
    ///
    /// # Errors
    /// Returns [`RecipientError::NoContactPoints`] if `contact_points` is empty,
    /// or any other variant on persistence failure.
    fn register(
        &self,
        tenant_id: TenantId,
        contact_points: Vec<ContactPoint>,
    ) -> Result<RecipientId, RecipientError>;

    /// Adds an additional contact point to an existing recipient.
    ///
    /// # Errors
    /// Returns [`RecipientError::NotFound`] if the recipient does not exist
    /// in `tenant_id`.
    fn add_contact_point(
        &self,
        tenant_id: &TenantId,
        id: &RecipientId,
        cp: ContactPoint,
    ) -> Result<(), RecipientError>;
}

#[cfg(test)]
mod tests {}
