# Feature Specification: SqlGuid

**Feature Branch**: `011-sql-guid`
**Created**: 2026-03-02
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlGuid` — a nullable 128-bit GUID with SQL Server's non-standard comparison byte ordering

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect GUID values (Priority: P1)

A developer constructs `SqlGuid` values from byte arrays, string representations, or component parts. They inspect the stored GUID, check for NULL, and retrieve the raw 16-byte representation.

**Why this priority**: Construction and inspection are the foundation — no other functionality works without reliable GUID creation and retrieval.

**Independent Test**: Create GUIDs from bytes, strings, and components; verify `value()`, `is_null()`, `to_byte_array()` return correct results.

**Acceptance Scenarios**:

1. **Given** `SqlGuid::new([u8; 16])`, **When** `value()` called, **Then** returns `Ok(&[u8; 16])` matching input bytes
2. **Given** `SqlGuid::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlGuid::NULL`, **When** `value()` called, **Then** returns `Err(SqlTypeError::NullValue)`
4. **Given** `SqlGuid::parse("6F9619FF-8B86-D011-B42D-00CF4FC964FF")`, **When** `value()` called, **Then** returns the correctly decoded 16-byte array
5. **Given** `SqlGuid::new(bytes)`, **When** `to_byte_array()` called, **Then** returns the same 16-byte array
6. **Given** `SqlGuid::parse("6f9619ff-8B86-D011-b42d-00CF4FC964FF")` (mixed case), **When** parsed, **Then** succeeds with same result as uppercase

---

### User Story 2 - SQL Server comparison ordering (Priority: P1)

SQL Server compares GUIDs using a non-standard byte order: bytes are compared in the order `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`. This is critical for SQL Server compatibility and must be replicated exactly.

**Why this priority**: Incorrect GUID ordering breaks SQL Server sort compatibility — this is the defining behavioral requirement of `SqlGuid`.

**Independent Test**: Compare pairs of GUIDs that differ in specific byte positions and verify the comparison result matches SQL Server's byte ordering.

**Acceptance Scenarios**:

1. **Given** two GUIDs differing only in byte 10, **When** SQL compared, **Then** byte 10 determines order
2. **Given** two GUIDs differing in byte 0 and byte 10, **When** SQL compared, **Then** byte 10 takes priority (it's compared first)
3. **Given** two GUIDs differing only in byte 3, **When** SQL compared, **Then** byte 3 determines order (last group)
4. **Given** any SQL comparison with NULL, **Then** returns `SqlBoolean::NULL`
5. **Given** two identical GUIDs, **When** `sql_equals()` called, **Then** returns `SqlBoolean::TRUE`
6. **Given** NULL compared with NULL, **When** `sql_equals()` called, **Then** returns `SqlBoolean::NULL`

---

### User Story 3 - Display and parsing (Priority: P2)

Developers display GUIDs in standard `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` format and parse GUIDs from string input, including case-insensitive hex.

**Why this priority**: String representation is essential for debugging, logging, and interoperability, but not needed for core GUID operations.

**Independent Test**: Format a GUID to string; parse strings (uppercase, lowercase, mixed); verify round-trip fidelity.

**Acceptance Scenarios**:

1. **Given** a `SqlGuid` with known bytes, **When** `Display` is called, **Then** outputs `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` format (lowercase hex)
2. **Given** `SqlGuid::NULL`, **When** `Display` is called, **Then** outputs `"Null"`
3. **Given** string `"6F9619FF-8B86-D011-B42D-00CF4FC964FF"`, **When** `FromStr` is called, **Then** returns correctly parsed `SqlGuid`
4. **Given** string `"null"` (case-insensitive), **When** `FromStr` is called, **Then** returns `SqlGuid::NULL`
5. **Given** an invalid string like `"not-a-guid"`, **When** `FromStr` is called, **Then** returns `Err(SqlTypeError::ParseError)`
6. **Given** a GUID, **When** formatted and re-parsed, **Then** the result is equal to the original

---

### User Story 4 - Conversions to/from SqlBinary (Priority: P2)

Developers convert between `SqlGuid` and `SqlBinary` for byte-level interoperability. A `SqlGuid` converts to a 16-byte `SqlBinary`, and a 16-byte `SqlBinary` converts back to `SqlGuid`.

**Why this priority**: Cross-type conversion is needed for binary protocol handling but depends on `SqlBinary` existing.

**Independent Test**: Convert `SqlGuid` -> `SqlBinary`, verify 16 bytes; convert 16-byte `SqlBinary` -> `SqlGuid`, verify equality; test NULL propagation.

**Acceptance Scenarios**:

1. **Given** a `SqlGuid`, **When** `to_sql_binary()` called, **Then** returns `SqlBinary` with 16 bytes matching the GUID
2. **Given** `SqlGuid::NULL`, **When** `to_sql_binary()` called, **Then** returns `SqlBinary::NULL`
3. **Given** a `SqlBinary` with exactly 16 bytes, **When** `SqlGuid::from_sql_binary()` called, **Then** returns the correct `SqlGuid`
4. **Given** a `SqlBinary` with fewer or more than 16 bytes, **When** `SqlGuid::from_sql_binary()` called, **Then** returns `Err(SqlTypeError::ParseError)`
5. **Given** `SqlBinary::NULL`, **When** `SqlGuid::from_sql_binary()` called, **Then** returns `Ok(SqlGuid::NULL)`

---

### Edge Cases

- SQL Server comparison byte order `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` is critical — standard byte-by-byte comparison is WRONG
- GUID string format must support both uppercase and lowercase hex characters
- All-zeros GUID (`00000000-0000-0000-0000-000000000000`) is valid, not NULL
- `Ord`/`PartialOrd` should use SQL Server byte ordering (for use in `BTreeMap` etc.)
- `Eq`/`PartialEq` compares all 16 bytes directly (byte equality, no ordering concerns)
- `Hash` must be consistent with `Eq`
- `SqlGuid` should be `Copy + Clone` since it is a fixed 16-byte value

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlGuid` MUST be `Copy + Clone`, storing `Option<[u8; 16]>` internally
- **FR-002**: MUST provide `SqlGuid::NULL` constant
- **FR-003**: MUST provide `new([u8; 16]) -> SqlGuid` constructor
- **FR-004**: MUST provide `is_null() -> bool` and `value() -> Result<&[u8; 16], SqlTypeError>`
- **FR-005**: MUST provide `to_byte_array() -> Result<[u8; 16], SqlTypeError>`
- **FR-006**: MUST implement SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean`
- **FR-007**: SQL comparisons MUST use SQL Server's non-standard byte order: `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`
- **FR-008**: Any comparison involving NULL MUST return `SqlBoolean::NULL`
- **FR-009**: MUST implement `Display` outputting standard GUID format `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` (lowercase hex), NULL as `"Null"`
- **FR-010**: MUST implement `FromStr` with case-insensitive hex parsing, `"Null"` returning `SqlGuid::NULL`
- **FR-011**: MUST implement `PartialEq`/`Eq` based on byte equality (NULL == NULL for Rust trait purposes)
- **FR-012**: MUST implement `Hash` consistent with `Eq`
- **FR-013**: MUST implement `PartialOrd`/`Ord` using SQL Server byte ordering (NULL < non-NULL)
- **FR-014**: MUST provide `to_sql_binary() -> SqlBinary` conversion
- **FR-015**: MUST provide `from_sql_binary(SqlBinary) -> Result<SqlGuid, SqlTypeError>` requiring exactly 16 bytes

### Key Entities

- **SqlGuid**: Nullable 128-bit GUID. Internal representation: `Option<[u8; 16]>`. Fixed size, `Copy`.
- **SQL Server comparison order**: Constant array `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` defining the byte comparison sequence.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: SQL Server comparison ordering verified with known test vectors (at least 6 pairs testing each byte-group boundary)
- **SC-002**: Round-trip string parsing verified (`Display` -> `FromStr` -> equality check) for at least 5 distinct GUIDs
- **SC-003**: NULL propagation verified for all 6 SQL comparison methods
- **SC-004**: Conversion to/from `SqlBinary` round-trips correctly
- **SC-005**: All tests pass with `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt` clean
- **SC-006**: >=95% code coverage on `SqlGuid` module

## Assumptions

- GUID string format uses lowercase hex for Display output (standard convention)
- `Ord` implementation uses SQL Server byte ordering for consistency with SQL comparisons
- `SqlBinary` type exists (or will exist) for conversion methods — if not yet available, conversion methods can be added later
- No external UUID crate dependency — pure Rust byte-array implementation
