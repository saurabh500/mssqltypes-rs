# Implementation Plan: SqlBinary

**Branch**: `012-sql-binary` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/012-sql-binary/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement `SqlBinary` — a nullable variable-length byte sequence equivalent to C# `System.Data.SqlTypes.SqlBinary`. Core features: construction from `Vec<u8>`, NULL handling, indexed access (`get`), concatenation via `Add` with NULL propagation, SQL comparisons with trailing-zero-padded semantics, `Display` as lowercase hex, and standard Rust traits (`Eq`, `Hash`, `Ord`) all using trailing-zero-padded normalization.

## Technical Context

**Language/Version**: Rust (Edition 2024, stable)
**Primary Dependencies**: None (std-only library)
**Storage**: N/A
**Testing**: `cargo test` (built-in), TDD mandatory per constitution
**Target Platform**: All platforms supported by Rust std
**Project Type**: Library
**Performance Goals**: No unnecessary heap allocations in comparisons or hashing
**Constraints**: No `unsafe` code, no external dependencies, no panics
**Scale/Scope**: Single type implementation (~200-400 lines)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | Trailing-zero-padded comparison matches C# `PerformCompareByte`. Display deviates from C# `"SqlBinary({len})"` → lowercase hex per spec (justified: more useful for debugging). |
| II. Idiomatic Rust | ✅ PASS | `Option<Vec<u8>>`, `Result` returns, standard traits (`Clone`, `Debug`, `Display`, `Eq`, `Hash`, `Ord`), operator overloading (`Add`). Not `Copy` — heap-allocated. |
| III. TDD | ✅ PASS | Tests before implementation. ≥95% coverage target. |
| IV. Type Coverage | ✅ PASS | SqlBinary is one of the required types. |
| V. Zero Unsafe | ✅ PASS | No unsafe code needed. |
| VI. No External Deps | ✅ PASS | std-only. |
| VII. Versioning | ✅ PASS | Pre-1.0, new type addition (minor). |

### Post-Design Re-check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | Comparison algorithm is a direct port of C#. Hash normalization matches C# `GetHashCode`. Display format is a justified deviation. |
| II. Idiomatic Rust | ✅ PASS | `value()` returns `&[u8]` (borrowed) instead of cloning like C#. `get()` method instead of panicking `Index` trait. `From<&[u8]>` and `From<Vec<u8>>` conversions. |
| III. TDD | ✅ PASS | All phases include test tasks before implementation. |

## Project Structure

### Documentation (this feature)

```text
specs/012-sql-binary/
├── plan.md              # This file
├── research.md          # Phase 0 output — 12 research decisions
├── data-model.md        # Phase 1 output — entity model
├── quickstart.md        # Phase 1 output — usage examples
├── contracts/
│   └── public-api.md    # Phase 1 output — full API surface
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Add: pub mod sql_binary; pub use sql_binary::SqlBinary;
├── sql_binary.rs         # NEW: SqlBinary type + tests
└── error.rs              # Existing: SqlTypeError (OutOfRange variant already present)
```

**Structure Decision**: Single new file `src/sql_binary.rs` following established pattern. Tests inline in the module (same pattern as all existing types). No new dependencies or error variants needed — `SqlTypeError::OutOfRange` and `SqlTypeError::NullValue` already exist.

## Complexity Tracking

No violations to justify. Single file, single type, no new dependencies.
