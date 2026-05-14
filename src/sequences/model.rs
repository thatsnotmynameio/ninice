#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Sequence and Enrollment aggregates.

use std::fmt;
use std::time::Duration;

use crate::channels::ChannelKind;
use crate::notifications::Content;
use crate::recipients::RecipientId;
use crate::tenants::TenantId;

/// Identifier of a sequence definition within a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceId(uuid::Uuid);

impl SequenceId {
    /// Generates a new random sequence id (UUID v4).
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Wraps an existing UUID into a `SequenceId`.
    #[must_use]
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for SequenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifier of an enrollment (a recipient's instance of a sequence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnrollmentId(uuid::Uuid);

impl EnrollmentId {
    /// Generates a new random enrollment id (UUID v4).
    #[must_use]
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Wraps an existing UUID into an `EnrollmentId`.
    #[must_use]
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

impl fmt::Display for EnrollmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// One step in a sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceStep {
    /// Wait time after the previous step (or after enrollment for the first step).
    pub delay: Duration,
    /// Channel to use for this step.
    pub channel: ChannelKind,
    /// Payload for this step.
    pub content: Content,
}

/// A multi-step sequence definition, scoped to a tenant.
///
/// Construct via [`Sequence::builder`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    /// Stable identifier.
    pub id: SequenceId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// Human-readable name.
    pub name: String,
    /// Ordered steps; non-empty by construction.
    pub steps: Vec<SequenceStep>,
}

/// Progress status of an enrollment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrollmentStatus {
    /// The enrollment is in progress; `next_step_index` is the index of
    /// the next step to deliver in the parent sequence's `steps` vector.
    Active {
        /// Index of the next step to fire.
        next_step_index: usize,
    },
    /// All steps were delivered.
    Completed,
    /// Cancelled before completion.
    Cancelled,
}

/// A recipient's progress through a sequence.
///
/// Construct via [`Enrollment::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrollment {
    /// Stable identifier.
    pub id: EnrollmentId,
    /// Owning tenant.
    pub tenant_id: TenantId,
    /// The sequence being walked.
    pub sequence_id: SequenceId,
    /// The recipient walking it.
    pub recipient_id: RecipientId,
    /// Current status.
    pub status: EnrollmentStatus,
    /// When enrollment was created.
    pub enrolled_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_id_generate_produces_distinct() {
        assert_ne!(SequenceId::generate(), SequenceId::generate());
    }

    #[test]
    fn enrollment_id_generate_produces_distinct() {
        assert_ne!(EnrollmentId::generate(), EnrollmentId::generate());
    }
}
