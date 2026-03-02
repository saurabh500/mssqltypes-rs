# Feature Specification: SqlDateTime

**Feature Branch**: `010-sql-datetime`  
**Created**: 2025-07-17  
**Status**: Draft  
**Input**: Rust equivalent of C# `System.Data.SqlTypes.SqlDateTime` — a date/time value stored as days since 1900-01-01 and time as 1/300-second ticks, with range 1753-01-01 to 9999-12-31, NULL support, duration arithmetic, and SQL three-valued comparison logic

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and inspect values from calendar components (Priority: P1)

A library consumer creates `SqlDateTime` values from year, month, day, hour, minute, second, and optional millisecond components, then inspects them. Construction validates that the date falls within the SQL Server `DATETIME` range (1753-01-01 to 9999-12-31) and that all time components are within valid bounds. Milliseconds are rounded to the nearest 1/300-second SQL tick (~3.33ms precision).

**Why this priority**: Core value construction from calendar components is the most natural way to create date/time values. Without this, the type cannot be populated with meaningful data.

**Independent Test**: Can be fully tested by constructing values with calendar components and calling accessors. Delivers the ability to represent SQL DATETIME values safely in Rust.

**Acceptance Scenarios**:

1. **Given** construction with (2025, 7, 17, 12, 30, 0, 0.0), **When** inspected, **Then** day_ticks and time_ticks are correctly computed
2. **Given** construction with (1900, 1, 1, 0, 0, 0, 0.0), **When** `day_ticks()` called, **Then** returns `0` (epoch date)
3. **Given** construction with (1899, 12, 31, 0, 0, 0, 0.0), **When** `day_ticks()` called, **Then** returns `-1` (one day before epoch)
4. **Given** construction with (1753, 1, 1, 0, 0, 0, 0.0), **When** inspected, **Then** day_ticks is `-53690` (MinDay)
5. **Given** construction with (9999, 12, 31, 23, 59, 59, 997.0), **When** inspected, **Then** day_ticks is `2958463` (MaxDay) and time_ticks is `25919999` (MaxTime)
6. **Given** construction with year `1752`, **Then** returns out-of-range error (before minimum date)
7. **Given** construction with year `10000`, **Then** returns out-of-range error (after maximum date)
8. **Given** construction with month `0` or `13`, **Then** returns out-of-range error
9. **Given** construction with day `0` or `32`, **Then** returns out-of-range error
10. **Given** construction with hour `24`, minute `60`, or second `60`, **Then** returns out-of-range error
11. **Given** construction with millisecond `1000.0` or negative, **Then** returns out-of-range error

---

### User Story 2 - Create and inspect values from raw ticks (Priority: P1)

A library consumer creates `SqlDateTime` values directly from raw day_ticks and time_ticks, and inspects them using accessors. This is the low-level constructor for interop and deserialization. The NULL sentinel is also created this way.

**Why this priority**: Raw tick construction is essential for deserialization from wire protocols and internal conversions. Combined with Story 1, this completes the construction surface.

**Independent Test**: Can be fully tested by constructing values with raw ticks and calling `day_ticks()`, `time_ticks()`, `is_null()`, and `value()`.

**Acceptance Scenarios**:

1. **Given** `SqlDateTime::from_ticks(0, 0)`, **When** inspected, **Then** represents 1900-01-01 00:00:00.000
2. **Given** `SqlDateTime::from_ticks(-53690, 0)`, **When** inspected, **Then** represents 1753-01-01 (MinValue)
3. **Given** `SqlDateTime::from_ticks(2958463, 25919999)`, **When** inspected, **Then** represents 9999-12-31 23:59:59.997 (MaxValue)
4. **Given** `SqlDateTime::NULL`, **When** `is_null()` called, **Then** returns `true`
5. **Given** `SqlDateTime::NULL`, **When** `value()` called, **Then** returns `Err(NullValue)`
6. **Given** `SqlDateTime::MIN_VALUE`, **When** `day_ticks()` called, **Then** returns `-53690`
7. **Given** `SqlDateTime::MAX_VALUE`, **When** `time_ticks()` called, **Then** returns `25919999`
8. **Given** `SqlDateTime::from_ticks(-53691, 0)`, **Then** returns out-of-range error (day below MinDay)
9. **Given** `SqlDateTime::from_ticks(2958464, 0)`, **Then** returns out-of-range error (day above MaxDay)
10. **Given** `SqlDateTime::from_ticks(0, -1)`, **Then** returns out-of-range error (negative time ticks)
11. **Given** `SqlDateTime::from_ticks(0, 25920000)`, **Then** returns out-of-range error (time ticks >= TicksPerDay)

---

### User Story 3 - Millisecond rounding to 1/300-second precision (Priority: P1)

A library consumer creates `SqlDateTime` values with millisecond precision. The milliseconds are rounded to the nearest SQL Server tick (1/300 of a second ≈ 3.333ms). This rounding matches C#'s formula: `time_ticks = (int)(millisecond * 0.3 + 0.5)`. When rounding causes time to overflow midnight, the day rolls forward by one.

**Why this priority**: Millisecond rounding is a fundamental invariant of the SQL DATETIME type. Getting it wrong produces data mismatches between Rust and SQL Server.

**Independent Test**: Can be fully tested by constructing values with various millisecond values and verifying the resulting time_ticks match the C# rounding formula.

**Acceptance Scenarios**:

1. **Given** construction with millisecond `0.0`, **Then** adds 0 ticks to time component (formula: `(int)(0.0 * 0.3 + 0.5) = 0`)
2. **Given** construction with millisecond `3.33`, **Then** rounds to 1 SQL tick
3. **Given** construction with millisecond `500.0`, **Then** rounds correctly per formula
4. **Given** construction with time 23:59:59 and millisecond 998.0, **When** rounding causes time_ticks to exceed 25919999, **Then** time resets to 0 and day increments by 1
5. **Given** construction with time 23:59:59 and millisecond 998.0 on December 31, 9999, **When** day rollover would exceed MaxDay, **Then** returns out-of-range error

---

### User Story 4 - Duration arithmetic (Priority: P1)

A library consumer adds or subtracts a duration (days and/or time offset) to a `SqlDateTime`. Only duration-based arithmetic is supported — you cannot add or subtract two `SqlDateTime` values directly. The result must remain within the valid range. NULL propagates through all arithmetic.

**Why this priority**: Duration arithmetic is the primary operation for date manipulation — adjusting dates by intervals is a core use case.

**Independent Test**: Can be fully tested by adding/subtracting durations and verifying the resulting `SqlDateTime` is correct.

**Acceptance Scenarios**:

1. **Given** `SqlDateTime(2025, 1, 15, 12, 0, 0)` + duration of 1 day, **Then** returns `SqlDateTime(2025, 1, 16, 12, 0, 0)`
2. **Given** `SqlDateTime(2025, 1, 15, 12, 0, 0)` - duration of 1 day, **Then** returns `SqlDateTime(2025, 1, 14, 12, 0, 0)`
3. **Given** `SqlDateTime(2025, 1, 15, 23, 0, 0)` + duration of 2 hours, **Then** returns `SqlDateTime(2025, 1, 16, 1, 0, 0)` (day rollover)
4. **Given** `SqlDateTime(2025, 1, 1, 1, 0, 0)` - duration of 2 hours, **Then** returns `SqlDateTime(2024, 12, 31, 23, 0, 0)` (day rollback)
5. **Given** `SqlDateTime::MAX_VALUE` + duration of 1 tick, **Then** returns out-of-range error
6. **Given** `SqlDateTime::MIN_VALUE` - duration of 1 tick, **Then** returns out-of-range error
7. **Given** any arithmetic op with `SqlDateTime::NULL` operand, **Then** returns `SqlDateTime::NULL`

---

### User Story 5 - Comparison returning SqlBoolean (Priority: P2)

A library consumer compares `SqlDateTime` values using SQL three-valued logic. Comparison is lexicographic: first by day_ticks, then by time_ticks. Comparisons return `SqlBoolean` (not `bool`), and any comparison involving NULL returns `SqlBoolean::NULL`.

**Why this priority**: Comparisons are essential for ordering and filtering date/time data but depend on the type already being constructable.

**Independent Test**: Can be fully tested by comparing pairs of values and verifying the returned `SqlBoolean`.

**Acceptance Scenarios**:

1. **Given** two SqlDateTimes for the same date and time, **When** `sql_equals` called, **Then** returns `SqlBoolean::TRUE`
2. **Given** two SqlDateTimes for the same date but different time, **When** `sql_less_than` called, **Then** ordered by time
3. **Given** two SqlDateTimes for different dates, **When** `sql_less_than` called, **Then** ordered by date first (day_ticks compared before time_ticks)
4. **Given** `SqlDateTime(2025, 1, 1).sql_less_than(&SqlDateTime(2025, 1, 2))`, **Then** returns `SqlBoolean::TRUE`
5. **Given** `SqlDateTime(2025, 1, 2).sql_greater_than(&SqlDateTime(2025, 1, 1))`, **Then** returns `SqlBoolean::TRUE`
6. **Given** `SqlDateTime(2025, 1, 1).sql_less_than_or_equal(&SqlDateTime(2025, 1, 1))`, **Then** returns `SqlBoolean::TRUE`
7. **Given** `SqlDateTime(2025, 1, 1).sql_greater_than_or_equal(&SqlDateTime(2025, 1, 1))`, **Then** returns `SqlBoolean::TRUE`
8. **Given** `SqlDateTime(2025, 1, 1).sql_not_equals(&SqlDateTime(2025, 1, 2))`, **Then** returns `SqlBoolean::TRUE`
9. **Given** any comparison with `SqlDateTime::NULL` operand, **Then** returns `SqlBoolean::NULL`

---

### User Story 6 - Display and parsing (Priority: P2)

A library consumer converts `SqlDateTime` to and from string representations. NULL displays as `"Null"`. Parsing supports standard date/time formats. Invalid strings return a parse error.

**Why this priority**: String conversion is needed for diagnostics, logging, and data interchange.

**Independent Test**: Can be fully tested by formatting values with `Display` and parsing strings with `FromStr`.

**Acceptance Scenarios**:

1. **Given** `SqlDateTime(2025, 7, 17, 12, 30, 0)`, **When** formatted with `Display`, **Then** produces a recognizable date/time string
2. **Given** `SqlDateTime::NULL`, **When** formatted with `Display`, **Then** produces `"Null"`
3. **Given** string `"2025-07-17 12:30:00"`, **When** parsed as `SqlDateTime`, **Then** returns the correct value
4. **Given** string `"abc"`, **When** parsed as `SqlDateTime`, **Then** returns parse error
5. **Given** string representing a date before 1753-01-01, **When** parsed, **Then** returns out-of-range error
6. **Given** string representing a date after 9999-12-31, **When** parsed, **Then** returns out-of-range error

---

### User Story 7 - Leap year and calendar correctness (Priority: P2)

A library consumer creates `SqlDateTime` values around leap year boundaries and verifies that the calendar calculations are correct. February 29 is valid in leap years and invalid in non-leap years.

**Why this priority**: Calendar math is error-prone and must be verified independently.

**Independent Test**: Can be fully tested by constructing dates around Feb 28/29 in leap and non-leap years.

**Acceptance Scenarios**:

1. **Given** construction with (2024, 2, 29), **Then** succeeds (2024 is a leap year)
2. **Given** construction with (2023, 2, 29), **Then** returns out-of-range error (2023 is not a leap year)
3. **Given** construction with (2000, 2, 29), **Then** succeeds (2000 is a century leap year)
4. **Given** construction with (1900, 2, 29), **Then** returns out-of-range error (1900 is not a leap year — divisible by 100 but not 400)
5. **Given** construction with (2024, 2, 28) + duration of 1 day, **Then** returns (2024, 2, 29)
6. **Given** construction with (2023, 2, 28) + duration of 1 day, **Then** returns (2023, 3, 1)

---

### User Story 8 - Accessors and component extraction (Priority: P3)

A library consumer extracts individual date/time components (year, month, day, hour, minute, second, millisecond) from a `SqlDateTime` value. This enables inspection and manipulation of individual fields.

**Why this priority**: Component extraction is a convenience feature that depends on the internal tick representations being correctly established.

**Independent Test**: Can be fully tested by constructing known date/time values and verifying each extracted component.

**Acceptance Scenarios**:

1. **Given** `SqlDateTime(2025, 7, 17, 14, 30, 45, 333.0)`, **When** year extracted, **Then** returns `2025`
2. **Given** same value, **When** month extracted, **Then** returns `7`
3. **Given** same value, **When** day extracted, **Then** returns `17`
4. **Given** same value, **When** hour extracted, **Then** returns `14`
5. **Given** same value, **When** minute extracted, **Then** returns `30`
6. **Given** same value, **When** second extracted, **Then** returns `45`
7. **Given** `SqlDateTime::NULL`, **When** any component extracted, **Then** returns `Err(NullValue)`

---

### Edge Cases

- 1900-01-01 has day_ticks = 0 (the epoch); dates before 1900 have negative day_ticks
- Minimum date 1753-01-01 corresponds to day_ticks = -53690
- Maximum date 9999-12-31 corresponds to day_ticks = 2958463
- Maximum time_ticks = 25919999 (one tick before midnight; TicksPerDay - 1)
- Millisecond rounding formula: `(int)(milliseconds * 0.3 + 0.5)` — matches C# exactly
- Time overflow at midnight: when rounding causes time_ticks > 25919999, time resets to 0 and day increments by 1
- Day overflow from time rollover at 9999-12-31 23:59:59.998+ MUST return out-of-range error
- Leap year rules: divisible by 4 AND (not divisible by 100 OR divisible by 400)
- February has 28 days normally, 29 in leap years
- Months with 30 days: April, June, September, November
- All other months have 31 days
- Duration arithmetic may cause day rollover (time overflow at midnight) or rollback (time underflow at midnight)
- Duration arithmetic result MUST remain within [MinDay, MaxDay] range
- NULL propagates through all arithmetic, comparison, and accessor operations
- `PartialEq` / `Eq` — Rust-level equality (distinct from `sql_equals` which returns `SqlBoolean`). Two NULL values are equal for Rust `PartialEq`, but `sql_equals` returns `SqlBoolean::NULL`
- `PartialOrd` / `Ord` — NULL values sort before all non-NULL values; non-NULL values sort by (day_ticks, time_ticks) lexicographically
- Day-of-month validation: construction with (2025, 4, 31) MUST fail (April has 30 days)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `SqlDateTime` MUST be `Copy + Clone`, storing an `i32` day_ticks (days since 1900-01-01) and `i32` time_ticks (1/300-second units since midnight) with NULL support
- **FR-002**: MUST validate date range: day_ticks MUST be in [-53690, 2958463] (1753-01-01 to 9999-12-31); time_ticks MUST be in [0, 25919999]
- **FR-003**: MUST provide construction from calendar components (year, month, day, hour, minute, second, millisecond) with full validation of each component's range and correct calendar math
- **FR-004**: MUST provide construction from raw (day_ticks, time_ticks) with range validation
- **FR-005**: MUST round milliseconds to the nearest 1/300-second SQL tick using the formula: `ticks = (int)(milliseconds * 0.3 + 0.5)` — matching C# behavior exactly
- **FR-006**: MUST handle time overflow at midnight: when rounding causes time_ticks to exceed 25919999, time resets to 0 and day increments by 1; if day exceeds MaxDay, return out-of-range error
- **FR-007**: MUST implement duration arithmetic (add/subtract a duration to/from a `SqlDateTime`) returning `Result<SqlDateTime, SqlTypeError>`, with range validation on the result
- **FR-008**: MUST NOT support adding or subtracting two `SqlDateTime` values directly — only duration-based arithmetic
- **FR-009**: MUST implement SQL comparison methods (`sql_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, `sql_not_equals`) returning `SqlBoolean`, comparing by (day_ticks, time_ticks) lexicographically
- **FR-010**: MUST implement `Display` (NULL displays as `"Null"`) and `FromStr` (invalid input returns `ParseError`; out-of-range dates return `OutOfRange`)
- **FR-011**: MUST provide constants: `NULL`, `MIN_VALUE` (1753-01-01 00:00:00.000), `MAX_VALUE` (9999-12-31 23:59:59.997)
- **FR-012**: MUST provide accessors: `day_ticks()`, `time_ticks()` returning the raw internal tick values
- **FR-013**: MUST provide calendar component extraction: year, month, day, hour, minute, second — each returning `Result<value, SqlTypeError>` (error if NULL)
- **FR-014**: MUST implement correct leap year handling following the Gregorian calendar rule (divisible by 4, except centuries not divisible by 400)
- **FR-015**: MUST implement `Hash`, `PartialEq`, `Eq` — two NULL values are equal for Rust equality
- **FR-016**: MUST implement `PartialOrd`, `Ord` — NULL sorts before all non-NULL values; non-NULL values sort by (day_ticks, time_ticks)
- **FR-017**: NULL propagation MUST apply to all arithmetic and comparison operations — any NULL operand produces a NULL result
- **FR-018**: MUST provide public tick-rate constants: `TICKS_PER_SECOND` (300), `TICKS_PER_MINUTE` (18000), `TICKS_PER_HOUR` (1080000), `TICKS_PER_DAY` (25920000)

### Key Entities

- **SqlDateTime**: A nullable date/time value with SQL Server DATETIME semantics. Internal representation: `Option<(i32, i32)>` where `None` = SQL NULL, `Some((day_ticks, time_ticks))` = a valid date/time. Day_ticks counts days from 1900-01-01 (negative for dates before 1900). Time_ticks counts 1/300-second intervals since midnight. Fixed-size, stack-allocated.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Date range boundaries tested — construction at exactly 1753-01-01 (MinDay) and 9999-12-31 (MaxDay) succeeds; one day before MinDay and one day after MaxDay fails with out-of-range error
- **SC-002**: Millisecond rounding verified against C# formula for at least 10 representative millisecond values, including boundary cases (0.0, 1.0, 3.33, 500.0, 996.0, 997.0, 998.0)
- **SC-003**: Leap year handling correct for all four rule categories: divisible by 4 (leap), divisible by 100 (not leap), divisible by 400 (leap), and non-divisible by 4 (not leap)
- **SC-004**: Time overflow at midnight tested — rounding at 23:59:59.998 rolls day forward correctly
- **SC-005**: Code coverage ≥ 95% for the `SqlDateTime` module
- **SC-006**: All comparison methods tested with equal, earlier, later, and NULL operand combinations
- **SC-007**: Duration arithmetic tested with day rollover, day rollback, and out-of-range error conditions
- **SC-008**: Calendar component extraction (year, month, day, hour, minute, second) round-trips correctly for representative dates across the valid range

## Assumptions

- Day_ticks calculation from calendar components uses the standard Gregorian calendar formula: `y = year - 1; dayticks = y*365 + y/4 - y/100 + y/400 + DaysToMonth[month-1] + day - 1 - DayBase` where DayBase = 693595 (Jan 1, 1900 in absolute days)
- Duration is represented using a Rust-native approach (likely `std::time::Duration` or a custom struct with days and ticks) — the exact type will be determined during planning
- No timezone support — `SqlDateTime` represents a local date/time without timezone information, matching SQL Server `DATETIME` semantics
- Display format defaults to ISO 8601-like format (e.g., `"2025-07-17 12:30:00.000"`) for consistency and unambiguous parsing; C# uses culture-sensitive formatting by default but for a Rust library a deterministic format is more appropriate
- `FromStr` parsing supports at minimum ISO 8601 format (`"YYYY-MM-DD HH:MM:SS"` and `"YYYY-MM-DDTHH:MM:SS"`); additional SQL Server-style formats may be added in a future iteration
- The `bilisecond` constructor variant from C# (taking fraction as integer/1000) is deferred as a convenience; the millisecond-based constructor is sufficient
- Conversions to/from `SqlString` are deferred until the `SqlString` type is implemented
- The internal representation may use a struct with named fields rather than a tuple, but the observable behavior is identical — this is an implementation detail for the planning phase
- `PartialOrd` / `Ord`: NULL values sort before all non-NULL values (consistent with Rust convention for `Option`)
