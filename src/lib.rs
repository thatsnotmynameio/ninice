//! `ninice` is a multi-channel, multi-tenant notification management library.

pub mod channels;
pub mod notifications;
pub mod recipients;
pub mod sequences;
pub mod tenants;

pub use channels::{Channel, ChannelError, ChannelKind, ContactPoint, WebhookUrl};
pub use notifications::{
    Content, Notification, NotificationError, NotificationId, NotificationRepository,
    NotificationService, NotificationStatus,
};
pub use recipients::{
    Recipient, RecipientError, RecipientId, RecipientRepository, RecipientService,
};
pub use sequences::{
    Enrollment, EnrollmentId, EnrollmentRepository, EnrollmentStatus, Sequence, SequenceBuilder,
    SequenceError, SequenceId, SequenceRepository, SequenceService, SequenceStep,
};
pub use tenants::TenantId;

/// Returns the banner the placeholder binary prints (`"ninice <CARGO_PKG_VERSION>"`).
///
/// Extracted so the binary's logic is exercised by unit tests; once the real
/// entry point (CLI/HTTP/worker) is decided, this is the first thing to retire.
#[must_use]
pub fn version_banner() -> String {
    format!("ninice {}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_banner_includes_crate_name_and_version() {
        let banner = version_banner();
        assert!(banner.starts_with("ninice "), "banner: {banner}");
        assert!(
            banner.ends_with(env!("CARGO_PKG_VERSION")),
            "banner: {banner}"
        );
    }
}
