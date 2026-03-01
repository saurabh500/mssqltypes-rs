# Feature Specification: SqlBinary

**Feature Branch**: `feature/sql-binary`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlBinary`

## Overview

`SqlBinary` represents a SQL Server `BINARY`/`VARBINARY` type — a variable-length byte sequence with NULL support. Comparison uses byte-by-byte ordering with shorter values padded with zeros.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlBinary::new(vec![1, 2, 3])`, **When** `value()` called, **Then** returns `Ok(&[1, 2, 3])`
2. **Given** `SqlBinary::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlBinary::new(vec![1, 2, 3])`, **When** `len()` called, **Then** returns `3`
4. **Given** `SqlBinary::new(vec![1, 2, 3])`, **When** indexed at `1`, **Then** returns `2`

---

### User Story 2 - Concatenation (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlBinary([1,2]) + SqlBinary([3,4])`, **Then** returns `SqlBinary([1,2,3,4])`
2. **Given** `SqlBinary([1,2]) + SqlBinary::NULL`, **Then** returns `SqlBinary::NULL`

---

### User Story 3 - Comparison with zero-padding (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlBinary([1,2])` and `SqlBinary([1,2,0,0])`, **When** compared, **Then** they are equal (trailing zero padding)
2. **Given** `SqlBinary([1,2])` and `SqlBinary([1,3])`, **When** compared, **Then** first is less
3. **Given** any comparison with NULL, **Then** returns `SqlBoolean::NULL`

---

### Edge Cases

- Empty binary: `SqlBinary(vec![])` is valid, not NULL
- Trailing zero comparison semantics
- Deep copy on `value()` access (ownership returned)
- Indexing out of bounds returns error

## Requirements

### Functional Requirements

- **FR-001**: `SqlBinary` MUST be `Clone`, wrapping `Option<Vec<u8>>`
- **FR-002**: MUST implement concatenation via `Add`
- **FR-003**: MUST implement comparison with trailing-zero padding
- **FR-004**: MUST provide `len()`, indexing, `value()` returning `&[u8]`
- **FR-005**: MUST implement `Display` (hex string) and conversion to/from `SqlGuid`
- **FR-006**: MUST provide `SqlBinary::NULL`

## Success Criteria

- **SC-001**: Trailing-zero comparison matches C# behavior
- **SC-002**: ≥95% code coverage
