# Implementation Plan: SqlInt64

**Branch**: `005-sql-int64` | **Date**: 2026-03-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/005-sql-int64/spec.md`

## Summary

Implement `SqlInt64` — a Rust equivalent of C# `System.Data.SqlTypes.SqlInt64`. Signed 64-bit integer (−9,223,372,036,854,775,808 to 9,223,372,036,854,775,807) with SQL NULL support via `Option<i64>`. Arithmetic returns `Result<SqlInt64, SqlTypeError>` with overflow detection using Rust's `checked_*` methods (idiomatic replacement for C#'s sign-bit checks and split-half multiplication). Comparisons return `SqlBoolean` with three-valued NULL logic. Bitwise operations are infallible with NULL propagation. Follows patterns established by `SqlByte`, `SqlInt16`, `SqlInt32`, and `SqlBoolean`.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)
**Target Platform**: All platforms (pure Rust)
**Project Type**: Library
**Performance Goals**: Stack-allocated, zero-allocation arithmetic/comparison
**Constraints**: No `unsafe`, no external deps, no panics
**Scale/Scope**: Single module (~700 lines including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Pre-Design | Post-Design | Notes |
|---|-----------|-----------|-------------|-------|
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLInt64.cs (603 lines). Matches all arithmetic, comparison, bitwise, conversion behavior. Negation of MIN_VALUE: C# silently wraps (bug, no overflow check at line 92) — Rust detects overflow (stricter, correct). Multiplication: C# uses split-half approach (high/low 32-bit words) — Rust uses `checked_mul` (equivalent detection, idiomatic). |
| II | Idiomatic Rust | PASS | PASS | `Option<i64>` repr, `checked_*` for overflow, standard traits, `Result` for fallible ops |
| III | Test-First Development | PASS | PASS | Spec defines 47 acceptance scenarios + edge cases |
| IV | Comprehensive Coverage | PASS | PASS | SqlInt64 is a required numeric type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All safe integer arithmetic via `checked_*` |
| VI | No External Deps | PASS | PASS | std only |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **Overflow detection via `checked_*`** (not C# sign-bit/split-half patterns) — see [research.md](research.md#r1-overflow-detection-for-signed-i64)
2. **Negation of MIN_VALUE returns Overflow** (stricter than C#) — see [research.md](research.md#r2-negation-of-min_value)
3. **MIN_VALUE % -1 returns Overflow** (matches C#) — see [research.md](research.md#r3-remainder-min_value--1)
4. **Conversions scoped to existing types** (SqlBoolean, SqlByte, SqlInt16, SqlInt32) — see [research.md](research.md#r4-available-conversions-scoped-to-existing-types)
5. **Bitwise ops use native Rust operators** (no casting needed) — see [research.md](research.md#r5-bitwise-operations--no-casting-needed-in-rust)
6. **C# named static methods not implemented** (Rust operators are idiomatic) — see [research.md](research.md#r7-c-named-static-methods-add-subtract-etc)

## Project Structure

### Documentation (this feature)

```text
specs/005-sql-int64/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── public-api.md    # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # MODIFY: add sql_int64 module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons + conversion)
├── sql_byte.rs          # NO CHANGES (dependency for conversion)
├── sql_int16.rs         # NO CHANGES (dependency for conversion)
├── sql_int32.rs         # NO CHANGES (dependency for conversion)
└── sql_int64.rs         # NEW: SqlInt64 implementation + inline tests
```

## Complexity Tracking

No violations — nothing to justify.
