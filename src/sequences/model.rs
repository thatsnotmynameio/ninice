//! Sequence and Enrollment aggregates.

use std::fmt;
use std::time::Duration;

use crate::channels::ChannelKind;
use crate::notifications::Content;
use crate::recipients::RecipientId;
use crate::sequences::SequenceError;
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
///
/// Immutable: construct via [`SequenceStep::new`], read via accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceStep {
    delay: Duration,
    channel: ChannelKind,
    content: Content,
}

impl SequenceStep {
    /// Creates a step with the given delay, channel, and content.
    #[must_use]
    pub fn new(delay: Duration, channel: ChannelKind, content: Content) -> Self {
        Self {
            delay,
            channel,
            content,
        }
    }

    /// Wait time after the previous step (or after enrollment for the first step).
    #[must_use]
    pub fn delay(&self) -> Duration {
        self.delay
    }

    /// Channel to use for this step.
    #[must_use]
    pub fn channel(&self) -> ChannelKind {
        self.channel
    }

    /// Payload for this step.
    #[must_use]
    pub fn content(&self) -> &Content {
        &self.content
    }
}

/// A multi-step sequence definition, scoped to a tenant.
///
/// Immutable. Construct via [`Sequence::builder`]; once built, the value
/// can only be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    id: SequenceId,
    tenant_id: TenantId,
    name: String,
    steps: Vec<SequenceStep>,
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

    /// Stable identifier.
    #[must_use]
    pub fn id(&self) -> SequenceId {
        self.id
    }

    /// Owning tenant.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Ordered steps; non-empty by construction.
    #[must_use]
    pub fn steps(&self) -> &[SequenceStep] {
        &self.steps
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
    /// Returns a new builder with `step` appended.
    #[must_use]
    pub fn add_step(self, step: SequenceStep) -> Self {
        let mut steps = self.steps;
        steps.push(step);
        Self { steps, ..self }
    }

    /// Finalizes the sequence.
    ///
    /// # Errors
    /// Returns [`SequenceError::EmptySteps`] if no steps were added.
    pub fn build(self) -> Result<Sequence, SequenceError> {
        if self.steps.is_empty() {
            return Err(SequenceError::EmptySteps);
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
/// Immutable. Construct via [`Enrollment::new`]; transitions
/// ([`Enrollment::advance`], [`Enrollment::cancel`]) consume the value and
/// return a new one with the same identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrollment {
    id: EnrollmentId,
    tenant_id: TenantId,
    sequence_id: SequenceId,
    recipient_id: RecipientId,
    status: EnrollmentStatus,
    enrolled_at: chrono::DateTime<chrono::Utc>,
}

impl Enrollment {
    /// Enrolls `recipient_id` into `sequence_id` under `tenant_id`.
    #[must_use]
    pub fn new(tenant_id: TenantId, sequence_id: SequenceId, recipient_id: RecipientId) -> Self {
        Self {
            id: EnrollmentId::generate(),
            tenant_id,
            sequence_id,
            recipient_id,
            status: EnrollmentStatus::Active { next_step_index: 0 },
            enrolled_at: chrono::Utc::now(),
        }
    }

    /// Stable identifier.
    #[must_use]
    pub fn id(&self) -> EnrollmentId {
        self.id
    }

    /// Owning tenant.
    #[must_use]
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// The sequence being walked.
    #[must_use]
    pub fn sequence_id(&self) -> SequenceId {
        self.sequence_id
    }

    /// The recipient walking it.
    #[must_use]
    pub fn recipient_id(&self) -> RecipientId {
        self.recipient_id
    }

    /// Current status.
    #[must_use]
    pub fn status(&self) -> &EnrollmentStatus {
        &self.status
    }

    /// When enrollment was created.
    #[must_use]
    pub fn enrolled_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.enrolled_at
    }

    /// Returns a new enrollment advanced by one step.
    ///
    /// - `Active { i }` becomes `Active { i + 1 }` when `i + 1 < total_steps`.
    /// - `Active { i }` becomes `Completed` when `i + 1 >= total_steps`.
    /// - `Completed` and `Cancelled` are terminal — returns the same status unchanged.
    #[must_use]
    pub fn advance(self, total_steps: usize) -> Self {
        let next_status = match self.status {
            EnrollmentStatus::Active { next_step_index } => {
                let next = next_step_index + 1;
                if next >= total_steps {
                    EnrollmentStatus::Completed
                } else {
                    EnrollmentStatus::Active {
                        next_step_index: next,
                    }
                }
            }
            terminal => terminal,
        };
        Self {
            status: next_status,
            ..self
        }
    }

    /// Returns a cancelled copy of this enrollment regardless of its prior state.
    #[must_use]
    pub fn cancel(self) -> Self {
        Self {
            status: EnrollmentStatus::Cancelled,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn sequence_id_generate_produces_distinct() {
        assert_ne!(SequenceId::generate(), SequenceId::generate());
    }

    #[test]
    fn sequence_id_from_uuid_round_trips_through_display() {
        let raw = uuid::Uuid::new_v4();
        let id = SequenceId::from_uuid(raw);
        assert_eq!(id.to_string(), raw.to_string());
    }

    #[test]
    fn enrollment_id_generate_produces_distinct() {
        assert_ne!(EnrollmentId::generate(), EnrollmentId::generate());
    }

    #[test]
    fn enrollment_id_from_uuid_round_trips_through_display() {
        let raw = uuid::Uuid::new_v4();
        let id = EnrollmentId::from_uuid(raw);
        assert_eq!(id.to_string(), raw.to_string());
    }

    fn sample_step() -> SequenceStep {
        SequenceStep::new(
            Duration::from_secs(60),
            ChannelKind::Webhook,
            Content::new("hi"),
        )
    }

    #[test]
    fn sequence_step_exposes_fields() {
        let s = sample_step();
        assert_eq!(s.delay(), Duration::from_secs(60));
        assert_eq!(s.channel(), ChannelKind::Webhook);
        assert_eq!(s.content(), &Content::new("hi"));
    }

    #[test]
    fn builder_rejects_empty_steps() {
        let tenant = TenantId::generate();
        let result = Sequence::builder(tenant, "welcome").build();
        assert!(matches!(result, Err(SequenceError::EmptySteps)));
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
        assert_eq!(seq.tenant_id(), tenant);
        assert_eq!(seq.name(), "welcome");
        assert_eq!(seq.steps().len(), 2);
        assert_eq!(&seq.steps()[0], &s);
    }

    #[test]
    fn each_built_sequence_has_a_distinct_id() {
        let tenant = TenantId::generate();
        let a = Sequence::builder(tenant, "x")
            .add_step(sample_step())
            .build()
            .unwrap();
        let b = Sequence::builder(tenant, "x")
            .add_step(sample_step())
            .build()
            .unwrap();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn enrollment_new_starts_active_at_index_zero() {
        let tenant = TenantId::generate();
        let seq = SequenceId::generate();
        let rec = RecipientId::generate();
        let before = chrono::Utc::now();

        let e = Enrollment::new(tenant, seq, rec);

        let after = chrono::Utc::now();
        assert_eq!(e.tenant_id(), tenant);
        assert_eq!(e.sequence_id(), seq);
        assert_eq!(e.recipient_id(), rec);
        assert_eq!(e.status(), &EnrollmentStatus::Active { next_step_index: 0 });
        assert!(e.enrolled_at() >= before && e.enrolled_at() <= after);
    }

    #[test]
    fn advance_returns_new_enrollment_at_next_step() {
        let e = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        );
        let original_id = e.id();

        let advanced = e.advance(3);

        assert_eq!(advanced.id(), original_id);
        assert_eq!(
            advanced.status(),
            &EnrollmentStatus::Active { next_step_index: 1 }
        );
    }

    #[test]
    fn advance_to_last_step_completes() {
        let e = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        )
        .advance(2) // Active{0} -> Active{1}
        .advance(2); // Active{1} -> Completed
        assert_eq!(e.status(), &EnrollmentStatus::Completed);
    }

    #[test]
    fn advance_is_noop_when_completed() {
        let completed = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        )
        .advance(1); // Active{0} -> Completed
        assert_eq!(completed.status(), &EnrollmentStatus::Completed);

        let still_completed = completed.advance(1);
        assert_eq!(still_completed.status(), &EnrollmentStatus::Completed);
    }

    #[test]
    fn cancel_returns_cancelled_enrollment() {
        let cancelled = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        )
        .cancel();
        assert_eq!(cancelled.status(), &EnrollmentStatus::Cancelled);
    }

    #[test]
    fn cancel_overrides_completed() {
        let cancelled = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        )
        .advance(1) // -> Completed
        .cancel();
        assert_eq!(cancelled.status(), &EnrollmentStatus::Cancelled);
    }

    #[test]
    fn advance_does_not_affect_prior_snapshot() {
        let active = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        );
        let snapshot = active.clone();

        let _advanced = active.advance(3);

        assert_eq!(
            snapshot.status(),
            &EnrollmentStatus::Active { next_step_index: 0 }
        );
    }

    #[test]
    fn cancel_does_not_affect_prior_snapshot() {
        let active = Enrollment::new(
            TenantId::generate(),
            SequenceId::generate(),
            RecipientId::generate(),
        );
        let snapshot = active.clone();

        let _cancelled = active.cancel();

        assert_eq!(
            snapshot.status(),
            &EnrollmentStatus::Active { next_step_index: 0 }
        );
    }
}
