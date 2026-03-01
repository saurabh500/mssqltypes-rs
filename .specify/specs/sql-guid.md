# Feature Specification: SqlGuid

**Feature Branch**: `feature/sql-guid`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlGuid`

## Overview

`SqlGuid` represents a SQL Server `UNIQUEIDENTIFIER` type — a 128-bit GUID with NULL support. Critically, SQL Server uses a NON-STANDARD byte ordering for GUID comparisons (bytes: 10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3), which this implementation MUST replicate.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlGuid::new(uuid_bytes)`, **When** `value()` called, **Then** returns the GUID bytes
2. **Given** `SqlGuid::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** `SqlGuid::parse("6F9619FF-8B86-D011-B42D-00CF4FC964FF")`, **Then** parsed correctly

---

### User Story 2 - SQL Server comparison ordering (Priority: P1)

The comparison order MUST use SQL Server's non-standard byte ordering:
bytes compared in order: 10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3

**Acceptance Scenarios**:

1. **Given** two GUIDs differing only in byte 10, **When** compared, **Then** byte 10 determines order
2. **Given** two GUIDs differing in byte 0 and byte 10, **When** compared, **Then** byte 10 takes priority
3. **Given** comparison with NULL, **Then** returns `SqlBoolean::NULL`

---

### User Story 3 - Conversions (Priority: P2)

**Acceptance Scenarios**:

1. **Given** `SqlGuid`, **When** `to_byte_array()` called, **Then** returns 16-byte array
2. **Given** `SqlGuid`, **When** converted to `SqlBinary`, **Then** returns correct bytes
3. **Given** `SqlBinary` of 16 bytes, **When** converted to `SqlGuid`, **Then** correct GUID

---

### Edge Cases

- Non-standard comparison byte order is critical for SQL Server compatibility
- GUID string format: `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`
- Case-insensitive hex parsing

## Requirements

### Functional Requirements

- **FR-001**: `SqlGuid` MUST be `Copy + Clone`, storing 16 bytes with NULL
- **FR-002**: MUST implement comparison using SQL Server byte order: [10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]
- **FR-003**: MUST implement `Display` (standard GUID format) and `FromStr`
- **FR-004**: MUST provide `to_byte_array()` returning `[u8; 16]`
- **FR-005**: MUST provide conversions to/from `SqlBinary`
- **FR-006**: MUST provide `SqlGuid::NULL`
- **FR-007**: MUST support construction from string, byte array, and component parts

## Success Criteria

- **SC-001**: SQL Server comparison ordering verified with known test vectors
- **SC-002**: Round-trip string parsing verified
- **SC-003**: ≥95% code coverage
