# Feature Specification: crates.io Publish Readiness

**Feature Branch**: `chore/crates-io-publish`
**Created**: 2026-03-02
**Status**: Draft
**Input**: Prepare the mssqltypes crate for initial publication on crates.io

## Overview

The `mssqltypes` crate is functionally complete (1,469 tests passing, zero clippy warnings, clean doc build) but the packaging metadata, documentation, and file exclusions need attention before publishing to crates.io. This spec captures the requirements to make the crate publish-ready.

## User Scenarios & Testing

### User Story 1 - Cargo.toml metadata for crates.io (Priority: P1)

A user searching crates.io for SQL Server type libraries must be able to discover this crate via keywords, read a description, and navigate to the repository.

**Why this priority**: Without required metadata (`description`, `license`), `cargo publish` emits warnings and the crate page on crates.io is incomplete/undiscoverable.

**Independent Test**: Run `cargo publish --dry-run --allow-dirty` and confirm zero warnings about missing metadata.

**Acceptance Scenarios**:

1. **Given** `Cargo.toml`, **When** inspected, **Then** it contains `description`, `license`, `repository`, `homepage`, `documentation`, `readme`, `keywords`, and `categories`
2. **Given** `cargo publish --dry-run --allow-dirty`, **When** run, **Then** zero warnings about missing metadata
3. **Given** `keywords` field, **When** inspected, **Then** contains at most 5 keywords relevant to SQL Server types (crates.io limit is 5)
4. **Given** `categories` field, **When** inspected, **Then** contains valid crates.io categories (e.g., `database`, `data-structures`)
5. **Given** `license` field, **When** inspected, **Then** value is `MIT` matching the LICENSE file

---

### User Story 2 - Exclude non-source files from package (Priority: P1)

A downstream user running `cargo add mssqltypes` should download a lean package containing only source code, license, and README — not specs, CI workflows, or agent prompts.

**Why this priority**: The current package ships 168 files / 1.4 MB including 93 spec files and 52 `.github`/`.specify` files. This is unnecessary bloat that wastes bandwidth and clutters the crates.io source view.

**Independent Test**: Run `cargo package --list --allow-dirty` and confirm only `src/`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `LICENSE`, `README.md`, and `.cargo_vcs_info.json` are included.

**Acceptance Scenarios**:

1. **Given** `Cargo.toml` `exclude` field, **When** inspected, **Then** it excludes `.github/`, `.specify/`, `specs/`, and `target/`
2. **Given** `cargo package --list --allow-dirty`, **When** run, **Then** no files from `.github/`, `.specify/`, or `specs/` appear
3. **Given** packaged crate, **When** size measured, **Then** compressed size is under 100 KB (down from ~320 KB)

---

### User Story 3 - Crate-level documentation in lib.rs (Priority: P1)

A developer browsing docs.rs/mssqltypes should see a meaningful landing page with an overview and quick-start example.

**Why this priority**: Currently there are zero `//!` doc comments in `lib.rs`, so the docs.rs landing page is blank. This is the first thing users see.

**Independent Test**: Run `cargo doc --open` and verify the crate root page has a description and a working code example.

**Acceptance Scenarios**:

1. **Given** `src/lib.rs`, **When** inspected, **Then** it begins with `//!` crate-level doc comments including a description and usage example
2. **Given** `cargo test --doc`, **When** run, **Then** at least one doc-test passes (from the crate-level example)
3. **Given** docs.rs rendering, **When** the crate root is viewed, **Then** it displays a type overview table, quick-start example, and links to key types

---

### User Story 4 - Declare minimum supported Rust version (Priority: P2)

A user on an older Rust toolchain should get a clear error at dependency resolution time rather than a cryptic compile failure.

**Why this priority**: The crate uses `edition = "2024"` which requires Rust 1.85+. Without `rust-version`, users on older toolchains get confusing errors.

**Independent Test**: Confirm `rust-version` is set in `Cargo.toml` and matches the minimum toolchain that supports edition 2024.

**Acceptance Scenarios**:

1. **Given** `Cargo.toml`, **When** inspected, **Then** `rust-version = "1.85"` is present
2. **Given** a Rust toolchain older than 1.85, **When** `cargo build` is run, **Then** cargo reports a clear MSRV error

---

### User Story 5 - Enable missing_docs lint (Priority: P2)

All public API items should have documentation. Enforcing this via a lint prevents regressions.

**Why this priority**: Currently ~307 public items exist across 16 files. Some key types (`SqlBoolean`, `SqlInt32`, `SqlTypeError`) are missing struct/enum-level doc comments. A lint ensures coverage stays high.

**Independent Test**: Add `#![warn(missing_docs)]` to `lib.rs` and run `cargo doc`. All warnings should be resolved.

**Acceptance Scenarios**:

1. **Given** `src/lib.rs`, **When** inspected, **Then** contains `#![warn(missing_docs)]`
2. **Given** `cargo doc 2>&1`, **When** run, **Then** zero `missing_docs` warnings remain
3. **Given** any public `fn`, `struct`, `enum`, `const`, or `type`, **When** inspected, **Then** it has a `///` doc comment

---

### User Story 6 - Evaluate edition 2024 vs 2021 (Priority: P3)

The team should make a deliberate choice about using edition 2024 vs 2021 based on feature needs and compatibility reach.

**Why this priority**: Edition 2024 limits users to Rust 1.85+. Edition 2021 (stable since Rust 1.56) gives much broader compatibility. This is a trade-off decision, not a strict requirement.

**Independent Test**: If edition is changed to 2021, `cargo test` must still pass with zero failures.

**Acceptance Scenarios**:

1. **Given** the codebase, **When** audited for edition-2024-specific features, **Then** a documented decision exists on whether to keep edition 2024 or downgrade to 2021
2. **Given** the chosen edition, **When** `cargo test` is run, **Then** all 1,469+ tests pass

---

### Edge Cases

- `keywords` must not exceed 5 entries (crates.io hard limit)
- `categories` must use valid slugs from [crates.io/category_slugs](https://crates.io/category_slugs)
- `description` should be under 1000 characters
- `exclude` patterns must not accidentally exclude `src/` or other required files
- Crate name `mssqltypes` must not be already taken on crates.io (verify before publish)

## Requirements

### Functional Requirements

- **FR-001**: `Cargo.toml` MUST contain `description` (concise, under 1000 chars)
- **FR-002**: `Cargo.toml` MUST contain `license = "MIT"`
- **FR-003**: `Cargo.toml` MUST contain `repository` pointing to the GitHub repo
- **FR-004**: `Cargo.toml` MUST contain `keywords` with at most 5 relevant terms
- **FR-005**: `Cargo.toml` MUST contain `categories` with valid crates.io slugs
- **FR-006**: `Cargo.toml` MUST contain `readme = "README.md"`
- **FR-007**: `Cargo.toml` MUST contain `exclude` removing `.github/`, `.specify/`, `specs/`, `target/`
- **FR-008**: `Cargo.toml` MUST contain `rust-version = "1.85"` (or appropriate MSRV)
- **FR-009**: `src/lib.rs` MUST contain crate-level `//!` documentation with overview and example
- **FR-010**: `src/lib.rs` SHOULD contain `#![warn(missing_docs)]`
- **FR-011**: All public items SHOULD have `///` doc comments
- **FR-012**: `cargo publish --dry-run` MUST complete with zero warnings
- **FR-013**: `cargo test --doc` MUST pass with at least one doc-test
- **FR-014**: Packaged crate MUST contain only source code, `Cargo.toml`, `LICENSE`, and `README.md` (plus cargo-generated files)

### Key Entities

- **Cargo.toml `[package]` section**: Crate metadata consumed by crates.io for discovery, display, and validation
- **Crate-level docs (`//!`)**: Content rendered on the docs.rs landing page
- **Package file list**: The set of files included in the `.crate` tarball uploaded to crates.io

## Success Criteria

### Measurable Outcomes

- **SC-001**: `cargo publish --dry-run --allow-dirty` completes with zero warnings
- **SC-002**: `cargo package --list --allow-dirty` shows fewer than 25 files (down from 168)
- **SC-003**: Compressed package size is under 100 KB (down from 320 KB)
- **SC-004**: `cargo test --doc` runs at least 1 doc-test successfully
- **SC-005**: `cargo doc` generates docs with a populated crate root page
- **SC-006**: All 1,469+ existing tests continue to pass
