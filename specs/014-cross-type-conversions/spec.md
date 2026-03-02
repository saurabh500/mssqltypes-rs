# Feature Specification: Cross-Type Conversions

**Feature Branch**: `014-cross-type-conversions`
**Created**: 2026-03-02
**Status**: Draft
**Input**: Deferred cross-type conversion methods across all `System.Data.SqlTypes` equivalents. All 13 SQL types are now implemented; this feature wires up the remaining conversions that were deferred until target types existed.

## Overview

All 13 SQL types (`SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney`, `SqlBinary`, `SqlString`, `SqlDateTime`, `SqlGuid`) are implemented. During initial implementation, conversions to types that didn't yet exist were deferred. This feature completes the remaining 43 cross-type conversion methods following C# `System.Data.SqlTypes` conversion semantics.

Widening (lossless) conversions are infallible and use `From` trait implementations. Narrowing (lossy or range-restricted) conversions return `Result<T, SqlTypeError>`. `SqlString` serves as the universal interchange hub — every type converts to/from string. NULL propagation is universal: NULL input always produces NULL output.

## User Scenarios & Testing *(mandatory)*
-->

### User Story 1 — Widening Integer Conversions (Priority: P1)

A developer widens smaller integer types into larger ones without data loss. These conversions are infallible (except NULL propagation).

**Why this priority**: Widening conversions are the most common cross-type operation and are needed for arithmetic between mixed-width integers.

**Independent Test**: Convert `SqlByte(100)` → `SqlInt32`, verify `value() == 100`. Convert `SqlInt16(1000)` → `SqlInt64`, verify `value() == 1000`. Convert NULL → NULL.

**Acceptance Scenarios**:

1. **Given** `SqlByte(200)`, **When** converted to `SqlInt32`, **Then** returns `SqlInt32(200)`
2. **Given** `SqlByte(200)`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64(200)`
3. **Given** `SqlInt16(30000)`, **When** converted to `SqlInt32`, **Then** returns `SqlInt32(30000)`
4. **Given** `SqlInt16(30000)`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64(30000)`
5. **Given** `SqlInt32(2_000_000)`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64(2_000_000)`
6. **Given** `SqlByte::NULL`, **When** converted to `SqlInt32`, **Then** returns `SqlInt32::NULL`
7. **Given** `SqlInt16::NULL`, **When** converted to `SqlInt64`, **Then** returns `SqlInt64::NULL`

---

### User Story 2 — Boolean ↔ Numeric Conversions (Priority: P1)

A developer converts between `SqlBoolean` and numeric types. Non-zero values become `TRUE`; zero becomes `FALSE`. `TRUE` converts to `1`, `FALSE` to `0`.

**Why this priority**: Boolean/numeric interop is essential for SQL Server compatibility where `BIT` columns frequently interact with integer expressions.

**Independent Test**: Convert `SqlInt32(42)` → `SqlBoolean::TRUE`. Convert `SqlInt32(0)` → `SqlBoolean::FALSE`. Convert `SqlInt64::NULL` → `SqlBoolean::NULL`.

**Acceptance Scenarios**:

1. **Given** `SqlInt32(0)`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::FALSE`
2. **Given** `SqlInt32(42)`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::TRUE`
3. **Given** `SqlInt32(-1)`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::TRUE`
4. **Given** `SqlInt32::NULL`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::NULL`
5. **Given** `SqlInt64(0)`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::FALSE`
6. **Given** `SqlInt64(1)`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlInt64::NULL`, **When** `to_sql_boolean()` called, **Then** returns `SqlBoolean::NULL`

---

### User Story 3 — Float ↔ Float Conversions (Priority: P1)

A developer converts between `SqlSingle` and `SqlDouble`. `SqlSingle` → `SqlDouble` is widening (lossless). `SqlDouble` → `SqlSingle` is narrowing (can overflow or lose precision).

**Why this priority**: Float interop is fundamental for mixed-precision numeric expressions.

**Independent Test**: Convert `SqlSingle(3.14)` → `SqlDouble`, verify value. Convert `SqlDouble(1e300)` → `SqlSingle`, verify overflow error.

**Acceptance Scenarios**:

1. **Given** `SqlSingle(3.14)`, **When** `from_sql_single()` called on `SqlDouble`, **Then** returns `SqlDouble` with widened value
2. **Given** `SqlSingle::NULL`, **When** `from_sql_single()` called on `SqlDouble`, **Then** returns `SqlDouble::NULL`
3. **Given** `SqlDouble(3.14)`, **When** `to_sql_single()` called, **Then** returns `Ok(SqlSingle)` with narrowed value
4. **Given** `SqlDouble(1e300)`, **When** `to_sql_single()` called, **Then** returns `Err(SqlTypeError::Overflow)` (out of `f32` range)
5. **Given** `SqlDouble::NULL`, **When** `to_sql_single()` called, **Then** returns `SqlSingle::NULL`

---

### User Story 4 — SqlString ↔ All Types (Priority: P2)

A developer converts any SQL type to/from `SqlString`. `to_sql_string()` uses the type's display formatting. Parsing from `SqlString` uses the type's string parsing logic.

**Why this priority**: String is the universal interchange format in SQL. All C# SqlTypes support string conversion.

**Independent Test**: Convert `SqlInt32(42)` → `SqlString("42")`. Parse `SqlString("42")` → `SqlInt32(42)`. Convert NULL ↔ NULL.

**Acceptance Scenarios**:

1. **Given** `SqlBoolean::TRUE`, **When** `to_sql_string()` called, **Then** returns `SqlString("True")`
2. **Given** `SqlInt32(42)`, **When** `to_sql_string()` called, **Then** returns `SqlString("42")`
3. **Given** `SqlInt64::NULL`, **When** `to_sql_string()` called, **Then** returns `SqlString::NULL`
4. **Given** `SqlDouble(3.14)`, **When** `to_sql_string()` called, **Then** returns `SqlString("3.14")`
5. **Given** `SqlString("42")`, **When** `to_sql_int32()` called, **Then** returns `Ok(SqlInt32(42))`
6. **Given** `SqlString("true")`, **When** `to_sql_boolean()` called, **Then** returns `Ok(SqlBoolean::TRUE)`
7. **Given** `SqlString("not_a_number")`, **When** `to_sql_int32()` called, **Then** returns `Err(SqlTypeError::ParseError)`
8. **Given** `SqlString::NULL`, **When** `to_sql_int32()` called, **Then** returns `Ok(SqlInt32::NULL)`
9. **Given** `SqlDateTime`, **When** `to_sql_string()` called, **Then** returns formatted date string
10. **Given** `SqlGuid`, **When** `to_sql_string()` called, **Then** returns hyphenated GUID string
11. **Given** `SqlString("6f9619ff-8b86-d011-b42d-00cf4fc964ff")`, **When** `to_sql_guid()` called, **Then** returns correct `SqlGuid`

---

### User Story 5 — SqlDecimal ↔ Float/Money Conversions (Priority: P2)

A developer converts between `SqlDecimal` and `SqlSingle`/`SqlDouble`/`SqlMoney`. Float→Decimal is explicit (precision concerns). Decimal→Float is explicit (precision loss). Decimal↔Money is safe for common ranges.

**Why this priority**: Decimal/float/money interop is needed for financial calculations that mix precision levels.

**Independent Test**: Convert `SqlDecimal(100)` → `SqlDouble(100.0)`. Convert `SqlSingle(3.14)` → `SqlDecimal`. Convert `SqlMoney` ↔ `SqlDecimal`. NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDecimal` value `100.50`, **When** `to_sql_double()` called, **Then** returns `SqlDouble(100.50)`
2. **Given** `SqlDecimal` value `100.50`, **When** `to_sql_single()` called, **Then** returns `SqlSingle(100.50)`
3. **Given** `SqlDecimal` value `100.5000`, **When** `to_sql_money()` called, **Then** returns `SqlMoney(100.5000)`
4. **Given** `SqlDouble(3.14)`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` approximation
5. **Given** `SqlSingle(3.14)`, **When** converted to `SqlDecimal`, **Then** returns `SqlDecimal` approximation
6. **Given** `SqlDecimal::NULL`, **When** `to_sql_double()` called, **Then** returns `SqlDouble::NULL`
7. **Given** `SqlMoney` value, **When** converted to `SqlDecimal`, **Then** preserves 4-decimal-place value exactly

---

### User Story 6 — SqlMoney ↔ Float Conversions (Priority: P2)

A developer converts between `SqlMoney` and floating-point types. These are all narrowing/explicit conversions with potential precision loss.

**Why this priority**: Money/float interop is needed but involves precision trade-offs that require explicit handling.

**Independent Test**: Convert `SqlMoney(100.50)` → `SqlDouble(100.50)`. Convert `SqlDouble(100.50)` → `SqlMoney`.

**Acceptance Scenarios**:

1. **Given** `SqlMoney` value `100.50`, **When** `to_sql_single()` called, **Then** returns `SqlSingle(100.50)`
2. **Given** `SqlMoney` value `100.50`, **When** `to_sql_double()` called, **Then** returns `SqlDouble(100.50)`
3. **Given** `SqlSingle(99.99)`, **When** converted to `SqlMoney`, **Then** returns `SqlMoney` with 4-decimal scale
4. **Given** `SqlDouble(99.99)`, **When** converted to `SqlMoney`, **Then** returns `SqlMoney` with 4-decimal scale
5. **Given** `SqlMoney::NULL`, **When** `to_sql_single()` called, **Then** returns `SqlSingle::NULL`
6. **Given** `SqlDouble` value exceeding `SqlMoney` range, **When** converted, **Then** returns `Err(SqlTypeError::Overflow)`

---

### User Story 7 — SqlDateTime ↔ SqlString (Priority: P3)

A developer converts between `SqlDateTime` and `SqlString` for formatted date interchange.

**Why this priority**: DateTime/String conversion is useful but less common than numeric conversions.

**Independent Test**: Convert `SqlDateTime(2025, 1, 15, ...)` → string. Parse string back. NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlDateTime`, **When** `to_sql_string()` called, **Then** returns formatted date string as `SqlString`
2. **Given** `SqlString` with valid date, **When** `SqlDateTime::from_sql_string()` called, **Then** returns parsed `SqlDateTime`
3. **Given** `SqlString` with invalid date, **When** `SqlDateTime::from_sql_string()` called, **Then** returns `Err(SqlTypeError::ParseError)`
4. **Given** `SqlDateTime::NULL`, **When** `to_sql_string()` called, **Then** returns `SqlString::NULL`
5. **Given** `SqlString::NULL`, **When** `SqlDateTime::from_sql_string()` called, **Then** returns `Ok(SqlDateTime::NULL)`

---

### Edge Cases

- **Overflow on narrowing**: `SqlInt64(i64::MAX)` → `SqlInt32` must return `Err(Overflow)`, never panic
- **NaN/Infinity rejection**: `SqlDouble(f64::NAN)` → `SqlDecimal` must return an error
- **Money range**: `SqlDouble(1e18)` → `SqlMoney` may overflow the internal storage and must return `Err(Overflow)`
- **Boolean non-zero**: Any non-zero value → `TRUE`, only exact zero → `FALSE`
- **String parsing errors**: Invalid strings → `Err(ParseError)`, never panic
- **NULL propagation is universal**: NULL input always → NULL output (or `Ok(T::NULL)` for fallible conversions)

---

## Requirements *(mandatory)*

### Functional Requirements

#### Widening Integer (infallible, `From` impls)

- **FR-001**: `From<SqlByte> for SqlInt32` MUST be implemented — NULL propagates
- **FR-002**: `From<SqlInt16> for SqlInt32` MUST be implemented — NULL propagates
- **FR-003**: `From<SqlByte> for SqlInt64` MUST be implemented — NULL propagates
- **FR-004**: `From<SqlInt16> for SqlInt64` MUST be implemented — NULL propagates
- **FR-005**: `From<SqlInt32> for SqlInt64` MUST be implemented — NULL propagates

#### Boolean ↔ Numeric (narrowing)

- **FR-006**: `SqlInt32` MUST provide `to_sql_boolean()` — non-zero → TRUE, zero → FALSE, NULL → NULL
- **FR-007**: `SqlInt64` MUST provide `to_sql_boolean()` — non-zero → TRUE, zero → FALSE, NULL → NULL

#### Float ↔ Float

- **FR-008**: `SqlDouble` MUST provide `from_sql_single(SqlSingle)` — widening, NULL propagates
- **FR-009**: `SqlDouble` MUST provide `to_sql_single()` returning a result — narrowing, overflow check

#### SqlDecimal ↔ Float

- **FR-010**: `From<SqlSingle> for SqlDecimal` MUST be implemented — NULL propagates, reject NaN/Infinity
- **FR-011**: `From<SqlDouble> for SqlDecimal` MUST be implemented — NULL propagates, reject NaN/Infinity
- **FR-012**: `SqlDecimal` MUST provide `to_sql_single()` — NULL propagates
- **FR-013**: `SqlDecimal` MUST provide `to_sql_double()` — NULL propagates

#### SqlDecimal ↔ SqlMoney

- **FR-014**: `From<SqlMoney> for SqlDecimal` MUST be implemented — NULL propagates, preserves 4-decimal scale
- **FR-015**: `SqlDecimal` MUST provide `to_sql_money()` returning a result — NULL propagates, range check

#### SqlMoney ↔ Float

- **FR-016**: `SqlMoney` MUST provide `from_sql_single()` or equivalent — NULL propagates, range check
- **FR-017**: `SqlMoney` MUST provide `from_sql_double()` or equivalent — NULL propagates, range check
- **FR-018**: `SqlMoney` MUST provide `to_sql_single()` — NULL propagates
- **FR-019**: `SqlMoney` MUST provide `to_sql_double()` — NULL propagates

#### SqlString → All Types (parsing)

- **FR-020**: `SqlString` MUST provide `to_sql_boolean()` returning a result with parse error handling
- **FR-021**: `SqlString` MUST provide `to_sql_byte()` returning a result with parse error handling
- **FR-022**: `SqlString` MUST provide `to_sql_int16()` returning a result with parse error handling
- **FR-023**: `SqlString` MUST provide `to_sql_int32()` returning a result with parse error handling
- **FR-024**: `SqlString` MUST provide `to_sql_int64()` returning a result with parse error handling
- **FR-025**: `SqlString` MUST provide `to_sql_single()` returning a result with parse error handling
- **FR-026**: `SqlString` MUST provide `to_sql_double()` returning a result with parse error handling
- **FR-027**: `SqlString` MUST provide `to_sql_decimal()` returning a result with parse error handling
- **FR-028**: `SqlString` MUST provide `to_sql_money()` returning a result with parse error handling
- **FR-029**: `SqlString` MUST provide `to_sql_date_time()` returning a result with parse error handling
- **FR-030**: `SqlString` MUST provide `to_sql_guid()` returning a result with parse error handling

#### All Types → SqlString (formatting)

- **FR-031**: `SqlBoolean` MUST provide `to_sql_string()` — uses display format, NULL → NULL
- **FR-032**: `SqlByte` MUST provide `to_sql_string()` — NULL → NULL
- **FR-033**: `SqlInt16` MUST provide `to_sql_string()` — NULL → NULL
- **FR-034**: `SqlInt32` MUST provide `to_sql_string()` — NULL → NULL
- **FR-035**: `SqlInt64` MUST provide `to_sql_string()` — NULL → NULL
- **FR-036**: `SqlSingle` MUST provide `to_sql_string()` — NULL → NULL
- **FR-037**: `SqlDouble` MUST provide `to_sql_string()` — NULL → NULL
- **FR-038**: `SqlDecimal` MUST provide `to_sql_string()` — NULL → NULL
- **FR-039**: `SqlMoney` MUST provide `to_sql_string()` — NULL → NULL
- **FR-040**: `SqlDateTime` MUST provide `to_sql_string()` — NULL → NULL
- **FR-041**: `SqlGuid` MUST provide `to_sql_string()` — NULL → NULL

#### SqlDateTime ↔ SqlString

- **FR-042**: `SqlDateTime` MUST provide `from_sql_string(SqlString)` returning a result — parses date string
- **FR-043**: `SqlDateTime` MUST provide `to_sql_string()` — formats as date string (covered by FR-040)

### Non-Functional Requirements

- **NFR-001**: No `unsafe` code in any conversion implementation
- **NFR-002**: No new external dependencies added to the core library
- **NFR-003**: All conversions MUST propagate NULL → NULL
- **NFR-004**: Narrowing conversions MUST return results with overflow/parse errors — never panic
- **NFR-005**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` MUST all pass
- **NFR-006**: Each conversion method MUST have test coverage (at least 3 tests: normal value, NULL, and edge case)

### Key Entities

- **Widening conversion**: Lossless, infallible — implemented as `From<Source> for Target`; the source type's full range fits within the target type's range
- **Narrowing conversion**: Lossy or range-restricted — implemented as fallible methods returning results with overflow or parse errors
- **SqlString hub**: Universal text interchange — `to_sql_string()` on every type for formatting, `SqlString::to_sql_*()` methods for parsing back to typed values

---

## Audit: Existing vs Missing Conversions

### Conversion Matrix (rows = source, columns = target)

Legend: **E** = Exists, **M** = Missing (in scope), **—** = same type, **N/A** = not applicable

| Source ↓ \ Target → | Bool | Byte | I16  | I32   | I64   | Single | Double | Decimal | Money | String | DateTime | Guid | Binary |
| ------------------- | ---- | ---- | ---- | ----- | ----- | ------ | ------ | ------- | ----- | ------ | -------- | ---- | ------ |
| **SqlBoolean**      | —    | E    | E    | E     | E     | E      | E      | E       | E     | **M**  | N/A      | N/A  | N/A    |
| **SqlByte**         | E    | —    | E    | **M** | **M** | E      | E      | E       | E     | **M**  | N/A      | N/A  | N/A    |
| **SqlInt16**        | E    | E    | —    | **M** | **M** | E      | E      | E       | E     | **M**  | N/A      | N/A  | N/A    |
| **SqlInt32**        | **M** | E   | E    | —     | **M** | E      | E      | E       | E     | **M**  | N/A      | N/A  | N/A    |
| **SqlInt64**        | **M** | E   | E    | E     | —     | E      | E      | E       | E     | **M**  | N/A      | N/A  | N/A    |
| **SqlSingle**       | E    | N/A  | N/A  | N/A   | N/A   | —      | E      | **M**   | **M** | **M**  | N/A      | N/A  | N/A    |
| **SqlDouble**       | E    | N/A  | N/A  | N/A   | N/A   | **M**  | —      | **M**   | **M** | **M**  | N/A      | N/A  | N/A    |
| **SqlDecimal**      | E    | E    | E    | E     | E     | **M**  | **M**  | —       | **M** | **M**  | N/A      | N/A  | N/A    |
| **SqlMoney**        | E    | E    | E    | E     | E     | **M**  | **M**  | E       | —     | **M**  | N/A      | N/A  | N/A    |
| **SqlString**       | **M** | **M** | **M** | **M** | **M** | **M** | **M** | **M** | **M** | —    | **M**    | **M** | N/A   |
| **SqlDateTime**     | N/A  | N/A  | N/A  | N/A   | N/A   | N/A    | N/A    | N/A     | N/A   | **M**  | —        | N/A  | N/A    |
| **SqlGuid**         | N/A  | N/A  | N/A  | N/A   | N/A   | N/A    | N/A    | N/A     | N/A   | **M**  | N/A      | —    | E      |

### Summary of Missing Conversions: 43 methods

| Category             | Count | Requirements     |
| -------------------- | ----- | ---------------- |
| Widening integer     | 5     | FR-001 – FR-005  |
| to_sql_boolean()     | 2     | FR-006, FR-007   |
| Float ↔ Float        | 2     | FR-008, FR-009   |
| Decimal ↔ Float      | 4     | FR-010 – FR-013  |
| Decimal ↔ Money      | 2     | FR-014, FR-015   |
| Money ↔ Float        | 4     | FR-016 – FR-019  |
| SqlString → types    | 11    | FR-020 – FR-030  |
| Types → SqlString    | 11    | FR-031 – FR-041  |
| DateTime ↔ SqlString | 2     | FR-042, FR-043   |

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 43 missing conversion methods are implemented with correct NULL propagation
- **SC-002**: Each conversion method has at least 3 tests: normal value, NULL, and edge case (overflow/parse error where applicable)
- **SC-003**: Widening conversions are infallible (no error returns possible for valid non-NULL input)
- **SC-004**: Narrowing conversions return results with appropriate error variants and never panic
- **SC-005**: All existing tests continue to pass (no regressions)
- **SC-006**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` all pass
- **SC-007**: No new `unsafe` code and no new external dependencies

---

## Assumptions

- All 13 SQL types are implemented and stable
- Display and string parsing are already implemented on all types — `to_sql_string()` delegates to display formatting, `SqlString::to_sql_*()` delegates to string parsing
- Narrowing float→integer conversions are out of scope (e.g., `SqlDouble` → `SqlInt32`); these exist in C# but are complex and rarely used — can be added in a follow-up
- Chrono crate integration for `SqlDateTime` is out of scope (feature-flagged item for later)
- Serde support is out of scope (separate feature-flagged item)
- `SqlBinary::from_hex()` is out of scope (convenience method, not a C# conversion)
- `SqlGuid` braced/parenthesized parsing and component constructor are out of scope (separate feature)
