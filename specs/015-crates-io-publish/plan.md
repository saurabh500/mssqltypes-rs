# Implementation Plan: crates.io Publish Readiness

**Branch**: `015-crates-io-publish` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/015-crates-io-publish/spec.md`

## Summary

Prepare the `mssqltypes` crate for initial publication on crates.io by adding required package metadata to `Cargo.toml`, excluding non-source files from the package, adding crate-level documentation to `lib.rs`, declaring MSRV, and enabling the `missing_docs` lint. No behavioral or API changes — this is purely packaging and documentation.

## Technical Context

**Language/Version**: Rust 1.93.1 (stable), edition 2024
**Primary Dependencies**: None (zero-dependency library)
**Storage**: N/A
**Testing**: `cargo test` (1,469 tests), `cargo test --doc` (currently 0 doc-tests)
**Target Platform**: All platforms (pure Rust library, no platform-specific code)
**Project Type**: Library (published to crates.io)
**Performance Goals**: N/A (packaging changes only)
**Constraints**: Package must be <100 KB compressed; zero publish warnings
**Scale/Scope**: 16 source files, ~307 public API items across 14 types + error enum

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Constitution Principle | Status | Notes |
|----------------------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | No behavioral changes — packaging only |
| II. Idiomatic Rust Design | ✅ PASS | Adding standard Rust docs and metadata |
| III. Test-First Development | ✅ PASS | Doc-tests will be added; all existing 1,469 tests unaffected |
| IV. Comprehensive Type Coverage | ✅ PASS | No type changes |
| V. Zero Unsafe Code | ✅ PASS | No code changes |
| VI. No External Runtime Dependencies | ✅ PASS | No new dependencies |
| VII. Versioning & Breaking Changes | ✅ PASS | Version stays 0.1.0; no API changes |
| MSRV documented in Cargo.toml | ✅ ADDRESSED | Adding `rust-version = "1.85"` per constitution requirement |
| Code Quality Gates (fmt, clippy, test) | ✅ PASS | Currently passing; this feature adds doc-tests |

**Gate Result**: ALL PASS — no violations, no justifications needed.

## Project Structure

### Documentation (this feature)

```text
specs/015-crates-io-publish/
├── plan.md              # This file
├── research.md          # Phase 0: crates.io best practices, edition decision
├── data-model.md        # Phase 1: Cargo.toml metadata model
├── quickstart.md        # Phase 1: How to verify publish readiness
├── contracts/           # Phase 1: N/A (no external API changes)
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
# Files modified by this feature:
Cargo.toml               # Add metadata, exclude, rust-version
src/lib.rs               # Add crate-level //! docs, #![warn(missing_docs)]
src/*.rs                 # Add missing /// doc comments to public items
```

**Structure Decision**: This feature modifies existing files only (`Cargo.toml`, `src/lib.rs`, and any `src/*.rs` files missing doc comments). No new source files or directories are created.

## Complexity Tracking

No constitution violations — this table is intentionally empty.
