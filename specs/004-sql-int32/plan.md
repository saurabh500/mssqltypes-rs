# Implementation Plan: SqlInt32

**Branch**: `004-sql-int32` | **Date**: 2026-03-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/004-sql-int32/spec.md`

## Summary

Implement `SqlInt32` — a Rust equivalent of C# `System.Data.SqlTypes.SqlInt32`. Signed 32-bit integer (−2,147,483,648 to 2,147,483,647) with SQL NULL support via `Option<i32>`. Arithmetic returns `Result<SqlInt32, SqlTypeError>` with overflow detection using Rust's `checked_*` methods (idiomatic equivalent of C#'s sign-bit checks and long-widening). Comparisons return `SqlBoolean` with three-valued NULL logic. Bitwise operations are infallible with NULL propagation. Follows patterns established by `SqlByte`, `SqlBoolean`, and `SqlInt16`.

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
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLInt32.cs (545 lines). Matches all arithmetic, comparison, bitwise, conversion behavior. Negation of MIN_VALUE: C# silently wraps (bug) — Rust detects overflow (stricter, correct). |
| II | Idiomatic Rust | PASS | PASS | `Option<i32>` repr, `checked_*` for overflow, standard traits, `Result` for fallible ops |
| III | Test-First Development | PASS | PASS | Spec defines 44 acceptance scenarios + edge cases |
| IV | Comprehensive Coverage | PASS | PASS | SqlInt32 is a required numeric type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All safe integer arithmetic via `checked_*` |
| VI | No External Deps | PASS | PASS | std only |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **Overflow detection via `checked_*`** (not C# sign-bit/widening patterns) — see [research.md](research.md#r1-overflow-detection-for-signed-i32)
2. **Negation of MIN_VALUE returns Overflow** (stricter than C#) — see [research.md](research.md#r2-negation-of-min_value)
3. **MIN_VALUE % -1 returns Overflow** (matches C#) — see [research.md](research.md#r3-remainder-min_value--1)
4. **Conversions scoped to existing types** (SqlBoolean, SqlByte, SqlInt16) — see [research.md](research.md#r4-available-conversions-scoped-to-existing-types)
5. **Bitwise ops use native Rust operators** (no casting needed) — see [research.md](research.md#r6-bitwise-operations--no-casting-needed-in-rust)
6. **C# named static methods not implemented** (Rust operators are idiomatic) — see [research.md](research.md#r8-c-named-static-methods-add-subtract-etc)

## Project Structure

### Documentation (this feature)

```text
specs/004-sql-int32/
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
├── lib.rs               # MODIFY: add sql_int32 module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons + conversion)
├── sql_byte.rs          # NO CHANGES (dependency for conversion)
├── sql_int16.rs         # NO CHANGES (dependency for conversion)
└── sql_int32.rs         # NEW: SqlInt32 implementation + inline tests
```

## Complexity Tracking

No violations — nothing to justify.
