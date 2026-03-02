# Research: SqlDouble

**Feature**: 009-sql-double
**Date**: 2026-03-02
**Status**: Complete

## R1: NaN/Infinity Rejection Strategy

**Decision**: Validate with `f64::is_finite()` on construction and after every arithmetic operation.

**Rationale**: Direct match of C# `double.IsFinite(value)` in the constructor and `double.IsInfinity(result)` after arithmetic. The C# reference (SQLDouble.cs line 39) throws `OverflowException` for non-finite values on construction, and checks `IsInfinity` (not `IsNaN`) after arithmetic — since the inputs are always finite, arithmetic can only produce Infinity (never NaN) except for `0.0 / 0.0` which is caught by the explicit zero-divisor check.

**Alternatives considered**:
- Validate only on construction, trust arithmetic — rejected because `f64::MAX + f64::MAX = Infinity`
- Use `is_nan()` and `is_infinite()` separately — rejected because `is_finite()` catches both and is simpler

## R2: Division by Zero Handling

**Decision**: Check if divisor is `0.0` before computing division. Return `Err(SqlTypeError::DivideByZero)`.

**Rationale**: C# reference (SQLDouble.cs line 151) explicitly checks `y.m_value == 0.0` before computing. This catches both `x/0.0` (which produces ±Infinity) and `0.0/0.0` (which produces NaN). By checking first, we avoid needing to distinguish NaN from Infinity in the result.

**Alternatives considered**:
- Compute first, then check result — rejected because `0.0/0.0` produces NaN which is a different error class than overflow-to-infinity
- Use `DivideByZero` for all non-finite division results — rejected because `MAX * 2.0 = Infinity` is overflow, not divide-by-zero

## R3: Hash Implementation for f64

**Decision**: Use `f64::to_bits()` for hashing, with normalization of `-0.0` to `0.0` before converting to bits.

**Rationale**: `f64` does not implement `Hash` in Rust because of NaN != NaN. Since SqlDouble guarantees no NaN values exist, we can safely implement `Hash`. The `-0.0` normalization is needed because IEEE 754 says `0.0 == -0.0` but they have different bit representations.

**Alternatives considered**:
- Hash the raw bits without normalization — rejected because it would violate the contract that `a == b` implies `hash(a) == hash(b)` for `-0.0` and `0.0`
- Use `OrderedFloat` crate — rejected because of no-external-dependencies constraint

## R4: Eq Implementation for f64

**Decision**: Implement `PartialEq` and `Eq` manually. Two non-NULL values compare by `f64` equality. Two NULLs are equal (Rust structural equality).

**Rationale**: `f64` normally cannot implement `Eq` because NaN != NaN. Since SqlDouble's invariant guarantees no NaN ever exists in a value, this is safe. This matches C# `SqlDouble.Equals()` behavior (SQLDouble.cs line 391).

**Alternatives considered**:
- Derive PartialEq — could work but `Option<f64>` derives equality that works correctly since NaN is excluded
- Only implement PartialEq without Eq — rejected because Ord requires Eq, and we need Ord for consistent ordering

## R5: Negation Behavior

**Decision**: Negation is infallible — returns `SqlDouble` directly (not `Result`). Implements `std::ops::Neg`.

**Rationale**: C# `operator -` for SqlDouble (SQLDouble.cs line 98-100) does not throw — it simply negates the value. For finite `f64`, negation always produces a finite result (just flips the sign bit). `-f64::MAX = f64::MIN` and vice versa. `-0.0 = -0.0` per IEEE 754.

**Alternatives considered**:
- Return `Result` — rejected because negation of a finite value is always finite; no error is possible

## R6: Arithmetic Operator Trait Pattern

**Decision**: Use the `SqlMoney` pattern — implement `Add`, `Sub`, `Mul`, `Div` for `SqlDouble` with `Output = Result<SqlDouble, SqlTypeError>`. Also implement for `&SqlDouble` references.

**Rationale**: This is the established pattern in the codebase (SqlMoney uses the same approach). The `Result` output allows callers to use `?` operator for error propagation.

**Alternatives considered**:
- `checked_*` methods only (no operator traits) — rejected because operator traits are more ergonomic and match existing codebase pattern
- Operator traits with panic — rejected because constitution says no panics in library code

## R7: SqlSingle Dependency

**Decision**: Defer `from_sql_single()` and `to_sql_single()` conversions. SqlSingle (008) is not yet implemented. These methods will be added when SqlSingle is implemented.

**Rationale**: SqlSingle does not exist in the codebase yet. Adding conversion methods for a non-existent type would not compile. The spec lists conversions from SqlSingle as FR-009 and narrowing to SqlSingle as FR-010/FR-016, but these require SqlSingle to exist first.

**Alternatives considered**:
- Implement SqlSingle first — rejected because 009-sql-double is the current feature branch
- Add stub types — rejected because it adds complexity with no benefit
- Add conversions behind feature flag — rejected because overcomplicated for a temporary gap

## R8: From<f64> vs TryFrom<f64>

**Decision**: Implement `From<f64>` that panics on NaN/Infinity (matching C# implicit operator behavior), and provide `new()` as the fallible constructor returning `Result`.

**Rationale**: C# `implicit operator SqlDouble(double x)` calls the constructor which throws on non-finite values. Rust's `From` trait should match this — panic on invalid input. The `new()` method provides the explicit fallible alternative. This matches FR-011.

**Alternatives considered**:
- Only `TryFrom<f64>` — rejected because `From` is more ergonomic for the common case (valid values)
- Both `From<f64>` and `TryFrom<f64>` — could work but `new()` already serves as the fallible path

## R9: SqlMoney Conversion

**Decision**: Implement `from_sql_money()` — converts `SqlMoney` to `SqlDouble` by extracting the inner `i64` value and dividing by 10,000.0 to get the `f64` representation.

**Rationale**: C# has implicit conversion from SqlMoney to SqlDouble via `SqlMoney.ToDouble()`. SqlMoney stores values as `i64 * 10,000`, so conversion is `value as f64 / 10_000.0`. This is always finite since SqlMoney's range (±922 trillion) fits easily within f64 range (±1.7e308).

**Alternatives considered**:
- Skip SqlMoney conversion — rejected because C# supports it and FR-009 requires it

## R10: Display Format

**Decision**: Use Rust's default `f64` Display formatting (which matches `double.ToString()` in C# for most values).

**Rationale**: C# `SQLDouble.ToString()` uses `m_value.ToString((IFormatProvider)null!)` which produces culture-invariant output. Rust's `Display` for `f64` similarly produces a decimal representation. NULL displays as `"Null"`.

**Alternatives considered**:
- Custom formatting with specific decimal places — rejected because C# uses default formatting
- `{:?}` (Debug) format — rejected because that's for debugging, not display
