# Implementation Plan: SqlString

**Branch**: `013-sql-string` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/013-sql-string/spec.md`

## Summary

Implement `SqlString` — a nullable string type equivalent to C# `System.Data.SqlTypes.SqlString` with configurable comparison options. Uses `Option<String>` internally with a `SqlCompareOptions` enum controlling comparison behavior (case-insensitive by default, ordinal, or binary sort). Includes a companion `SqlCompareOptions` enum with 4 variants. Unlike numeric SQL types, `SqlString` is `Clone` but not `Copy` due to heap-allocated `String`. Concatenation via `+` operator is infallible (returns `SqlString` directly). SQL comparisons return `SqlBoolean` with NULL propagation. Left operand's compare options govern mixed-options comparisons.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A — in-memory heap-allocated string
**Testing**: `cargo test` (built-in Rust test framework)
**Target Platform**: All platforms supporting Rust stable
**Project Type**: Library
**Performance Goals**: Minimal allocation on comparison (trim + lowercase are stack-local or iterators where possible)
**Constraints**: No `unsafe` code; no external dependencies; ASCII case folding only (no ICU)
**Scale/Scope**: Two types (~800-1100 LOC including tests): `SqlCompareOptions` enum + `SqlString` struct

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | Port of C# `SQLString.cs` — NULL propagation, comparison options, concatenation, trailing space trimming. Locale/collation dropped (no Rust equivalent without external deps) with explicit justification (R1, R2) |
| II. Idiomatic Rust | ✅ PASS | `Option<String>`, simple enum (not bitflags), `Add` trait for concatenation, `&str` constructors, `FromStr`, standard traits |
| III. Test-First Development | ✅ PASS | TDD mandatory per constitution; tests before implementation |
| IV. Comprehensive Type Coverage | ✅ PASS | SqlString is one of the required types in the project overview |
| V. Zero Unsafe Code | ✅ PASS | Pure safe Rust |
| VI. No External Dependencies | ✅ PASS | std only — this is WHY we drop locale/ICU support |
| VII. Versioning | ✅ PASS | Pre-1.0, additive change |

No gate violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/013-sql-string/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── public-api.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Module registration: pub mod sql_string; pub mod sql_compare_options;
├── error.rs             # Shared SqlTypeError enum (existing)
├── sql_boolean.rs       # SqlBoolean dependency — comparison return type (existing)
├── sql_compare_options.rs  # NEW — SqlCompareOptions enum
└── sql_string.rs        # NEW — SqlString implementation + inline tests
```

**Structure Decision**: Two new files: `src/sql_compare_options.rs` for the `SqlCompareOptions` enum (small, reusable, may be referenced by future types) and `src/sql_string.rs` for the `SqlString` struct with inline `#[cfg(test)] mod tests`. Both registered in `src/lib.rs`. Follows the established one-type-per-file pattern.
