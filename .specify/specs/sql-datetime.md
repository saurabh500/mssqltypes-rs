# Feature Specification: SqlDateTime

**Feature Branch**: `feature/sql-datetime`
**Created**: 2026-03-01
**Status**: Draft
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDateTime`

## Overview

`SqlDateTime` represents a SQL Server `DATETIME` type — a date/time value stored as days since 1900-01-01 and time as 1/300-second ticks. Range: 1753-01-01 to 9999-12-31. Time precision: ~3.33 milliseconds.

## User Scenarios & Testing

### User Story 1 - Create and inspect values (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlDateTime::new(2026, 3, 1, 12, 30, 0)`, **When** inspected, **Then** day_ticks and time_ticks are correct
2. **Given** `SqlDateTime::NULL`, **When** `is_null()` called, **Then** returns `true`
3. **Given** date before 1753-01-01, **Then** returns out-of-range error
4. **Given** date after 9999-12-31, **Then** returns out-of-range error

---

### User Story 2 - Date arithmetic (Priority: P1)

**Acceptance Scenarios**:

1. **Given** `SqlDateTime` + duration of 1 day, **Then** day_ticks incremented by 1
2. **Given** `SqlDateTime` - duration of 1 hour, **Then** time_ticks decremented correctly
3. **Given** any op with NULL, **Then** returns NULL

---

### User Story 3 - Comparison (Priority: P2)

**Acceptance Scenarios**:

1. **Given** two SqlDateTimes for same date, different time, **When** compared, **Then** ordered by time
2. **Given** two SqlDateTimes for different dates, **When** compared, **Then** ordered by date first
3. **Given** comparison with NULL, **Then** returns `SqlBoolean::NULL`

---

### User Story 4 - Time precision (Priority: P2)

**Acceptance Scenarios**:

1. **Given** time with milliseconds, **When** stored, **Then** rounded to nearest 1/300 second (~3.33ms)
2. **Given** `SqlDateTime` with time 12:00:00.005, **When** stored, **Then** rounded appropriately

---

### Edge Cases

- 1900-01-01 = day 0
- Dates before 1900-01-01 have negative day_ticks
- Leap year handling
- Time wrapping at midnight
- 1/300 second precision loss

## Requirements

### Functional Requirements

- **FR-001**: `SqlDateTime` MUST be `Copy + Clone`, storing `i32` day_ticks + `i32` time_ticks
- **FR-002**: MUST validate date range: 1753-01-01 to 9999-12-31
- **FR-003**: MUST store time as 1/300 second units (0 to 25,919,999)
- **FR-004**: MUST round time to nearest 1/300 second on construction
- **FR-005**: MUST implement comparison returning `SqlBoolean`
- **FR-006**: MUST implement `Display` (ISO 8601 format) and `FromStr`
- **FR-007**: MUST provide `day_ticks()`, `time_ticks()` accessors
- **FR-008**: MUST provide constants: `NULL`, `MIN_VALUE`, `MAX_VALUE`
- **FR-009**: MUST support addition/subtraction with durations

## Success Criteria

- **SC-001**: Date range boundaries tested
- **SC-002**: 1/300 second rounding verified against C# behavior
- **SC-003**: Leap year handling correct
- **SC-004**: ≥95% code coverage
