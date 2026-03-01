# Feature Specification: SqlDouble

**Feature Branch**: `feature/sql-double`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDouble`

## Overview

`SqlDouble` represents a SQL Server `FLOAT` — a 64-bit IEEE 754 floating-point number with NULL support. Like `SqlSingle`, NaN and Infinity results are NOT allowed and produce errors.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlDouble::new(3.14159265358979)`, **When** `value()` called, **Then** returns `Ok(3.14159265358979)`
2. **Given** `SqlDouble::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** attempt to create with `f64::NAN`, **Then** returns error
4. **Given** attempt to create with `f64::INFINITY`, **Then** returns error

---

### User Story 2 - Arithmetic rejecting NaN/Infinity (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlDouble(f64::MAX) + SqlDouble(f64::MAX)`, **Then** returns overflow error
2. **Given** `SqlDouble(1.0) / SqlDouble(0.0)`, **Then** returns divide-by-zero error
3. **Given** `SqlDouble(2.5) * SqlDouble(4.0)`, **Then** returns `SqlDouble(10.0)`
4. **Given** any op with NULL, **Then** returns NULL

---

### User Story 3 - Comparison and conversions (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlDouble(1.0).sql_equals(SqlDouble(1.0))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlDouble` from `SqlInt32(42)`, **Then** returns `SqlDouble(42.0)`

---

### Edge Cases

- NaN/Infinity rejection
- Very small/large values near f64 limits
- Conversion from all integer types (implicit widening)

## Requirements

### Functional Requirements

- **FR-001**: `SqlDouble` MUST be `Copy + Clone`, representing `f64` with NULL
- **FR-002**: MUST implement `Add`, `Sub`, `Mul`, `Div` returning `Result`, rejecting NaN/Infinity
- **FR-003**: MUST implement `Neg`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST accept implicit conversions from all integer Sql types and SqlSingle

## Success Criteria

- **SC-001**: NaN/Infinity never produced
- **SC-002**: ≥95% code coverage
