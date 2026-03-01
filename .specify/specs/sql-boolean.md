# Feature Specification: SqlBoolean

**Feature Branch**: `feature/sql-boolean`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlBoolean`

## Overview

`SqlBoolean` is a three-state boolean type representing SQL Server's `BIT` type with NULL support. It implements three-valued logic where NULL propagates through logical operations (AND, OR, XOR, NOT).

## User Scenarios & Testing

### User Story 1 - Create and inspect SqlBoolean values (Priority: P1)

A developer needs to create SqlBoolean values representing `true`, `false`, and `NULL`, and inspect their state.

**Acceptance Scenarios**:

1. **Given** `SqlBoolean::TRUE`, **When** `is_true()` called, **Then** returns `true`
2. **Given** `SqlBoolean::FALSE`, **When** `is_false()` called, **Then** returns `true`
3. **Given** `SqlBoolean::NULL`, **When** `is_null()` called, **Then** returns `true`
4. **Given** a `SqlBoolean::TRUE`, **When** `value()` called, **Then** returns `Ok(true)`
5. **Given** a `SqlBoolean::NULL`, **When** `value()` called, **Then** returns `Err(SqlTypeError::NullValue)`

---

### User Story 2 - Three-valued logical operations (Priority: P1)

A developer needs to perform AND, OR, XOR, NOT operations that correctly propagate NULL per SQL semantics.

**Acceptance Scenarios**:

1. **Given** `TRUE & NULL`, **When** evaluated, **Then** result is `NULL`
2. **Given** `FALSE & NULL`, **When** evaluated, **Then** result is `FALSE` (short-circuit)
3. **Given** `TRUE | NULL`, **When** evaluated, **Then** result is `TRUE` (short-circuit)
4. **Given** `FALSE | NULL`, **When** evaluated, **Then** result is `NULL`
5. **Given** `!NULL`, **When** evaluated, **Then** result is `NULL`
6. **Given** `TRUE ^ FALSE`, **When** evaluated, **Then** result is `TRUE`
7. **Given** `TRUE ^ NULL`, **When** evaluated, **Then** result is `NULL`

---

### User Story 3 - Comparison operations (Priority: P2)

Comparison operators return `SqlBoolean` (not `bool`) to support NULL propagation.

**Acceptance Scenarios**:

1. **Given** `TRUE == TRUE`, **When** evaluated, **Then** result is `SqlBoolean::TRUE`
2. **Given** `TRUE == NULL`, **When** evaluated, **Then** result is `SqlBoolean::NULL`
3. **Given** `TRUE != FALSE`, **When** evaluated, **Then** result is `SqlBoolean::TRUE`
4. **Given** `TRUE > FALSE`, **When** evaluated, **Then** result is `SqlBoolean::TRUE`

---

### User Story 4 - Type conversions (Priority: P2)

Convert SqlBoolean to/from other SQL types and Rust primitives.

**Acceptance Scenarios**:

1. **Given** `SqlBoolean::TRUE`, **When** converted to `SqlByte`, **Then** result is `SqlByte(1)`
2. **Given** `SqlBoolean::FALSE`, **When** converted to `SqlInt32`, **Then** result is `SqlInt32(0)`
3. **Given** `SqlBoolean::NULL`, **When** converted to any numeric type, **Then** result is `NULL`
4. **Given** `SqlByte(0)`, **When** converted to `SqlBoolean`, **Then** result is `FALSE`
5. **Given** `SqlByte(nonzero)`, **When** converted to `SqlBoolean`, **Then** result is `TRUE`
6. **Given** string `"true"`, **When** parsed, **Then** result is `SqlBoolean::TRUE`
7. **Given** string `"1"`, **When** parsed, **Then** result is `SqlBoolean::TRUE`

---

### Edge Cases

- Three-valued AND truth table: `FALSE & anything = FALSE`
- Three-valued OR truth table: `TRUE | anything = TRUE`
- Byte representation: `TRUE=1, FALSE=0, NULL=undefined`

## Requirements

### Functional Requirements

- **FR-001**: `SqlBoolean` MUST be a `Copy + Clone` type with three states: True, False, Null
- **FR-002**: `SqlBoolean` MUST implement `BitAnd`, `BitOr`, `BitXor`, `Not` with three-valued logic
- **FR-003**: `SqlBoolean` MUST provide `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than` returning `SqlBoolean`
- **FR-004**: `SqlBoolean` MUST provide `value() -> Result<bool, SqlTypeError>`
- **FR-005**: `SqlBoolean` MUST implement `Display` (outputs "True", "False", or "Null")
- **FR-006**: `SqlBoolean` MUST implement `FromStr` supporting "true", "false", "1", "0" (case-insensitive)
- **FR-007**: `SqlBoolean` MUST provide constants: `TRUE`, `FALSE`, `NULL`, `ZERO`, `ONE`
- **FR-008**: `SqlBoolean` MUST provide `byte_value() -> Result<u8, SqlTypeError>` returning 0 or 1

### Key Entities

- **SqlBoolean**: Internal representation as `u8` (0=Null, 1=False, 2=True) matching C# layout

## Success Criteria

- **SC-001**: All three-valued logic truth tables pass exhaustive testing
- **SC-002**: All type conversions match C# behavior
- **SC-003**: ≥95% code coverage for this type
