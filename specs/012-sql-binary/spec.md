# Feature Specification: SqlBinary

**Feature Branch**: `012-sql-binary`
**Created**: 2026-03-02
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlBinary` — a nullable variable-length byte sequence with trailing-zero-padded comparison

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect binary values (Priority: P1)

A developer constructs `SqlBinary` values from byte vectors, inspects their contents, checks for NULL, and retrieves the length and individual bytes.

**Why this priority**: Construction and inspection are the foundation — all other binary operations require reliable creation and access.

**Independent Test**: Create binaries from byte vectors; verify `value()`, `is_null()`, `len()`, and indexing return correct results.

**Acceptance Scenarios**:

1. **Given** `SqlBinary::new(vec![1, 2, 3])`, **When** `value()` called, **Then** returns `Ok(&[1, 2, 3])`
2. **Given** `SqlBinary::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlBinary::NULL`, **When** `value()` called, **Then** returns `Err(SqlTypeError::NullValue)`
4. **Given** `SqlBinary::new(vec![1, 2, 3])`, **When** `len()` called, **Then** returns `3`
5. **Given** `SqlBinary::new(vec![10, 20, 30])`, **When** indexed at position `1`, **Then** returns `Ok(20)`
6. **Given** `SqlBinary::new(vec![10, 20, 30])`, **When** indexed at position `5`, **Then** returns `Err(SqlTypeError::OutOfRange)`
7. **Given** `SqlBinary::new(vec![])`, **When** `is_null()` called, **Then** returns `false` (empty is not NULL)
8. **Given** `SqlBinary::new(vec![])`, **When** `len()` called, **Then** returns `0`

---

### User Story 2 - Concatenation (Priority: P1)

Developers concatenate two `SqlBinary` values using the `+` operator to produce a new byte sequence containing all bytes from both operands. NULL propagation applies.

**Why this priority**: Concatenation is the core operation for building binary data, analogous to string concatenation.

**Independent Test**: Concatenate binary values; verify result bytes; test NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlBinary([1,2]) + SqlBinary([3,4])`, **Then** returns `SqlBinary([1,2,3,4])`
2. **Given** `SqlBinary([1,2]) + SqlBinary::NULL`, **Then** returns `SqlBinary::NULL`
3. **Given** `SqlBinary::NULL + SqlBinary([3,4])`, **Then** returns `SqlBinary::NULL`
4. **Given** `SqlBinary::NULL + SqlBinary::NULL`, **Then** returns `SqlBinary::NULL`
5. **Given** `SqlBinary([]) + SqlBinary([1,2])`, **Then** returns `SqlBinary([1,2])`

---

### User Story 3 - Comparison with trailing-zero padding (Priority: P2)

Developers compare binary values using SQL comparison methods. Shorter values are logically padded with trailing zeros before comparison, matching C# `SqlBinary` behavior.

**Why this priority**: Comparison semantics are important for correctness but less commonly used than construction and concatenation.

**Independent Test**: Compare pairs of binary values with different lengths; verify trailing-zero padding semantics; test NULL propagation.

**Acceptance Scenarios**:

1. **Given** `SqlBinary([1,2])` and `SqlBinary([1,2,0,0])`, **When** `sql_equals()` called, **Then** returns `SqlBoolean::TRUE` (trailing zeros match)
2. **Given** `SqlBinary([1,2])` and `SqlBinary([1,3])`, **When** `sql_less_than()` called, **Then** returns `SqlBoolean::TRUE`
3. **Given** `SqlBinary([1,2,1])` and `SqlBinary([1,2])`, **When** `sql_greater_than()` called, **Then** returns `SqlBoolean::TRUE` (extra non-zero byte)
4. **Given** any comparison with NULL, **Then** returns `SqlBoolean::NULL`
5. **Given** two empty binaries, **When** `sql_equals()` called, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlBinary([0])` and `SqlBinary([])`, **When** `sql_equals()` called, **Then** returns `SqlBoolean::TRUE` (trailing zero padding)

---

### User Story 4 - Display (Priority: P2)

Developers display binary values as hex strings for debugging and logging.

**Why this priority**: Display is essential for debugging but not for core binary manipulation.

**Independent Test**: Format binary values; verify hex output.

**Acceptance Scenarios**:

1. **Given** `SqlBinary([0x0A, 0xFF])`, **When** `Display` is called, **Then** outputs `"0aff"` (lowercase hex with no separators)
2. **Given** `SqlBinary::NULL`, **When** `Display` is called, **Then** outputs `"Null"`
3. **Given** `SqlBinary([])`, **When** `Display` is called, **Then** outputs `""` (empty string)

---

### Edge Cases

- Empty binary (`vec![]`) is valid, not NULL
- Trailing-zero padding for comparison: `[1,2]` equals `[1,2,0,0]`
- Indexing out of bounds returns `Err(SqlTypeError::OutOfRange)`, not panic
- `value()` returns a borrowed slice `&[u8]` (zero-copy access)
- `SqlBinary` is `Clone` but NOT `Copy` (contains `Vec<u8>`)
- `Eq`/`PartialEq` should use trailing-zero-padded semantics (consistent with SQL comparisons)
- `Hash` must be consistent with `Eq` — normalize trailing zeros before hashing
- `Ord`/`PartialOrd` should use trailing-zero-padded byte ordering

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlBinary` MUST be `Clone`, storing `Option<Vec<u8>>` internally
- **FR-002**: MUST provide `SqlBinary::NULL` constant
- **FR-003**: MUST provide `new(Vec<u8>) -> SqlBinary` constructor
- **FR-004**: MUST provide `is_null() -> bool` and `value() -> Result<&[u8], SqlTypeError>`
- **FR-005**: MUST provide `len() -> Result<usize, SqlTypeError>` returning byte count
- **FR-006**: MUST provide indexed access `get(usize) -> Result<u8, SqlTypeError>` returning error for out-of-bounds or NULL
- **FR-007**: MUST implement concatenation via `Add` operator with NULL propagation
- **FR-008**: MUST implement SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean`
- **FR-009**: SQL comparisons MUST use trailing-zero-padded semantics (shorter values padded with zeros to match length)
- **FR-010**: Any comparison involving NULL MUST return `SqlBoolean::NULL`
- **FR-011**: MUST implement `Display` outputting lowercase hex (no separators), NULL as `"Null"`
- **FR-012**: MUST implement `PartialEq`/`Eq` using trailing-zero-padded semantics (consistent with SQL comparisons)
- **FR-013**: MUST implement `Hash` consistent with `Eq` (normalize trailing zeros)
- **FR-014**: MUST implement `PartialOrd`/`Ord` using trailing-zero-padded byte ordering (NULL < non-NULL)

### Key Entities

- **SqlBinary**: Nullable variable-length byte sequence. Internal representation: `Option<Vec<u8>>`. Not `Copy` (heap-allocated).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Trailing-zero-padded comparison verified with at least 5 test pairs of different lengths
- **SC-002**: Concatenation verified with at least 4 test cases including NULL propagation and empty values
- **SC-003**: NULL propagation verified for all 6 SQL comparison methods
- **SC-004**: Indexing boundary conditions verified (valid, out-of-bounds, NULL)
- **SC-005**: All tests pass with `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt` clean
- **SC-006**: >=95% code coverage on `SqlBinary` module

## Assumptions

- Display uses lowercase hex with no separators or prefix (e.g., `"0aff"` not `"0x0AFF"`)
- `Eq` uses trailing-zero-padded comparison to stay consistent with SQL semantics
- `value()` returns a borrowed `&[u8]` slice for zero-copy access
- Empty binary is a valid value (zero-length), not NULL
