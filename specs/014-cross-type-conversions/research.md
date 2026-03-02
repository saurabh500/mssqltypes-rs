# Research: Cross-Type Conversions

**Feature**: 014-cross-type-conversions
**Date**: 2026-03-02

## R1: Conversion Categorization (Widening vs Narrowing)

**Decision**: Use `From` trait for widening (infallible) conversions and `to_sql_*() -> Result<T, SqlTypeError>` for narrowing (fallible) conversions.

**Rationale**: This mirrors the C# pattern of `implicit operator` (widening) vs `explicit operator` (narrowing), expressed idiomatically in Rust. `From` trait enables `.into()` ergonomics for lossless conversions. `Result` return types for narrowing conversions align with the constitution's requirement to never panic.

**Alternatives considered**:
- `TryFrom` trait for narrowing: Rejected because `TryFrom` doesn't allow NULL propagation as cleanly (it maps `None→None` but the error type must be fixed). Named methods like `to_sql_int32()` are more discoverable and match the C# `ToSqlInt32()` naming.
- Unified `to_sql_*()` for everything (including widening): Rejected because `From` trait interop provides stronger type-system guarantees and is idiomatic Rust.

## R2: SqlString Conversion Strategy

**Decision**: `to_sql_string()` on each type delegates to its `Display` implementation. `SqlString::to_sql_*()` parsing methods delegate to the target type's `FromStr` implementation.

**Rationale**: All 12 non-SqlBinary types already implement `Display` and `FromStr`. This avoids duplicating formatting/parsing logic and ensures consistency between `format!("{}", value)` and `to_sql_string()`.

**Alternatives considered**:
- Separate formatting logic in conversion methods: Rejected because it would duplicate `Display` implementations and risk inconsistency.
- `From<SqlString>` trait impls: Rejected because string parsing is fallible and `From` must be infallible.

**Special case — SqlBinary**: `SqlBinary` does NOT implement `FromStr`. `SqlString::to_sql_binary()` is explicitly out of scope (marked N/A in the conversion matrix). No `SqlBinary::to_sql_string()` is needed beyond what `Display` already provides.

## R3: SqlBoolean Display Format for to_sql_string()

**Decision**: Use "True" / "False" (capitalized) to match C# `SqlBoolean.ToString()` behavior.

**Rationale**: C# `SqlBoolean.ToString()` returns "True" or "False" (not "true"/"false"). The existing `Display` impl for `SqlBoolean` already outputs "True"/"False", so `to_sql_string()` will use that directly.

**Alternatives considered**:
- Lowercase "true"/"false": Rejected because it deviates from C# behavior without justification.

## R4: NaN Handling in Float → SqlDecimal

**Decision**: Reject NaN with `Err(SqlTypeError::OutOfRange)` rather than silently converting to zero.

**Rationale**: C# `SqlDecimal(double.NaN)` silently produces zero due to IEEE comparison quirks in the constructor. This is widely considered a bug. Rust's explicit rejection is safer and more consistent with the library's philosophy of never silently losing data.

**Alternatives considered**:
- Replicate C# behavior (NaN → zero): Rejected because it would silently corrupt data. Constitution Principle I allows documented deviations when the C# behavior is buggy.
- Return `Err(SqlTypeError::Overflow)`: Rejected because NaN is not an overflow — `OutOfRange` is semantically more correct.

## R5: SqlDouble → SqlSingle Overflow Detection

**Decision**: Cast `f64` to `f32`, then check if the result is infinite (when the input was finite). If so, return `Err(SqlTypeError::Overflow)`.

**Rationale**: This matches C# behavior where `checked((float)doubleValue)` throws `OverflowException` when the result is infinity. Rust's `as f32` cast saturates to infinity for out-of-range values, making the post-cast infinity check sufficient.

**Alternatives considered**:
- Pre-check against `f32::MAX`/`f32::MIN` before casting: More complex and fragile for denormalized values. Post-cast check is simpler and covers all edge cases.

## R6: SqlMoney ↔ Float Conversion Strategy

**Decision**: `SqlMoney::from_sql_single()` and `from_sql_double()` convert the float to the internal `i64 × 10,000` representation with range checking. `to_sql_single()` and `to_sql_double()` convert the internal `i64` to float by dividing by 10,000.

**Rationale**: C# `SqlMoney(double)` goes through `decimal` as an intermediary, but Rust has no native `decimal` type. Direct conversion via `(f64 * 10_000.0).round() as i64` is equivalent for the SqlMoney range and avoids external dependencies.

**Alternatives considered**:
- Route through `SqlDecimal` as intermediary (matching C# chain): Rejected because it adds unnecessary complexity and allocation for a simple range check.
- Use `f64::to_bits()` for exact conversion: Overly complex for this use case.

## R7: SqlString → SqlMoney Parsing

**Decision**: Use `FromStr` implementation, which parses decimal notation. Do NOT support currency symbols or locale-specific formatting.

**Rationale**: C# uses `NumberStyles.Currency` with culture-sensitive parsing, but the library has no locale support (Constitution Principle VI — no external dependencies). The existing `FromStr` impl for `SqlMoney` already handles decimal notation like "100.50".

**Alternatives considered**:
- Strip common currency symbols ($, €, £) before parsing: Rejected because it's partial locale support and creates false expectations. Better to be explicit about what's supported.

## R8: Circular Import Resolution

**Decision**: No special handling needed. All types are sibling modules in the same crate. Each file uses `use crate::sql_*::Sql*;` imports as needed.

**Rationale**: Rust's module system allows any module to import any other module in the same crate. Unlike C# where each class is independently compiled, Rust compiles the entire crate together, so circular references between modules are handled naturally.

**Alternatives considered**:
- Centralized conversion module (`src/conversions.rs`): Rejected because it breaks the one-type-per-file pattern and makes it harder to find conversions. C# defines conversions on either the source or target type — Rust should follow the same proximity principle.

## R9: Where to Place Each Conversion

**Decision**: Follow C# placement conventions:
- `From<Source> for Target` → placed in the **target** type's file (matches C# implicit operator placement on target)
- `to_sql_*()` methods → placed on the **source** type (matches C# `ToSql*()` method placement)
- `from_sql_*()` static methods → placed on the **target** type (matches C# explicit operator placement on target)
- `SqlString::to_sql_*()` parsing methods → placed on `SqlString` (matches C# `SqlString.ToSql*()`)

**Rationale**: Mirrors C# conventions and matches existing patterns in the codebase (e.g., `From<SqlBoolean> for SqlInt32` is in `sql_int32.rs`, `SqlSingle::from_sql_double()` is in `sql_single.rs`).
