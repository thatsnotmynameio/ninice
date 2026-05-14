//! Channel value objects.

use crate::channels::ChannelError;

/// Kind of communication channel a notification travels through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChannelKind {
    /// HTTP webhook target.
    Webhook,
}

/// A validated webhook URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebhookUrl(String);

impl WebhookUrl {
    /// Parses `raw` as a webhook target URL.
    ///
    /// Validation is intentionally minimal in this stub iteration:
    /// the value must be non-empty and start with `http://` or `https://`.
    /// Real URL parsing (via the `url` crate) is deferred to the impl phase.
    ///
    /// # Errors
    ///
    /// Returns [`ChannelError::InvalidWebhookUrl`] when validation fails.
    pub fn parse(raw: impl Into<String>) -> Result<Self, ChannelError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(ChannelError::InvalidWebhookUrl);
        }
        if !raw.starts_with("http://") && !raw.starts_with("https://") {
            return Err(ChannelError::InvalidWebhookUrl);
        }
        Ok(Self(raw))
    }

    /// Returns the raw URL string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A single addressable destination for a recipient.
///
/// Immutable: construct via [`ContactPoint::new`], read via accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactPoint {
    kind: ChannelKind,
    address: String,
}

impl ContactPoint {
    /// Creates a contact point pairing a channel with its address.
    #[must_use]
    pub fn new(kind: ChannelKind, address: impl Into<String>) -> Self {
        Self {
            kind,
            address: address.into(),
        }
    }

    /// The channel kind to use for this destination.
    #[must_use]
    pub fn kind(&self) -> ChannelKind {
        self.kind
    }

    /// The channel-specific address (for [`ChannelKind::Webhook`], a URL).
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::channels::ChannelError;

    #[test]
    fn parses_https_url() {
        let url = WebhookUrl::parse("https://example.com/hook").unwrap();
        assert_eq!(url.as_str(), "https://example.com/hook");
    }

    #[test]
    fn parses_http_url() {
        let url = WebhookUrl::parse("http://example.com/hook").unwrap();
        assert_eq!(url.as_str(), "http://example.com/hook");
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!(
            WebhookUrl::parse(""),
            Err(ChannelError::InvalidWebhookUrl)
        ));
    }

    #[test]
    fn rejects_missing_scheme() {
        assert!(matches!(
            WebhookUrl::parse("example.com/hook"),
            Err(ChannelError::InvalidWebhookUrl)
        ));
    }

    #[test]
    fn contact_point_exposes_kind_and_address() {
        let cp = ContactPoint::new(ChannelKind::Webhook, "https://example.com/h");
        assert_eq!(cp.kind(), ChannelKind::Webhook);
        assert_eq!(cp.address(), "https://example.com/h");
    }
}
