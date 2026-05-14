//! Recipient aggregate and value objects.

use std::fmt;

use crate::channels::ContactPoint;
use crate::recipients::RecipientError;
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
/// Immutable: every transition (e.g. [`Recipient::with_contact_point`]) consumes
/// the value and returns a new one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    id: RecipientId,
    tenant_id: TenantId,
    contact_points: Vec<ContactPoint>,
}

impl Recipient {
    /// Creates a new recipient under `tenant_id`.
    ///
    /// # Errors
    /// Returns [`RecipientError::NoContactPoints`] if `contact_points` is empty.
    pub fn new(
        tenant_id: TenantId,
        contact_points: Vec<ContactPoint>,
    ) -> Result<Self, RecipientError> {
        if contact_points.is_empty() {
            return Err(RecipientError::NoContactPoints);
        }
        Ok(Self {
            id: RecipientId::generate(),
            tenant_id,
            contact_points,
        })
    }

    /// Stable identifier.
    #[must_use]
    pub fn id(&self) -> RecipientId {
        self.id
    }

    /// Owning tenant.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// At least one addressable destination.
    #[must_use]
    pub fn contact_points(&self) -> &[ContactPoint] {
        &self.contact_points
    }

    /// Returns a new recipient with `cp` appended to its contact points.
    ///
    /// The invariant "at least one contact point" is monotonic: this
    /// operation can only grow the list.
    #[must_use]
    pub fn with_contact_point(self, cp: ContactPoint) -> Self {
        let mut contact_points = self.contact_points;
        contact_points.push(cp);
        Self {
            contact_points,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::channels::{ChannelKind, ContactPoint};
    use crate::tenants::TenantId;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(RecipientId::generate(), RecipientId::generate());
    }

    fn webhook_cp() -> ContactPoint {
        ContactPoint::new(ChannelKind::Webhook, "https://example.com/h")
    }

    #[test]
    fn new_assigns_tenant_and_contact_points() {
        let tenant = TenantId::generate();
        let cp = webhook_cp();
        let r = Recipient::new(tenant, vec![cp.clone()]).unwrap();
        assert_eq!(r.tenant_id(), tenant);
        assert_eq!(r.contact_points(), &[cp][..]);
    }

    #[test]
    fn new_rejects_empty_contact_points() {
        let tenant = TenantId::generate();
        assert!(matches!(
            Recipient::new(tenant, vec![]),
            Err(RecipientError::NoContactPoints)
        ));
    }

    #[test]
    fn with_contact_point_returns_new_recipient_with_appended_point() {
        let tenant = TenantId::generate();
        let cp1 = webhook_cp();
        let original = Recipient::new(tenant, vec![cp1.clone()]).unwrap();
        let original_id = original.id();

        let cp2 = ContactPoint::new(ChannelKind::Webhook, "https://other.example/h");
        let updated = original.with_contact_point(cp2.clone());

        assert_eq!(updated.id(), original_id);
        assert_eq!(updated.tenant_id(), tenant);
        assert_eq!(updated.contact_points(), &[cp1, cp2][..]);
    }

    #[test]
    fn with_contact_point_does_not_affect_prior_snapshot() {
        let tenant = TenantId::generate();
        let cp1 = webhook_cp();
        let snapshot = Recipient::new(tenant, vec![cp1.clone()]).unwrap();
        let before = snapshot.clone();

        let cp2 = ContactPoint::new(ChannelKind::Webhook, "https://other.example/h");
        let _updated = snapshot.with_contact_point(cp2);

        assert_eq!(before.contact_points(), &[cp1][..]);
    }
}
