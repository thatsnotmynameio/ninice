//! Sequence and Enrollment repository ports.

use crate::sequences::{Enrollment, EnrollmentId, Sequence, SequenceError, SequenceId};
use crate::tenants::TenantId;

/// Persistence port for [`Sequence`] definitions.
pub trait SequenceRepository: Send + Sync {
    /// Persists `s`.
    ///
    /// # Errors
    /// Returns a [`SequenceError`] when the backing store fails.
    fn save(&self, s: &Sequence) -> Result<(), SequenceError>;

    /// Finds a sequence scoped to `tenant_id`.
    ///
    /// # Errors
    /// Returns a [`SequenceError`] when the backing store fails.
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &SequenceId,
    ) -> Result<Option<Sequence>, SequenceError>;
}

/// Persistence port for [`Enrollment`] instances.
pub trait EnrollmentRepository: Send + Sync {
    /// Persists `e`.
    ///
    /// # Errors
    /// Returns a [`SequenceError`] when the backing store fails.
    fn save(&self, e: &Enrollment) -> Result<(), SequenceError>;

    /// Finds an enrollment scoped to `tenant_id`.
    ///
    /// # Errors
    /// Returns a [`SequenceError`] when the backing store fails.
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &EnrollmentId,
    ) -> Result<Option<Enrollment>, SequenceError>;

    /// Returns all active enrollments in `tenant_id` whose next step is
    /// due at or before `now`.
    ///
    /// # Errors
    /// Returns a [`SequenceError`] when the backing store fails.
    fn find_active_due(
        &self,
        tenant_id: &TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Enrollment>, SequenceError>;
}

#[cfg(test)]
mod tests {}
