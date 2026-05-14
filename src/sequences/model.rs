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

impl Sequence {
    /// Starts building a sequence under `tenant_id` with the given `name`.
    #[must_use]
    pub fn builder(tenant_id: TenantId, name: impl Into<String>) -> SequenceBuilder {
        SequenceBuilder {
            tenant_id,
            name: name.into(),
            steps: Vec::new(),
        }
    }
}

/// Fluent builder for [`Sequence`].
#[derive(Debug)]
pub struct SequenceBuilder {
    tenant_id: TenantId,
    name: String,
    steps: Vec<SequenceStep>,
}

impl SequenceBuilder {
    /// Appends `step` to the sequence under construction.
    #[must_use]
    pub fn add_step(mut self, step: SequenceStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Finalizes the sequence.
    ///
    /// # Errors
    /// Returns [`crate::sequences::SequenceError::EmptySteps`] if no steps were added.
    pub fn build(self) -> Result<Sequence, crate::sequences::SequenceError> {
        if self.steps.is_empty() {
            return Err(crate::sequences::SequenceError::EmptySteps);
        }
        Ok(Sequence {
            id: SequenceId::generate(),
            tenant_id: self.tenant_id,
            name: self.name,
            steps: self.steps,
        })
    }
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

impl Enrollment {
    /// Enrolls `recipient_id` into `sequence_id` under `tenant_id`.
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        sequence_id: SequenceId,
        recipient_id: RecipientId,
    ) -> Self {
        Self {
            id: EnrollmentId::generate(),
            tenant_id,
            sequence_id,
            recipient_id,
            status: EnrollmentStatus::Active { next_step_index: 0 },
            enrolled_at: chrono::Utc::now(),
        }
    }
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

    fn sample_step() -> SequenceStep {
        SequenceStep {
            delay: Duration::from_secs(60),
            channel: ChannelKind::Webhook,
            content: Content::new("hi"),
        }
    }

    #[test]
    fn builder_rejects_empty_steps() {
        let tenant = TenantId::generate();
        let result = Sequence::builder(tenant, "welcome").build();
        assert!(matches!(result, Err(crate::sequences::SequenceError::EmptySteps)));
    }

    #[test]
    fn builder_accumulates_steps() {
        let tenant = TenantId::generate();
        let s = sample_step();
        let seq = Sequence::builder(tenant, "welcome")
            .add_step(s.clone())
            .add_step(s.clone())
            .build()
            .unwrap();
        assert_eq!(seq.tenant_id, tenant);
        assert_eq!(seq.name, "welcome");
        assert_eq!(seq.steps.len(), 2);
        assert_eq!(seq.steps[0], s);
    }

    #[test]
    fn enrollment_new_starts_active_at_index_zero() {
        let tenant = TenantId::generate();
        let seq = SequenceId::generate();
        let rec = RecipientId::generate();
        let before = chrono::Utc::now();

        let e = Enrollment::new(tenant, seq, rec);

        let after = chrono::Utc::now();
        assert_eq!(e.tenant_id, tenant);
        assert_eq!(e.sequence_id, seq);
        assert_eq!(e.recipient_id, rec);
        assert_eq!(e.status, EnrollmentStatus::Active { next_step_index: 0 });
        assert!(e.enrolled_at >= before && e.enrolled_at <= after);
    }
}
