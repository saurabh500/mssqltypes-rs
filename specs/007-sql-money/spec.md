# Feature Specification: SqlMoney

**Feature Branch**: `007-sql-money`
**Created**: 2025-07-15
**Status**: Draft
**Input**: User description: "SqlMoney — a fixed-point currency type with 4 decimal places, stored as i64 × 10,000"

## User Scenarios & Testing

### User Story 1 — Create and Inspect Currency Values (Priority: P1)

Users can create SqlMoney values from integers, floating-point numbers, and raw scaled representations. They can inspect the stored value, check for NULL, and access boundary constants. This is the foundation — without it, no other operations are possible.

**Why this priority**: Construction and inspection are prerequisites for every other user story. A SqlMoney that can't be created or read has no value.

**Independent Test**: `SqlMoney::from_i32(100)` creates a value representing 100.0000. `SqlMoney::NULL.is_null()` returns `true`. `SqlMoney::MAX_VALUE` and `SqlMoney::MIN_VALUE` are accessible and have correct internal representations. `SqlMoney::ZERO` represents 0.0000.

**Acceptance Scenarios**:

1. **Given** `SqlMoney::from_i32(100)`, **When** `to_i64()` called, **Then** returns `Ok(100)`
2. **Given** `SqlMoney::from_i64(922_337_203_685_477)`, **When** `to_f64()` called, **Then** returns approximately `922337203685477.0`
3. **Given** `SqlMoney::from_i64(value)` where value exceeds `i64::MAX / 10_000`, **Then** returns `Err(Overflow)`
4. **Given** `SqlMoney::NULL`, **When** `is_null()` called, **Then** returns `true`
5. **Given** `SqlMoney::NULL`, **When** `to_i64()` called, **Then** returns `Err(NullValue)`
6. **Given** `SqlMoney::from_f64(123.4567)`, **When** inspected, **Then** internal value is `1_234_567` (123.4567 × 10,000)
7. **Given** `SqlMoney::from_f64(123.45678)`, **When** constructed, **Then** value is rounded to 4 decimal places (123.4568)
8. **Given** `SqlMoney::ZERO`, **When** inspected, **Then** internal value is `0`
9. **Given** `SqlMoney::MAX_VALUE`, **When** inspected, **Then** internal value is `i64::MAX` (922,337,203,685,477.5807)
10. **Given** `SqlMoney::MIN_VALUE`, **When** inspected, **Then** internal value is `i64::MIN` (−922,337,203,685,477.5808)
11. **Given** `SqlMoney::from_scaled(raw_i64)`, **When** created with any `i64`, **Then** stores that exact value without range checking (direct internal construction)

---

### User Story 2 — Arithmetic with Overflow Detection (Priority: P1)

Users can perform checked addition, subtraction, multiplication, division, and negation on SqlMoney values. Overflow beyond the i64 range returns an error. Division by zero returns an error. NULL propagates through all operations.

**Why this priority**: Arithmetic is essential for currency calculations — the primary use case. Without it, SqlMoney is just a container.

**Independent Test**: `SqlMoney(100.00) + SqlMoney(50.25)` returns `SqlMoney(150.25)`. `SqlMoney(MAX) + SqlMoney(0.0001)` returns `Err(Overflow)`. `SqlMoney(100.00) / SqlMoney(0.00)` returns `Err(DivideByZero)`. Any operation with NULL returns NULL.

**Acceptance Scenarios**:

1. **Given** `SqlMoney(100.0000) + SqlMoney(50.2500)`, **Then** returns `SqlMoney(150.2500)`
2. **Given** `SqlMoney(100.0000) - SqlMoney(200.0000)`, **Then** returns `SqlMoney(-100.0000)`
3. **Given** `SqlMoney(100.0000) * SqlMoney(2.5000)`, **Then** returns `SqlMoney(250.0000)`
4. **Given** `SqlMoney(100.0000) / SqlMoney(3.0000)`, **Then** returns `SqlMoney(33.3333)` (truncated to 4 decimal places per currency semantics)
5. **Given** `SqlMoney(MAX_VALUE) + SqlMoney(0.0001)`, **Then** returns `Err(Overflow)`
6. **Given** `SqlMoney(100.0000) / SqlMoney(0.0000)`, **Then** returns `Err(DivideByZero)`
7. **Given** `-SqlMoney(100.0000)`, **Then** returns `SqlMoney(-100.0000)`
8. **Given** `-SqlMoney(MIN_VALUE)`, **Then** returns `Err(Overflow)` (i64::MIN cannot be negated)
9. **Given** any arithmetic with `SqlMoney::NULL`, **Then** returns `Ok(SqlMoney::NULL)`
10. **Given** `SqlMoney(100.0000) * SqlMoney(0.0000)`, **Then** returns `SqlMoney(0.0000)`
11. **Given** add/subtract, **When** internal i64 values are added/subtracted directly with checked arithmetic, **Then** result is exact (no rounding)

---

### User Story 3 — Comparison Returning SqlBoolean (Priority: P1)

Users can compare SqlMoney values using SQL three-valued logic. Comparisons return SqlBoolean. NULL compared with anything returns SqlBoolean::NULL.

**Why this priority**: Comparisons are essential for conditional logic and sorting in SQL operations.

**Independent Test**: `SqlMoney(100.00).sql_equals(&SqlMoney(100.00))` returns `SqlBoolean::TRUE`. `SqlMoney(100.00).sql_less_than(&SqlMoney(200.00))` returns `SqlBoolean::TRUE`. Any comparison with NULL returns `SqlBoolean::NULL`.

**Acceptance Scenarios**:

1. **Given** `SqlMoney(100.0000).sql_equals(&SqlMoney(100.0000))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlMoney(100.0000).sql_equals(&SqlMoney(200.0000))`, **Then** returns `SqlBoolean::FALSE`
3. **Given** `SqlMoney(100.0000).sql_less_than(&SqlMoney(200.0000))`, **Then** returns `SqlBoolean::TRUE`
4. **Given** `SqlMoney(200.0000).sql_greater_than(&SqlMoney(100.0000))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlMoney(100.0000).sql_less_than_or_equal(&SqlMoney(100.0000))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlMoney(100.0000).sql_greater_than_or_equal(&SqlMoney(100.0000))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlMoney(100.0000).sql_not_equals(&SqlMoney(200.0000))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** any comparison with `SqlMoney::NULL`, **Then** returns `SqlBoolean::NULL`

---

### User Story 4 — Display and Parsing (Priority: P2)

Users can convert SqlMoney to and from string representations. Display shows at least 2 decimal places and up to 4, trimming trailing zeros beyond the 2nd place (matching C# format `"#0.00##"`). NULL displays as `"Null"`. Parsing supports decimal notation with optional sign.

**Why this priority**: String conversion is important for debugging, logging, and user-facing output but is not needed for core computation.

**Independent Test**: `format!("{}", SqlMoney(123.4567))` produces `"123.4567"`. `format!("{}", SqlMoney(100.0000))` produces `"100.00"`. `"123.45".parse::<SqlMoney>()` returns a valid SqlMoney.

**Acceptance Scenarios**:

1. **Given** `SqlMoney(123.4567)`, **When** displayed, **Then** shows `"123.4567"` (all 4 decimal digits significant)
2. **Given** `SqlMoney(123.4500)`, **When** displayed, **Then** shows `"123.45"` (trailing zeros trimmed to minimum of 2)
3. **Given** `SqlMoney(100.0000)`, **When** displayed, **Then** shows `"100.00"` (minimum 2 decimal places)
4. **Given** `SqlMoney(-50.1000)`, **When** displayed, **Then** shows `"-50.10"` (trailing zeros trimmed to minimum of 2)
5. **Given** `SqlMoney::NULL`, **When** displayed, **Then** shows `"Null"`
6. **Given** `"123.4567"`, **When** parsed as SqlMoney, **Then** returns `SqlMoney(123.4567)`
7. **Given** `"-50.10"`, **When** parsed as SqlMoney, **Then** returns `SqlMoney(-50.1000)`
8. **Given** `"abc"`, **When** parsed as SqlMoney, **Then** returns `Err(ParseError)`
9. **Given** `"Null"`, **When** parsed as SqlMoney, **Then** returns `SqlMoney::NULL`
10. **Given** a string with more than 4 decimal places, **When** parsed, **Then** value is rounded to 4 decimal places

---

### User Story 5 — Conversions To and From Other SqlTypes (Priority: P2)

Users can convert between SqlMoney and other SqlTypes. Widening conversions from integers always succeed. Narrowing conversions check range.

**Why this priority**: Cross-type conversions enable SqlMoney to interoperate with the rest of the type system but are secondary to core functionality.

**Independent Test**: `SqlMoney::from(SqlInt32::new(42))` returns `SqlMoney(42.0000)`. `SqlMoney(42.9999).to_sql_int64()` returns `SqlInt64(43)` (rounded). `SqlMoney(42.0000).to_sql_decimal()` returns a SqlDecimal with scale=4.

**Acceptance Scenarios**:

1. **Given** `SqlMoney::from(SqlInt32::new(42))`, **Then** returns `SqlMoney(42.0000)` with internal value `420000`
2. **Given** `SqlMoney::from(SqlInt64::new(100))`, **Then** returns `SqlMoney(100.0000)`
3. **Given** `SqlMoney::from(SqlInt64::new(value))` where value exceeds money range, **Then** returns `Err(Overflow)`
4. **Given** `SqlMoney::from(SqlBoolean::TRUE)`, **Then** returns `SqlMoney(1.0000)`
5. **Given** `SqlMoney::from(SqlBoolean::FALSE)`, **Then** returns `SqlMoney(0.0000)`
6. **Given** `SqlMoney::from(SqlByte::new(255))`, **Then** returns `SqlMoney(255.0000)`
7. **Given** `SqlMoney::from(SqlInt16::new(1000))`, **Then** returns `SqlMoney(1000.0000)`
8. **Given** any conversion from NULL SqlType, **Then** returns `SqlMoney::NULL`
9. **Given** `SqlMoney(42.9999).to_sql_int64()`, **Then** returns `SqlInt64(43)` (round-half-away-from-zero)
10. **Given** `SqlMoney(42.0000).to_sql_int32()`, **Then** returns `SqlInt32(42)`
11. **Given** `SqlMoney(42.5000).to_sql_decimal()`, **Then** returns SqlDecimal with value 42.5000 and scale=4
12. **Given** `SqlMoney(42.0000).to_f64()`, **Then** returns `42.0`
13. **Given** `SqlMoney::NULL.to_sql_boolean()`, **Then** returns `SqlBoolean::NULL`
14. **Given** `SqlMoney(0.0000).to_sql_boolean()`, **Then** returns `SqlBoolean::FALSE`
15. **Given** `SqlMoney(1.0000).to_sql_boolean()`, **Then** returns `SqlBoolean::TRUE`

---

### User Story 6 — Standard Rust Traits (Priority: P3)

Users can use SqlMoney with standard Rust trait operations: equality, hashing, and ordering. This enables use in HashMaps, BTreeMaps, sorting, and pattern matching.

**Why this priority**: Standard trait compliance is important for Rust ergonomics but not for core SQL type behavior.

**Independent Test**: Two equal SqlMoney values compare as equal and produce the same hash. NULL == NULL per Rust semantics. SqlMoney values can be sorted with NULL < any value.

**Acceptance Scenarios**:

1. **Given** `SqlMoney(100.0000) == SqlMoney(100.0000)`, **Then** returns `true` (Rust PartialEq)
2. **Given** `SqlMoney::NULL == SqlMoney::NULL`, **Then** returns `true` (Rust semantics, not SQL)
3. **Given** equal SqlMoney values, **When** hashed, **Then** produce same hash
4. **Given** `SqlMoney::NULL`, **When** compared via Ord, **Then** NULL < any non-NULL value
5. **Given** `SqlMoney(-100.0000) < SqlMoney(100.0000)`, **Then** returns `true`

---

### Edge Cases

- Negation of `MIN_VALUE` (`i64::MIN`) overflows — must return `Err(Overflow)`
- Construction from `f64`: rounding to 4 decimal places; NaN and Infinity must be rejected
- Multiplication intermediate result may overflow i64 — must use wider intermediate (i128 or decimal approach)
- Division by zero must return `Err(DivideByZero)`, not panic
- `to_i64()` rounds the value (round-half-away-from-zero per C# reference), not truncation
- `to_i32()` rounds then checks i32 range
- Display format: minimum 2 decimal places, maximum 4 (trim trailing zeros only beyond 2)
- Parsing should handle leading/trailing whitespace and optional sign

## Requirements

### Functional Requirements

- **FR-001**: `SqlMoney` MUST be `Copy + Clone + Debug`, internally `Option<i64>` where `None` = SQL NULL and `Some(v)` = value × 10,000
- **FR-002**: MUST provide constants: `NULL`, `ZERO` (internal 0), `MIN_VALUE` (internal `i64::MIN`), `MAX_VALUE` (internal `i64::MAX`)
- **FR-003**: MUST provide constructors: `from_i32(i32)`, `from_i64(i64) -> Result` (range-checked), `from_f64(f64) -> Result` (rounded to 4dp, reject NaN/Infinity), `from_scaled(i64)` (direct internal value)
- **FR-004**: MUST provide accessors: `is_null() -> bool`, `to_i64() -> Result` (rounded), `to_i32() -> Result` (rounded + range check), `to_f64() -> Result`, `scaled_value() -> Result<i64>` (raw internal)
- **FR-005**: MUST implement checked arithmetic: `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_neg` — all returning `Result<SqlMoney, SqlTypeError>`
- **FR-006**: Add and subtract MUST use checked i64 arithmetic (exact, no rounding)
- **FR-007**: Multiply MUST use i128 intermediate to avoid overflow during computation, result rounded/truncated back to i64 scale
- **FR-008**: Divide MUST check for zero divisor → `Err(DivideByZero)`, use i128 intermediate for scale-preserving division
- **FR-009**: Negation of `i64::MIN` MUST return `Err(Overflow)`
- **FR-010**: All arithmetic with NULL operand MUST return `Ok(SqlMoney::NULL)`
- **FR-011**: MUST implement operator traits: `Add`, `Sub`, `Mul`, `Div`, `Neg` for both owned and borrowed, `Output = Result<SqlMoney, SqlTypeError>`
- **FR-012**: MUST implement 6 SQL comparison methods returning `SqlBoolean` with NULL propagation
- **FR-013**: MUST implement `Display` with format: minimum 2 decimal places, maximum 4, trimming trailing zeros beyond 2nd place
- **FR-014**: MUST implement `FromStr` with support for decimal notation, optional sign, `"Null"` → NULL, reject invalid input
- **FR-015**: MUST implement `From<SqlBoolean>`, `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>`, `From<SqlInt64>` (the last with range checking via `TryFrom` pattern or returning NULL-safe value)
- **FR-016**: MUST implement `to_sql_int64()`, `to_sql_int32()`, `to_sql_int16()`, `to_sql_byte()` with rounding and range checking
- **FR-017**: MUST implement `to_sql_decimal()` returning SqlDecimal with scale=4
- **FR-018**: MUST implement `to_sql_boolean()`: zero → FALSE, non-zero → TRUE, NULL → NULL
- **FR-019**: MUST implement `PartialEq`/`Eq` (NULL == NULL per Rust semantics), `Hash`, `PartialOrd`/`Ord` (NULL < any value)
- **FR-020**: Construction from `f64` MUST reject NaN and Infinity with `Err(OutOfRange)`

### Key Entities

- **SqlMoney**: Fixed-point currency value. Internal representation is `Option<i64>` where the i64 stores the value multiplied by 10,000. Range of the monetary value is −922,337,203,685,477.5808 to 922,337,203,685,477.5807. Always exactly 4 decimal places of internal precision.
- **Scale factor**: Constant 10,000 (10^4). All values are stored as `actual_value × 10,000`.

## Assumptions

- SqlMoney is a fixed-size type, so it implements `Copy` (unlike SqlDecimal which is `Clone`-only)
- The `from_i64` constructor checks that the value fits when scaled by 10,000 (i.e., `value * 10_000` must fit in i64), matching the C# behavior where `new SqlMoney(long)` range-checks
- The `from_scaled` constructor takes a raw i64 directly without scaling — this is the equivalent of the C# internal constructor `SqlMoney(long value, int _)` and is used for `MIN_VALUE`, `MAX_VALUE`, and TDS interop
- Multiplication uses a wider intermediate (i128) to multiply the two scaled values, then dividing by 10,000 to bring back to the correct scale
- Division scales the dividend by 10,000 before dividing to preserve 4 decimal places in the result
- Display format `"#0.00##"` means: always show at least 2 decimal digits, show 3rd and 4th only if non-zero
- `to_i64()` uses round-half-away-from-zero (matching C# `SqlMoney.ToInt64()`)
- No `unsafe` code

## Success Criteria

### Measurable Outcomes

- **SC-001**: Internal i64 representation preserves exact 4-decimal-place precision for all values in range
- **SC-002**: All arithmetic operations correctly detect overflow and return errors instead of wrapping
- **SC-003**: ≥95% code coverage across all public methods
- **SC-004**: `cargo fmt` and `cargo clippy -- -D warnings` pass with zero warnings
- **SC-005**: All acceptance scenarios from all user stories pass as automated tests
- **SC-006**: NULL propagation is correct for every operation that accepts SqlMoney operands
