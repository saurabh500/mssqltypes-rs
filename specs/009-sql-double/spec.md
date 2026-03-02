# Feature Specification: SqlDouble

**Feature Branch**: `009-sql-double`  
**Created**: 2025-07-17  
**Status**: Draft  
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDouble` — a 64-bit IEEE 754 floating-point number (SQL Server `FLOAT`) with NULL support, NaN/Infinity rejection, checked arithmetic, and SQL three-valued comparison logic

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect values (Priority: P1)

A library consumer creates `SqlDouble` values from 64-bit floats and inspects them, including the SQL NULL sentinel. Construction MUST reject NaN and Infinity — these are not valid SQL FLOAT values. This is the foundational capability upon which all other operations depend.

**Why this priority**: Core value construction, NULL representation, and NaN/Infinity rejection are prerequisites for every other feature. Without safe construction, the type cannot guarantee its invariant.

**Independent Test**: Can be fully tested by constructing values with `new()`, constants, and calling `value()` / `is_null()`. Delivers the ability to represent SQL FLOAT values safely in Rust.

**Acceptance Scenarios**:

1. **Given** `SqlDouble::new(3.14159265358979)`, **When** `value()` called, **Then** returns `Ok(3.14159265358979)`
2. **Given** `SqlDouble::new(-2.718281828)`, **When** `value()` called, **Then** returns `Ok(-2.718281828)`
3. **Given** `SqlDouble::new(0.0)`, **When** `value()` called, **Then** returns `Ok(0.0)`
4. **Given** `SqlDouble::NULL`, **When** `is_null()` called, **Then** returns `true`
5. **Given** `SqlDouble::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`
6. **Given** `SqlDouble::ZERO`, **When** `value()` called, **Then** returns `Ok(0.0)`
7. **Given** `SqlDouble::MIN_VALUE`, **When** `value()` called, **Then** returns `Ok(f64::MIN)`
8. **Given** `SqlDouble::MAX_VALUE`, **When** `value()` called, **Then** returns `Ok(f64::MAX)`
9. **Given** attempt to create with `f64::NAN`, **Then** returns error (construction rejected)
10. **Given** attempt to create with `f64::INFINITY`, **Then** returns error (construction rejected)
11. **Given** attempt to create with `f64::NEG_INFINITY`, **Then** returns error (construction rejected)

---

### User Story 2 - Arithmetic with overflow and NaN/Infinity detection (Priority: P1)

A library consumer performs arithmetic on `SqlDouble` values. All four operations (add, subtract, multiply, divide) MUST check the result for NaN and Infinity, returning errors instead of producing invalid values. Division by zero returns a divide-by-zero error. NULL propagates through all arithmetic.

**Why this priority**: Checked arithmetic with NaN/Infinity rejection is the primary value proposition of `SqlDouble` over raw `f64` — it prevents silent data corruption from invalid floating-point results.

**Independent Test**: Can be fully tested by performing arithmetic operations and verifying results, overflow/infinity errors, divide-by-zero errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDouble(2.5) + SqlDouble(3.5)`, **Then** returns `Ok(SqlDouble(6.0))`
2. **Given** `SqlDouble(10.0) - SqlDouble(3.0)`, **Then** returns `Ok(SqlDouble(7.0))`
3. **Given** `SqlDouble(4.0) * SqlDouble(2.5)`, **Then** returns `Ok(SqlDouble(10.0))`
4. **Given** `SqlDouble(10.0) / SqlDouble(4.0)`, **Then** returns `Ok(SqlDouble(2.5))`
5. **Given** `SqlDouble(f64::MAX) + SqlDouble(f64::MAX)`, **Then** returns overflow error (result is Infinity)
6. **Given** `SqlDouble(-f64::MAX) - SqlDouble(f64::MAX)`, **Then** returns overflow error (result is -Infinity)
7. **Given** `SqlDouble(f64::MAX) * SqlDouble(2.0)`, **Then** returns overflow error (result is Infinity)
8. **Given** `SqlDouble(1.0) / SqlDouble(0.0)`, **Then** returns divide-by-zero error
9. **Given** `SqlDouble(0.0) / SqlDouble(0.0)`, **Then** returns divide-by-zero error (result would be NaN)
10. **Given** any arithmetic op with `SqlDouble::NULL` operand, **Then** returns `SqlDouble::NULL`

---

### User Story 3 - Negation (Priority: P1)

A library consumer negates `SqlDouble` values. Negation inverts the sign and propagates NULL.

**Why this priority**: Negation is a core unary arithmetic operation needed for completeness of the arithmetic story.

**Independent Test**: Can be fully tested by negating values and checking the result.

**Acceptance Scenarios**:

1. **Given** `-SqlDouble(5.0)`, **Then** returns `SqlDouble(-5.0)`
2. **Given** `-SqlDouble(-3.14)`, **Then** returns `SqlDouble(3.14)`
3. **Given** `-SqlDouble(0.0)`, **Then** returns `SqlDouble(-0.0)` (IEEE 754 negative zero)
4. **Given** `-SqlDouble::NULL`, **Then** returns `SqlDouble::NULL`

---

### User Story 4 - Comparison returning SqlBoolean (Priority: P2)

A library consumer compares `SqlDouble` values using SQL three-valued logic. Comparisons return `SqlBoolean` (not `bool`), and any comparison involving NULL returns `SqlBoolean::NULL`.

**Why this priority**: Comparisons are essential for conditional logic but depend on the type already being constructable.

**Independent Test**: Can be fully tested by comparing pairs of values and verifying the returned `SqlBoolean`.

**Acceptance Scenarios**:

1. **Given** `SqlDouble(1.0).sql_equals(&SqlDouble(1.0))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlDouble(1.0).sql_equals(&SqlDouble(2.0))`, **Then** returns `SqlBoolean::FALSE`
3. **Given** `SqlDouble(1.0).sql_less_than(&SqlDouble(2.0))`, **Then** returns `SqlBoolean::TRUE`
4. **Given** `SqlDouble(2.0).sql_greater_than(&SqlDouble(1.0))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlDouble(1.0).sql_less_than_or_equal(&SqlDouble(1.0))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlDouble(1.0).sql_greater_than_or_equal(&SqlDouble(1.0))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlDouble(1.0).sql_not_equals(&SqlDouble(2.0))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** any comparison with `SqlDouble::NULL` operand, **Then** returns `SqlBoolean::NULL`

---

### User Story 5 - Display and parsing (Priority: P2)

A library consumer converts `SqlDouble` to and from string representations. NULL displays as `"Null"`. Parsing invalid strings returns a parse error.

**Why this priority**: String conversion is needed for diagnostics, logging, and data interchange.

**Independent Test**: Can be fully tested by formatting values with `Display` and parsing strings with `FromStr`.

**Acceptance Scenarios**:

1. **Given** `SqlDouble::new(3.14159265358979)`, **When** formatted with `Display`, **Then** produces `"3.14159265358979"`
2. **Given** `SqlDouble::NULL`, **When** formatted with `Display`, **Then** produces `"Null"`
3. **Given** `SqlDouble::new(0.0)`, **When** formatted with `Display`, **Then** produces `"0"`
4. **Given** string `"3.14159265358979"`, **When** parsed as `SqlDouble`, **Then** returns `SqlDouble(3.14159265358979)`
5. **Given** string `"abc"`, **When** parsed as `SqlDouble`, **Then** returns parse error
6. **Given** string `"NaN"`, **When** parsed as `SqlDouble`, **Then** returns parse error (NaN not allowed)
7. **Given** string `"Infinity"`, **When** parsed as `SqlDouble`, **Then** returns parse error (Infinity not allowed)
8. **Given** string representing a value too large for `f64`, **When** parsed, **Then** returns overflow error

---

### User Story 6 - Conversions to and from other SqlTypes (Priority: P3)

A library consumer converts between `SqlDouble` and other SQL types. `SqlDouble` is the widest floating-point SQL type, so it accepts widening conversions from all integer types and from `SqlSingle`. Conversions from `SqlBoolean` follow C# semantics (TRUE=1.0, FALSE=0.0).

**Why this priority**: Cross-type conversions enable interoperability between SQL types but are only useful after core arithmetic and comparison are working.

**Independent Test**: Can be fully tested by converting values between types and verifying results, errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlByte(42)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(42.0)`
2. **Given** `SqlInt16(1000)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(1000.0)`
3. **Given** `SqlInt32(100_000)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(100000.0)`
4. **Given** `SqlInt64(1_000_000_000)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(1000000000.0)`
5. **Given** `SqlSingle(3.14)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(3.14...)` (widened)
6. **Given** `SqlMoney(42.5)`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(42.5)`
7. **Given** `SqlBoolean::TRUE`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(1.0)`
8. **Given** `SqlBoolean::FALSE`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble(0.0)`
9. **Given** `SqlBoolean::NULL`, **When** converted to `SqlDouble`, **Then** returns `SqlDouble::NULL`
10. **Given** `SqlDouble::NULL`, **When** converted to any type, **Then** returns NULL of target type
11. **Given** `SqlDouble(42.0)`, **When** converted to `SqlBoolean`, **Then** returns `SqlBoolean::TRUE`
12. **Given** `SqlDouble(0.0)`, **When** converted to `SqlBoolean`, **Then** returns `SqlBoolean::FALSE`
13. **Given** `SqlDouble(3.14)`, **When** narrowing conversion to `SqlSingle` requested, **Then** checks if value fits in `f32` range and returns result or overflow error

---

### Edge Cases

- NaN and Infinity MUST be rejected both on construction and after every arithmetic operation
- Negative zero (`-0.0`) is a valid IEEE 754 value and MUST be accepted; `-0.0 == 0.0` in comparisons (IEEE 754 semantics)
- Subnormal (denormalized) values near the limits of `f64` precision are valid and MUST be accepted
- Very small subnormal values (e.g., `f64::MIN_POSITIVE * 0.5`) MUST be handled correctly
- Division: `0.0 / 0.0` produces NaN in IEEE 754 — MUST return divide-by-zero error
- Division: `x / 0.0` where x ≠ 0 produces Infinity in IEEE 754 — MUST return divide-by-zero error
- Negation of `f64::MIN` and `f64::MAX` are valid (just flips sign, no overflow possible with floats)
- NULL propagates through all arithmetic, comparison, and conversion operations
- Widening from `SqlSingle` to `SqlDouble` is lossless (every `f32` value is exactly representable as `f64`)
- Narrowing from `SqlDouble` to `SqlSingle` may lose precision or overflow — must check range
- Widening integer-to-float conversion may lose precision for very large integers (e.g., `i64::MAX` as `f64` loses low-order bits) — this is acceptable and matches C# behavior
- `From<f64>` trait provides ergonomic construction but will panic on NaN/Infinity (use `try_from` for fallible construction)
- `PartialEq` / `Eq` — Rust-level equality (distinct from `sql_equals` which returns `SqlBoolean`). Two NULL values are equal for Rust `PartialEq`, but `sql_equals` returns `SqlBoolean::NULL`
- `PartialOrd` / `Ord` — NULL values sort before all non-NULL values (consistent with Rust `Option` convention)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlDouble` MUST be `Copy + Clone`, representing `f64` with NULL support via an internal `Option<f64>`
- **FR-002**: MUST reject NaN and Infinity on construction — `new()` MUST validate that the value is finite; a fallible constructor (`try_new()` or similar) MUST return `Err(SqlTypeError::Overflow)` for non-finite values
- **FR-003**: MUST implement checked `Add`, `Sub`, `Mul`, `Div` — all return `Result<SqlDouble, SqlTypeError>` and reject any result that is NaN or Infinity
- **FR-004**: MUST implement `Neg` — infallible for valid values; propagates NULL
- **FR-005**: Division by zero MUST return `Err(SqlTypeError::DivideByZero)` — checked before computing the result
- **FR-006**: MUST implement SQL comparison methods (`sql_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, `sql_not_equals`) returning `SqlBoolean`
- **FR-007**: MUST implement `Display` (NULL displays as `"Null"`) and `FromStr` (invalid input returns `ParseError`; NaN/Infinity strings rejected)
- **FR-008**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE` (f64::MIN), `MAX_VALUE` (f64::MAX)
- **FR-009**: MUST provide widening conversions from `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlSingle`, `SqlMoney`, and `SqlBoolean` (TRUE=1.0, FALSE=0.0, NULL=NULL)
- **FR-010**: MUST provide narrowing conversion to `SqlSingle` — checks if value fits in `f32` range; returns `Err(SqlTypeError::Overflow)` if not; NULL input returns NULL
- **FR-011**: MUST implement `From<f64>` (panics on NaN/Infinity) and a fallible conversion for safe construction
- **FR-012**: MUST implement `Hash`, `PartialEq`, `Eq` — two NULL values are equal for Rust equality; two non-NULL values compare by their `f64` value (using total-order bit comparison for Hash consistency)
- **FR-013**: MUST implement `PartialOrd`, `Ord` — NULL sorts before all non-NULL values
- **FR-014**: NULL propagation MUST apply to all arithmetic and comparison operations — any NULL operand produces a NULL result
- **FR-015**: MUST provide `to_sql_boolean()` — zero maps to FALSE, non-zero maps to TRUE, NULL maps to NULL
- **FR-016**: MUST provide `to_sql_single()` — narrowing conversion to `SqlSingle` with range check

### Key Entities

- **SqlDouble**: A nullable 64-bit IEEE 754 floating-point number. Internal representation: `Option<f64>` where `None` = SQL NULL, `Some(v)` = a finite value. Fixed-size, stack-allocated. Invariant: the contained `f64` is always finite (never NaN or Infinity).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: NaN and Infinity are never representable — all construction and arithmetic paths that would produce non-finite values are detected and returned as errors, with 100% of such paths tested
- **SC-002**: Code coverage ≥ 95% for the `SqlDouble` module
- **SC-003**: All four arithmetic operations have at least one positive test, one overflow/infinity error test, one divide-by-zero test (for division), and one NULL propagation test
- **SC-004**: `Display` and `FromStr` round-trip correctly for all non-NULL finite values
- **SC-005**: All widening conversions from integer types and `SqlSingle` tested with representative values and NULL propagation
- **SC-006**: All six comparison methods tested with equal, unequal, and NULL operand combinations
- **SC-007**: Narrowing conversion to `SqlSingle` tested at boundary values (values within and outside `f32` range)

## Assumptions

- Overflow detection for floating-point arithmetic uses `f64::is_finite()` after the operation — if the result is not finite, return `Err(SqlTypeError::Overflow)` (matches C# `double.IsInfinity()` checks)
- Division by zero is checked by testing if the divisor is `0.0` before computing, returning `Err(SqlTypeError::DivideByZero)` (matches C# explicit zero checks)
- Negative zero (`-0.0`) is accepted as valid — IEEE 754 treats `0.0 == -0.0`, and this matches C# behavior
- Subnormal values are accepted — no additional validation beyond `is_finite()`
- `Hash` implementation uses `f64::to_bits()` for bit-exact hashing, with special handling so that `0.0` and `-0.0` hash identically (they are equal under IEEE 754)
- Widening from `SqlSingle` to `SqlDouble` is lossless — every finite `f32` is exactly representable as `f64`
- Widening conversions from integer types may lose precision for very large values (e.g., `i64::MAX` → `f64` loses low bits) — this is expected and matches C# behavior
- Narrowing to `SqlSingle`: checks if `f64` value fits in `f32` range (`f32::MIN..=f32::MAX`); subnormal results from narrowing are acceptable
- Widening conversions to `SqlDecimal` are deferred until that type is implemented
- `PartialOrd` / `Ord` for Rust-level ordering: NULL values sort before all non-NULL values (consistent with Rust convention for `Option`)
