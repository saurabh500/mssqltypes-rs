# Feature Specification: SqlString

**Feature Branch**: `feature/sql-string`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlString`

## Overview

`SqlString` represents SQL Server `CHAR`/`VARCHAR`/`NCHAR`/`NVARCHAR` types — a string value with NULL support and configurable comparison options (case sensitivity, locale, binary sort). In the Rust implementation, we simplify locale handling to focus on the core comparison behaviors.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlString::new("hello")`, **When** `value()` called, **Then** returns `Ok("hello")`
2. **Given** `SqlString::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlString::new("hello")`, **When** `len()` called, **Then** returns `5`

---

### User Story 2 - Concatenation (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlString("hello") + SqlString(" world")`, **Then** returns `SqlString("hello world")`
2. **Given** `SqlString("hello") + SqlString::NULL`, **Then** returns `SqlString::NULL`

---

### User Story 3 - Comparison with options (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlString("ABC")` and `SqlString("abc")` with `IgnoreCase`, **When** compared, **Then** equal
2. **Given** `SqlString("ABC")` and `SqlString("abc")` with `BinarySort`, **When** compared, **Then** not equal
3. **Given** any comparison with NULL, **Then** returns `SqlBoolean::NULL`

---

### User Story 4 - Parsing and display (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlString("123")`, **When** displayed, **Then** outputs `"123"`
2. **Given** string `"hello"`, **When** parsed, **Then** returns `SqlString("hello")`

---

### Edge Cases

- Empty string is valid (not NULL)
- Default comparison: case-insensitive
- Binary sort: byte-by-byte comparison
- Unicode handling via Rust's native UTF-8

## Requirements

### Functional Requirements

- **FR-001**: `SqlString` MUST be `Clone`, wrapping `Option<String>`
- **FR-002**: MUST implement concatenation via `Add`
- **FR-003**: MUST support `SqlCompareOptions`: `None`, `IgnoreCase`, `BinarySort`, `BinarySort2`
- **FR-004**: MUST implement comparison returning `SqlBoolean`, respecting compare options
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide `len()`, `value() -> Result<&str, SqlTypeError>`
- **FR-007**: MUST provide `SqlString::NULL`
- **FR-008**: MUST support construction with explicit compare options

### Key Entities

- **SqlString**: `Option<String>` + `SqlCompareOptions`
- **SqlCompareOptions**: Enum/flags for comparison behavior

## Success Criteria

- **SC-001**: Case-insensitive comparison works correctly
- **SC-002**: Binary sort comparison matches byte ordering
- **SC-003**: ≥95% code coverage
