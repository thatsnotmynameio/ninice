//! `Channel` port.

use crate::channels::ChannelKind;

/// A channel adapter that knows how to identify itself.
///
/// The `send` operation is intentionally not declared yet; its shape
/// (sync/async, payload, retry semantics) is a downstream decision.
pub trait Channel: Send + Sync {
    /// Returns the kind of channel this adapter handles.
    fn kind(&self) -> ChannelKind;
}

#[cfg(test)]
mod tests {}
