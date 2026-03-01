# Feature Specification: SqlInt64

**Feature Branch**: `feature/sql-int64`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlInt64`

## Overview

`SqlInt64` represents a SQL Server `BIGINT` — a signed 64-bit integer (−2^63 to 2^63−1) with NULL support.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt64::new(9_000_000_000)`, **When** `value()` called, **Then** returns `Ok(9_000_000_000)`
2. **Given** `SqlInt64::NULL`, **When** `is_null()` called, **Then** returns `true`

---

### User Story 2 - Arithmetic with overflow (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlInt64(i64::MAX) + SqlInt64(1)`, **Then** overflow error
2. **Given** `SqlInt64(i64::MIN) - SqlInt64(1)`, **Then** overflow error
3. **Given** `SqlInt64(i64::MIN) / SqlInt64(-1)`, **Then** overflow error
4. **Given** `SqlInt64(i64::MAX) * SqlInt64(2)`, **Then** overflow error
5. **Given** `SqlInt64(10) / SqlInt64(0)`, **Then** divide-by-zero error
6. **Given** `-SqlInt64(i64::MIN)`, **Then** overflow error

---

### User Story 3 - Bitwise and comparison (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlInt64(0xFF) & SqlInt64(0x0F)`, **Then** returns `SqlInt64(0x0F)`
2. **Given** `SqlInt64(100).sql_less_than(SqlInt64(200))`, **Then** returns `SqlBoolean::TRUE`

---

### Edge Cases

- Multiplication overflow: tracked via sign analysis (both operands' signs)
- i64::MIN / -1 → overflow
- NULL propagation in all operations

## Requirements

### Functional Requirements

- **FR-001**: `SqlInt64` MUST be `Copy + Clone`, representing `i64` with NULL
- **FR-002**: MUST implement checked `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
- **FR-003**: MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`

## Success Criteria

- **SC-001**: All overflow cases tested exhaustively
- **SC-002**: ≥95% code coverage
