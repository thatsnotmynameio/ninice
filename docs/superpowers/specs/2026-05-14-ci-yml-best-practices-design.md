# CI workflow reorganization (GitHub best practices)

**Status:** Design, ready for implementation planning
**Date:** 2026-05-14
**Branch:** `chore/ci-best-practices`

## Context

`.github/workflows/ci.yml` (204 lines today) was assembled incrementally over several iterations. It is functionally correct and already follows several GitHub best practices: `permissions: {}` at the workflow level with per-job `contents: read`, a sensible `concurrency` policy (`cancel-in-progress` only on non-`main` refs), a `Swatinem/rust-cache` strategy with `save-if` gated on `main`, and an aggregating `conclusion` job that lets branch protection require a single status check.

The remaining gaps are typical for a workflow that grew organically:

- Repeated setup boilerplate (checkout + toolchain + cache) across 9 of 11 jobs.
- No `timeout-minutes` on any job, so the runner default (6 hours) applies.
- Bare `uses:` / `run:` steps with no `name:` keys; UI logs are harder to scan.
- Third-party actions pinned by mutable tag (`@v6`, `@v2`, `@stable`) rather than commit SHA.
- No Dependabot configuration to keep those pins current.
- A 25-line commented-out `semver-checks` block that lives in git history already.
- `pull_request` trigger runs full CI on draft PRs.
- `cargo-deny` is gated only on PR/push activity; new RustSec advisories published while the repo is idle go unnoticed.

This spec captures the design for closing those gaps. Implementation is a single PR on the `chore/ci-best-practices` branch.

## Scope

**In scope**

- Extract repeated Rust setup into a local composite action.
- Add `timeout-minutes: 3` to every job.
- Add descriptive `name:` to every step.
- SHA-pin all third-party actions (with `# vX.Y.Z` comments).
- Add `.github/dependabot.yml` for `cargo` and `github-actions` ecosystems.
- Split `cargo-deny` into a separate `audit.yml` workflow with a daily cron schedule (in addition to the existing PR/push/merge_group triggers).
- Add a `codeql.yml` workflow scanning both `rust` and `actions`. This satisfies the existing `code_scanning` rule in the `Protect main` ruleset.
- Filter draft PRs out of the workflow triggers.
- Delete the commented-out `semver-checks` block.
- Add `actions/checkout` `persist-credentials: false` everywhere.

**Out of scope**

- Re-enabling `cargo-semver-checks`. Will be a follow-up once the crate has a real baseline (currently unpublished).
- Release automation (`release.yml`), doc deployment, benchmarks.
- Changes to `sonar-project.properties`, `deny.toml`, `Cargo.toml`.
- Tightening the `Protect main` ruleset further (e.g. requiring approving reviewers). The ruleset surgery in this spec is limited to wiring the new checks into branch protection.

## Resolved action versions (as of 2026-05-14)

All third-party actions in this spec are pinned to the commit SHAs below. The
`# vX.Y.Z` comment after each `@<sha>` is what Dependabot reads to know which
semver line to follow when bumping.

| Action | Reference | Commit SHA |
|---|---|---|
| `actions/checkout` | v6.0.2 | `de0fac2e4500dabe0009e67214ff5f5447ce83dd` |
| `dtolnay/rust-toolchain` | `stable` (branch — not a tag) | `29eef336d9b2848a0b548edc03f92a220660cdb8` |
| `Swatinem/rust-cache` | v2.9.1 | `c19371144df3bb44fab255c43d04cbc2ab54d1c4` |
| `taiki-e/install-action` | v2.78.0 | `e1c4cd42111751368541a7cb5db3522bd1f846a4` |
| `crate-ci/typos` | v1.46.1 | `5374cbf686e897b15713110e233094e2874de7ef` |
| `github/codeql-action/init` & `/analyze` | v4.35.4 | `68bde559dea0fdcac2102bfdf6230c5f70eb485e` |
| `SonarSource/sonarqube-scan-action` | v8.0.0 (already pinned in current ci.yml) | `59db25f34e16620e48ab4bb9e4a5dce155cb5432` |

Notes:

- `dtolnay/rust-toolchain` doesn't follow semver — it has per-Rust-version branches (`1.85`, `1.85.1`, etc.) and a moving `stable` branch. The comment is the branch name, not a version. Dependabot still tracks it via that comment.
- `github/codeql-action` has `v3` and `v4` lines maintained in parallel. v4 is GA (Node.js v24 runtime is the only v3→v4 breaking change, irrelevant to us). The floating `v4` tag and the specific `v4.35.4` tag point to the same commit today; we pin with the specific version comment so Dependabot bumps on the patch line.

## File layout

```
.github/
├── actions/
│   └── setup-rust/
│       └── action.yml                NEW   composite action (checkout + toolchain + cache)
├── dependabot.yml                    NEW   cargo + github-actions, weekly Friday 00:00 UTC
└── workflows/
    ├── audit.yml                     NEW   cargo-deny: PR/push/merge_group + daily cron
    ├── codeql.yml                    NEW   CodeQL: matrix [rust, actions]
    └── ci.yml                        EDIT  composite usage, timeouts, polish, drop deny job
```

No other files change.

## Design

### Composite action: `.github/actions/setup-rust/action.yml`

A single local composite action with three optional inputs, so each job declares only what it needs and skips integrations it doesn't:

```yaml
name: Set up Rust
description: Checkout sources, install Rust toolchain (+ optional components/tools), restore cargo cache.

inputs:
  components:
    description: "Comma-separated rustup components (e.g. 'clippy,rustfmt')."
    required: false
    default: ""
  tools:
    description: "Comma-separated cargo tools for taiki-e/install-action (e.g. 'cargo-nextest')."
    required: false
    default: ""
  cache-key:
    description: "Shared cache key for Swatinem/rust-cache. Leave empty to skip caching."
    required: false
    default: ""

runs:
  using: composite
  steps:
    - name: Checkout sources
      uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      with:
        persist-credentials: false

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable
      with:
        components: ${{ inputs.components }}

    - name: Install cargo tools
      if: inputs.tools != ''
      uses: taiki-e/install-action@e1c4cd42111751368541a7cb5db3522bd1f846a4 # v2.78.0
      with:
        tool: ${{ inputs.tools }}

    - name: Restore cargo cache
      if: inputs.cache-key != ''
      uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1
      with:
        shared-key: ${{ inputs.cache-key }}
        save-if: ${{ github.ref == 'refs/heads/main' }}
```

**Design choices:**

- `if: inputs.X != ''` guards each integration so jobs only pay for what they use. Avoids the "kitchen-sink composite" anti-pattern.
- `save-if: github.ref == 'refs/heads/main'` lives inside the composite, preserving the current policy that PRs read from the cache but only `main` writes to it. Inherited automatically by every job.
- `persist-credentials: false` on `actions/checkout` prevents the `GITHUB_TOKEN` from being written to `.git/config`, where later steps could read it. Cheap security hardening.
- SHAs above were resolved on 2026-05-14 (see the *Resolved action versions* table). Dependabot will keep them current after the first scheduled run.

### `ci.yml` after refactor

Every Rust job becomes ~6 lines instead of ~12:

```yaml
clippy:
  runs-on: ubuntu-latest
  timeout-minutes: 3
  permissions:
    contents: read
  steps:
    - uses: ./.github/actions/setup-rust
      with:
        components: clippy
        cache-key: lint
    - name: Run clippy on all targets and features
      run: cargo clippy --all-targets --all-features -- -D warnings
```

**Two jobs do not use the composite** (no Rust setup needed):
- `typos` — only `actions/checkout` + `crate-ci/typos`. Still gets `timeout-minutes`, step names, `persist-credentials: false`, and SHA pins.
- `conclusion` — unchanged logic; gets `timeout-minutes: 3`.

**Other ci.yml changes:**

- `on.pull_request.types: [opened, synchronize, reopened, ready_for_review]` — skips drafts.
- The `deny` job is **removed** from `ci.yml` (it moves to `audit.yml`).
- `conclusion.needs` updated to: `[fmt, clippy, typos, docs, rdme, msrv, hack, test, sonarqube]`. `deny` is removed because it now gates via a separate workflow's required check.
- The commented-out `semver-checks` block (current lines 141–161) is deleted. Reasoning is preserved in commit `114f5cb`'s history.
- Every `run:` step gets a `name:` (e.g. `Run clippy on all targets and features`, `Generate LCOV coverage report`, `Run unit tests with nextest`, `Run doctests`).
- The `sonarqube` job keeps its inline `env: RUSTFLAGS: ""` / `RUSTDOCFLAGS: ""` override and its existing comment explaining why; that workaround is still needed.

### `audit.yml` (new)

```yaml
name: Audit

on:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]
  merge_group:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # daily at 00:00 UTC

permissions: {}

concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}

jobs:
  deny:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - name: Checkout sources
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false
      - name: Install cargo-deny
        uses: taiki-e/install-action@e1c4cd42111751368541a7cb5db3522bd1f846a4 # v2.78.0
        with:
          tool: cargo-deny
      - name: Run cargo deny
        run: cargo deny check
```

**Why a separate workflow:** the `schedule:` trigger is the value-add — it catches RustSec advisories published while no PR is open. Same triggers as ci.yml are kept so PRs still gate on `audit / deny`. The concurrency expression matches ci.yml so a fresh push cancels stale audit runs on PR refs but not on `main`.

### `codeql.yml` (new)

Scans both Rust source and workflows. Two matrix entries because each runs its own analysis pass.

```yaml
name: CodeQL

on:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 1'  # weekly, Monday 00:00 UTC

permissions: {}

concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}

jobs:
  analyze:
    name: Analyze (${{ matrix.language }})
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
      security-events: write
      actions: read
    strategy:
      fail-fast: false
      matrix:
        language: [rust, actions]
    steps:
      - name: Checkout sources
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false
      - name: Initialize CodeQL
        uses: github/codeql-action/init@68bde559dea0fdcac2102bfdf6230c5f70eb485e # v4.35.4
        with:
          languages: ${{ matrix.language }}
      - name: Perform CodeQL Analysis
        uses: github/codeql-action/analyze@68bde559dea0fdcac2102bfdf6230c5f70eb485e # v4.35.4
        with:
          category: "/language:${{ matrix.language }}"
```

**Notes:**

- `permissions: security-events: write` is the *only* place outside repo defaults where elevated perms are granted, and it's scoped to this one job. Required for uploading SARIF results.
- `fail-fast: false` so a flake in one language doesn't mask findings in the other.
- The `actions` language analyzes `.github/workflows/*.yml` for things like command injection via `${{ github.event.* }}` in `run:` blocks — relevant for our supply chain.
- The `rust` language analyzes the crate. GA since early 2026; supports Rust editions 2021 and 2024. The action handles `rustup`/`cargo` install automatically.

**Timeout caveat:** 3 minutes may be tight for `rust` (CodeQL builds the project to extract its database). If the job consistently approaches the limit, bump to 5 in a follow-up; the spec keeps the uniform 3-minute policy as the starting point per the agreed approach.

### `.github/dependabot.yml`

```yaml
version: 2
updates:
  # Cargo dependencies: weekly, grouped by update kind to bundle PRs.
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
      day: friday
      time: "00:00"
      timezone: Etc/UTC
    open-pull-requests-limit: 2
    rebase-strategy: auto
    groups:
      rust-patch-updates:
        patterns: ["*"]
        update-types: [patch]
      rust-minor-updates:
        patterns: ["*"]
        update-types: [minor]

  # GitHub Actions: weekly, single grouped PR.
  # Dependabot reads the `# vX.Y.Z` comment after each SHA pin and updates
  # both the SHA and the comment in lockstep.
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
      day: friday
      time: "00:00"
      timezone: Etc/UTC
    open-pull-requests-limit: 2
    rebase-strategy: auto
    groups:
      github-actions:
        patterns: ["*"]
        update-types: [minor, patch]
```

**Notes:**

- `Etc/UTC` is the canonical IANA timezone for UTC/GMT (DST-free, stable year-round).
- `open-pull-requests-limit: 2` caps each ecosystem to 2 open PRs at any time, preventing PR noise.
- Cargo groups split `patch` and `minor` so blame stays clear when something breaks.
- Action major-version bumps (e.g. v6 → v7) are not in the `update-types` groups, so they come as individual PRs — major bumps usually have migration notes worth reading.

### Per-job timeouts

| Job (file) | timeout-minutes |
|---|---:|
| `fmt` (ci.yml) | 3 |
| `clippy` (ci.yml) | 3 |
| `typos` (ci.yml) | 3 |
| `docs` (ci.yml) | 3 |
| `rdme` (ci.yml) | 3 |
| `msrv` (ci.yml) | 3 |
| `hack` (ci.yml) | 3 |
| `test` (ci.yml) | 3 |
| `sonarqube` (ci.yml) | 3 |
| `conclusion` (ci.yml) | 3 |
| `deny` (audit.yml) | 3 |
| `analyze` (codeql.yml) | 3 |

**Rationale:** uniform 3-minute policy as the deliberate starting point. ninice is small enough that any job taking longer is a signal worth investigating. Bumps to 5 (or higher) are one-line follow-ups, and easier to justify after observing real runtime than to set defensively up front. Highest-risk jobs to monitor: `msrv`, `hack`, `sonarqube`, `codeql/rust`.

### Required status checks (ruleset)

The `Protect main` ruleset (`id: 16402601`) was updated **before** writing this spec to:

- Set `allowed_merge_methods: ["squash"]` (was `["merge", "squash"]`; the `merge` option was unusable under `required_linear_history`).
- Add a `required_status_checks` rule requiring the `conclusion` check.

**Post-merge follow-up (manual step after this PR ships):**

Once `audit.yml` exists on `main`, add `audit / deny` to the required checks. The `gh api` payload to run is the same shape as the existing one, plus one more check object:

```bash
gh api repos/thatsnotmynameio/ninice/rulesets/16402601 \
  --method PUT \
  --input - <<'JSON'
{
  "name": "Protect main",
  "target": "branch",
  "enforcement": "active",
  "conditions": {"ref_name": {"exclude": [], "include": ["~DEFAULT_BRANCH"]}},
  "bypass_actors": [],
  "rules": [
    {"type":"deletion"},
    {"type":"non_fast_forward"},
    {"type":"required_linear_history"},
    {"type":"code_quality","parameters":{"severity":"warnings"}},
    {"type":"code_scanning","parameters":{"code_scanning_tools":[{"tool":"CodeQL","security_alerts_threshold":"high_or_higher","alerts_threshold":"errors"}]}},
    {"type":"pull_request","parameters":{"required_approving_review_count":0,"dismiss_stale_reviews_on_push":false,"required_reviewers":[],"require_code_owner_review":false,"require_last_push_approval":false,"required_review_thread_resolution":true,"allowed_merge_methods":["squash"]}},
    {"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"conclusion"},{"context":"deny"}],"strict_required_status_checks_policy":false}}
  ]
}
JSON
```

The check context for `audit.yml`'s `deny` job is `deny` (job key when no `job.name` is set). If a workflow run reports a different check name in the GitHub UI, use that exact string.

## Verification

After implementation, the following must hold on the chore branch before merging:

1. `cargo fmt --all --check`, `cargo clippy ...`, `cargo nextest run`, `cargo test --doc`, `cargo deny check`, `cargo rdme --check`, `cargo msrv verify ...`, `cargo hack check ...` all pass locally on the branch.
2. The PR's CI run shows: `fmt`, `clippy`, `typos`, `docs`, `rdme`, `msrv`, `hack`, `test`, `sonarqube`, `conclusion` (in ci.yml), `deny` (in audit.yml), and `analyze (rust)` + `analyze (actions)` (in codeql.yml) all green.
3. Branch protection's `conclusion` required check resolves green.
4. No job exceeds the 3-minute timeout. If any does, the spec is amended (with the new value documented) before that job's timeout is raised in code.
5. Dependabot's "Dependency graph → Dependabot" page in GitHub UI shows two ecosystems configured: cargo and github-actions.
6. `actionlint` (if run locally) reports no errors against the new and edited workflows.

## Open questions / known constraints

- **CodeQL/rust on 3-minute timeout.** Realistic risk this trips on first PR. Mitigation: monitor the first run; if it consistently lands within 30 seconds of the limit, raise to 5 in a follow-up commit on this same branch before merge.
- **`taiki-e/install-action` patch versions move frequently.** SHA pinning means PRs bumping it land weekly via Dependabot. Acceptable.
- **The post-merge ruleset PATCH is manual.** Could be automated via a `release.yml` style workflow later, but for now a documented `gh api` command is enough.
- **`code_scanning` rule behaviour on the chore PR itself.** The rule is already active in `Protect main`, but `codeql.yml` does not yet exist on `main`. GitHub's documented behaviour is to evaluate the rule against *findings*, so "no scan ran" should not block — but this is the first time the rule will see a PR. If the merge button reports the rule as unsatisfied before `codeql.yml` runs successfully, the workaround is a one-off `gh api PUT` to temporarily drop the `code_scanning` rule, merge this PR, then re-add it (the JSON payload is the same shape as the post-merge follow-up above).

## Implementation plan

A separate plan document (per the brainstorming → writing-plans handoff) will sequence the file additions and the ci.yml edits into discrete commits. The writing-plans skill is invoked after this design is approved.
