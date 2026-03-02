# Implementation Plan: SqlDouble

**Branch**: `009-sql-double` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-sql-double/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement `SqlDouble` — a nullable 64-bit IEEE 754 floating-point type equivalent to C# `System.Data.SqlTypes.SqlDouble` / SQL Server `FLOAT`. Uses `Option<f64>` internally with NaN/Infinity rejection on construction and after every arithmetic operation. Follows the established `SqlMoney` pattern for operator traits, SQL comparison methods, Display/FromStr, and standard traits.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A — in-memory stack-allocated type
**Testing**: `cargo test` (built-in Rust test framework)
**Target Platform**: All platforms supporting Rust stable
**Project Type**: Library
**Performance Goals**: Zero-allocation arithmetic; `Copy + Clone` for stack value semantics
**Constraints**: No `unsafe` code; no external dependencies
**Scale/Scope**: Single type (~600-800 LOC including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | Direct port of C# `SQLDouble.cs` — constructor rejects non-finite, arithmetic checks `IsInfinity`, division checks zero, NULL propagation |
| II. Idiomatic Rust | ✅ PASS | `Option<f64>`, operator traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`), `Result` returns, standard traits |
| III. Test-First Development | ✅ PASS | TDD mandatory per constitution; tests before implementation |
| IV. Comprehensive Type Coverage | ✅ PASS | SqlDouble is one of the required numeric types |
| V. Zero Unsafe Code | ✅ PASS | Pure safe Rust |
| VI. No External Dependencies | ✅ PASS | std only |
| VII. Versioning | ✅ PASS | Pre-1.0, additive change |

No gate violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/009-sql-double/
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
├── lib.rs               # Module registration: pub mod sql_double
├── error.rs             # Shared SqlTypeError enum (existing)
├── sql_boolean.rs       # SqlBoolean dependency (existing)
├── sql_byte.rs          # SqlByte dependency for conversions (existing)
├── sql_int16.rs         # SqlInt16 dependency for conversions (existing)
├── sql_int32.rs         # SqlInt32 dependency for conversions (existing)
├── sql_int64.rs         # SqlInt64 dependency for conversions (existing)
├── sql_money.rs         # SqlMoney dependency for conversions (existing)
└── sql_double.rs        # NEW — SqlDouble implementation + inline tests
```

**Structure Decision**: Single file `src/sql_double.rs` with inline `#[cfg(test)] mod tests`, following the established pattern from SqlMoney, SqlDateTime, and all other types. Registered in `src/lib.rs`.
