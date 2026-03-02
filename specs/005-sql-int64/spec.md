# Feature Specification: SqlInt64

**Feature Branch**: `005-sql-int64`  
**Created**: 2026-03-01  
**Status**: Draft  
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlInt64` — a signed 64-bit integer (BIGINT) with NULL support, checked arithmetic, bitwise operations, and SQL three-valued comparison logic

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect values (Priority: P1)

A library consumer creates `SqlInt64` values from 64-bit integers and inspects them, including the SQL NULL sentinel. This is the foundational capability — without constructing and reading values, no other operation is meaningful.

**Why this priority**: Core value construction and NULL representation are prerequisites for every other feature.

**Independent Test**: Can be fully tested by constructing values with `new()`, constants, and calling `value()` / `is_null()`. Delivers the ability to represent SQL BIGINT values in Rust.

**Acceptance Scenarios**:

1. **Given** `SqlInt64::new(9_000_000_000)`, **When** `value()` called, **Then** returns `Ok(9_000_000_000)`
2. **Given** `SqlInt64::new(-9_223_372_036_854_775_808)`, **When** `value()` called, **Then** returns `Ok(-9_223_372_036_854_775_808)`
3. **Given** `SqlInt64::NULL`, **When** `is_null()` called, **Then** returns `true`
4. **Given** `SqlInt64::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`
5. **Given** `SqlInt64::ZERO`, **When** `value()` called, **Then** returns `Ok(0)`
6. **Given** `SqlInt64::MIN_VALUE`, **When** `value()` called, **Then** returns `Ok(-9_223_372_036_854_775_808)`
7. **Given** `SqlInt64::MAX_VALUE`, **When** `value()` called, **Then** returns `Ok(9_223_372_036_854_775_807)`

---

### User Story 2 - Arithmetic with overflow detection (Priority: P1)

A library consumer performs arithmetic on `SqlInt64` values. All six operations (add, subtract, multiply, divide, remainder, negate) detect overflow and division by zero, returning errors instead of wrapping or panicking. NULL propagates through all arithmetic.

**Why this priority**: Checked arithmetic is the primary value proposition of SQL types — it prevents silent data corruption from overflow.

**Independent Test**: Can be fully tested by performing arithmetic operations and verifying results, overflow errors, divide-by-zero errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlInt64(100) + SqlInt64(200)`, **Then** returns `Ok(SqlInt64(300))`
2. **Given** `SqlInt64(i64::MAX) + SqlInt64(1)`, **Then** returns overflow error
3. **Given** `SqlInt64(i64::MIN) - SqlInt64(1)`, **Then** returns overflow error
4. **Given** `SqlInt64(i64::MIN) / SqlInt64(-1)`, **Then** returns overflow error
5. **Given** `SqlInt64(i64::MAX) * SqlInt64(2)`, **Then** returns overflow error
6. **Given** `SqlInt64(5_000_000_000) * SqlInt64(5_000_000_000)`, **Then** returns overflow error
7. **Given** `SqlInt64(10) / SqlInt64(0)`, **Then** returns divide-by-zero error
8. **Given** `SqlInt64(10) % SqlInt64(0)`, **Then** returns divide-by-zero error
9. **Given** `SqlInt64(7) % SqlInt64(3)`, **Then** returns `Ok(SqlInt64(1))`
10. **Given** `-SqlInt64(i64::MIN)`, **Then** returns overflow error
11. **Given** any arithmetic op with `SqlInt64::NULL` operand, **Then** returns `SqlInt64::NULL`

---

### User Story 3 - Bitwise operations (Priority: P2)

A library consumer performs bitwise operations (AND, OR, XOR, NOT) on `SqlInt64` values. Bitwise operations are infallible (never overflow) but propagate NULL.

**Why this priority**: Bitwise operations are secondary to arithmetic but required for SQL Server flag/mask manipulation patterns.

**Independent Test**: Can be fully tested by applying bitwise operators and verifying results and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlInt64(0xFF00) & SqlInt64(0x0FF0)`, **Then** returns `SqlInt64(0x0F00)`
2. **Given** `SqlInt64(0xFF00) | SqlInt64(0x00FF)`, **Then** returns `SqlInt64(0xFFFF)`
3. **Given** `SqlInt64(0xFF) ^ SqlInt64(0x0F)`, **Then** returns `SqlInt64(0xF0)`
4. **Given** `!SqlInt64(0)`, **Then** returns `SqlInt64(-1)` (all bits set)
5. **Given** bitwise op with `SqlInt64::NULL` operand, **Then** returns `SqlInt64::NULL`

---

### User Story 4 - Comparison returning SqlBoolean (Priority: P2)

A library consumer compares `SqlInt64` values using SQL three-valued logic. Comparisons return `SqlBoolean` (not `bool`), and any comparison involving NULL returns `SqlBoolean::NULL`.

**Why this priority**: Comparisons are essential for conditional logic but depend on the type already being constructable.

**Independent Test**: Can be fully tested by comparing pairs of values and verifying the returned `SqlBoolean`.

**Acceptance Scenarios**:

1. **Given** `SqlInt64(100).sql_equals(&SqlInt64(100))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlInt64(100).sql_equals(&SqlInt64(200))`, **Then** returns `SqlBoolean::FALSE`
3. **Given** `SqlInt64(100).sql_less_than(&SqlInt64(200))`, **Then** returns `SqlBoolean::TRUE`
4. **Given** `SqlInt64(200).sql_greater_than(&SqlInt64(100))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlInt64(100).sql_less_than_or_equal(&SqlInt64(100))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlInt64(100).sql_greater_than_or_equal(&SqlInt64(100))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlInt64(100).sql_not_equals(&SqlInt64(200))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** any comparison with `SqlInt64::NULL` operand, **Then** returns `SqlBoolean::NULL`

---

### User Story 5 - Display and parsing (Priority: P2)

A library consumer converts `SqlInt64` to and from string representations. NULL displays as `"Null"`. Parsing invalid strings returns a parse error.

**Why this priority**: String conversion is needed for diagnostics, logging, and data interchange.

**Independent Test**: Can be fully tested by formatting values with `Display` and parsing strings with `FromStr`.

**Acceptance Scenarios**:

1. **Given** `SqlInt64::new(9_000_000_000)`, **When** formatted with `Display`, **Then** produces `"9000000000"`
2. **Given** `SqlInt64::NULL`, **When** formatted with `Display`, **Then** produces `"Null"`
3. **Given** string `"9000000000"`, **When** parsed as `SqlInt64`, **Then** returns `SqlInt64(9_000_000_000)`
4. **Given** string `"abc"`, **When** parsed as `SqlInt64`, **Then** returns parse error
5. **Given** string `"99999999999999999999"` (out of i64 range), **When** parsed as `SqlInt64`, **Then** returns parse error

---

### User Story 6 - Conversions to and from other SqlTypes (Priority: P3)

A library consumer converts between `SqlInt64` and other SQL types. Narrowing conversions (to `SqlInt32`, `SqlInt16`, `SqlByte`) check for range overflow. Conversions from `SqlBoolean` follow C# semantics (TRUE=1, FALSE=0).

**Why this priority**: Cross-type conversions enable interoperability between SQL types but are only useful after core arithmetic and comparison are working.

**Independent Test**: Can be fully tested by converting values between types and verifying results, range errors, and NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlInt64(100)`, **When** converted to `SqlInt32`, **Then** returns `SqlInt32(100)`
2. **Given** `SqlInt64(3_000_000_000)`, **When** converted to `SqlInt32`, **Then** returns overflow error
3. **Given** `SqlInt64(100)`, **When** converted to `SqlInt16`, **Then** returns `SqlInt16(100)`
4. **Given** `SqlInt64(100_000)`, **When** converted to `SqlInt16`, **Then** returns overflow error
5. **Given** `SqlInt64(200)`, **When** converted to `SqlByte`, **Then** returns `SqlByte(200)`
6. **Given** `SqlInt64(300)`, **When** converted to `SqlByte`, **Then** returns overflow error
7. **Given** `SqlInt64(-1)`, **When** converted to `SqlByte`, **Then** returns overflow error
8. **Given** `SqlBoolean::TRUE`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64(1)`
9. **Given** `SqlBoolean::FALSE`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64(0)`
10. **Given** `SqlBoolean::NULL`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64::NULL`
11. **Given** `SqlInt64::NULL`, **When** converted to any type, **Then** returns NULL of target type

---

### Edge Cases

- `i64::MIN / -1` → overflow (because `|i64::MIN|` > `i64::MAX`)
- Negation of `i64::MIN` → overflow (same reason)
- `i64::MIN % -1` → overflow (Rust panics on this; must be caught)
- Multiplication overflow detected via `checked_mul`
- NULL propagates through all arithmetic, bitwise, and comparison operations
- `From<i64>` trait provides ergonomic construction
- `Hash` implementation: NULL values hash consistently; equal values produce equal hashes
- `PartialEq` / `Eq` for Rust-level equality (distinct from `sql_equals` which returns `SqlBoolean`)
- Values exceeding `i32::MAX` or below `i32::MIN` must be correctly handled in narrowing conversions
- Large magnitude multiplication (e.g., `5_000_000_000 * 5_000_000_000`) must detect overflow

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlInt64` MUST be `Copy + Clone`, storing an `i64` with NULL support
- **FR-002**: MUST implement checked `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` — all return `Result<SqlInt64, SqlTypeError>` with overflow and divide-by-zero detection
- **FR-003**: MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not` — infallible, with NULL propagation
- **FR-004**: MUST implement SQL comparison methods (`sql_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, `sql_not_equals`) returning `SqlBoolean`
- **FR-005**: MUST implement `Display` (NULL displays as `"Null"`) and `FromStr` (invalid input returns `ParseError`)
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST provide narrowing conversions to `SqlInt32`, `SqlInt16`, and `SqlByte` with range checking (return `Err(Overflow)` if value is out of target range; NULL input returns NULL of target type)
- **FR-008**: MUST provide widening conversion from `SqlBoolean` (TRUE=1, FALSE=0, NULL=NULL)
- **FR-009**: MUST implement `From<i64>`, `Hash`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`
- **FR-010**: NULL propagation MUST apply to all arithmetic, bitwise, and comparison operations — any NULL operand produces a NULL result

### Key Entities

- **SqlInt64**: A nullable 64-bit signed integer. Internal representation: `Option<i64>` where `None` = SQL NULL, `Some(v)` = a value. Fixed-size, stack-allocated.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All overflow boundary conditions tested — `i64::MAX + 1`, `i64::MIN - 1`, `i64::MIN / -1`, `i64::MIN % -1`, `-i64::MIN`, `i64::MAX * 2`
- **SC-002**: Code coverage ≥ 95% for the `SqlInt64` module
- **SC-003**: All six arithmetic operations, four bitwise operations, and six comparison methods have at least one positive test, one overflow/error test (where applicable), and one NULL propagation test
- **SC-004**: `Display` and `FromStr` round-trip correctly for all non-NULL values
- **SC-005**: All narrowing conversions tested at boundary values (max valid, first invalid, negative for unsigned target)

## Assumptions

- Overflow detection uses Rust's `checked_*` methods (idiomatic equivalent of C#'s explicit overflow checks)
- Negation of `i64::MIN` returns `Err(SqlTypeError::Overflow)` — matches correct mathematical behavior
- `i64::MIN % -1` returns `Err(SqlTypeError::Overflow)` — matches C# behavior and avoids Rust's panic on this operation
- Widening conversions to `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney` are deferred until those types are implemented
- Widening conversions FROM `SqlByte`, `SqlInt16`, and `SqlInt32` (`From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>`) and narrowing `to_sql_boolean()` are deferred — will be added in a follow-up or when those types' specs request it
- `PartialOrd` / `Ord` for Rust-level ordering follows standard integer ordering; NULL values are treated as less than all non-NULL values (consistent with Rust convention for `Option`)
