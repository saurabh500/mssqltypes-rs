# Feature Specification: SqlInt32

**Feature Branch**: `feature/sql-int32`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlInt32`

## Overview

`SqlInt32` represents a SQL Server `INT` — a signed 32-bit integer (−2,147,483,648 to 2,147,483,647) with NULL support.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt32::new(100000)`, **When** `value()` called, **Then** returns `Ok(100000)`
2. **Given** `SqlInt32::NULL`, **When** `is_null()` called, **Then** returns `true`

---

### User Story 2 - Arithmetic with overflow (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt32(i32::MAX) + SqlInt32(1)`, **Then** overflow error
2. **Given** `SqlInt32(i32::MIN) - SqlInt32(1)`, **Then** overflow error
3. **Given** `SqlInt32(i32::MIN) / SqlInt32(-1)`, **Then** overflow error
4. **Given** `SqlInt32(100_000) * SqlInt32(100_000)`, **Then** overflow error
5. **Given** `SqlInt32(10) / SqlInt32(0)`, **Then** divide-by-zero error
6. **Given** any op with `SqlInt32::NULL`, **Then** returns `SqlInt32::NULL`

---

### User Story 3 - Bitwise and comparison (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlInt32(0xFF00) & SqlInt32(0x0FF0)`, **Then** returns `SqlInt32(0x0F00)`
2. **Given** `SqlInt32(10).sql_equals(SqlInt32(10))`, **Then** returns `SqlBoolean::TRUE`

---

### Edge Cases

- i32::MIN / -1 → overflow
- Negation of i32::MIN → overflow
- Multiplication overflow detected via 64-bit intermediate

## Requirements

### Functional Requirements

- **FR-001**: `SqlInt32` MUST be `Copy + Clone`, representing `i32` with NULL
- **FR-002**: MUST implement checked `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
- **FR-003**: MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST provide widening conversions to SqlInt64, SqlSingle, SqlDouble, SqlDecimal, SqlMoney

## Success Criteria

- **SC-001**: All overflow boundary conditions tested
- **SC-002**: ≥95% code coverage
