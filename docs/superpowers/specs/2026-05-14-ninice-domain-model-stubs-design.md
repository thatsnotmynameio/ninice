# ninice — Domain Model Stubs Design

**Date:** 2026-05-14
**Status:** Draft (awaiting review)
**Scope:** Map the multi-tenant domain model and create compilable stubs. No real implementations of repositories, services, or channel adapters.

## 1. Context

`ninice` is a Rust crate (`lib` + `bin`) currently containing a `greeting()` placeholder. The goal of this iteration is to replace the placeholder with the **domain skeleton** of a multi-channel, multi-tenant notification management system, designed with DDD-style bounded contexts.

Out of scope for this iteration:

- Concrete repository implementations (no in-memory, no DB).
- Concrete channel adapters (no HTTP webhook client).
- The binary entry point (CLI? HTTP? worker?). Deferred.
- Persistence schema, migrations, transactions.
- Scheduling/worker loop for enrollments (only the data model and trait surface to enable it later).
- Tenant lifecycle (registration, billing, settings, quotas). External to this lib.
- Auth, observability.

## 2. Goals

1. Define bounded contexts as top-level modules.
2. Declare aggregate roots, value objects, and domain errors per context.
3. Declare repository and service ports (traits) per context — no implementations.
4. Implement only domain-pure methods (state transitions, invariant-enforcing constructors).
5. Make every aggregate tenant-scoped so the library cannot accidentally leak data across tenants.
6. Keep the crate compiling with the existing strict lint config (`unwrap_used = "deny"`, `expect_used = "deny"`, etc.).
7. Provide test-module placeholders so the impl phase can fill them in following TDD.

## 3. Bounded Contexts

Five modules, each a bounded context:

| Module           | Aggregate root(s)        | Purpose                                                          |
| ---------------- | ------------------------ | ---------------------------------------------------------------- |
| `channels`       | — (value objects only)   | Channel kind taxonomy and the `Channel` port.                    |
| `tenants`        | — (`TenantId` only)      | Tenant identity. Lifecycle handled externally; lib only honors.  |
| `recipients`     | `Recipient`              | Persistent identity of who receives notifications, per tenant.   |
| `notifications` | `Notification`           | Intent to deliver a single one-shot message, per tenant.         |
| `sequences`      | `Sequence`, `Enrollment` | Multi-step drip flows and per-recipient progress, per tenant.    |

Webhook is the first (and only) channel kind in this iteration. Email/SMS/Push are deferred.

## 4. Project Layout

```
ninice/
├── src/
│   ├── lib.rs              # crate-level docs + re-exports
│   ├── main.rs             # domain stub (prints version)
│   ├── channels/
│   │   ├── mod.rs          # ChannelError, re-exports
│   │   ├── model.rs        # ChannelKind, WebhookUrl, ContactPoint
│   │   └── service.rs      # Channel trait
│   ├── tenants/
│   │   ├── mod.rs          # re-exports
│   │   └── model.rs        # TenantId
│   ├── recipients/
│   │   ├── mod.rs          # RecipientError, re-exports
│   │   ├── model.rs        # RecipientId, Recipient
│   │   ├── service.rs      # RecipientService trait
│   │   └── repository.rs   # RecipientRepository trait
│   ├── notifications/
│   │   ├── mod.rs          # NotificationError, re-exports
│   │   ├── model.rs        # NotificationId, Content, NotificationStatus, Notification
│   │   ├── service.rs      # NotificationService trait
│   │   └── repository.rs   # NotificationRepository trait
│   └── sequences/
│       ├── mod.rs          # SequenceError, re-exports
│       ├── model.rs        # SequenceId, EnrollmentId, SequenceStep,
│       │                   #   Sequence + SequenceBuilder,
│       │                   #   EnrollmentStatus, Enrollment
│       ├── service.rs      # SequenceService trait
│       └── repository.rs   # SequenceRepository, EnrollmentRepository traits
├── tests/                  # (existing greeting.rs replaced; see §11)
├── Cargo.toml
├── deny.toml
└── ...
```

Convention per module:

- `mod.rs` — declares submodules, defines the module's `Error` enum (when applicable), re-exports the public surface.
- `model.rs` — entities, value objects, IDs, status enums, domain-pure methods.
- `service.rs` — `Service` trait(s).
- `repository.rs` — `Repository` trait(s).

`channels/` has no repository (no persistence concern). `tenants/` has no service, repository, or error (no operations beyond carrying an ID).

## 5. `channels/`

### 5.1 Value objects

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    Webhook,
    // Email, Sms, Push deferred
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebhookUrl(String);

impl WebhookUrl {
    /// Parses `raw` as a webhook target URL.
    ///
    /// # Errors
    /// Returns `ChannelError::InvalidWebhookUrl` if `raw` is empty or
    /// does not start with `http://` or `https://`.
    pub fn parse(raw: impl Into<String>) -> Result<Self, ChannelError>;

    pub fn as_str(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactPoint {
    pub kind: ChannelKind,
    pub address: String,
}
```

Validation in `WebhookUrl::parse` is intentionally minimal (non-empty + scheme prefix). Real URL parsing waits for the impl phase, where the `url` crate may be added.

### 5.2 Port

```rust
pub trait Channel: Send + Sync {
    fn kind(&self) -> ChannelKind;
}
```

The `send` method is **not** declared in this iteration. Its shape (sync vs async, payload type, retry semantics) is a downstream decision tied to the binary's entry point.

### 5.3 Error

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ChannelError {
    #[error("invalid webhook URL")]
    InvalidWebhookUrl,
}
```

## 6. `tenants/`

This is the smallest BC. It holds only the identity type that every other aggregate carries.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantId(uuid::Uuid);

impl TenantId {
    pub fn generate() -> Self;
    pub fn from_uuid(id: uuid::Uuid) -> Self;
}

impl std::fmt::Display for TenantId { /* delegates to inner */ }
```

No `Tenant` aggregate, no service, no repository, no error. Tenant lifecycle (creation, deletion, settings) is **outside** the library — the caller hands us `TenantId` values it already has.

`generate()` is provided as a convenience for tests and prototypes. Production tenants are expected to come from an external system (e.g., a control plane). Documented as such.

## 7. `recipients/`

### 7.1 Aggregate root

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipientId(uuid::Uuid);

impl RecipientId {
    pub fn generate() -> Self;
    pub fn from_uuid(id: uuid::Uuid) -> Self;
}

impl std::fmt::Display for RecipientId { /* delegates to inner */ }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    pub id: RecipientId,
    pub tenant_id: TenantId,
    pub contact_points: Vec<ContactPoint>,
}

impl Recipient {
    /// Creates a new recipient with at least one contact point.
    ///
    /// # Errors
    /// Returns `RecipientError::NoContactPoints` if `contact_points` is empty.
    pub fn new(
        tenant_id: TenantId,
        contact_points: Vec<ContactPoint>,
    ) -> Result<Self, RecipientError>;

    pub fn add_contact_point(&mut self, cp: ContactPoint);
}
```

Invariants:

- A `Recipient` always has ≥ 1 `ContactPoint`. Enforced in `new`; `add_contact_point` only adds.
- `tenant_id` is immutable after construction. No setter.

### 7.2 Ports

```rust
pub trait RecipientRepository: Send + Sync {
    fn save(&self, recipient: &Recipient) -> Result<(), RecipientError>;
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &RecipientId,
    ) -> Result<Option<Recipient>, RecipientError>;
}

pub trait RecipientService: Send + Sync {
    fn register(
        &self,
        tenant_id: TenantId,
        contact_points: Vec<ContactPoint>,
    ) -> Result<RecipientId, RecipientError>;
    fn add_contact_point(
        &self,
        tenant_id: &TenantId,
        id: &RecipientId,
        cp: ContactPoint,
    ) -> Result<(), RecipientError>;
}
```

Tenant scoping is enforced by repositories: `find(tenant_id, id)` returns `Ok(None)` if the recipient with that `id` belongs to a different tenant. Cross-tenant access is indistinguishable from non-existence — no information leak.

### 7.3 Error

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RecipientError {
    #[error("recipient not found")]
    NotFound,
    #[error("recipient must have at least one contact point")]
    NoContactPoints,
}
```

## 8. `notifications/`

### 8.1 Aggregate root

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(uuid::Uuid);

impl NotificationId {
    pub fn generate() -> Self;
    pub fn from_uuid(id: uuid::Uuid) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub body: String,
}

impl Content {
    pub fn new(body: impl Into<String>) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: NotificationId,
    pub tenant_id: TenantId,
    pub recipient_id: RecipientId,
    pub channel: ChannelKind,
    pub content: Content,
    pub status: NotificationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Notification {
    pub fn new(
        tenant_id: TenantId,
        recipient_id: RecipientId,
        channel: ChannelKind,
        content: Content,
    ) -> Self;

    pub fn mark_sent(&mut self);
    pub fn mark_failed(&mut self, reason: impl Into<String>);
}
```

State transitions:

- `new` → `status = Pending`, `created_at = now`, `sent_at = None`.
- `mark_sent` → `status = Sent`, `sent_at = Some(now)`.
- `mark_failed(reason)` → `status = Failed { reason }`, `sent_at = None`.

Idempotency of state transitions is **not** enforced in stubs; the impl phase decides whether double-mark is a domain error.

### 8.2 Ports

```rust
pub trait NotificationRepository: Send + Sync {
    fn save(&self, n: &Notification) -> Result<(), NotificationError>;
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &NotificationId,
    ) -> Result<Option<Notification>, NotificationError>;
}

pub trait NotificationService: Send + Sync {
    fn send_one_off(
        &self,
        tenant_id: TenantId,
        recipient_id: RecipientId,
        channel: ChannelKind,
        content: Content,
    ) -> Result<NotificationId, NotificationError>;
}
```

The service must verify that `recipient_id` belongs to `tenant_id` before creating the notification. If not, returns `NotificationError::RecipientNotFound` (same response as if the recipient genuinely did not exist).

### 8.3 Error

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NotificationError {
    #[error("notification not found")]
    NotFound,
    #[error("recipient {0} not found")]
    RecipientNotFound(RecipientId),
    #[error("channel {0:?} unavailable")]
    ChannelUnavailable(ChannelKind),
}
```

## 9. `sequences/`

### 9.1 Aggregate roots

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceId(uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnrollmentId(uuid::Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceStep {
    pub delay: std::time::Duration,
    pub channel: ChannelKind,
    pub content: Content, // imported from notifications::model
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    pub id: SequenceId,
    pub tenant_id: TenantId,
    pub name: String,
    pub steps: Vec<SequenceStep>,
}

impl Sequence {
    pub fn builder(tenant_id: TenantId, name: impl Into<String>) -> SequenceBuilder;
}

#[derive(Debug)]
pub struct SequenceBuilder { /* private fields */ }

impl SequenceBuilder {
    pub fn add_step(self, step: SequenceStep) -> Self;

    /// Finalizes the sequence.
    ///
    /// # Errors
    /// Returns `SequenceError::EmptySteps` if no steps were added.
    pub fn build(self) -> Result<Sequence, SequenceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrollmentStatus {
    Active { next_step_index: usize },
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrollment {
    pub id: EnrollmentId,
    pub tenant_id: TenantId,
    pub sequence_id: SequenceId,
    pub recipient_id: RecipientId,
    pub status: EnrollmentStatus,
    pub enrolled_at: chrono::DateTime<chrono::Utc>,
}

impl Enrollment {
    pub fn new(
        tenant_id: TenantId,
        sequence_id: SequenceId,
        recipient_id: RecipientId,
    ) -> Self;
    // status = Active { next_step_index: 0 }, enrolled_at = now

    pub fn advance(&mut self, total_steps: usize);
    // Active{i} -> Active{i+1} if i+1 < total_steps, else Completed.
    // Completed/Cancelled remain unchanged.

    pub fn cancel(&mut self);
    // Any state -> Cancelled.
}
```

Why two aggregates: `Sequence` is the (effectively immutable) definition; `Enrollment` is per-recipient mutable progress. Keeping them separate avoids an aggregate whose size grows with audience.

Both carry `tenant_id`. Both repositories filter by tenant on read.

### 9.2 Ports

```rust
pub trait SequenceRepository: Send + Sync {
    fn save(&self, s: &Sequence) -> Result<(), SequenceError>;
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &SequenceId,
    ) -> Result<Option<Sequence>, SequenceError>;
}

pub trait EnrollmentRepository: Send + Sync {
    fn save(&self, e: &Enrollment) -> Result<(), SequenceError>;
    fn find(
        &self,
        tenant_id: &TenantId,
        id: &EnrollmentId,
    ) -> Result<Option<Enrollment>, SequenceError>;
    fn find_active_due(
        &self,
        tenant_id: &TenantId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Enrollment>, SequenceError>;
}

pub trait SequenceService: Send + Sync {
    fn enroll(
        &self,
        tenant_id: TenantId,
        sequence_id: SequenceId,
        recipient_id: RecipientId,
    ) -> Result<EnrollmentId, SequenceError>;
    fn cancel(
        &self,
        tenant_id: &TenantId,
        enrollment_id: &EnrollmentId,
    ) -> Result<(), SequenceError>;
}
```

`find_active_due` is intentionally per-tenant. A scheduler that processes all tenants iterates over them externally (avoids the lib needing a "list all tenants" capability, which would conflict with §6's stance that tenant lifecycle is external).

The service must verify that `sequence_id` and `recipient_id` both belong to `tenant_id` before creating an enrollment.

### 9.3 Error

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SequenceError {
    #[error("sequence not found")]
    NotFound,
    #[error("enrollment not found")]
    EnrollmentNotFound,
    #[error("sequence must have at least one step")]
    EmptySteps,
    #[error("recipient {0} not found")]
    RecipientNotFound(RecipientId),
}
```

## 10. Cross-cutting

### 10.1 `lib.rs` public surface

```rust
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
pub use recipients::{Recipient, RecipientError, RecipientId, RecipientRepository, RecipientService};
pub use sequences::{
    Enrollment, EnrollmentId, EnrollmentRepository, EnrollmentStatus, Sequence, SequenceBuilder,
    SequenceError, SequenceId, SequenceRepository, SequenceService, SequenceStep,
};
pub use tenants::TenantId;
```

`greeting()` is removed.

### 10.2 `main.rs`

```rust
//! Binary entry point. Currently a domain stub; real entry point (CLI/HTTP/worker)
//! is decided in a later iteration.

fn main() {
    println!("ninice {}", env!("CARGO_PKG_VERSION"));
}
```

### 10.3 IDs

All `XxxId` types are `pub struct XxxId(uuid::Uuid)` with:

- `generate()` → wraps `Uuid::new_v4()`
- `from_uuid(uuid::Uuid) -> Self`
- `impl Display` delegating to the inner UUID

Naming `generate` (rather than `new`) signals that the operation is non-deterministic and sidesteps Clippy's `new_without_default` lint.

### 10.4 Errors

- One `thiserror::Error` enum per bounded context that has operations (channels, recipients, notifications, sequences).
- `tenants/` has no error type (no operations).
- All marked `#[non_exhaustive]` to allow future variants without breaking change.
- `anyhow` is **removed** from `Cargo.toml`. Anti-pattern in libraries.

### 10.5 Tenant scoping rules

- Every aggregate root (`Recipient`, `Notification`, `Sequence`, `Enrollment`) carries `tenant_id`.
- Constructors require `tenant_id` as their first parameter.
- Repository read methods (`find`, `find_active_due`) take `&TenantId` and return `Ok(None)` / empty list when scoped out, never an error variant. Cross-tenant access is indistinguishable from non-existence.
- Repository write methods (`save`) take an aggregate that already carries its tenant; the repo persists it as-is.
- Services validate cross-aggregate tenant consistency before mutations (e.g., enrolling a recipient into a sequence requires both belong to the same tenant).

### 10.6 Test module placeholders

Every `model.rs` / `service.rs` / `repository.rs` ends with:

```rust
#[cfg(test)]
mod tests {
    // Tests live here. Stub phase has no implementations to test
    // beyond pure value objects (covered in model.rs tests).
}
```

To allow real tests (which may use `unwrap`/`expect`) without fighting the strict lints, every file with `mod tests` gets, at its very top:

```rust
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
```

In this stub iteration, only `model.rs` files will have populated test bodies (covering invariant constructors and state transitions). `service.rs` and `repository.rs` have empty `mod tests` blocks pending impls in the next phase.

### 10.7 Dependencies

`Cargo.toml` changes:

```toml
[dependencies]
thiserror = "2"
uuid     = { version = "1", features = ["v4"] }
chrono   = { version = "0.4", default-features = false, features = ["clock"] }
# anyhow removed
```

Dev-dependencies for the **next** phase (impl), not added now:

```toml
# [dev-dependencies]
# mockall  = "0.13"
# rstest   = "0.23"
# proptest = "1"
```

## 11. Impact on existing files

| File                       | Action                                                                                  |
| -------------------------- | --------------------------------------------------------------------------------------- |
| `src/lib.rs`               | Remove `greeting()`, add module declarations + re-exports (see §10.1).                  |
| `src/main.rs`              | Replace with version stub (see §10.2).                                                  |
| `tests/greeting.rs`        | **Delete**. Replaced in impl phase with per-context integration tests.                  |
| `Cargo.toml`               | Remove `anyhow`, add `uuid`, `chrono`. Keep `thiserror`. Lints unchanged.               |
| `README.md`                | Re-run `cargo rdme` so the synced section reflects the new crate-level doc in `lib.rs`. |
| `sonar-project.properties` | No change (still scans `src/`).                                                         |
| `.github/workflows/ci.yml` | No change.                                                                              |
| `deny.toml`                | No change.                                                                              |

## 12. Stub Strategy — What is Implemented vs Declared

**Implemented (real code):**

- All structs, enums, newtypes, with their `Debug`/`Clone`/`PartialEq`/etc derives.
- All `Error` enums (`thiserror`).
- All trait declarations.
- Invariant-enforcing constructors: `WebhookUrl::parse`, `Recipient::new`, `Sequence::builder().build()`.
- Domain-pure mutators: `Notification::mark_sent`/`mark_failed`, `Recipient::add_contact_point`, `Enrollment::advance`/`cancel`.
- `Sequence::builder` + `SequenceBuilder::add_step`/`build`.
- ID generation helpers (`XxxId::generate`/`from_uuid` + `Display`).

**Declared only (trait signatures, no `impl`):**

- All `Service` and `Repository` traits across the four operational contexts.

**Not addressed in this iteration:**

- Binary entry point design (CLI vs HTTP vs worker).
- Channel send semantics (sync/async, retry, payload format).
- Persistence backends.
- Scheduler/worker for `find_active_due` → step execution.
- Observability (`tracing`).
- Serialization (`serde` derives).
- Tenant lifecycle (no `Tenant` aggregate, no tenant CRUD).
- Cross-aggregate tenant consistency enforcement in service impls (the requirement is documented; the impl is future work).

## 13. Tech debt / known smells

1. **`sequences::SequenceStep` imports `notifications::Content`.** Cross-context coupling. Acceptable while the model is small. When friction shows up (e.g., sequences want richer content metadata that notifications don't need), either lift `Content` into a shared module or duplicate per-context.
2. **`Channel` trait is open (not sealed).** Means external crates can implement `Channel`. Revisit when the `send` method shape is defined.
3. **State-transition idempotency unenforced.** `mark_sent` on an already-sent `Notification` silently succeeds. May want `NotificationError::InvalidTransition` later.
4. **`SequenceBuilder` validates only non-empty steps.** Future validations (monotonic delays? channel-support check?) deferred.
5. **`WebhookUrl::parse` does minimal validation** (non-empty + scheme prefix). Replace with `url::Url` parsing in the impl phase.
6. **No `Tenant` aggregate.** Tenants are referenced by ID only. If/when the library needs to manage tenant settings, quotas, or lifecycle, promote `tenants/` to a full BC with aggregate, service, and repository.
7. **Tenant scoping is enforced by convention, not the type system.** A repo impl that forgets to filter by `tenant_id` will compile and run. Could be tightened later via a `TenantScoped<T>` wrapper or by making `tenant_id` mandatory in every query method (already done in trait signatures, but the body can still ignore it).
8. **Lint `expect_used = "deny"` collides with idiomatic test patterns.** Worked around with file-level `#![cfg_attr(test, allow(...))]`; revisit if too noisy.

## 14. Validation criteria for "done"

This spec is implemented correctly when:

1. `cargo build` succeeds.
2. `cargo clippy -- -D warnings` is clean.
3. `cargo test --lib` runs (test bodies in `model.rs` files; service/repo test modules empty).
4. `cargo rdme --check` passes (README synced).
5. `cargo deny check` passes.
6. Every trait and struct in §5–§9 exists with the documented signature.
7. Every aggregate root carries a `tenant_id` field, immutable after construction.
8. Every repository read method accepts a `&TenantId` parameter.
9. `greeting()` and `tests/greeting.rs` are gone.

## 15. Next steps

Once this spec is approved, the next agent (writing-plans) produces an ordered implementation plan, broken into commits.
