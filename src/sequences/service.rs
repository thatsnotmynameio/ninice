//! Sequence service port.

use crate::recipients::RecipientId;
use crate::sequences::{EnrollmentId, SequenceError, SequenceId};
use crate::tenants::TenantId;

/// Application service for sequence enrollment lifecycle.
pub trait SequenceService: Send + Sync {
    /// Enrolls `recipient_id` into `sequence_id`.
    ///
    /// Implementations must verify both belong to `tenant_id` before
    /// creating the enrollment.
    ///
    /// # Errors
    /// See [`SequenceError`] variants.
    fn enroll(
        &self,
        tenant_id: TenantId,
        sequence_id: SequenceId,
        recipient_id: RecipientId,
    ) -> Result<EnrollmentId, SequenceError>;

    /// Cancels an active enrollment.
    ///
    /// # Errors
    /// Returns [`SequenceError::EnrollmentNotFound`] when the enrollment
    /// does not exist in `tenant_id`.
    fn cancel(
        &self,
        tenant_id: &TenantId,
        enrollment_id: &EnrollmentId,
    ) -> Result<(), SequenceError>;
}

#[cfg(test)]
mod tests {}
