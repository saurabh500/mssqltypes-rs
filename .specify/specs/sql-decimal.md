# Feature Specification: SqlDecimal

**Feature Branch**: `feature/sql-decimal`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDecimal`

## Overview

`SqlDecimal` represents a SQL Server `DECIMAL`/`NUMERIC` type — a fixed-point decimal number with up to 38 digits of precision and configurable scale, with NULL support. Internally stored as four `u32` components plus precision, scale, and sign metadata.

## User Scenarios & Testing

### User Story 1 - Create with precision and scale (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlDecimal::new(10, 2, true, 12345, 0, 0, 0)` (precision=10, scale=2, positive), **When** inspected, **Then** represents `123.45`
2. **Given** `SqlDecimal::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** precision > 38, **Then** returns error
4. **Given** scale > precision, **Then** returns error

---

### User Story 2 - Arithmetic with precision management (Priority: P1)

**Acceptance Scenarios**:

1. **Given** two SqlDecimals added, **When** evaluated, **Then** result has correct precision/scale per SQL Server rules
2. **Given** multiplication, **When** evaluated, **Then** precision = p1 + p2 + 1, scale = s1 + s2 (capped at 38)
3. **Given** division, **When** evaluated, **Then** appropriate scale selected
4. **Given** overflow (result > 38 digits), **Then** returns overflow error
5. **Given** any op with NULL, **Then** returns NULL

---

### User Story 3 - Scale adjustment and rounding (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlDecimal` with scale 4, **When** adjusted to scale 2, **Then** rounds correctly
2. **Given** `SqlDecimal::parse("123.456")`, **When** adjusted to scale 2, **Then** returns `123.46`

---

### User Story 4 - Conversions (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlDecimal`, **When** converted to `f64`, **Then** closest double value
2. **Given** `SqlDecimal`, **When** converted to `i64`, **Then** truncates fractional part
3. **Given** `SqlInt32(42)`, **When** converted to `SqlDecimal`, **Then** precision/scale set appropriately

---

### Edge Cases

- Maximum precision: 38 digits
- Scale = 0 for integer-like decimals
- Negative zero handling
- Very large values using all four u32 components
- Division rounding modes

## Requirements

### Functional Requirements

- **FR-001**: `SqlDecimal` MUST support precision 1–38 and scale 0–precision
- **FR-002**: MUST store value as sign + four `u32` components (128-bit mantissa)
- **FR-003**: MUST implement `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` with precision/scale propagation
- **FR-004**: MUST implement `adjust_scale(scale, round)` for scale adjustment
- **FR-005**: MUST implement comparison returning `SqlBoolean`
- **FR-006**: MUST implement `Display` (decimal string) and `FromStr`
- **FR-007**: MUST provide `precision()`, `scale()`, `sign()`, `is_positive()` accessors
- **FR-008**: MUST provide constants: `NULL`, `MIN_VALUE`, `MAX_VALUE`
- **FR-009**: MUST provide conversions to/from integer types, f64, and string

### Key Entities

- **SqlDecimal**: `Clone` (not `Copy` due to size), fields: status, len, precision, scale, data1–data4

## Success Criteria

- **SC-001**: Precision/scale correctly propagated through all arithmetic
- **SC-002**: Round-trip string parsing verified
- **SC-003**: ≥90% code coverage (complex type)
