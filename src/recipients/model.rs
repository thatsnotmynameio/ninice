#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Recipient aggregate and value objects.

use std::fmt;

use crate::channels::ContactPoint;
use crate::tenants::TenantId;

/// Identifier of a recipient within a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipientId(uuid::Uuid);

impl RecipientId {
    /// Generates a new random recipient id (UUID v4).
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Wraps an existing UUID into a `RecipientId`.
    #[must_use]
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for RecipientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A recipient: a tenant-scoped identity with at least one contact point.
///
/// Constructor and mutators are added in subsequent tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    /// Stable identifier.
    pub id: RecipientId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// At least one addressable destination.
    pub contact_points: Vec<ContactPoint>,
}

impl Recipient {
    /// Creates a new recipient under `tenant_id`.
    ///
    /// # Errors
    /// Returns [`RecipientError::NoContactPoints`] if `contact_points` is empty.
    pub fn new(
        tenant_id: TenantId,
        contact_points: Vec<ContactPoint>,
    ) -> Result<Self, crate::recipients::RecipientError> {
        if contact_points.is_empty() {
            return Err(crate::recipients::RecipientError::NoContactPoints);
        }
        Ok(Self {
            id: RecipientId::generate(),
            tenant_id,
            contact_points,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{ChannelKind, ContactPoint};
    use crate::recipients::RecipientError;
    use crate::tenants::TenantId;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(RecipientId::generate(), RecipientId::generate());
    }

    fn webhook_cp() -> ContactPoint {
        ContactPoint {
            kind: ChannelKind::Webhook,
            address: "https://example.com/h".into(),
        }
    }

    #[test]
    fn new_assigns_tenant_and_contact_points() {
        let tenant = TenantId::generate();
        let cp = webhook_cp();
        let r = Recipient::new(tenant, vec![cp.clone()]).unwrap();
        assert_eq!(r.tenant_id, tenant);
        assert_eq!(r.contact_points, vec![cp]);
    }

    #[test]
    fn new_rejects_empty_contact_points() {
        let tenant = TenantId::generate();
        assert!(matches!(
            Recipient::new(tenant, vec![]),
            Err(RecipientError::NoContactPoints)
        ));
    }
}
