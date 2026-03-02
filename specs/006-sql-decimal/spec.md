# Feature Specification: SqlDecimal

**Feature Branch**: `006-sql-decimal`  
**Created**: 2026-03-02  
**Status**: Draft  
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDecimal` — a fixed-point decimal number with up to 38 digits of precision and configurable scale, NULL support, checked arithmetic with precision/scale propagation, and SQL three-valued comparison logic

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect values (Priority: P1)

A library consumer creates `SqlDecimal` values by specifying precision, scale, sign, and up to four `u32` data components representing the 128-bit mantissa. The consumer can inspect the value's precision, scale, sign, and whether it is NULL.

**Why this priority**: Value construction and inspection are prerequisites for every other operation — without them, nothing else is testable.

**Independent Test**: Can be fully tested by constructing values with `new()`, constants, and calling `value()`, `is_null()`, `precision()`, `scale()`, `is_positive()`.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal::new(10, 2, true, 12345, 0, 0, 0)` (precision=10, scale=2, positive), **When** inspected, **Then** represents `123.45` with `precision()` returning `10` and `scale()` returning `2`
2. **Given** `SqlDecimal::new(10, 2, false, 12345, 0, 0, 0)` (negative), **When** inspected, **Then** represents `-123.45` and `is_positive()` returns `false`
3. **Given** `SqlDecimal::NULL`, **When** `is_null()` called, **Then** returns `true`
4. **Given** `SqlDecimal::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`
5. **Given** precision of `0` or greater than `38`, **When** constructing, **Then** returns error
6. **Given** scale greater than precision, **When** constructing, **Then** returns error
7. **Given** `SqlDecimal::MAX_VALUE`, **When** inspected, **Then** represents `10^38 - 1` with precision `38` and scale `0`
8. **Given** `SqlDecimal::MIN_VALUE`, **When** inspected, **Then** represents `-(10^38 - 1)` with precision `38` and scale `0`
9. **Given** all four `u32` components populated (large value), **When** inspected, **Then** correctly represents the full 128-bit mantissa
10. **Given** a value with trailing fractional zeros (e.g., `100.00`), **When** inspected, **Then** preserves the declared scale

---

### User Story 2 - Arithmetic with precision/scale propagation (Priority: P1)

A library consumer performs arithmetic (add, subtract, multiply, divide, remainder, negate) on `SqlDecimal` values. Result precision and scale follow SQL Server rules. Overflow (result exceeding 38 digits) returns an error. NULL propagates through all arithmetic.

**Why this priority**: Checked arithmetic with correct precision/scale propagation is the primary value proposition — it prevents silent data corruption and matches SQL Server behavior.

**Independent Test**: Can be fully tested by performing arithmetic operations and verifying result values, precision/scale, overflow errors, divide-by-zero errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal(precision=5, scale=2, value=123.45) + SqlDecimal(precision=5, scale=2, value=678.90)`, **Then** returns `SqlDecimal` representing `802.35` with precision/scale computed per SQL Server addition rules
2. **Given** `SqlDecimal(100.00) - SqlDecimal(200.00)`, **Then** returns `SqlDecimal` representing `-100.00`
3. **Given** `SqlDecimal(precision=5, scale=2) * SqlDecimal(precision=5, scale=2)`, **Then** result precision = `p1 + p2 + 1` and result scale = `s1 + s2` (both capped at 38)
4. **Given** `SqlDecimal(10.00) / SqlDecimal(3.00)`, **Then** returns a `SqlDecimal` with appropriate scale for the quotient
5. **Given** `SqlDecimal(10.00) / SqlDecimal(0)`, **Then** returns `Err(DivideByZero)`
6. **Given** `SqlDecimal(10.00) % SqlDecimal(3.00)`, **Then** returns `SqlDecimal` representing `1.00`
7. **Given** `SqlDecimal(10.00) % SqlDecimal(0)`, **Then** returns `Err(DivideByZero)`
8. **Given** negation of a positive `SqlDecimal`, **Then** returns the negative equivalent
9. **Given** negation of a negative `SqlDecimal`, **Then** returns the positive equivalent
10. **Given** overflow (result exceeds 38 digits of precision), **Then** returns `Err(Overflow)`
11. **Given** any arithmetic op with `SqlDecimal::NULL` operand, **Then** returns `SqlDecimal::NULL`

---

### User Story 3 - Comparison returning SqlBoolean (Priority: P1)

A library consumer compares `SqlDecimal` values using SQL three-valued logic. Comparisons return `SqlBoolean` (not `bool`), and any comparison involving NULL returns `SqlBoolean::NULL`. Values with different precision/scale but mathematically equal (e.g., `1.0` vs `1.00`) compare as equal.

**Why this priority**: Comparisons are essential for conditional logic and are a core SQL type behavior.

**Independent Test**: Can be fully tested by comparing pairs of values and verifying the returned `SqlBoolean`.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal(100.00).sql_equals(&SqlDecimal(100.00))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlDecimal(100.00).sql_equals(&SqlDecimal(100.0000))`, **Then** returns `SqlBoolean::TRUE` (different scale, same value)
3. **Given** `SqlDecimal(100.00).sql_equals(&SqlDecimal(200.00))`, **Then** returns `SqlBoolean::FALSE`
4. **Given** `SqlDecimal(100.00).sql_less_than(&SqlDecimal(200.00))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlDecimal(200.00).sql_greater_than(&SqlDecimal(100.00))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlDecimal(100.00).sql_less_than_or_equal(&SqlDecimal(100.00))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlDecimal(100.00).sql_greater_than_or_equal(&SqlDecimal(100.00))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** `SqlDecimal(100.00).sql_not_equals(&SqlDecimal(200.00))`, **Then** returns `SqlBoolean::TRUE`
9. **Given** any comparison with `SqlDecimal::NULL` operand, **Then** returns `SqlBoolean::NULL`
10. **Given** comparison of negative and positive values, **Then** negative is less than positive

---

### User Story 4 - Display and parsing (Priority: P2)

A library consumer converts `SqlDecimal` to and from string representations. NULL displays as `"Null"`. Parsing supports decimal notation with optional sign. Invalid strings return a parse error.

**Why this priority**: String conversion is needed for diagnostics, logging, and data interchange but depends on core construction being functional.

**Independent Test**: Can be fully tested by formatting values with `Display` and parsing strings with `FromStr`.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal` representing `123.45`, **When** formatted with `Display`, **Then** produces `"123.45"`
2. **Given** `SqlDecimal` representing `-123.45`, **When** formatted with `Display`, **Then** produces `"-123.45"`
3. **Given** `SqlDecimal` representing `100.00`, **When** formatted with `Display`, **Then** produces `"100.00"` (preserves scale)
4. **Given** `SqlDecimal::NULL`, **When** formatted with `Display`, **Then** produces `"Null"`
5. **Given** string `"123.45"`, **When** parsed as `SqlDecimal`, **Then** returns `SqlDecimal` with value `123.45`, appropriate precision and scale
6. **Given** string `"-0.001"`, **When** parsed as `SqlDecimal`, **Then** returns negative `SqlDecimal` with scale `3`
7. **Given** string `"abc"`, **When** parsed as `SqlDecimal`, **Then** returns parse error
8. **Given** string with more than 38 significant digits, **When** parsed as `SqlDecimal`, **Then** returns overflow or parse error
9. **Given** string `"42"` (no decimal point), **When** parsed as `SqlDecimal`, **Then** returns `SqlDecimal` with scale `0`

---

### User Story 5 - Scale adjustment and rounding (Priority: P2)

A library consumer adjusts the scale of a `SqlDecimal`, either increasing it (zero-padding) or decreasing it (with truncation or rounding). This is needed when combining values with different scales or when storing into a column with a specific scale.

**Why this priority**: Scale adjustment is a secondary operation that builds on the core type but is essential for practical decimal manipulation.

**Independent Test**: Can be fully tested by calling `adjust_scale()` and verifying the resulting value and scale.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal` with value `123.4500` (scale=4), **When** `adjust_scale(2, true)` called (round=true), **Then** returns `SqlDecimal` with value `123.45` and scale `2`
2. **Given** `SqlDecimal` with value `123.456` (scale=3), **When** `adjust_scale(2, true)` called, **Then** returns `SqlDecimal` with value `123.46` (rounded up)
3. **Given** `SqlDecimal` with value `123.454` (scale=3), **When** `adjust_scale(2, true)` called, **Then** returns `SqlDecimal` with value `123.45` (rounded down)
4. **Given** `SqlDecimal` with value `123.456` (scale=3), **When** `adjust_scale(2, false)` called (truncate), **Then** returns `SqlDecimal` with value `123.45` (truncated)
5. **Given** `SqlDecimal` with value `123.45` (scale=2), **When** `adjust_scale(4, true)` called, **Then** returns `SqlDecimal` with value `123.4500` and scale `4`
6. **Given** `SqlDecimal::NULL`, **When** `adjust_scale` called, **Then** returns `SqlDecimal::NULL`
7. **Given** scale adjustment that would require precision > 38, **Then** returns overflow error

---

### User Story 6 - Conversions to and from other types (Priority: P3)

A library consumer converts between `SqlDecimal` and other Rust and SQL types. Conversions from integers are widening (always succeed for non-NULL). Conversion to `f64` provides the closest double-precision approximation. Conversion to integer types truncates the fractional part and checks for range overflow.

**Why this priority**: Cross-type conversions enable interoperability but are only useful after core arithmetic, comparison, and parsing are working.

**Independent Test**: Can be fully tested by converting values between types and verifying results, range errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal` representing `42.99`, **When** converted to `f64`, **Then** returns closest double value to `42.99`
2. **Given** `SqlDecimal` representing `42.99`, **When** converted to integer, **Then** returns `42` (truncated)
3. **Given** `SqlDecimal` representing a value exceeding `i64::MAX`, **When** converted to `i64`, **Then** returns overflow error
4. **Given** `SqlInt32(42)`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` with value `42`, precision `10`, scale `0`
5. **Given** `SqlInt64(9_000_000_000)`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` with appropriate precision
6. **Given** `SqlDecimal::NULL`, **When** converted to any target type, **Then** returns NULL of target type
7. **Given** `SqlBoolean::TRUE`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` with value `1`
8. **Given** `SqlBoolean::FALSE`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` with value `0`

---

### User Story 7 - Mathematical functions (Priority: P3)

A library consumer uses mathematical helper functions on `SqlDecimal` values: `abs()`, `floor()`, `ceiling()`, `round()`, `truncate()`, `sign()`, and `power()`.

**Why this priority**: Mathematical functions extend the type's utility but are not needed for basic usage.

**Independent Test**: Can be fully tested by calling each function and verifying results and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal(-123.45)`, **When** `abs()` called, **Then** returns `SqlDecimal(123.45)`
2. **Given** `SqlDecimal(123.45)`, **When** `floor()` called, **Then** returns `SqlDecimal(123)`
3. **Given** `SqlDecimal(123.45)`, **When** `ceiling()` called, **Then** returns `SqlDecimal(124)`
4. **Given** `SqlDecimal(123.456)`, **When** `round(2)` called, **Then** returns `SqlDecimal(123.46)`
5. **Given** `SqlDecimal(123.456)`, **When** `truncate(2)` called, **Then** returns `SqlDecimal(123.45)`
6. **Given** `SqlDecimal(-5.00)`, **When** `sign()` called, **Then** returns `-1`
7. **Given** `SqlDecimal(0.00)`, **When** `sign()` called, **Then** returns `0`
8. **Given** `SqlDecimal(5.00)`, **When** `power(3)` called, **Then** returns `SqlDecimal(125.000000)`
9. **Given** `SqlDecimal::NULL`, **When** any math function called, **Then** returns NULL

---

### Edge Cases

- Maximum precision: 38 digits using all four `u32` components
- Scale = 0 for integer-like decimals (e.g., `SqlDecimal` representing `42`)
- Negative zero: `-0` normalizes to `0` (sign is positive for zero values)
- Very large values near the 128-bit mantissa limit (`u32::MAX` in all four data components)
- Division by values that produce non-terminating decimals (e.g., `1 / 3`)
- Precision/scale overflow during arithmetic (result would exceed precision 38)
- Arithmetic between values with very different scales (e.g., scale=0 + scale=38)
- Rounding at the midpoint (banker's rounding / round-half-to-even per SQL Server convention)
- Parsing strings with leading zeros (e.g., `"007.50"`)
- Parsing strings with trailing zeros that affect scale (e.g., `"1.0"` vs `"1.00"`)
- `adjust_scale` to scale 0 (effectively converting to integer)
- NULL propagation through all arithmetic, comparison, scale adjustment, and math operations

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlDecimal` MUST support precision from 1 to 38 and scale from 0 to precision. Construction with out-of-range precision or scale MUST return an error.
- **FR-002**: MUST store the numeric value as a sign flag plus four `u32` components forming a 128-bit unsigned mantissa, along with precision and scale metadata. The struct MUST implement `Clone` and `Debug` (not `Copy`, due to struct size).
- **FR-003**: MUST implement checked `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` — all return `Result<SqlDecimal, SqlTypeError>`. Precision and scale of the result MUST follow SQL Server computation rules. Overflow beyond 38 digits MUST return `Err(SqlTypeError::Overflow)`. Division by zero MUST return `Err(SqlTypeError::DivideByZero)`.
- **FR-004**: MUST implement `adjust_scale(new_scale, round)` — adjusts the value's scale. When `round` is true, uses round-half-up rounding. When false, truncates. Scale increase zero-pads. Returns `Err(SqlTypeError::Overflow)` if the adjustment would exceed precision 38.
- **FR-005**: MUST implement SQL comparison methods (`sql_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, `sql_not_equals`) returning `SqlBoolean`. Values with different precision/scale but mathematically equal MUST compare as equal.
- **FR-006**: MUST implement `Display` producing a decimal string representation (NULL displays as `"Null"`, preserves declared scale) and `FromStr` parsing decimal strings with optional sign (invalid input returns `Err(SqlTypeError::ParseError)`).
- **FR-007**: MUST provide accessor methods: `precision()`, `scale()`, `is_positive()`, `is_null()`, `value()`, and `data()` (returning the four `u32` components).
- **FR-008**: MUST provide constants: `NULL`, `MAX_VALUE` (38 nines, scale 0), `MIN_VALUE` (negative 38 nines, scale 0), and `MAX_PRECISION` (38).
- **FR-009**: MUST provide conversions: from integer types (`i32`, `i64`, `SqlInt32`, `SqlInt64`, `SqlByte`, `SqlInt16`) and `SqlBoolean` to `SqlDecimal`; from `SqlDecimal` to `f64` and to integer types with truncation and range checking.
- **FR-010**: MUST implement mathematical functions: `abs()`, `floor()`, `ceiling()`, `round(position)`, `truncate(position)`, `sign()` (returns `-1`, `0`, or `1`), and `power(exponent)`.
- **FR-011**: NULL propagation MUST apply to all arithmetic, comparison, scale adjustment, mathematical, and conversion operations — any NULL operand produces a NULL result.
- **FR-012**: MUST implement `PartialEq`, `Eq`, `Hash` for Rust-level equality semantics. `PartialOrd` and `Ord` for Rust-level ordering (NULL values treated as less than all non-NULL values, consistent with `Option` convention).

### Key Entities

- **SqlDecimal**: A nullable fixed-point decimal number. Internal representation: `Option<InnerDecimal>` where `None` = SQL NULL. `InnerDecimal` contains: `precision: u8` (1–38), `scale: u8` (0–precision), `positive: bool` (sign flag), `data: [u32; 4]` (128-bit unsigned mantissa in little-endian order). The struct is `Clone` but not `Copy` due to its size.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Precision and scale are correctly propagated through all six arithmetic operations, verified by tests covering same-scale operands, different-scale operands, and boundary precision values
- **SC-002**: Round-trip string parsing (`Display` → `FromStr` → `Display`) produces identical output for all non-NULL values across a range of precisions and scales
- **SC-003**: Code coverage ≥ 90% for the `SqlDecimal` module
- **SC-004**: All arithmetic overflow and divide-by-zero boundary conditions tested — including maximum precision operations and results that would exceed 38 digits
- **SC-005**: Scale adjustment verified for rounding, truncation, and zero-padding at boundary values
- **SC-006**: All comparison methods tested with equal values of different scales, positive/negative values, and NULL propagation
- **SC-007**: All conversion paths tested at boundary values with range-overflow detection for narrowing conversions

## Assumptions

- The four `u32` mantissa components are stored in little-endian order (data[0] is the least significant), matching the C# `SqlDecimal` internal layout
- Precision/scale propagation for arithmetic follows SQL Server rules as implemented in the C# `SqlDecimal` reference (e.g., addition: `max(s1,s2) + max(p1-s1, p2-s2) + 1` for precision, `max(s1,s2)` for scale)
- Negative zero is normalized to positive zero — the sign flag is always `true` (positive) when the value is zero
- `round()` and `adjust_scale(..., true)` use round-half-up (away from zero) rounding, consistent with C# `SqlDecimal.AdjustScale` behavior
- Conversions to/from `SqlSingle`, `SqlDouble`, `SqlMoney`, `SqlString`, and `SqlDateTime` are deferred until those types are implemented
- The struct implements `Clone` but not `Copy` — while the struct is fixed-size (approximately 24 bytes), the additional fields make `Copy` less appropriate for a complex numeric type
- `PartialOrd` / `Ord` for Rust-level ordering: NULL values are treated as less than all non-NULL values (consistent with Rust convention for `Option`)
- `Hash` implementation: NULL values hash consistently; mathematically equal values with different precision/scale produce equal hashes
