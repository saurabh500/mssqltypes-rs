# Tasks: crates.io Publish Readiness

**Input**: Design documents from `/specs/015-crates-io-publish/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/packaging.md ✅, quickstart.md ✅

**Tests**: Doc-tests are included as part of US3 (crate-level docs). No separate test phase — validation uses cargo built-in commands per quickstart.md.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `Cargo.toml` at repository root
- This feature modifies existing files only — no new source files

---

## Phase 1: Setup

**Purpose**: No project initialization needed — this feature modifies an existing, working crate. No setup tasks required.

_(No tasks — the project is already initialized and building.)_

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: No foundational infrastructure needed — all changes are independent modifications to existing files.

_(No tasks — the crate already builds, tests, and lints cleanly.)_

---

## Phase 3: User Story 1 — Cargo.toml Metadata (Priority: P1) 🎯 MVP

**Goal**: Add all required and recommended crates.io metadata to `Cargo.toml` so `cargo publish --dry-run` produces zero metadata warnings.

**Independent Test**: `cargo publish --dry-run --allow-dirty 2>&1 | grep -c 'warning: manifest'` returns `0`.

### Implementation for User Story 1

- [ ] T001 [US1] Add `description` field to `[package]` section in Cargo.toml — value: `"Faithful Rust equivalents of C# System.Data.SqlTypes with SQL NULL semantics, checked arithmetic, and three-valued logic"`
- [ ] T002 [US1] Add `license = "MIT"` to `[package]` section in Cargo.toml
- [ ] T003 [US1] Add `repository = "https://github.com/saurabh500/mssqltypes-rs"` to `[package]` section in Cargo.toml
- [ ] T004 [US1] Add `readme = "README.md"` to `[package]` section in Cargo.toml
- [ ] T005 [US1] Add `keywords = ["sql-server", "sql-types", "mssql", "database", "tds"]` to `[package]` section in Cargo.toml
- [ ] T006 [US1] Add `categories = ["database", "data-structures"]` to `[package]` section in Cargo.toml

> **Note**: Tasks T001–T006 all modify the same file (`Cargo.toml`) so they should be applied as a single edit. They are listed separately for traceability to FR-001 through FR-006.

**Checkpoint**: `cargo publish --dry-run --allow-dirty` should produce zero `warning: manifest has no ...` messages.

---

## Phase 4: User Story 2 — Exclude Non-Source Files (Priority: P1)

**Goal**: Add `exclude` patterns to `Cargo.toml` so the published package contains only source code, license, and README (~21 files, <100 KB compressed).

**Independent Test**: `cargo package --list --allow-dirty 2>/dev/null | grep -cE '^\.(github|specify)/|^specs/'` returns `0`.

### Implementation for User Story 2

- [ ] T007 [US2] Add `exclude = [".github/", ".specify/", "specs/"]` to `[package]` section in Cargo.toml
- [ ] T008 [US2] Verify package contents by running `cargo package --list --allow-dirty` and confirming no `.github/`, `.specify/`, or `specs/` files appear
- [ ] T009 [US2] Verify compressed package size is under 100 KB by running `cargo package --allow-dirty 2>&1 | grep Packaged`

**Checkpoint**: Package contains ~21 files and compressed size is <100 KB (down from 170 files / 322 KB).

---

## Phase 5: User Story 3 — Crate-Level Documentation (Priority: P1)

**Goal**: Add `//!` crate-level documentation to `src/lib.rs` with overview, type table, quick-start example (as doc-test), and feature flags section. The docs.rs landing page should be populated.

**Independent Test**: `cargo test --doc` passes with ≥1 doc-test; `cargo doc` generates a populated crate root page.

### Implementation for User Story 3

- [ ] T010 [US3] Add crate-level `//!` doc block at top of src/lib.rs with: crate title, one-paragraph description, key features list
- [ ] T011 [US3] Add type overview table to `//!` doc block in src/lib.rs mapping all 14 types (Rust type → SQL Server type → C# equivalent)
- [ ] T012 [US3] Add quick-start code example to `//!` doc block in src/lib.rs using `SqlInt32` and `SqlBoolean` — must compile as a doc-test
- [ ] T013 [US3] Add feature flags section to `//!` doc block in src/lib.rs documenting optional `serde` support
- [ ] T014 [US3] Run `cargo test --doc` and verify ≥1 doc-test passes
- [ ] T015 [US3] Run `cargo doc` and visually verify the crate root page is populated with description, type table, and example

> **Note**: Tasks T010–T013 all modify the same file (`src/lib.rs`) so they should be applied as a single edit. They are listed separately for traceability to the crate-level documentation contract.

**Checkpoint**: `cargo test --doc` passes with at least 1 doc-test. `cargo doc` shows a populated landing page.

---

## Phase 6: User Story 4 — Declare MSRV (Priority: P2)

**Goal**: Add `rust-version = "1.85"` to `Cargo.toml` so users on older toolchains get a clear error at dependency resolution time.

**Independent Test**: `grep 'rust-version' Cargo.toml` shows `"1.85"`.

### Implementation for User Story 4

- [ ] T016 [US4] Add `rust-version = "1.85"` to `[package]` section in Cargo.toml

**Checkpoint**: `rust-version` is set. Users on Rust <1.85 will get a clear MSRV error.

---

## Phase 7: User Story 5 — Enable missing_docs Lint (Priority: P2)

**Goal**: Add `#![warn(missing_docs)]` to `src/lib.rs` and fix all resulting warnings so doc coverage is enforced going forward.

**Independent Test**: `RUSTFLAGS="-W missing-docs" cargo check 2>&1 | grep -c 'missing documentation'` returns `0`.

### Implementation for User Story 5

- [ ] T017 [US5] Add `#![warn(missing_docs)]` attribute at top of src/lib.rs (after the `//!` doc block, before `pub mod` declarations)
- [ ] T018 [P] [US5] Add `//!` module-level doc comment to src/error.rs (currently missing)
- [ ] T019 [P] [US5] Add `//!` module-level doc comment to src/sql_binary.rs (currently has `//` comment, needs `//!`)
- [ ] T020 [P] [US5] Add `//!` module-level doc comment to src/sql_boolean.rs (currently missing)
- [ ] T021 [P] [US5] Add `//!` module-level doc comment to src/sql_byte.rs (currently has `//` comment, needs `//!`)
- [ ] T022 [P] [US5] Add `//!` module-level doc comment to src/sql_guid.rs (currently has `//` comment, needs `//!`)
- [ ] T023 [US5] Run `RUSTFLAGS="-W missing-docs" cargo check` and verify zero `missing documentation` warnings remain

**Checkpoint**: All modules have `//!` doc comments. `#![warn(missing_docs)]` is active with zero warnings.

---

## Phase 8: User Story 6 — Edition Decision (Priority: P3)

**Goal**: Document the deliberate decision to keep edition 2024, per research.md findings.

**Independent Test**: Decision is documented; `cargo test` still passes.

### Implementation for User Story 6

- [ ] T024 [US6] Confirm edition 2024 decision is documented in specs/015-crates-io-publish/research.md section 4 (already done during planning — verify and close)
- [ ] T025 [US6] Run `cargo test` and confirm all 1,469+ tests still pass with edition 2024

**Checkpoint**: Edition decision documented and validated.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories

- [ ] T026 Run `cargo publish --dry-run --allow-dirty` and verify zero warnings (SC-001)
- [ ] T027 Run `cargo package --list --allow-dirty` and verify <25 files, no excluded dirs (SC-002)
- [ ] T028 Verify compressed package size <100 KB (SC-003)
- [ ] T029 Run `cargo test --doc` and verify ≥1 doc-test passes (SC-004)
- [ ] T030 Run `cargo doc` and verify crate root page is populated (SC-005)
- [ ] T031 Run `cargo test` and verify all 1,469+ tests pass (SC-006)
- [ ] T032 Run `cargo clippy -- -D warnings` and verify zero warnings
- [ ] T033 Run `cargo fmt --check` and verify no formatting issues
- [ ] T034 Run quickstart.md full validation script to confirm end-to-end

**Checkpoint**: All success criteria (SC-001 through SC-006) met. Crate is ready for `cargo publish`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: N/A — no setup tasks
- **Foundational (Phase 2)**: N/A — no foundational tasks
- **US1 Metadata (Phase 3)**: No dependencies — can start immediately
- **US2 Exclude (Phase 4)**: No dependencies — can start immediately (different section of Cargo.toml)
- **US3 Crate Docs (Phase 5)**: No dependencies — can start immediately (different file: src/lib.rs)
- **US4 MSRV (Phase 6)**: No dependencies — can start immediately (single line in Cargo.toml)
- **US5 missing_docs (Phase 7)**: Depends on US3 completion (crate-level docs must exist before enabling the lint)
- **US6 Edition (Phase 8)**: No dependencies — verification only
- **Polish (Phase 9)**: Depends on US1–US5 completion

### User Story Dependencies

```text
US1 (Metadata)    ──┐
US2 (Exclude)     ──┤
US3 (Crate Docs)  ──┼──→ US5 (missing_docs) ──→ Polish (Phase 9)
US4 (MSRV)        ──┤
US6 (Edition)     ──┘
```

### Parallel Opportunities

- **US1 + US2 + US4**: All modify `Cargo.toml` `[package]` section — can be combined into a single edit
- **US3**: Modifies `src/lib.rs` — independent of Cargo.toml changes, parallelizable
- **US5 T018–T022**: All modify different `src/*.rs` files — fully parallelizable
- **US6**: Read-only verification — parallelizable with everything

### Within Each User Story

- Cargo.toml tasks (US1, US2, US4) can be combined into one atomic edit
- lib.rs crate-level doc tasks (US3 T010–T013) should be combined into one atomic edit
- Module doc tasks (US5 T018–T022) are independent and parallelizable

---

## Parallel Example: Maximum Parallelism

```text
# Batch 1: All independent edits (parallel)
T001–T007, T016  →  Cargo.toml (metadata + exclude + MSRV — single combined edit)
T010–T013        →  src/lib.rs (crate-level docs — single combined edit)
T024             →  Verify edition decision (read-only)

# Batch 2: After Batch 1 completes
T017             →  src/lib.rs (#![warn(missing_docs)] attribute)
T018–T022        →  src/error.rs, src/sql_binary.rs, src/sql_boolean.rs, src/sql_byte.rs, src/sql_guid.rs (module docs — all parallel)

# Batch 3: Verification
T008–T009, T014–T015, T023, T025–T034  →  All verification tasks
```

---

## Implementation Strategy

### MVP First (US1 + US2 + US3)

1. Apply Cargo.toml metadata + exclude (US1 + US2) — single edit
2. Add crate-level docs to lib.rs (US3) — single edit
3. **STOP and VALIDATE**: `cargo publish --dry-run --allow-dirty` should be warning-free
4. This is a publishable state — crate can go to crates.io at this point

### Full Delivery

1. MVP (US1 + US2 + US3) → publish-ready
2. Add MSRV (US4) → better user experience on old toolchains
3. Enable missing_docs lint (US5) → doc coverage enforcement
4. Confirm edition decision (US6) → documented governance
5. Run full validation (Polish) → all success criteria verified

---

## Notes

- All Cargo.toml changes (T001–T007, T016) should ideally be applied as a **single atomic edit** since they all modify the `[package]` section
- All crate-level doc changes (T010–T013) should be applied as a **single atomic edit** since they form one contiguous `//!` block
- The `<owner>` placeholder in the contracts doc has been resolved: repository is `https://github.com/saurabh500/mssqltypes-rs`
- Commit after each phase for clean git history
- Total: 34 tasks (10 implementation, 10 verification, 14 combined/atomic)
