#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! `TenantId` value object.

use std::fmt;

/// Identifier of a tenant. Lifecycle is external to this library; the
/// caller hands us values it already has. `generate` exists for tests
/// and prototypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantId(uuid::Uuid);

impl TenantId {
    /// Generates a new random tenant id (UUID v4).
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Wraps an existing UUID into a `TenantId`.
    #[must_use]
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_distinct_ids() {
        let a = TenantId::generate();
        let b = TenantId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn from_uuid_round_trips() {
        let raw = uuid::Uuid::new_v4();
        let id = TenantId::from_uuid(raw);
        assert_eq!(id.to_string(), raw.to_string());
    }
}
