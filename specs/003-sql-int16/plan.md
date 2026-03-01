# Implementation Plan: SqlInt16

**Branch**: `003-sql-int16` | **Date**: 2026-03-01 | **Spec**: [spec.md](../../.specify/specs/003-sql-int16/spec.md)
**Input**: Feature specification from `.specify/specs/sql-int16.md`

## Summary

Implement `SqlInt16` — a Rust equivalent of C# `System.Data.SqlTypes.SqlInt16`. Signed 16-bit integer (−32,768 to 32,767) with SQL NULL support via `Option<i16>`. Arithmetic returns `Result<SqlInt16, SqlTypeError>` with overflow detection using Rust's `checked_*` methods (idiomatic equivalent of C#'s widened-to-int bit-shift checks). Comparisons return `SqlBoolean` with three-valued NULL logic. Bitwise operations are infallible with NULL propagation. Follows patterns established by `SqlByte` and `SqlBoolean`.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)
**Target Platform**: All platforms (pure Rust)
**Project Type**: Library
**Performance Goals**: Stack-allocated, zero-allocation arithmetic/comparison
**Constraints**: No `unsafe`, no external deps, no panics
**Scale/Scope**: Single module (~600 lines including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Pre-Design | Post-Design | Notes |
|---|-----------|-----------|-------------|-------|
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLInt16.cs (530 lines). Matches all arithmetic, comparison, bitwise, conversion behavior. Negation of MIN_VALUE: Rust detects overflow (stricter than C# which silently wraps — C# bug, not intentional behavior). |
| II | Idiomatic Rust | PASS | PASS | `Option<i16>` repr, `checked_*` for overflow, standard traits, `Result` for fallible ops |
| III | Test-First Development | PASS | PASS | Spec defines 13 acceptance scenarios + edge cases |
| IV | Comprehensive Coverage | PASS | PASS | SqlInt16 is a required numeric type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All safe integer arithmetic via `checked_*` |
| VI | No External Deps | PASS | PASS | std only |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **Overflow detection via `checked_*`** (not C# bit-shift pattern) — see [research.md](research.md#r1-overflow-detection-for-signed-i16)
2. **Negation of MIN_VALUE returns Overflow** (stricter than C#) — see [research.md](research.md#r2-negation-of-min_value)
3. **MIN_VALUE % -1 returns Overflow** (matches C#) — see [research.md](research.md#r3-remainder-min_value--1)
4. **Conversions scoped to existing types** (SqlBoolean, SqlByte) — see [research.md](research.md#r4-available-conversions)
5. **Bitwise ops use native Rust operators** (no C# ushort dance) — see [research.md](research.md#r6-bitor-casting)

## Project Structure

### Documentation (this feature)

```text
specs/003-sql-int16/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── public-api.md    # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # MODIFY: add sql_int16 module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons + conversion)
├── sql_byte.rs          # NO CHANGES (dependency for conversion)
└── sql_int16.rs         # NEW: SqlInt16 implementation + inline tests
```

## Complexity Tracking

No violations — nothing to justify.
