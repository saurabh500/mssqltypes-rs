# Feature Specification: SqlInt16

**Feature Branch**: `feature/sql-int16`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlInt16`

## Overview

`SqlInt16` represents a SQL Server `SMALLINT` — a signed 16-bit integer (−32,768 to 32,767) with NULL support. All arithmetic operations detect overflow.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt16::new(1000)`, **When** `value()` called, **Then** returns `Ok(1000)`
2. **Given** `SqlInt16::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlInt16::new(-32768)`, **When** `value()` called, **Then** returns `Ok(-32768)`

---

### User Story 2 - Arithmetic with overflow (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt16(32767) + SqlInt16(1)`, **Then** returns overflow error
2. **Given** `SqlInt16(-32768) - SqlInt16(1)`, **Then** returns overflow error
3. **Given** `SqlInt16(-32768) / SqlInt16(-1)`, **Then** returns overflow error
4. **Given** `SqlInt16(100) * SqlInt16(400)`, **Then** returns overflow error
5. **Given** `SqlInt16(10) / SqlInt16(0)`, **Then** returns divide-by-zero error
6. **Given** `SqlInt16(7) % SqlInt16(3)`, **Then** returns `SqlInt16(1)`
7. **Given** `-SqlInt16(-32768)`, **Then** returns overflow error

---

### User Story 3 - Bitwise and comparison (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlInt16(0xFF) & SqlInt16(0x0F)`, **Then** returns `SqlInt16(0x0F)`
2. **Given** `SqlInt16(10) < SqlInt16(20)`, **Then** returns `SqlBoolean::TRUE`
3. **Given** `SqlInt16(10) == SqlInt16::NULL`, **Then** returns `SqlBoolean::NULL`

---

### Edge Cases

- MIN_VALUE / -1 → overflow (because |MIN_VALUE| > MAX_VALUE)
- Negation of MIN_VALUE → overflow
- NULL propagates through all operations

## Requirements

### Functional Requirements

- **FR-001**: `SqlInt16` MUST be `Copy + Clone`, representing `i16` with NULL
- **FR-002**: MUST implement checked `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
- **FR-003**: MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST provide widening conversions to SqlInt32, SqlInt64, SqlSingle, SqlDouble, SqlDecimal, SqlMoney

## Success Criteria

- **SC-001**: Overflow detected at all boundaries
- **SC-002**: ≥95% code coverage
