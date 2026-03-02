# Feature Specification: SqlSingle

**Feature Branch**: `008-sql-single`  
**Created**: 2025-07-17  
**Status**: Draft  
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlSingle` — a 32-bit IEEE 754 floating-point number (SQL Server `REAL`) with NULL support, NaN/Infinity rejection, checked arithmetic, and SQL three-valued comparison logic

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect values (Priority: P1)

A library consumer creates `SqlSingle` values from 32-bit floats and inspects them, including the SQL NULL sentinel. Construction MUST reject NaN and Infinity — these are not valid SQL REAL values. This is the foundational capability upon which all other operations depend.

**Why this priority**: Core value construction, NULL representation, and NaN/Infinity rejection are prerequisites for every other feature. Without safe construction, the type cannot guarantee its invariant.

**Independent Test**: Can be fully tested by constructing values with `new()`, constants, and calling `value()` / `is_null()`. Delivers the ability to represent SQL REAL values safely in Rust.

**Acceptance Scenarios**:

1. **Given** `SqlSingle::new(3.14)`, **When** `value()` called, **Then** returns `Ok(3.14)`
2. **Given** `SqlSingle::new(-2.5)`, **When** `value()` called, **Then** returns `Ok(-2.5)`
3. **Given** `SqlSingle::new(0.0)`, **When** `value()` called, **Then** returns `Ok(0.0)`
4. **Given** `SqlSingle::NULL`, **When** `is_null()` called, **Then** returns `true`
5. **Given** `SqlSingle::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`
6. **Given** `SqlSingle::ZERO`, **When** `value()` called, **Then** returns `Ok(0.0)`
7. **Given** `SqlSingle::MIN_VALUE`, **When** `value()` called, **Then** returns `Ok(f32::MIN)`
8. **Given** `SqlSingle::MAX_VALUE`, **When** `value()` called, **Then** returns `Ok(f32::MAX)`
9. **Given** attempt to create with `f32::NAN`, **Then** returns error (construction rejected)
10. **Given** attempt to create with `f32::INFINITY`, **Then** returns error (construction rejected)
11. **Given** attempt to create with `f32::NEG_INFINITY`, **Then** returns error (construction rejected)

---

### User Story 2 - Arithmetic with overflow and NaN/Infinity detection (Priority: P1)

A library consumer performs arithmetic on `SqlSingle` values. All four operations (add, subtract, multiply, divide) MUST check the result for NaN and Infinity, returning errors instead of producing invalid values. Division by zero returns a divide-by-zero error. NULL propagates through all arithmetic.

**Why this priority**: Checked arithmetic with NaN/Infinity rejection is the primary value proposition of `SqlSingle` over raw `f32` — it prevents silent data corruption from invalid floating-point results.

**Independent Test**: Can be fully tested by performing arithmetic operations and verifying results, overflow/infinity errors, divide-by-zero errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlSingle(2.5) + SqlSingle(3.5)`, **Then** returns `Ok(SqlSingle(6.0))`
2. **Given** `SqlSingle(10.0) - SqlSingle(3.0)`, **Then** returns `Ok(SqlSingle(7.0))`
3. **Given** `SqlSingle(4.0) * SqlSingle(2.5)`, **Then** returns `Ok(SqlSingle(10.0))`
4. **Given** `SqlSingle(10.0) / SqlSingle(4.0)`, **Then** returns `Ok(SqlSingle(2.5))`
5. **Given** `SqlSingle(f32::MAX) + SqlSingle(f32::MAX)`, **Then** returns overflow error (result is Infinity)
6. **Given** `SqlSingle(-f32::MAX) - SqlSingle(f32::MAX)`, **Then** returns overflow error (result is -Infinity)
7. **Given** `SqlSingle(f32::MAX) * SqlSingle(2.0)`, **Then** returns overflow error (result is Infinity)
8. **Given** `SqlSingle(1.0) / SqlSingle(0.0)`, **Then** returns divide-by-zero error
9. **Given** `SqlSingle(0.0) / SqlSingle(0.0)`, **Then** returns divide-by-zero error (result would be NaN)
10. **Given** any arithmetic op with `SqlSingle::NULL` operand, **Then** returns `SqlSingle::NULL`

---

### User Story 3 - Negation (Priority: P1)

A library consumer negates `SqlSingle` values. Negation inverts the sign and propagates NULL.

**Why this priority**: Negation is a core unary arithmetic operation needed for completeness of the arithmetic story.

**Independent Test**: Can be fully tested by negating values and checking the result.

**Acceptance Scenarios**:

1. **Given** `-SqlSingle(5.0)`, **Then** returns `SqlSingle(-5.0)`
2. **Given** `-SqlSingle(-3.14)`, **Then** returns `SqlSingle(3.14)`
3. **Given** `-SqlSingle(0.0)`, **Then** returns `SqlSingle(-0.0)` (IEEE 754 negative zero)
4. **Given** `-SqlSingle::NULL`, **Then** returns `SqlSingle::NULL`

---

### User Story 4 - Comparison returning SqlBoolean (Priority: P2)

A library consumer compares `SqlSingle` values using SQL three-valued logic. Comparisons return `SqlBoolean` (not `bool`), and any comparison involving NULL returns `SqlBoolean::NULL`.

**Why this priority**: Comparisons are essential for conditional logic but depend on the type already being constructable.

**Independent Test**: Can be fully tested by comparing pairs of values and verifying the returned `SqlBoolean`.

**Acceptance Scenarios**:

1. **Given** `SqlSingle(1.0).sql_equals(&SqlSingle(1.0))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlSingle(1.0).sql_equals(&SqlSingle(2.0))`, **Then** returns `SqlBoolean::FALSE`
3. **Given** `SqlSingle(1.0).sql_less_than(&SqlSingle(2.0))`, **Then** returns `SqlBoolean::TRUE`
4. **Given** `SqlSingle(2.0).sql_greater_than(&SqlSingle(1.0))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlSingle(1.0).sql_less_than_or_equal(&SqlSingle(1.0))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlSingle(1.0).sql_greater_than_or_equal(&SqlSingle(1.0))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlSingle(1.0).sql_not_equals(&SqlSingle(2.0))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** any comparison with `SqlSingle::NULL` operand, **Then** returns `SqlBoolean::NULL`

---

### User Story 5 - Display and parsing (Priority: P2)

A library consumer converts `SqlSingle` to and from string representations. NULL displays as `"Null"`. Parsing invalid strings returns a parse error.

**Why this priority**: String conversion is needed for diagnostics, logging, and data interchange.

**Independent Test**: Can be fully tested by formatting values with `Display` and parsing strings with `FromStr`.

**Acceptance Scenarios**:

1. **Given** `SqlSingle::new(3.14)`, **When** formatted with `Display`, **Then** produces `"3.14"`
2. **Given** `SqlSingle::NULL`, **When** formatted with `Display`, **Then** produces `"Null"`
3. **Given** `SqlSingle::new(0.0)`, **When** formatted with `Display`, **Then** produces `"0"`
4. **Given** string `"3.14"`, **When** parsed as `SqlSingle`, **Then** returns `SqlSingle(3.14)`
5. **Given** string `"abc"`, **When** parsed as `SqlSingle`, **Then** returns parse error
6. **Given** string `"NaN"`, **When** parsed as `SqlSingle`, **Then** returns parse error (NaN not allowed)
7. **Given** string `"Infinity"`, **When** parsed as `SqlSingle`, **Then** returns parse error (Infinity not allowed)
8. **Given** string representing a value too large for `f32`, **When** parsed, **Then** returns overflow error

---

### User Story 6 - Conversions to and from other SqlTypes (Priority: P3)

A library consumer converts between `SqlSingle` and other SQL types. Widening conversions from integer types are infallible (except for NULL propagation). Conversion from `SqlBoolean` follows C# semantics (TRUE=1.0, FALSE=0.0).

**Why this priority**: Cross-type conversions enable interoperability between SQL types but are only useful after core arithmetic and comparison are working.

**Independent Test**: Can be fully tested by converting values between types and verifying results, errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlByte(42)`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(42.0)`
2. **Given** `SqlInt16(1000)`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(1000.0)`
3. **Given** `SqlInt32(100_000)`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(100000.0)`
4. **Given** `SqlInt64(1_000_000)`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(1000000.0)`
5. **Given** `SqlBoolean::TRUE`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(1.0)`
6. **Given** `SqlBoolean::FALSE`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(0.0)`
7. **Given** `SqlBoolean::NULL`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle::NULL`
8. **Given** `SqlSingle::NULL`, **When** converted to any type, **Then** returns NULL of target type
9. **Given** `SqlMoney(42.5)`, **When** converted to `SqlSingle`, **Then** returns `SqlSingle(42.5)`
10. **Given** `SqlSingle(42.0)`, **When** converted to `SqlBoolean`, **Then** returns `SqlBoolean::TRUE`
11. **Given** `SqlSingle(0.0)`, **When** converted to `SqlBoolean`, **Then** returns `SqlBoolean::FALSE`

---

### Edge Cases

- NaN and Infinity MUST be rejected both on construction and after every arithmetic operation
- Negative zero (`-0.0`) is a valid IEEE 754 value and MUST be accepted; `-0.0 == 0.0` in comparisons (IEEE 754 semantics)
- Subnormal (denormalized) values near the limits of `f32` precision are valid and MUST be accepted
- Very small subnormal values (e.g., `f32::MIN_POSITIVE * 0.5`) MUST be handled correctly
- Division: `0.0 / 0.0` produces NaN in IEEE 754 — MUST return divide-by-zero error
- Division: `x / 0.0` where x ≠ 0 produces Infinity in IEEE 754 — MUST return divide-by-zero error
- Negation of `f32::MIN` and `f32::MAX` are valid (just flips sign, no overflow possible with floats)
- NULL propagates through all arithmetic, comparison, and conversion operations
- Widening integer-to-float conversion may lose precision for very large integers (e.g., `i32::MAX` as `f32` loses low-order bits) — this is acceptable and matches C# behavior
- `From<f32>` trait provides ergonomic construction but will panic on NaN/Infinity (use `try_from` for fallible construction)
- `PartialEq` / `Eq` — Rust-level equality (distinct from `sql_equals` which returns `SqlBoolean`). Two NULL values are equal for Rust `PartialEq`, but `sql_equals` returns `SqlBoolean::NULL`
- `PartialOrd` / `Ord` — NULL values sort before all non-NULL values (consistent with Rust `Option` convention)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlSingle` MUST be `Copy + Clone`, representing `f32` with NULL support via an internal `Option<f32>`
- **FR-002**: MUST reject NaN and Infinity on construction — `new()` MUST validate that the value is finite; a fallible constructor (`try_new()` or similar) MUST return `Err(SqlTypeError::Overflow)` for non-finite values
- **FR-003**: MUST implement checked `Add`, `Sub`, `Mul`, `Div` — all return `Result<SqlSingle, SqlTypeError>` and reject any result that is NaN or Infinity
- **FR-004**: MUST implement `Neg` — infallible for valid values; propagates NULL
- **FR-005**: Division by zero MUST return `Err(SqlTypeError::DivideByZero)` — checked before computing the result
- **FR-006**: MUST implement SQL comparison methods (`sql_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, `sql_not_equals`) returning `SqlBoolean`
- **FR-007**: MUST implement `Display` (NULL displays as `"Null"`) and `FromStr` (invalid input returns `ParseError`; NaN/Infinity strings rejected)
- **FR-008**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE` (f32::MIN), `MAX_VALUE` (f32::MAX)
- **FR-009**: MUST provide widening conversions from `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, and `SqlBoolean` (TRUE=1.0, FALSE=0.0, NULL=NULL)
- **FR-010**: MUST provide conversion from `SqlMoney` to `SqlSingle` (widening, may lose precision)
- **FR-011**: MUST implement `From<f32>` (panics on NaN/Infinity) and a fallible conversion for safe construction
- **FR-012**: MUST implement `Hash`, `PartialEq`, `Eq` — two NULL values are equal for Rust equality; two non-NULL values compare by their `f32` value (using total-order bit comparison for Hash consistency)
- **FR-013**: MUST implement `PartialOrd`, `Ord` — NULL sorts before all non-NULL values
- **FR-014**: NULL propagation MUST apply to all arithmetic and comparison operations — any NULL operand produces a NULL result
- **FR-015**: MUST provide `to_sql_double()` — widening conversion from `SqlSingle` to `SqlDouble` (infallible, NULL propagates)
- **FR-016**: MUST provide `to_sql_boolean()` — zero maps to FALSE, non-zero maps to TRUE, NULL maps to NULL

### Key Entities

- **SqlSingle**: A nullable 32-bit IEEE 754 floating-point number. Internal representation: `Option<f32>` where `None` = SQL NULL, `Some(v)` = a finite value. Fixed-size, stack-allocated. Invariant: the contained `f32` is always finite (never NaN or Infinity).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: NaN and Infinity are never representable — all construction and arithmetic paths that would produce non-finite values are detected and returned as errors, with 100% of such paths tested
- **SC-002**: Code coverage ≥ 95% for the `SqlSingle` module
- **SC-003**: All four arithmetic operations have at least one positive test, one overflow/infinity error test, one divide-by-zero test (for division), and one NULL propagation test
- **SC-004**: `Display` and `FromStr` round-trip correctly for all non-NULL finite values
- **SC-005**: All widening conversions from integer types tested with representative values and NULL propagation
- **SC-006**: All six comparison methods tested with equal, unequal, and NULL operand combinations

## Assumptions

- Overflow detection for floating-point arithmetic uses `f32::is_finite()` after the operation — if the result is not finite, return `Err(SqlTypeError::Overflow)` (matches C# `float.IsInfinity()` checks)
- Division by zero is checked by testing if the divisor is `0.0` before computing, returning `Err(SqlTypeError::DivideByZero)` (matches C# explicit zero checks)
- Negative zero (`-0.0`) is accepted as valid — IEEE 754 treats `0.0 == -0.0`, and this matches C# behavior
- Subnormal values are accepted — no additional validation beyond `is_finite()`
- `Hash` implementation uses `f32::to_bits()` for bit-exact hashing, with special handling so that `0.0` and `-0.0` hash identically (they are equal under IEEE 754)
- Widening conversions from integer types may lose precision for large values (e.g., `i32::MAX` → `f32` loses low bits) — this is expected and matches C# behavior
- Widening conversions to `SqlDouble` and `SqlDecimal` are deferred until those types are implemented (except `to_sql_double()` which is included here)
- `PartialOrd` / `Ord` for Rust-level ordering: NULL values sort before all non-NULL values (consistent with Rust convention for `Option`)
