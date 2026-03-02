# Feature Specification: SqlString

**Feature Branch**: `013-sql-string`
**Created**: 2026-03-02
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlString` — a nullable string with configurable comparison options (case sensitivity, binary sort)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect string values (Priority: P1)

A developer constructs `SqlString` values from Rust strings, inspects their contents, checks for NULL, and retrieves the stored string value and its length.

**Why this priority**: Construction and inspection are the foundation — all other string operations require reliable creation and access.

**Independent Test**: Create strings from `&str` and `String`; verify `value()`, `is_null()`, `len()` return correct results.

**Acceptance Scenarios**:

1. **Given** `SqlString::new("hello")`, **When** `value()` called, **Then** returns `Ok("hello")`
2. **Given** `SqlString::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlString::NULL`, **When** `value()` called, **Then** returns `Err(SqlTypeError::NullValue)`
4. **Given** `SqlString::new("hello")`, **When** `len()` called, **Then** returns `Ok(5)`
5. **Given** `SqlString::new("")`, **When** `is_null()` called, **Then** returns `false` (empty is not NULL)
6. **Given** `SqlString::new("")`, **When** `len()` called, **Then** returns `Ok(0)`
7. **Given** `SqlString::new("hello")` with default options, **Then** compare options are `IgnoreCase`

---

### User Story 2 - Concatenation (Priority: P1)

Developers concatenate two `SqlString` values using the `+` operator to produce a new string containing the characters from both operands. NULL propagation applies.

**Why this priority**: Concatenation is the primary string manipulation operation and critical for building dynamic strings.

**Independent Test**: Concatenate string values; verify result; test NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlString("hello") + SqlString(" world")`, **Then** returns `SqlString("hello world")`
2. **Given** `SqlString("hello") + SqlString::NULL`, **Then** returns `SqlString::NULL`
3. **Given** `SqlString::NULL + SqlString("world")`, **Then** returns `SqlString::NULL`
4. **Given** `SqlString::NULL + SqlString::NULL`, **Then** returns `SqlString::NULL`
5. **Given** `SqlString("") + SqlString("hello")`, **Then** returns `SqlString("hello")`

---

### User Story 3 - Comparison with configurable options (Priority: P2)

Developers compare string values using SQL comparison methods that respect configurable comparison options. The default comparison is case-insensitive. Binary sort compares bytes directly. Options are set per-instance.

**Why this priority**: Configurable comparison is what distinguishes `SqlString` from a plain `Option<String>`, but construction and concatenation are needed first.

**Independent Test**: Compare string pairs with different comparison options; verify SqlBoolean results; test NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlString("ABC")` and `SqlString("abc")` with default options (IgnoreCase), **When** `sql_equals()` called, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlString("ABC")` and `SqlString("abc")` with `BinarySort`, **When** `sql_equals()` called, **Then** returns `SqlBoolean::FALSE`
3. **Given** `SqlString("apple")` and `SqlString("banana")` with `IgnoreCase`, **When** `sql_less_than()` called, **Then** returns `SqlBoolean::TRUE`
4. **Given** any comparison with NULL, **Then** returns `SqlBoolean::NULL`
5. **Given** `SqlString("hello")` with `IgnoreCase` and `SqlString("hello")` with `BinarySort`, **When** `sql_equals()` called, **Then** comparison uses the options of the left operand
6. **Given** `SqlString("A")` and `SqlString("a")` with `BinarySort`, **When** `sql_less_than()` called, **Then** returns `SqlBoolean::TRUE` (uppercase 'A' < lowercase 'a' in ASCII/UTF-8)

---

### User Story 4 - Construction with explicit compare options (Priority: P2)

Developers create `SqlString` values with explicit comparison options to control how the string participates in comparisons.

**Why this priority**: Explicit options are needed for full SQL compatibility but default options cover most use cases.

**Independent Test**: Create strings with different compare options; verify options are stored and used in comparisons.

**Acceptance Scenarios**:

1. **Given** `SqlString::with_options("hello", SqlCompareOptions::None)`, **When** compared with `SqlString("HELLO")`, **Then** returns `SqlBoolean::FALSE` (case-sensitive)
2. **Given** `SqlString::with_options("hello", SqlCompareOptions::IgnoreCase)`, **When** compared with `SqlString("HELLO")`, **Then** returns `SqlBoolean::TRUE`
3. **Given** `SqlString::with_options("hello", SqlCompareOptions::BinarySort)`, **When** compared, **Then** uses byte-by-byte ordering

---

### User Story 5 - Display and parsing (Priority: P2)

Developers display string values for output and parse strings from input.

**Why this priority**: Display and parsing are essential for I/O but built on top of core string operations.

**Independent Test**: Format string values; parse strings; verify round-trip.

**Acceptance Scenarios**:

1. **Given** `SqlString("hello")`, **When** `Display` is called, **Then** outputs `"hello"`
2. **Given** `SqlString::NULL`, **When** `Display` is called, **Then** outputs `"Null"`
3. **Given** string `"hello"`, **When** `FromStr` is called, **Then** returns `SqlString("hello")` with default compare options
4. **Given** string `"Null"` (case-insensitive), **When** `FromStr` is called, **Then** returns `SqlString::NULL`

---

### Edge Cases

- Empty string is valid (not NULL)
- Default comparison options: `IgnoreCase` (matching C# SqlString default behavior)
- When comparing two SqlStrings with different compare options, the left operand's options take precedence (matching C# behavior)
- `SqlCompareOptions::None` means case-sensitive, culture-insensitive comparison (ordinal)
- `BinarySort` compares raw UTF-8 bytes
- `BinarySort2` is equivalent to `BinarySort` in this implementation (no legacy distinction needed)
- Unicode strings are fully supported via Rust's native UTF-8 `String`
- `SqlString` is `Clone` but NOT `Copy` (contains heap-allocated `String`)
- `len()` returns byte length (consistent with Rust `String::len()`), not character count

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlString` MUST be `Clone`, storing `Option<String>` plus `SqlCompareOptions` internally
- **FR-002**: MUST provide `SqlString::NULL` constant
- **FR-003**: MUST provide `new(&str) -> SqlString` constructor with default `IgnoreCase` compare options
- **FR-004**: MUST provide `with_options(&str, SqlCompareOptions) -> SqlString` constructor
- **FR-005**: MUST provide `is_null() -> bool` and `value() -> Result<&str, SqlTypeError>`
- **FR-006**: MUST provide `len() -> Result<usize, SqlTypeError>` returning byte length
- **FR-007**: MUST implement concatenation via `Add` operator with NULL propagation; result inherits left operand's compare options
- **FR-008**: MUST implement SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean`
- **FR-009**: SQL comparisons MUST respect the left operand's `SqlCompareOptions`
- **FR-010**: `IgnoreCase` comparison MUST use case-insensitive ASCII comparison (via `to_ascii_lowercase()`)
- **FR-011**: `BinarySort` / `BinarySort2` comparison MUST compare raw UTF-8 bytes
- **FR-012**: `None` comparison MUST use case-sensitive ordinal comparison
- **FR-013**: Any comparison involving NULL MUST return `SqlBoolean::NULL`
- **FR-014**: MUST implement `Display` outputting the string value, NULL as `"Null"`
- **FR-015**: MUST implement `FromStr` parsing strings, `"Null"` (case-insensitive) returning `SqlString::NULL`
- **FR-016**: MUST define `SqlCompareOptions` enum with variants: `None`, `IgnoreCase`, `BinarySort`, `BinarySort2`
- **FR-017**: MUST implement `PartialEq`/`Eq` using case-insensitive comparison (matching default options and C# SqlString.Equals behavior)
- **FR-018**: MUST implement `Hash` consistent with `Eq`
- **FR-019**: MUST implement `PartialOrd`/`Ord` using case-insensitive ordering (NULL < non-NULL)

### Key Entities

- **SqlString**: Nullable string value. Internal representation: `Option<String>` + `SqlCompareOptions`. Not `Copy` (heap-allocated).
- **SqlCompareOptions**: Enum controlling comparison behavior. Variants: `None` (case-sensitive ordinal), `IgnoreCase` (case-insensitive ASCII), `BinarySort` (raw byte comparison), `BinarySort2` (identical to `BinarySort`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Case-insensitive comparison verified with at least 5 test pairs (mixed case, identical, different)
- **SC-002**: Binary sort comparison verified to differ from case-insensitive for mixed-case inputs
- **SC-003**: NULL propagation verified for all 6 SQL comparison methods
- **SC-004**: Concatenation verified with at least 5 test cases including NULL propagation and empty values
- **SC-005**: All tests pass with `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt` clean
- **SC-006**: >=95% code coverage on `SqlString` and `SqlCompareOptions` modules

## Assumptions

- Case-insensitive comparison uses ASCII case folding (`to_ascii_lowercase()`) — no full Unicode case folding (matching SQL Server's typical behavior for non-Unicode types)
- `BinarySort2` is functionally identical to `BinarySort` in this Rust implementation (C# distinguishes them for legacy compatibility; we don't need that distinction)
- `len()` returns byte length, not character count (consistent with Rust conventions)
- Default compare options are `IgnoreCase` (matching C# `SqlString` defaults)
- When two SqlStrings with different compare options are compared, the left operand's options govern
- No locale/collation support beyond case-insensitive and binary sort — full ICU collation is out of scope
