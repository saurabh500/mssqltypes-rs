# Implementation Plan: SqlMoney

**Branch**: `007-sql-money` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/007-sql-money/spec.md`

## Summary

Implement `SqlMoney` — a Rust equivalent of C# `System.Data.SqlTypes.SqlMoney`. Fixed-point currency type with 4 decimal places, stored internally as `Option<i64>` where the `i64` value represents the monetary amount multiplied by 10,000. Range: −922,337,203,685,477.5808 to 922,337,203,685,477.5807. Arithmetic returns `Result<SqlMoney, SqlTypeError>`: add/subtract use checked i64 directly; multiply/divide use i128 intermediate for precision preservation. Comparisons return `SqlBoolean` with three-valued NULL logic. Display format: `"#0.00##"` (minimum 2, maximum 4 decimal places). Follows patterns established by `SqlInt64` and reference C# implementation (636 lines).

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)
**Target Platform**: All platforms (pure Rust)
**Project Type**: Library
**Performance Goals**: Stack-allocated (`Copy`), zero-allocation arithmetic/comparison
**Constraints**: No `unsafe`, no external deps, no panics
**Scale/Scope**: Single module (~900–1100 lines including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Pre-Design | Post-Design | Notes |
|---|-----------|-----------|-------------|-------|
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLMoney.cs (636 lines). Matches all arithmetic, comparison, conversion, display behavior. Negation: C# checks `x._value == s_minLong` (which is `i64::MIN / 10000`, NOT `i64::MIN`) — a bug that misses the true overflow case. Rust uses `i64::checked_neg()` for correct detection. Mul/Div: C# routes through `decimal`; Rust uses i128 intermediate (equivalent precision, idiomatic). |
| II | Idiomatic Rust | PASS | PASS | `Option<i64>` repr, `checked_*` for add/sub overflow, i128 intermediate for mul/div, standard traits, `Result` for fallible ops |
| III | Test-First Development | PASS | PASS | Spec defines 60 acceptance scenarios across 6 user stories + 8 edge cases |
| IV | Comprehensive Coverage | PASS | PASS | SqlMoney is a required numeric type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All safe integer arithmetic via checked ops and i128 widening |
| VI | No External Deps | PASS | PASS | std only |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **Add/Sub via checked i64 arithmetic** (exact, no rounding) — see [research.md](research.md#r1-addsubtract-strategy)
2. **Mul/Div via i128 intermediate** (not C# decimal route) — see [research.md](research.md#r2-multiplydivide-strategy)
3. **Negation uses `checked_neg()`** (fixes C# bug) — see [research.md](research.md#r3-negation-overflow-detection)
4. **Display format `"#0.00##"`** (min 2, max 4 decimal places) — see [research.md](research.md#r4-display-format)
5. **`to_i64()` rounding: round-half-away-from-zero** — see [research.md](research.md#r5-to_i64-rounding-semantics)
6. **`from_f64()` rounding to 4dp + NaN/Inf rejection** — see [research.md](research.md#r6-from_f64-construction)
7. **Conversions scoped to existing types** — see [research.md](research.md#r7-conversion-scope)
8. **C# named static methods not implemented** — see [research.md](research.md#r8-c-named-static-methods)

## Project Structure

### Documentation (this feature)

```text
specs/007-sql-money/
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
├── lib.rs               # MODIFY: add sql_money module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons + conversion from)
├── sql_byte.rs          # NO CHANGES (dependency for conversion from)
├── sql_int16.rs         # NO CHANGES (dependency for conversion from)
├── sql_int32.rs         # NO CHANGES (dependency for conversion from)
├── sql_int64.rs         # NO CHANGES (dependency for conversion from)
├── sql_decimal.rs       # NO CHANGES (dependency for to_sql_decimal conversion)
└── sql_money.rs         # NEW: SqlMoney implementation + inline tests
```

## Complexity Tracking

No violations — nothing to justify.
