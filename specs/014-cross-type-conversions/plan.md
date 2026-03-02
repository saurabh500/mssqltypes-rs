# Implementation Plan: Cross-Type Conversions

**Branch**: `014-cross-type-conversions` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/014-cross-type-conversions/spec.md`

## Summary

Complete the remaining 43 cross-type conversion methods across all 13 SQL types. All types are already implemented; this feature adds missing conversion bridges that were deferred until target types existed. Conversions fall into 3 categories: (1) widening — infallible `From` impls; (2) narrowing — `to_sql_*()` methods returning `Result`; (3) string hub — `to_sql_string()` on every type and `SqlString::to_sql_*()` parsing methods. No new files, types, or dependencies — only methods and trait impls added to existing source files.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A — in-memory types
**Testing**: `cargo test` (built-in Rust test framework)
**Target Platform**: All platforms supporting Rust stable
**Project Type**: Library
**Performance Goals**: Zero-cost abstraction; no heap allocation for fixed-size conversions
**Constraints**: No `unsafe` code; no external dependencies; all conversions propagate NULL
**Scale/Scope**: ~43 new methods/impls spread across 11 existing source files, ~400-600 LOC implementation + ~800-1200 LOC tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | All conversions mirror C# `System.Data.SqlTypes` operator/method semantics: implicit→`From`, explicit→`Result`, NULL propagation, overflow detection. One known C# quirk: `SqlDecimal(f64::NAN)` silently becomes zero in C# — Rust version will reject NaN explicitly (safer, documented deviation). |
| II. Idiomatic Rust | ✅ PASS | `From` trait for widening, `Result<T, SqlTypeError>` for narrowing, existing `Display`/`FromStr` traits leveraged for string conversions |
| III. Test-First Development | ✅ PASS | TDD mandatory; minimum 3 tests per conversion (normal, NULL, edge case) |
| IV. Comprehensive Type Coverage | ✅ PASS | Completes the conversion matrix for all 13 types |
| V. Zero Unsafe Code | ✅ PASS | Pure safe Rust |
| VI. No External Dependencies | ✅ PASS | std only |
| VII. Versioning | ✅ PASS | Pre-1.0, additive change — new methods only, no breaking changes |

No gate violations. No complexity tracking needed.

### Post-Design Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | C# SqlString→type conversions delegate to target type's Parse; Rust equivalently uses `FromStr`. SqlBoolean string display uses "True"/"False" (C# fidelity). NaN→SqlDecimal deviation documented. |
| II. Idiomatic Rust | ✅ PASS | Circular dependency between type files resolved by `use crate::*` imports — all types are siblings in the same crate. `SqlString` methods return `Result` even for NULL inputs (returns `Ok(T::NULL)`). |
| All others | ✅ PASS | No changes from pre-design check |

## Project Structure

### Documentation (this feature)

```text
specs/014-cross-type-conversions/
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
├── lib.rs               # No changes needed — all modules already registered
├── error.rs             # No changes needed — existing SqlTypeError variants sufficient
├── sql_boolean.rs       # Add: to_sql_string()
├── sql_byte.rs          # Add: to_sql_string()
├── sql_int16.rs         # Add: to_sql_string()
├── sql_int32.rs         # Add: to_sql_boolean(), to_sql_string(), From<SqlByte>, From<SqlInt16>
├── sql_int64.rs         # Add: to_sql_boolean(), to_sql_string(), From<SqlByte>, From<SqlInt16>, From<SqlInt32>
├── sql_single.rs        # Add: to_sql_string()
├── sql_double.rs        # Add: from_sql_single(), to_sql_single(), to_sql_string()
├── sql_decimal.rs       # Add: to_sql_single(), to_sql_double(), to_sql_money(), to_sql_string(), From<SqlSingle>, From<SqlDouble>, From<SqlMoney>
├── sql_money.rs         # Add: from_sql_single(), from_sql_double(), to_sql_single(), to_sql_double(), to_sql_string()
├── sql_string.rs        # Add: to_sql_boolean(), to_sql_byte(), to_sql_int16(), to_sql_int32(), to_sql_int64(), to_sql_single(), to_sql_double(), to_sql_decimal(), to_sql_money(), to_sql_date_time(), to_sql_guid()
├── sql_datetime.rs      # Add: to_sql_string(), from_sql_string()
└── sql_guid.rs          # Add: to_sql_string()
```

**Structure Decision**: No new files. All 43 conversion methods are added to existing source files following the established one-type-per-file pattern. Each file gains `use crate::sql_string::SqlString;` (or equivalent) imports as needed. Tests are added inline in each file's existing `#[cfg(test)] mod tests` block.
