//! `ninice` is a TODO command-line application.

/// Returns the canonical greeting printed by the binary.
///
/// # Examples
///
/// ```
/// assert_eq!(ninice::greeting(), "Hello, world!");
/// ```
pub fn greeting() -> &'static str {
    "Hello, world!"
}
