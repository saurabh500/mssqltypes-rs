# Implementation Plan: SqlGuid

**Branch**: `011-sql-guid` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-sql-guid/spec.md`

## Summary

Implement `SqlGuid` — a nullable 128-bit GUID type that replicates the behavior of C# `System.Data.SqlTypes.SqlGuid`. The defining feature is SQL Server's non-standard byte comparison ordering (`[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`). Internally stored as `Option<[u8; 16]>` with `Copy + Clone` semantics. No external dependencies; .NET mixed-endian byte layout used for string parsing/formatting. `PartialEq`/`Eq` use natural byte equality; `PartialOrd`/`Ord` use SQL Server byte ordering. All SQL comparison methods return `SqlBoolean` with NULL propagation.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)
**Target Platform**: All platforms supported by Rust std
**Project Type**: Library
**Performance Goals**: Stack-allocated; no heap allocations for SqlGuid operations
**Constraints**: Zero `unsafe` code; zero external runtime dependencies
**Scale/Scope**: Single file `src/sql_guid.rs` (~400-600 LOC including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Behavioral Fidelity | PASS | SQL byte ordering `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` replicated exactly. `Eq` uses natural bytes (matches C# `Equals`), `Ord` uses SQL byte order (matches C# `CompareTo`). Display uses lowercase hex "D" format. |
| II | Idiomatic Rust | PASS | Standard traits (`Display`, `FromStr`, `Eq`, `Ord`, `Hash`, `Copy`, `Clone`, `Debug`). `Result` for fallible ops. No operator overloading needed (no arithmetic). |
| III | Test-First Development | PASS | Will follow TDD: tests before implementation. Target ≥95% coverage per spec SC-006. |
| IV | Comprehensive Type Coverage | PASS | SqlGuid is listed in constitution as required type. |
| V | Zero Unsafe Code | PASS | Pure safe Rust; byte array operations only. |
| VI | No External Dependencies | PASS | No uuid crate; pure byte-array implementation with manual hex parsing. |
| VII | Versioning | PASS | New type addition = MINOR version bump (backward-compatible). |

**Gate result**: ALL PASS — proceed to Phase 0.

### Post-Design Re-Check

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Behavioral Fidelity | PASS | Research confirmed C# behavior: `Eq` = natural bytes, `CompareTo` = SQL order, `ToString` = lowercase "D" format, `Parse` accepts "Null". Mixed-endian byte layout for string parsing verified. |
| II | Idiomatic Rust | PASS | API follows established patterns from SqlByte/SqlInt32/SqlBinary. `From<[u8; 16]>` trait impl. |
| III | Test-First Development | PASS | Test vectors designed for all 6 byte-group boundaries in SQL ordering. |
| V | Zero Unsafe Code | PASS | No unsafe needed for hex parsing or byte reordering. |
| VI | No External Dependencies | PASS | Hand-written hex parser; no uuid/hex crate dependency. |

**Post-design gate result**: ALL PASS — proceed to Phase 2.

## Project Structure

### Documentation (this feature)

```text
specs/011-sql-guid/
├── plan.md              # This file
├── research.md          # Phase 0: 10 research decisions (R1-R10)
├── data-model.md        # Phase 1: entity model, byte layout, comparison semantics
├── quickstart.md        # Phase 1: usage examples for all 4 user stories
├── contracts/
│   └── public-api.md    # Phase 1: full public API surface
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Add: pub mod sql_guid; pub use sql_guid::SqlGuid;
├── sql_guid.rs           # NEW: SqlGuid implementation + inline tests
├── error.rs             # Existing: SqlTypeError (no changes needed)
├── sql_boolean.rs       # Existing: used by SQL comparison return types
└── sql_binary.rs        # Existing: used by to_sql_binary/from_sql_binary
```

**Structure Decision**: Single new file `src/sql_guid.rs` following the established pattern of one file per type. Module registration in `src/lib.rs`. No new error variants needed — `NullValue`, `ParseError`, and `OutOfRange` already exist in `SqlTypeError`.

## Complexity Tracking

> No constitution violations. No complexity justification needed.
