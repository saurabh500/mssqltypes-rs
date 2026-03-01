# Feature Specification: SqlByte

**Feature Branch**: `feature/sql-byte`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlByte`

## Overview

`SqlByte` represents a SQL Server `TINYINT` — an unsigned 8-bit integer (0–255) with NULL support. All arithmetic operations check for overflow and return errors instead of wrapping.

## User Scenarios & Testing

### User Story 1 - Create and inspect SqlByte values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlByte::new(42)`, **When** `value()` called, **Then** returns `Ok(42)`
2. **Given** `SqlByte::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlByte::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`

---

### User Story 2 - Arithmetic operations with overflow detection (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlByte(200) + SqlByte(100)`, **When** evaluated, **Then** returns overflow error
2. **Given** `SqlByte(10) + SqlByte(20)`, **When** evaluated, **Then** returns `SqlByte(30)`
3. **Given** `SqlByte(5) - SqlByte(10)`, **When** evaluated, **Then** returns overflow error
4. **Given** `SqlByte(10) / SqlByte(0)`, **When** evaluated, **Then** returns divide-by-zero error
5. **Given** `SqlByte(10) % SqlByte(3)`, **When** evaluated, **Then** returns `SqlByte(1)`
6. **Given** `SqlByte(15) * SqlByte(20)`, **When** evaluated, **Then** returns overflow error
7. **Given** any op with `SqlByte::NULL`, **When** evaluated, **Then** returns `SqlByte::NULL`

---

### User Story 3 - Bitwise operations (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlByte(0xFF) & SqlByte(0x0F)`, **When** evaluated, **Then** returns `SqlByte(0x0F)`
2. **Given** `SqlByte(0xF0) | SqlByte(0x0F)`, **When** evaluated, **Then** returns `SqlByte(0xFF)`
3. **Given** `SqlByte(0xFF) ^ SqlByte(0x0F)`, **When** evaluated, **Then** returns `SqlByte(0xF0)`
4. **Given** `!SqlByte(0x0F)`, **When** evaluated, **Then** returns `SqlByte(0xF0)`

---

### User Story 4 - Comparison and conversions (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlByte(10)` and `SqlByte(20)`, **When** compared, **Then** `sql_less_than` returns `SqlBoolean::TRUE`
2. **Given** string `"123"`, **When** parsed, **Then** returns `SqlByte(123)`
3. **Given** string `"256"`, **When** parsed, **Then** returns error
4. **Given** `SqlByte(100)`, **When** converted to `SqlInt16`, **Then** returns `SqlInt16(100)`

---

### Edge Cases

- `SqlByte::MIN_VALUE` = 0, `SqlByte::MAX_VALUE` = 255
- `0 - 1` → overflow error (unsigned)
- `255 + 1` → overflow error
- NULL propagation in all operations

## Requirements

### Functional Requirements

- **FR-001**: `SqlByte` MUST be `Copy + Clone`, representing `u8` with NULL
- **FR-002**: MUST implement `Add`, `Sub`, `Mul`, `Div`, `Rem` returning `Result<SqlByte, SqlTypeError>` with overflow checking
- **FR-003**: MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display` and `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST provide conversions to/from wider integer types

### Key Entities

- **SqlByte**: Internal `Option<u8>` (None = NULL)

## Success Criteria

- **SC-001**: All arithmetic overflow cases detected and return errors
- **SC-002**: All boundary values (0, 255) tested
- **SC-003**: ≥95% code coverage
