# Implementation Plan: SqlBoolean

**Branch**: `feature/sql-boolean` | **Date**: 2026-03-01 | **Spec**: [sql-boolean.md](../../specs/sql-boolean.md)
**Input**: Feature specification from `.specify/specs/sql-boolean.md`

## Summary

Implement `SqlBoolean`, a three-state boolean type representing SQL Server's `BIT` type with full NULL support. The type uses a `u8` internal representation (`0=Null, 1=False, 2=True`) matching the C# `System.Data.SqlTypes.SqlBoolean` layout. It implements three-valued logic (AND, OR, XOR, NOT) with SQL-standard NULL propagation and short-circuit semantics (`FALSE & NULL = FALSE`, `TRUE | NULL = TRUE`).

This is a foundational type — every other SqlType's comparison operator returns `SqlBoolean`, making it the first type that must be implemented.

## Technical Context

**Language/Version**: Rust (stable, edition 2024)
**Primary Dependencies**: None (std only)
**Storage**: N/A (in-memory stack type)
**Testing**: `cargo test` with `cargo tarpaulin` or `llvm-cov` for coverage
**Target Platform**: All platforms supported by Rust std
**Project Type**: Library
**Performance Goals**: Zero-allocation operations, `Copy` semantics
**Constraints**: No `unsafe` code, no external dependencies
**Scale/Scope**: Single type, ~300-400 lines of implementation + tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Behavioral Fidelity | ✅ PASS | C# reference reviewed (SQLBoolean.cs, 532 lines). Internal `u8` representation matches: `0=Null, 1=False, 2=True`. |
| II. Idiomatic Rust | ✅ PASS | Will implement `Display`, `FromStr`, `Clone`, `Copy`, `Debug`, `Hash`, `PartialEq`, `PartialOrd`, `BitAnd`, `BitOr`, `BitXor`, `Not`. Uses `Result` for fallible ops. |
| III. Test-First Development | ✅ PLAN | TDD workflow: tests written before implementation in each task. |
| IV. Type Coverage | ✅ PASS | SqlBoolean is listed as required type. |
| V. Zero Unsafe Code | ✅ PASS | No `unsafe` needed. |
| VI. No External Dependencies | ✅ PASS | Uses only `std`. |
| VII. Versioning | ✅ PASS | Pre-1.0, no breaking change concerns. |

## Project Structure

### Documentation (this feature)

```text
.specify/specs/sql-boolean/
├── plan.md              # This file
└── research.md          # C# reference analysis
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Module declarations, re-exports, SqlTypeError enum
├── error.rs             # SqlTypeError enum (shared across all types)
└── sql_boolean.rs       # SqlBoolean struct + impl + tests
```

**Structure Decision**: Single flat module per type under `src/`. Each type gets its own file. The shared `SqlTypeError` enum lives in `error.rs` and is re-exported from `lib.rs`. Tests are inline (`#[cfg(test)] mod tests`) within each type's file, following Rust convention.

## C# Reference Analysis

### Internal Representation (from SQLBoolean.cs)

```csharp
private byte m_value; // 0=Null, 1=False, 2=True
```

### Key Behaviors Observed

1. **Constructor**: `SqlBoolean(bool)` maps `true→2, false→1`. `SqlBoolean(int)` maps `0→False, nonzero→True`.
2. **NOT (`!`)**: True→False, False→True, Null→Null
3. **Bitwise complement (`~`)**: Same as NOT
4. **AND (`&`)**: Short-circuits — `FALSE & anything = FALSE`. Only returns True if both True. Otherwise Null.
5. **OR (`|`)**: Short-circuits — `TRUE | anything = TRUE`. Only returns False if both False. Otherwise Null.
6. **XOR (`^`)**: If either is Null, returns Null. Otherwise compares `m_value != y.m_value`.
7. **Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`)**: Null if either operand is Null. Otherwise compares `m_value` directly (False=1 < True=2).
8. **Parse**: Supports numeric strings (any non-zero int → True, 0 → False) and boolean strings ("true"/"false", case-insensitive).
9. **ByteValue**: Returns 0 for False, 1 for True, throws on Null.
10. **CompareTo**: Null < any non-null. Both nulls are equal. Otherwise compares ByteValue.
11. **Equals/GetHashCode**: Two Nulls are equal for `Equals`. Hash is 0 for Null, otherwise `Value.GetHashCode()`.
12. **Constants**: `True`, `False`, `Null`, `Zero` (=False), `One` (=True)
13. **`operator true`/`operator false`**: Enable `if (sqlBool)` syntax. In Rust, no direct equivalent; users call `.is_true()`.

### Type Conversions (from C#)

| Direction | Types | Behavior |
|-----------|-------|----------|
| From numeric → SqlBoolean | SqlByte, SqlInt16, SqlInt32, SqlInt64, SqlSingle, SqlDouble, SqlMoney, SqlDecimal | Null→Null, 0→False, non-zero→True |
| From SqlString → SqlBoolean | SqlString | Null→Null, otherwise parse string |
| SqlBoolean → numeric | SqlByte, SqlInt16, SqlInt32, SqlInt64, SqlSingle, SqlDouble, SqlMoney, SqlDecimal | Null→Null, True→1, False→0 |
| SqlBoolean → SqlString | SqlString | Null→Null, True→"True", False→"False" |

**Note**: Type conversions TO/FROM other SqlTypes will be deferred until those types are implemented. The initial SqlBoolean implementation will include only `From<bool>`, `value() -> Result<bool>`, `FromStr`, and `Display`.

## Implementation Phases

### Phase 0: Shared Infrastructure

Create the shared `SqlTypeError` enum and set up the module structure.

**Files**: `src/error.rs`, `src/lib.rs`

| Step | Description |
|------|-------------|
| 0.1 | Create `src/error.rs` with `SqlTypeError` enum (variants: `NullValue`, `Overflow`, `DivideByZero`, `ParseError`, `OutOfRange`) implementing `Display`, `Debug`, `Clone`, `PartialEq`, `std::error::Error` |
| 0.2 | Update `src/lib.rs` to declare modules (`mod error; mod sql_boolean;`) and re-export public types |

### Phase 1: Core Type (User Story 1 — P1)

Create `SqlBoolean` struct with construction, inspection, and value access.

**File**: `src/sql_boolean.rs`

| Step | Description | Tests |
|------|-------------|-------|
| 1.1 | Define `SqlBoolean` struct with `m_value: u8` field. Add constants: `NULL`, `TRUE`, `FALSE`, `ZERO`, `ONE`. | `test_constants_identity` |
| 1.2 | Implement `new(bool)`, `from_int(i32)`, `is_null()`, `is_true()`, `is_false()`. | `test_new_true`, `test_new_false`, `test_from_int_zero`, `test_from_int_nonzero`, `test_is_null` |
| 1.3 | Implement `value() -> Result<bool, SqlTypeError>` and `byte_value() -> Result<u8, SqlTypeError>`. | `test_value_true`, `test_value_false`, `test_value_null_error`, `test_byte_value` |
| 1.4 | Implement `From<bool> for SqlBoolean` (implicit) and derive/implement `Copy`, `Clone`, `Debug`. | `test_from_bool`, `test_copy_semantics` |

### Phase 2: Three-Valued Logic (User Story 2 — P1)

Implement logical operators with SQL NULL propagation and short-circuit rules.

**File**: `src/sql_boolean.rs`

| Step | Description | Tests |
|------|-------------|-------|
| 2.1 | Implement `Not` trait. | `test_not_true`, `test_not_false`, `test_not_null` |
| 2.2 | Implement `BitAnd` trait with FALSE short-circuit. | `test_and_truth_table` (all 9 combinations) |
| 2.3 | Implement `BitOr` trait with TRUE short-circuit. | `test_or_truth_table` (all 9 combinations) |
| 2.4 | Implement `BitXor` trait. | `test_xor_truth_table` (all 9 combinations) |

### Phase 3: Comparisons (User Story 3 — P2)

Implement SQL comparison methods returning `SqlBoolean`.

**File**: `src/sql_boolean.rs`

| Step | Description | Tests |
|------|-------------|-------|
| 3.1 | Implement `sql_equals(&self, &SqlBoolean) -> SqlBoolean` and `sql_not_equals`. | `test_equals_both_true`, `test_equals_with_null`, `test_not_equals` |
| 3.2 | Implement `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`. | `test_less_than_false_lt_true`, `test_ordering_with_null` |
| 3.3 | Implement `PartialEq` (Rust equality — two Nulls are equal per C# `Equals`). | `test_partialeq_null_null`, `test_partialeq_true_true` |
| 3.4 | Implement `Hash` (0 for Null, delegate to `bool` otherwise). | `test_hash_consistency` |
| 3.5 | Implement `Ord`/`PartialOrd` via `CompareTo` semantics (Null < non-null). | `test_ord_null_less_than_false`, `test_ord_false_less_than_true` |

### Phase 4: Display & Parsing (User Story 4 — P2)

Implement string conversion.

**File**: `src/sql_boolean.rs`

| Step | Description | Tests |
|------|-------------|-------|
| 4.1 | Implement `Display`: "True", "False", "Null". | `test_display_true`, `test_display_false`, `test_display_null` |
| 4.2 | Implement `FromStr`: parse "true"/"false" (case-insensitive), "1"/"0", numeric strings. Return `ParseError` on invalid input. | `test_parse_true`, `test_parse_false`, `test_parse_one`, `test_parse_zero`, `test_parse_negative`, `test_parse_invalid` |

### Phase 5: Polish & Validation

| Step | Description |
|------|-------------|
| 5.1 | Run `cargo fmt` and `cargo clippy -- -D warnings`. Fix any issues. |
| 5.2 | Run `cargo test` — all tests must pass. |
| 5.3 | Run coverage tool — verify ≥95% coverage for `sql_boolean.rs`. |
| 5.4 | Review public API surface — ensure only spec'd methods are public. |

## Complexity Tracking

No constitution violations. No complexity exceptions needed.

## Deferred Work

- **Type conversions** to/from `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlSingle`, `SqlDouble`, `SqlMoney`, `SqlDecimal`, `SqlString`: Implemented when those types are built.
- **Serde support**: Behind `serde` feature flag, implemented across all types together.
