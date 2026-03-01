# Feature Specification: SqlSingle

**Feature Branch**: `feature/sql-single`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlSingle`

## Overview

`SqlSingle` represents a SQL Server `REAL` — a 32-bit IEEE 754 floating-point number with NULL support. Unlike standard Rust `f32`, NaN and Infinity results are NOT allowed and produce errors.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlSingle::new(3.14)`, **When** `value()` called, **Then** returns `Ok(3.14)`
2. **Given** `SqlSingle::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** attempt to create with `f32::NAN`, **Then** returns error
4. **Given** attempt to create with `f32::INFINITY`, **Then** returns error

---

### User Story 2 - Arithmetic rejecting NaN/Infinity (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlSingle(f32::MAX) + SqlSingle(f32::MAX)`, **Then** returns overflow error
2. **Given** `SqlSingle(1.0) / SqlSingle(0.0)`, **Then** returns divide-by-zero error
3. **Given** `SqlSingle(2.5) + SqlSingle(3.5)`, **Then** returns `SqlSingle(6.0)`
4. **Given** any op with NULL, **Then** returns NULL

---

### User Story 3 - Comparison (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlSingle(1.0).sql_equals(SqlSingle(1.0))`, **Then** returns `SqlBoolean::TRUE`
2. **Given** `SqlSingle(1.0).sql_less_than(SqlSingle(2.0))`, **Then** returns `SqlBoolean::TRUE`

---

### Edge Cases

- NaN/Infinity rejection on construction and after arithmetic
- Negation: `-SqlSingle(x)` = `SqlSingle(-x)`
- Parse from string: `"3.14"` → `SqlSingle(3.14)`

## Requirements

### Functional Requirements

- **FR-001**: `SqlSingle` MUST be `Copy + Clone`, representing `f32` with NULL
- **FR-002**: MUST implement `Add`, `Sub`, `Mul`, `Div` returning `Result`, rejecting NaN/Infinity results
- **FR-003**: MUST implement `Neg`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display`, `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST reject NaN and Infinity on construction

## Success Criteria

- **SC-001**: NaN/Infinity never produced — always detected and returned as error
- **SC-002**: ≥95% code coverage
