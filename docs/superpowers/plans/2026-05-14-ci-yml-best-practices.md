# CI workflow reorganization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize `.github/workflows/ci.yml` to apply GitHub-recommended best practices (composite action for shared setup, SHA-pinned third-party actions, per-job timeouts, descriptive step names, draft-PR filter), split `cargo-deny` into a dedicated `audit.yml` with a daily cron schedule, add a CodeQL workflow scanning `rust` + `actions`, and configure Dependabot to keep all pins current.

**Architecture:** Five focused commits on the `chore/ci-best-practices` branch — each commit adds or modifies one CI file with `actionlint`-clean YAML and ends with a green local YAML-parse check. The composite action lands first so the ci.yml refactor in the third commit can reference it. Final commit is the push + PR verification using `gh`.

**Tech Stack:** GitHub Actions (workflow YAML, composite actions), Dependabot v2, GitHub Rulesets (ruleset surgery already complete; one follow-up documented), `cargo` toolchain via `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `taiki-e/install-action`, `crate-ci/typos`, `github/codeql-action`, `SonarSource/sonarqube-scan-action`. Local YAML validation via `python3 -c "import yaml; ..."` (no `actionlint`/`yq` installed; both are optional).

**Spec reference:** `docs/superpowers/specs/2026-05-14-ci-yml-best-practices-design.md`

---

## Working assumptions

- The branch `chore/ci-best-practices` already exists and the spec is committed there.
- The `Protect main` ruleset has already been updated to require `conclusion` and restrict to squash merges.
- `cargo-deny` will be in **both** the new `audit.yml` and the existing `ci.yml` for one commit (Task 2 ships `audit.yml` first; Task 3 removes the `deny` job from `ci.yml`). This avoids a window where no `deny` check runs on any incoming PR.
- All third-party action SHAs are pre-resolved in the spec (see *Resolved action versions*). Copy them verbatim.

## File structure

```
.github/
├── actions/
│   └── setup-rust/
│       └── action.yml          NEW  Task 1 — composite action
├── dependabot.yml              NEW  Task 5 — cargo + github-actions
└── workflows/
    ├── audit.yml               NEW  Task 2 — cargo-deny + daily cron
    ├── codeql.yml              NEW  Task 4 — CodeQL [rust, actions]
    └── ci.yml                  EDIT Task 3 — composite + timeouts + polish
```

No other files are touched.

## Order rationale

1. **Composite action first** — Task 3 references it. Must exist on the branch before the ci.yml refactor commit.
2. **`audit.yml` before removing `deny` from `ci.yml`** — keeps `cargo-deny` running every commit, even briefly duplicated, instead of leaving a window with no deny check.
3. **`ci.yml` refactor** — single large file change. Removes `deny` job (now in audit.yml), drops it from `conclusion.needs`, swaps every Rust job to the composite, adds timeouts/names/polish.
4. **`codeql.yml`** — independent, last among workflows.
5. **`dependabot.yml`** — last; configures Dependabot to watch all the other files we just created.
6. **Push + verify** — open PR, watch `gh pr checks`.

---

## Task 1: Composite action `setup-rust`

**Files:**
- Create: `.github/actions/setup-rust/action.yml`

- [ ] **Step 1.1: Create the composite action file**

Write `.github/actions/setup-rust/action.yml` with this exact content:

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

- [ ] **Step 1.2: Verify the YAML parses**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/actions/setup-rust/action.yml'))"
```

Expected: exits with no output and return code 0.

If you have `actionlint` installed (optional), additionally run:
```bash
actionlint -verbose
```
Expected: no errors. (`actionlint` reads workflow files but also catches composite action syntax issues.)

- [ ] **Step 1.3: Commit**

```bash
git add .github/actions/setup-rust/action.yml
git commit -m "ci: add setup-rust composite action

Checkout + toolchain + optional cargo tools + optional cache, all
gated by inputs so jobs only pay for the integrations they declare.
Pins actions/checkout, dtolnay/rust-toolchain, taiki-e/install-action,
and Swatinem/rust-cache to specific commit SHAs with version comments
Dependabot can read."
```

---

## Task 2: New `audit.yml` workflow

**Files:**
- Create: `.github/workflows/audit.yml`

- [ ] **Step 2.1: Create the audit workflow**

Write `.github/workflows/audit.yml` with this exact content:

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

- [ ] **Step 2.2: Verify the YAML parses**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/audit.yml'))"
```

Expected: exits with no output and return code 0.

- [ ] **Step 2.3: Commit**

```bash
git add .github/workflows/audit.yml
git commit -m "ci: add audit workflow with daily cargo-deny scan

Moves cargo-deny into a dedicated workflow that also runs on a daily
cron schedule, so RustSec advisories published between PRs are caught
within 24h. PR/push/merge_group triggers preserved so the existing CI
gating behaviour is unchanged. The deny job stays in ci.yml for one
commit until Task 3 removes it."
```

---

## Task 3: Refactor `ci.yml` (composite + timeouts + polish, drop deny)

**Files:**
- Modify: `.github/workflows/ci.yml` (full rewrite)

- [ ] **Step 3.1: Replace ci.yml with the refactored version**

Overwrite `.github/workflows/ci.yml` with this exact content:

```yaml
name: CI

on:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]
  merge_group:
  push:
    branches: [main]

permissions: {}

concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0
  RUSTFLAGS: "-D warnings"
  RUSTDOCFLAGS: "-D warnings -D rustdoc::broken_intra_doc_links"

jobs:
  fmt:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all --check

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

  typos:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - name: Checkout sources
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          persist-credentials: false
      - name: Run typos
        uses: crate-ci/typos@5374cbf686e897b15713110e233094e2874de7ef # v1.46.1

  docs:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          cache-key: lint
      - name: Build docs (no deps, all features)
        run: cargo doc --no-deps --all-features

  rdme:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          tools: cargo-rdme
      - name: Check README is in sync
        run: cargo rdme --check

  msrv:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          tools: cargo-msrv,cargo-hack
          cache-key: msrv
      - name: Verify MSRV
        run: cargo msrv verify -- cargo check --all-features

  hack:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          tools: cargo-hack
          cache-key: lint
      - name: Check feature powerset
        run: cargo hack check --feature-powerset --depth 2 --no-dev-deps

  test:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    steps:
      - uses: ./.github/actions/setup-rust
        with:
          tools: cargo-nextest
          cache-key: test
      - name: Run unit tests with nextest
        run: cargo nextest run --all-features
      - name: Run doctests
        run: cargo test --doc --all-features

  sonarqube:
    runs-on: ubuntu-latest
    timeout-minutes: 3
    permissions:
      contents: read
    # Sonar runs cargo clippy internally to collect findings; with the
    # workflow-level `RUSTFLAGS: -D warnings` every warning aborts the
    # scan instead of being reported. Reset it here so clippy completes
    # and the report reaches Sonar.
    env:
      RUSTFLAGS: ""
      RUSTDOCFLAGS: ""
    steps:
      - name: Checkout sources (full history for Sonar)
        uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
        with:
          fetch-depth: 0  # Shallow clones should be disabled for a better relevancy of analysis
          persist-credentials: false
      - name: Install Rust toolchain with clippy and llvm-tools
        uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable
        with:
          components: clippy,llvm-tools-preview
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@e1c4cd42111751368541a7cb5db3522bd1f846a4 # v2.78.0
        with:
          tool: cargo-llvm-cov
      - name: Restore cargo cache
        uses: Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1
        with:
          shared-key: sonarqube
          save-if: ${{ github.ref == 'refs/heads/main' }}
      - name: Generate LCOV coverage report
        run: cargo llvm-cov --lcov --output-path lcov.info --all-features
      - name: SonarQube Scan
        uses: SonarSource/sonarqube-scan-action@59db25f34e16620e48ab4bb9e4a5dce155cb5432 # v8.0.0
        env:
          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}

  conclusion:
    needs: [fmt, clippy, typos, docs, rdme, msrv, hack, test, sonarqube]
    if: ${{ !cancelled() }}
    runs-on: ubuntu-latest
    timeout-minutes: 3
    steps:
      - name: Verify all required checks succeeded
        run: |
          jq -e 'all(.[]; .result == "success" or .result == "skipped")' \
            <<< '${{ toJSON(needs) }}'
```

**What changed vs. the file on `main` (for reviewer sanity):**

- `on.pull_request.types` filter added (skips drafts).
- Every job got `timeout-minutes: 3`.
- Every step got a `name:` (and the `run:` line moved to its own keyed step).
- 9 of the 10 remaining jobs use `uses: ./.github/actions/setup-rust` instead of repeating checkout + toolchain + cache.
- `typos` and `sonarqube` keep direct action calls (they need non-standard setup); all third-party actions are now SHA-pinned with `# vX.Y.Z` comments.
- `actions/checkout` everywhere gained `persist-credentials: false`.
- `deny` job removed (lives in `audit.yml` now).
- `conclusion.needs` dropped `deny`.
- The 25-line commented-out `semver-checks` block is gone.

- [ ] **Step 3.2: Verify the YAML parses**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
```

Expected: exits with no output and return code 0.

- [ ] **Step 3.3: Sanity-check the conclusion `needs` list**

Run:
```bash
python3 -c "
import yaml
ci = yaml.safe_load(open('.github/workflows/ci.yml'))
needs = ci['jobs']['conclusion']['needs']
expected = ['fmt', 'clippy', 'typos', 'docs', 'rdme', 'msrv', 'hack', 'test', 'sonarqube']
assert needs == expected, f'needs mismatch: got {needs}'
assert 'deny' not in ci['jobs'], 'deny job should be removed from ci.yml'
print('ok')
"
```

Expected: prints `ok`.

- [ ] **Step 3.4: Confirm composite usage matches every Rust job**

Run:
```bash
grep -c 'uses: \./\.github/actions/setup-rust' .github/workflows/ci.yml
```

Expected: `7` (fmt, clippy, docs, rdme, msrv, hack, test — typos and sonarqube don't use it; conclusion doesn't either).

- [ ] **Step 3.5: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: refactor ci.yml to use composite, add timeouts and polish

Swap every Rust job to the new setup-rust composite (fmt, clippy, docs,
rdme, msrv, hack, test). Add timeout-minutes: 3 to every job. Name
every step for clearer log output. SHA-pin the direct action calls in
typos and sonarqube with # vX.Y.Z comments for Dependabot. Add
persist-credentials: false to every actions/checkout. Filter draft PRs
out of the pull_request trigger. Delete the 25-line commented-out
semver-checks block (preserved in commit 114f5cb's history). Remove
the deny job (now lives in audit.yml) and drop it from
conclusion.needs."
```

---

## Task 4: New `codeql.yml` workflow

**Files:**
- Create: `.github/workflows/codeql.yml`

- [ ] **Step 4.1: Create the CodeQL workflow**

Write `.github/workflows/codeql.yml` with this exact content:

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

- [ ] **Step 4.2: Verify the YAML parses**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/codeql.yml'))"
```

Expected: exits with no output and return code 0.

- [ ] **Step 4.3: Confirm matrix entries**

Run:
```bash
python3 -c "
import yaml
wf = yaml.safe_load(open('.github/workflows/codeql.yml'))
langs = wf['jobs']['analyze']['strategy']['matrix']['language']
assert langs == ['rust', 'actions'], f'matrix mismatch: {langs}'
perms = wf['jobs']['analyze']['permissions']
assert perms.get('security-events') == 'write', 'security-events: write required'
print('ok')
"
```

Expected: prints `ok`.

- [ ] **Step 4.4: Commit**

```bash
git add .github/workflows/codeql.yml
git commit -m "ci: add CodeQL workflow scanning rust and actions

Matrix runs two analyses per trigger: 'rust' for the crate source
(CodeQL Rust support is GA as of early 2026, editions 2021/2024) and
'actions' for the workflow files themselves (catches command
injection via untrusted github.event.* interpolation). Weekly cron
plus PR/push triggers so findings appear in the security tab and on
PRs. Satisfies the 'code_scanning' rule already active on the
Protect main ruleset."
```

---

## Task 5: Add `dependabot.yml`

**Files:**
- Create: `.github/dependabot.yml`

- [ ] **Step 5.1: Create the Dependabot config**

Write `.github/dependabot.yml` with this exact content:

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

- [ ] **Step 5.2: Verify the YAML parses and has both ecosystems**

Run:
```bash
python3 -c "
import yaml
db = yaml.safe_load(open('.github/dependabot.yml'))
assert db['version'] == 2, 'must be Dependabot v2'
ecosystems = sorted(u['package-ecosystem'] for u in db['updates'])
assert ecosystems == ['cargo', 'github-actions'], f'unexpected: {ecosystems}'
for u in db['updates']:
    assert u['schedule']['day'] == 'friday'
    assert u['schedule']['time'] == '00:00'
    assert u['schedule']['timezone'] == 'Etc/UTC'
print('ok')
"
```

Expected: prints `ok`.

- [ ] **Step 5.3: Commit**

```bash
git add .github/dependabot.yml
git commit -m "ci: add Dependabot config for cargo and github-actions

Weekly schedule on Friday 00:00 UTC for both ecosystems. Cargo updates
grouped by patch and minor separately to keep blame clean when
something breaks. GitHub Actions updates grouped into one weekly PR;
major bumps come as individual PRs since they typically need
migration review. open-pull-requests-limit: 2 caps PR noise per
ecosystem."
```

---

## Task 6: Push and verify on GitHub

**Files:** none — verification only.

- [ ] **Step 6.1: Confirm the branch has all five commits**

Run:
```bash
git log --oneline main..HEAD
```

Expected: 7 commits total — 2 spec commits already on the branch (`docs: design for ci.yml best-practices reorganization` and `docs: resolve action SHAs in ci.yml best-practices spec`) + 5 from Tasks 1–5. The latest 5 should be (most recent first):

```
ci: add Dependabot config for cargo and github-actions
ci: add CodeQL workflow scanning rust and actions
ci: refactor ci.yml to use composite, add timeouts and polish
ci: add audit workflow with daily cargo-deny scan
ci: add setup-rust composite action
```

If any commit message differs or order is wrong, fix it before pushing (use `git rebase -i` only if absolutely necessary; otherwise leave history alone).

- [ ] **Step 6.2: Push the branch to origin**

```bash
git push -u origin chore/ci-best-practices
```

Expected: branch pushed; remote tracking set up.

- [ ] **Step 6.3: Open the PR**

```bash
gh pr create --title "ci: reorganize workflows per GitHub best practices" --body "$(cat <<'EOF'
## Summary

Reorganizes `.github/workflows/ci.yml` and adds three new files under `.github/` per the design in `docs/superpowers/specs/2026-05-14-ci-yml-best-practices-design.md`.

**New files**

- `.github/actions/setup-rust/action.yml` — composite action: checkout + toolchain + optional cargo tools + optional cache.
- `.github/workflows/audit.yml` — `cargo-deny` on PR/push/merge_group + daily cron.
- `.github/workflows/codeql.yml` — CodeQL matrix `[rust, actions]`.
- `.github/dependabot.yml` — weekly cargo + github-actions, Friday 00:00 UTC.

**Edited**

- `.github/workflows/ci.yml` — all 9 Rust jobs use the new composite; every job has `timeout-minutes: 3`; every step has a `name:`; third-party actions SHA-pinned with `# vX.Y.Z` comments for Dependabot; draft PRs no longer trigger CI; `persist-credentials: false` on every checkout; commented-out `semver-checks` block deleted (history in commit `114f5cb`); `deny` job moved to `audit.yml` and removed from `conclusion.needs`.

## Test plan

- [ ] All ci.yml checks green: `fmt`, `clippy`, `typos`, `docs`, `rdme`, `msrv`, `hack`, `test`, `sonarqube`, `conclusion`.
- [ ] `audit / deny` check green.
- [ ] `codeql / Analyze (rust)` and `codeql / Analyze (actions)` checks green.
- [ ] No job exceeds the 3-minute timeout. If one does, document the new value in the spec before bumping in code.
- [ ] Dependabot tab in repo settings shows two ecosystems (cargo, github-actions).
- [ ] Branch-protection's `conclusion` required check resolves green.

## Follow-up after merge

Add `audit / deny` to the `Protect main` ruleset's `required_status_checks`. The exact `gh api` payload is in the spec under "Required status checks (ruleset) → Post-merge follow-up".
EOF
)"
```

Expected: PR URL printed.

- [ ] **Step 6.4: Wait for and verify all checks**

Run:
```bash
gh pr checks --watch
```

Expected: all of `fmt`, `clippy`, `typos`, `docs`, `rdme`, `msrv`, `hack`, `test`, `sonarqube`, `conclusion`, `deny`, `Analyze (rust)`, `Analyze (actions)` report `pass`. The watch command exits with code 0.

If any check fails, **do not merge**. Diagnose and fix:

- **Timeout exceeded** — the most likely failure mode for `msrv`, `hack`, `sonarqube`, or `Analyze (rust)`. Bump that single job's `timeout-minutes` (5 is a reasonable next stop), document the change in the spec's "Per-job timeouts" table, commit, push. Re-run the check.
- **YAML parse error** — `actionlint` (install via `brew install actionlint`) gives the most precise location.
- **Composite action not found** — verify the path is exactly `./.github/actions/setup-rust` (leading dot, no `.git` etc.) and that Task 1's commit landed on the branch.
- **`code_scanning` rule blocks merge** (see spec's *Open questions*) — temporary fix is to PUT the ruleset removing that one rule, merge, then re-add. JSON payload is the same as the post-merge follow-up minus the `code_scanning` block.

- [ ] **Step 6.5: Mark plan complete**

Once all checks are green and the PR is mergeable:
- Comment on the PR linking back to this plan document.
- Do **not** merge automatically — leave the merge to the user (matches the project's "ask before shared-state actions" preference).
- After the user merges, run the post-merge ruleset PATCH documented in the spec to add `audit / deny` to required checks.

---

## Self-review checklist (for the implementer before opening the PR)

- [ ] Every file in the *File structure* section has a corresponding commit.
- [ ] No commit deletes the `deny` job from `ci.yml` without `audit.yml` already existing on the branch.
- [ ] No `<sha>` placeholder strings remain in any workflow or composite action file.
- [ ] All commit subjects use Conventional Commits format (`ci:` type).
- [ ] `python3 -c "import yaml; ..."` passes on every YAML file we touched.
- [ ] `conclusion.needs` lists exactly 9 jobs and does not contain `deny`.
- [ ] `actions/checkout` has `persist-credentials: false` in every occurrence (composite, `typos`, `sonarqube`, `audit.yml`, `codeql.yml`).
- [ ] Every third-party action `uses:` line ends with `# vX.Y.Z` (or `# stable` for `dtolnay/rust-toolchain`).
