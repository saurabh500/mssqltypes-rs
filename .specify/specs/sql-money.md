# Feature Specification: SqlMoney

**Feature Branch**: `feature/sql-money`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlMoney`

## Overview

`SqlMoney` represents a SQL Server `MONEY` type — a fixed-point currency value with 4 decimal places of precision, stored internally as a scaled `i64` (value × 10,000). Range: −922,337,203,685,477.5808 to 922,337,203,685,477.5807.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlMoney::from_decimal(123.4567)`, **When** `value()` called, **Then** returns the decimal representation
2. **Given** `SqlMoney::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** value exceeding range, **Then** returns overflow error

---

### User Story 2 - Arithmetic with overflow detection (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlMoney(MAX) + SqlMoney(1)`, **Then** overflow error
2. **Given** `SqlMoney(100.00) + SqlMoney(50.25)`, **Then** returns `SqlMoney(150.25)`
3. **Given** `SqlMoney(100.00) * SqlMoney(2.00)`, **Then** returns `SqlMoney(200.00)`
4. **Given** `SqlMoney(100.00) / SqlMoney(0.00)`, **Then** divide-by-zero error
5. **Given** any op with NULL, **Then** returns NULL

---

### User Story 3 - Conversions and display (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlMoney(123.4567)`, **When** displayed, **Then** shows `"123.4567"`
2. **Given** `SqlMoney`, **When** converted to `SqlDecimal`, **Then** has scale 4
3. **Given** `SqlInt64(100)`, **When** converted to `SqlMoney`, **Then** returns `SqlMoney(100.0000)`

---

### Edge Cases

- Internal representation: `i64` scaled by 10,000
- Precision is always 4 decimal places
- Rounding on construction from floating-point
- Exact range boundaries

## Requirements

### Functional Requirements

- **FR-001**: `SqlMoney` MUST be `Copy + Clone`, internally `i64` scaled by 10,000
- **FR-002**: MUST implement `Add`, `Sub`, `Mul`, `Div` with overflow detection
- **FR-003**: MUST implement `Neg`
- **FR-004**: MUST implement comparison returning `SqlBoolean`
- **FR-005**: MUST implement `Display` (always 4 decimal places) and `FromStr`
- **FR-006**: MUST provide constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
- **FR-007**: MUST provide `to_decimal()`, `to_i64()`, `to_f64()` conversions
- **FR-008**: MUST provide constructors from `i32`, `i64`, `f64` with range checking

## Success Criteria

- **SC-001**: Internal i64 representation preserves exact 4-decimal-place precision
- **SC-002**: ≥95% code coverage
