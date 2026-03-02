# Implementation Plan: SqlSingle

**Branch**: `008-sql-single` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-sql-single/spec.md`

## Summary

Implement `SqlSingle` — a nullable 32-bit IEEE 754 floating-point type equivalent to C# `System.Data.SqlTypes.SqlSingle` / SQL Server `REAL`. Uses `Option<f32>` internally with NaN/Infinity rejection on construction and after every arithmetic operation. Follows the established `SqlDouble` pattern (its f64 sibling) for operator traits, SQL comparison methods, Display/FromStr, and standard traits. Includes widening `to_sql_double()` conversion.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A — in-memory stack-allocated type
**Testing**: `cargo test` (built-in Rust test framework)
**Target Platform**: All platforms supporting Rust stable
**Project Type**: Library
**Performance Goals**: Zero-allocation arithmetic; `Copy + Clone` for stack value semantics
**Constraints**: No `unsafe` code; no external dependencies
**Scale/Scope**: Single type (~700-900 LOC including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | Direct port of C# `SQLSingle.cs` — constructor rejects non-finite, arithmetic checks `IsInfinity`, division checks zero, NULL propagation |
| II. Idiomatic Rust | ✅ PASS | `Option<f32>`, operator traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`), `Result` returns, standard traits |
| III. Test-First Development | ✅ PASS | TDD mandatory per constitution; tests before implementation |
| IV. Comprehensive Type Coverage | ✅ PASS | SqlSingle is one of the required numeric types |
| V. Zero Unsafe Code | ✅ PASS | Pure safe Rust |
| VI. No External Dependencies | ✅ PASS | std only |
| VII. Versioning | ✅ PASS | Pre-1.0, additive change |

No gate violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/008-sql-single/
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
├── lib.rs               # Module registration: pub mod sql_single
├── error.rs             # Shared SqlTypeError enum (existing)
├── sql_boolean.rs       # SqlBoolean dependency (existing)
├── sql_byte.rs          # SqlByte dependency for conversions (existing)
├── sql_int16.rs         # SqlInt16 dependency for conversions (existing)
├── sql_int32.rs         # SqlInt32 dependency for conversions (existing)
├── sql_int64.rs         # SqlInt64 dependency for conversions (existing)
├── sql_money.rs         # SqlMoney dependency for conversions (existing)
├── sql_double.rs        # SqlDouble dependency for to_sql_double() (existing)
└── sql_single.rs        # NEW — SqlSingle implementation + inline tests
```

**Structure Decision**: Single file `src/sql_single.rs` with inline `#[cfg(test)] mod tests`, following the established pattern from SqlDouble, SqlMoney, SqlDateTime, and all other types. Registered in `src/lib.rs`.
