// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// ── T001: SqlDateTime module ──────────────────────────────────────────────────

//! `SqlDateTime` — a date/time value with SQL NULL support, equivalent to
//! C# `System.Data.SqlTypes.SqlDateTime` / SQL Server `DATETIME`.
//!
//! Stored internally as `Option<(i32, i32)>` where `(day_ticks, time_ticks)`:
//! - `day_ticks`: days since 1900-01-01 (negative for dates before 1900)
//! - `time_ticks`: 1/300-second intervals since midnight
//!
//! Valid date range: 1753-01-01 (day_ticks = −53690) to 9999-12-31 (day_ticks = 2958463).
//! Milliseconds are rounded to the nearest 1/300-second tick via `(int)(ms * 0.3 + 0.5)`.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

// ── T003: Struct definition ──────────────────────────────────────────────────

/// A date/time value (1753-01-01 to 9999-12-31, 1/300-second precision)
/// with SQL NULL support, equivalent to C# `System.Data.SqlTypes.SqlDateTime` /
/// SQL Server `DATETIME`.
///
/// Uses `Option<(i32, i32)>` internally: `None` = SQL NULL,
/// `Some((day_ticks, time_ticks))` = a valid date/time. All construction
/// validates ranges. Duration arithmetic returns `Result<SqlDateTime, SqlTypeError>`
/// with range checking. Comparisons return `SqlBoolean` for three-valued NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlDateTime {
    value: Option<(i32, i32)>,
}

// ── T004: Public constants ───────────────────────────────────────────────────
// ── T005: Private constants and helpers ──────────────────────────────────────

/// Absolute day number for 1900-01-01 (from year 1).
const DAY_BASE: i32 = 693_595;
/// Minimum day_ticks (1753-01-01).
const MIN_DAY: i32 = -53_690;
/// Maximum day_ticks (9999-12-31).
const MAX_DAY: i32 = 2_958_463;
/// Maximum time_ticks (one tick before midnight).
const MAX_TIME: i32 = 25_919_999;

/// Cumulative days per month (non-leap year), 13 entries (index 0..=12).
const DAYS_TO_MONTH_365: [i32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
/// Cumulative days per month (leap year), 13 entries (index 0..=12).
const DAYS_TO_MONTH_366: [i32; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

/// Checks whether a given year is a leap year.
/// Matches C# `DateTime.IsLeapYear` / `SqlDateTime.IsLeapYear`.
fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

impl SqlDateTime {
    // ── Public tick-rate constants ────────────────────────────────────────────

    /// Number of SQL ticks per second (1/300-second resolution).
    pub const TICKS_PER_SECOND: i32 = 300;
    /// Number of SQL ticks per minute (300 × 60).
    pub const TICKS_PER_MINUTE: i32 = 18_000;
    /// Number of SQL ticks per hour (300 × 3600).
    pub const TICKS_PER_HOUR: i32 = 1_080_000;
    /// Number of SQL ticks per day (300 × 86400).
    pub const TICKS_PER_DAY: i32 = 25_920_000;

    /// SQL NULL sentinel.
    pub const NULL: SqlDateTime = SqlDateTime { value: None };
    /// Minimum value: 1753-01-01 00:00:00.000.
    pub const MIN_VALUE: SqlDateTime = SqlDateTime {
        value: Some((MIN_DAY, 0)),
    };
    /// Maximum value: 9999-12-31 23:59:59.997.
    pub const MAX_VALUE: SqlDateTime = SqlDateTime {
        value: Some((MAX_DAY, MAX_TIME)),
    };
}

// ── T006: Private calendar helpers ───────────────────────────────────────────

/// Convert (year, month, day) to day_ticks (days since 1900-01-01).
/// Validates year ∈ [1753, 9999], month ∈ [1, 12], day ∈ [1, days_in_month].
/// Direct port of C# SqlDateTime constructor's calendar formula.
fn date_to_day_ticks(year: i32, month: i32, day: i32) -> Result<i32, SqlTypeError> {
    if !(1753..=9999).contains(&year) || !(1..=12).contains(&month) {
        return Err(SqlTypeError::OutOfRange(
            "Invalid date components".to_string(),
        ));
    }

    let days = if is_leap_year(year) {
        &DAYS_TO_MONTH_366
    } else {
        &DAYS_TO_MONTH_365
    };

    let days_in_month = days[month as usize] - days[(month - 1) as usize];
    if day < 1 || day > days_in_month {
        return Err(SqlTypeError::OutOfRange(
            "Invalid date components".to_string(),
        ));
    }

    let y = year - 1;
    let day_ticks =
        y * 365 + y / 4 - y / 100 + y / 400 + days[(month - 1) as usize] + day - 1 - DAY_BASE;

    if !(MIN_DAY..=MAX_DAY).contains(&day_ticks) {
        return Err(SqlTypeError::OutOfRange(
            "Date outside valid range".to_string(),
        ));
    }

    Ok(day_ticks)
}

/// Convert (hour, minute, second, millisecond) to `(time_ticks, day_carry)`.
/// Uses the C# rounding formula: `(int)(ms * 0.3 + 0.5)`.
/// If rounding causes overflow past MAX_TIME, returns `day_carry = 1` and
/// `time_ticks = 0` (midnight rollover).
fn time_to_ticks(
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: f64,
) -> Result<(i32, i32), SqlTypeError> {
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
        return Err(SqlTypeError::OutOfRange(
            "Invalid time components".to_string(),
        ));
    }
    if !(0.0..1000.0).contains(&millisecond) {
        return Err(SqlTypeError::OutOfRange(
            "Invalid time components".to_string(),
        ));
    }

    // C# formula: ticksForMillisecond = millisecond * TicksPerMillisecond + 0.5
    // where TicksPerMillisecond = 0.3
    let ticks_for_ms = (millisecond * 0.3 + 0.5) as i32;

    let time_ticks = hour * SqlDateTime::TICKS_PER_HOUR
        + minute * SqlDateTime::TICKS_PER_MINUTE
        + second * SqlDateTime::TICKS_PER_SECOND
        + ticks_for_ms;

    if time_ticks > MAX_TIME {
        // Only rounding up can cause this — time becomes exactly MAX_TIME+1 (midnight).
        // Reset time to 0 and carry 1 day forward.
        Ok((0, 1))
    } else {
        Ok((time_ticks, 0))
    }
}

/// Decompose day_ticks into (year, month, day) using the standard
/// 400/100/4/1-year Gregorian cycle algorithm.
/// Direct port of .NET DateTime's internal date extraction logic.
fn day_ticks_to_ymd(day_ticks: i32) -> (i32, i32, i32) {
    // Convert to absolute day number (0-based, day 0 = Jan 1, year 1)
    let mut d = (day_ticks + DAY_BASE) as i64;

    // 400-year cycles (146097 days each)
    let n400 = d / 146_097;
    d -= n400 * 146_097;

    // 100-year cycles within the 400-year period (36524 days each)
    // Clamped to 3: the last day of a 400-year cycle belongs to year 400 (leap)
    let mut n100 = d / 36_524;
    if n100 == 4 {
        n100 = 3;
    }
    d -= n100 * 36_524;

    // 4-year cycles (1461 days each)
    let n4 = d / 1_461;
    d -= n4 * 1_461;

    // Single years within the 4-year cycle (365 days each)
    // Clamped to 3: the last day of a 4-year cycle belongs to the leap year
    let mut n1 = d / 365;
    if n1 == 4 {
        n1 = 3;
    }
    d -= n1 * 365;

    let year = (400 * n400 + 100 * n100 + 4 * n4 + n1 + 1) as i32;

    // d is now the 0-based day-of-year
    let days = if is_leap_year(year) {
        &DAYS_TO_MONTH_366
    } else {
        &DAYS_TO_MONTH_365
    };

    // Find month: first m where days[m] > d
    let d = d as i32;
    let mut month = 1;
    while month <= 12 && days[month as usize] <= d {
        month += 1;
    }

    let day = d - days[(month - 1) as usize] + 1;

    (year, month, day)
}

// ── T007: from_ticks constructor ─────────────────────────────────────────────
// ── T008: new constructor ────────────────────────────────────────────────────
// ── T009: Accessors ──────────────────────────────────────────────────────────

impl SqlDateTime {
    /// Creates a `SqlDateTime` from raw tick values.
    /// Validates `day_ticks` ∈ [-53690, 2958463] and `time_ticks` ∈ [0, 25919999].
    pub fn from_ticks(day_ticks: i32, time_ticks: i32) -> Result<SqlDateTime, SqlTypeError> {
        if !(MIN_DAY..=MAX_DAY).contains(&day_ticks) || !(0..=MAX_TIME).contains(&time_ticks) {
            return Err(SqlTypeError::OutOfRange(
                "Day ticks or time ticks out of range".to_string(),
            ));
        }
        Ok(SqlDateTime {
            value: Some((day_ticks, time_ticks)),
        })
    }

    /// Creates a `SqlDateTime` from calendar components.
    /// Validates all components and range. Milliseconds are rounded to the
    /// nearest 1/300-second tick via the C# formula `(int)(ms * 0.3 + 0.5)`.
    /// Handles midnight overflow (time rolls over → day increments).
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: f64,
    ) -> Result<SqlDateTime, SqlTypeError> {
        let day_ticks = date_to_day_ticks(year, month, day)?;
        let (time_ticks, day_carry) = time_to_ticks(hour, minute, second, millisecond)?;

        let final_day = day_ticks + day_carry;

        // Validate the final result (day may have incremented from midnight rollover)
        Self::from_ticks(final_day, time_ticks)
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `(day_ticks, time_ticks)` tuple, or
    /// `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<(i32, i32), SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }

    /// Returns the day ticks (days since 1900-01-01), or
    /// `Err(SqlTypeError::NullValue)` if NULL.
    pub fn day_ticks(&self) -> Result<i32, SqlTypeError> {
        self.value.map(|(d, _)| d).ok_or(SqlTypeError::NullValue)
    }

    /// Returns the time ticks (1/300-second intervals since midnight), or
    /// `Err(SqlTypeError::NullValue)` if NULL.
    pub fn time_ticks(&self) -> Result<i32, SqlTypeError> {
        self.value.map(|(_, t)| t).ok_or(SqlTypeError::NullValue)
    }

    // ── T020: checked_add ────────────────────────────────────────────────────
    // ── T021: checked_sub ────────────────────────────────────────────────────
    // ── T022: checked_add_days, checked_add_ticks ────────────────────────────

    /// Add a duration expressed as day and time tick offsets.
    /// NULL propagation: if self is NULL, returns `Ok(SqlDateTime::NULL)`.
    /// Time overflow/underflow normalizes into day carry via `div_euclid`/`rem_euclid`.
    /// Returns `Err(OutOfRange)` if the result falls outside [MIN_DAY, MAX_DAY].
    pub fn checked_add(self, day_delta: i32, time_delta: i32) -> Result<SqlDateTime, SqlTypeError> {
        let (d, t) = match self.value {
            None => return Ok(SqlDateTime::NULL),
            Some(v) => v,
        };

        // Use i64 intermediate to avoid overflow when time_delta is large
        let new_time = t as i64 + time_delta as i64;

        // Normalize: always produce non-negative time remainder
        let day_carry = new_time.div_euclid(SqlDateTime::TICKS_PER_DAY as i64);
        let normalized_time = new_time.rem_euclid(SqlDateTime::TICKS_PER_DAY as i64);

        let new_day = d as i64 + day_delta as i64 + day_carry;

        // Range check
        if new_day < MIN_DAY as i64
            || new_day > MAX_DAY as i64
            || normalized_time < 0
            || normalized_time > MAX_TIME as i64
        {
            return Err(SqlTypeError::OutOfRange(
                "Result outside valid SqlDateTime range".to_string(),
            ));
        }

        Ok(SqlDateTime {
            value: Some((new_day as i32, normalized_time as i32)),
        })
    }

    /// Subtract a duration expressed as day and time tick offsets.
    /// Delegates to `checked_add` with negated deltas.
    pub fn checked_sub(self, day_delta: i32, time_delta: i32) -> Result<SqlDateTime, SqlTypeError> {
        // Negate deltas; handle i32::MIN by widening to i64 then narrowing safely
        let neg_day = -(day_delta as i64);
        let neg_time = -(time_delta as i64);

        // If the negated values don't fit in i32, the operation would be out of range anyway
        if neg_day < i32::MIN as i64
            || neg_day > i32::MAX as i64
            || neg_time < i32::MIN as i64
            || neg_time > i32::MAX as i64
        {
            return match self.value {
                None => Ok(SqlDateTime::NULL),
                Some(_) => Err(SqlTypeError::OutOfRange(
                    "Result outside valid SqlDateTime range".to_string(),
                )),
            };
        }

        self.checked_add(neg_day as i32, neg_time as i32)
    }

    /// Add days only (convenience for `checked_add(days, 0)`).
    pub fn checked_add_days(self, days: i32) -> Result<SqlDateTime, SqlTypeError> {
        self.checked_add(days, 0)
    }

    /// Add time ticks only (convenience for `checked_add(0, ticks)`).
    pub fn checked_add_ticks(self, ticks: i32) -> Result<SqlDateTime, SqlTypeError> {
        self.checked_add(0, ticks)
    }

    // ── T024: SQL comparison methods ─────────────────────────────────────────

    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    /// Compares `(day_ticks, time_ticks)` lexicographically.
    pub fn sql_equals(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 == d2 && t1 == t2),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 != d2 || t1 != t2),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than. Returns `SqlBoolean::NULL` if either operand is NULL.
    /// Uses lexicographic `(day_ticks, time_ticks)` ordering.
    pub fn sql_less_than(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 < d2 || (d1 == d2 && t1 < t2)),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 > d2 || (d1 == d2 && t1 > t2)),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 < d2 || (d1 == d2 && t1 <= t2)),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlDateTime) -> SqlBoolean {
        match (self.value, other.value) {
            (Some((d1, t1)), Some((d2, t2))) => SqlBoolean::new(d1 > d2 || (d1 == d2 && t1 >= t2)),
            _ => SqlBoolean::NULL,
        }
    }

    // ── T032: year(), month(), day() ─────────────────────────────────────────
    // ── T033: hour(), minute(), second() ─────────────────────────────────────

    /// Returns the year component (1753–9999), or `Err(NullValue)` if NULL.
    pub fn year(&self) -> Result<i32, SqlTypeError> {
        let (d, _) = self.value.ok_or(SqlTypeError::NullValue)?;
        let (y, _, _) = day_ticks_to_ymd(d);
        Ok(y)
    }

    /// Returns the month component (1–12), or `Err(NullValue)` if NULL.
    pub fn month(&self) -> Result<i32, SqlTypeError> {
        let (d, _) = self.value.ok_or(SqlTypeError::NullValue)?;
        let (_, m, _) = day_ticks_to_ymd(d);
        Ok(m)
    }

    /// Returns the day component (1–31), or `Err(NullValue)` if NULL.
    pub fn day(&self) -> Result<i32, SqlTypeError> {
        let (d, _) = self.value.ok_or(SqlTypeError::NullValue)?;
        let (_, _, day) = day_ticks_to_ymd(d);
        Ok(day)
    }

    /// Returns the hour component (0–23), or `Err(NullValue)` if NULL.
    pub fn hour(&self) -> Result<i32, SqlTypeError> {
        let (_, t) = self.value.ok_or(SqlTypeError::NullValue)?;
        Ok(t / Self::TICKS_PER_HOUR)
    }

    /// Returns the minute component (0–59), or `Err(NullValue)` if NULL.
    pub fn minute(&self) -> Result<i32, SqlTypeError> {
        let (_, t) = self.value.ok_or(SqlTypeError::NullValue)?;
        Ok((t % Self::TICKS_PER_HOUR) / Self::TICKS_PER_MINUTE)
    }

    /// Returns the second component (0–59), or `Err(NullValue)` if NULL.
    pub fn second(&self) -> Result<i32, SqlTypeError> {
        let (_, t) = self.value.ok_or(SqlTypeError::NullValue)?;
        Ok((t % Self::TICKS_PER_MINUTE) / Self::TICKS_PER_SECOND)
    }
}

// ── T027: Display ────────────────────────────────────────────────────────────

/// Extract (hour, minute, second, millisecond) from time_ticks.
fn time_ticks_to_hmsm(time_ticks: i32) -> (i32, i32, i32, i32) {
    let hour = time_ticks / SqlDateTime::TICKS_PER_HOUR;
    let rem = time_ticks % SqlDateTime::TICKS_PER_HOUR;
    let minute = rem / SqlDateTime::TICKS_PER_MINUTE;
    let rem = rem % SqlDateTime::TICKS_PER_MINUTE;
    let second = rem / SqlDateTime::TICKS_PER_SECOND;
    let remaining_ticks = rem % SqlDateTime::TICKS_PER_SECOND;
    // Convert SQL ticks to milliseconds with rounding: (ticks * 1000 + 150) / 300
    let millisecond = (remaining_ticks * 1000 + 150) / 300;
    (hour, minute, second, millisecond)
}

impl fmt::Display for SqlDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            None => write!(f, "Null"),
            Some((day, time)) => {
                let (year, month, day) = day_ticks_to_ymd(day);
                let (hour, minute, second, ms) = time_ticks_to_hmsm(time);
                write!(
                    f,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                    year, month, day, hour, minute, second, ms
                )
            }
        }
    }
}

// ── T028: FromStr ────────────────────────────────────────────────────────────

impl FromStr for SqlDateTime {
    type Err = SqlTypeError;

    /// Parse a string into a `SqlDateTime`.
    ///
    /// Accepted formats:
    /// - `"Null"` → `SqlDateTime::NULL`
    /// - `"YYYY-MM-DD HH:MM:SS.fff"` (space separator, with milliseconds)
    /// - `"YYYY-MM-DD HH:MM:SS"` (space separator, no milliseconds)
    /// - `"YYYY-MM-DDTHH:MM:SS.fff"` (T separator, with milliseconds)
    /// - `"YYYY-MM-DDTHH:MM:SS"` (T separator, no milliseconds)
    /// - `"YYYY-MM-DD"` (date only, time defaults to midnight)
    fn from_str(s: &str) -> Result<SqlDateTime, SqlTypeError> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("null") {
            return Ok(SqlDateTime::NULL);
        }

        // Must have at least "YYYY-MM-DD" (10 chars)
        if s.len() < 10 {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid SqlDateTime format: '{s}'"
            )));
        }

        // Parse date portion: YYYY-MM-DD
        let year = s[..4]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid year in: '{s}'")))?;
        if s.as_bytes()[4] != b'-' {
            return Err(SqlTypeError::ParseError(format!(
                "Expected '-' at position 4 in: '{s}'"
            )));
        }
        let month = s[5..7]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid month in: '{s}'")))?;
        if s.as_bytes()[7] != b'-' {
            return Err(SqlTypeError::ParseError(format!(
                "Expected '-' at position 7 in: '{s}'"
            )));
        }
        let day = s[8..10]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid day in: '{s}'")))?;

        // Date only → midnight
        if s.len() == 10 {
            return SqlDateTime::new(year, month, day, 0, 0, 0, 0.0);
        }

        // Check separator (space or 'T')
        let sep = s.as_bytes()[10];
        if sep != b' ' && sep != b'T' {
            return Err(SqlTypeError::ParseError(format!(
                "Expected ' ' or 'T' at position 10 in: '{s}'"
            )));
        }

        let time_str = &s[11..];

        // Must have at least "HH:MM:SS" (8 chars)
        if time_str.len() < 8 {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid time portion in: '{s}'"
            )));
        }

        let hour = time_str[..2]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid hour in: '{s}'")))?;
        if time_str.as_bytes()[2] != b':' {
            return Err(SqlTypeError::ParseError(format!(
                "Expected ':' in time portion of: '{s}'"
            )));
        }
        let minute = time_str[3..5]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid minute in: '{s}'")))?;
        if time_str.as_bytes()[5] != b':' {
            return Err(SqlTypeError::ParseError(format!(
                "Expected ':' in time portion of: '{s}'"
            )));
        }
        let second = time_str[6..8]
            .parse::<i32>()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid second in: '{s}'")))?;

        // Optional .fff
        let millisecond = if time_str.len() > 8 {
            if time_str.as_bytes()[8] != b'.' {
                return Err(SqlTypeError::ParseError(format!(
                    "Expected '.' for milliseconds in: '{s}'"
                )));
            }
            let ms_str = &time_str[9..];
            if ms_str.is_empty() || ms_str.len() > 3 {
                return Err(SqlTypeError::ParseError(format!(
                    "Invalid millisecond portion in: '{s}'"
                )));
            }
            // Pad to 3 digits: "1" → 100, "12" → 120, "123" → 123
            let padded = format!("{:0<3}", ms_str);
            padded
                .parse::<f64>()
                .map_err(|_| SqlTypeError::ParseError(format!("Invalid milliseconds in: '{s}'")))?
        } else {
            0.0
        };

        SqlDateTime::new(year, month, day, hour, minute, second, millisecond)
    }
}

// ── T037: PartialEq, Eq, Hash ───────────────────────────────────────────────

impl PartialEq for SqlDateTime {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlDateTime {}

impl Hash for SqlDateTime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NULL hashes as (0i32, 0i32) for consistency
        match self.value {
            Some((d, t)) => {
                d.hash(state);
                t.hash(state);
            }
            None => {
                0i32.hash(state);
                0i32.hash(state);
            }
        }
    }
}

// ── T038: PartialOrd, Ord ───────────────────────────────────────────────────

impl PartialOrd for SqlDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlDateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        // NULL < any non-NULL value, lexicographic (day_ticks, time_ticks) ordering
        match (self.value, other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── T010: US1 — new() with valid calendar components ─────────────────────

    #[test]
    fn new_epoch_date() {
        // 1900-01-01 00:00:00.000 → day_ticks = 0, time_ticks = 0
        let dt = SqlDateTime::new(1900, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), 0);
        assert_eq!(dt.time_ticks().unwrap(), 0);
    }

    #[test]
    fn new_date_before_epoch() {
        // 1899-12-31 → day_ticks = -1
        let dt = SqlDateTime::new(1899, 12, 31, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), -1);
    }

    #[test]
    fn new_representative_date() {
        // 2025-07-17 12:30:00.000
        let dt = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert!(!dt.is_null());
        // Verify day_ticks is reasonable (positive, after 1900)
        assert!(dt.day_ticks().unwrap() > 0);
        // time_ticks = 12*1080000 + 30*18000 = 12960000 + 540000 = 13500000
        assert_eq!(dt.time_ticks().unwrap(), 13_500_000);
    }

    #[test]
    fn new_min_value_date() {
        // 1753-01-01 → day_ticks = -53690
        let dt = SqlDateTime::new(1753, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), -53690);
        assert_eq!(dt.time_ticks().unwrap(), 0);
    }

    #[test]
    fn new_max_value_date() {
        // 9999-12-31 23:59:59.997 → day_ticks = 2958463, time_ticks = 25919999
        let dt = SqlDateTime::new(9999, 12, 31, 23, 59, 59, 997.0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), 2_958_463);
        assert_eq!(dt.time_ticks().unwrap(), 25_919_999);
    }

    #[test]
    fn new_value_accessor() {
        let dt = SqlDateTime::new(1900, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.value().unwrap(), (0, 0));
    }

    // ── T011: US1 — new() with invalid components ────────────────────────────

    #[test]
    fn new_year_too_low() {
        assert!(SqlDateTime::new(1752, 1, 1, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_year_too_high() {
        assert!(SqlDateTime::new(10000, 1, 1, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_month_zero() {
        assert!(SqlDateTime::new(2025, 0, 1, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_month_thirteen() {
        assert!(SqlDateTime::new(2025, 13, 1, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_day_zero() {
        assert!(SqlDateTime::new(2025, 1, 0, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_day_thirty_two() {
        assert!(SqlDateTime::new(2025, 1, 32, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_hour_twenty_four() {
        assert!(SqlDateTime::new(2025, 1, 1, 24, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_minute_sixty() {
        assert!(SqlDateTime::new(2025, 1, 1, 0, 60, 0, 0.0).is_err());
    }

    #[test]
    fn new_second_sixty() {
        assert!(SqlDateTime::new(2025, 1, 1, 0, 0, 60, 0.0).is_err());
    }

    #[test]
    fn new_millisecond_thousand() {
        assert!(SqlDateTime::new(2025, 1, 1, 0, 0, 0, 1000.0).is_err());
    }

    #[test]
    fn new_millisecond_negative() {
        assert!(SqlDateTime::new(2025, 1, 1, 0, 0, 0, -1.0).is_err());
    }

    // ── T012: US1 — day-of-month validation ──────────────────────────────────

    #[test]
    fn new_april_thirty_one_fails() {
        // April has 30 days
        assert!(SqlDateTime::new(2025, 4, 31, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_feb_twenty_nine_non_leap_fails() {
        // 2023 is not a leap year
        assert!(SqlDateTime::new(2023, 2, 29, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn new_valid_month_end_dates() {
        // Jan 31
        assert!(SqlDateTime::new(2025, 1, 31, 0, 0, 0, 0.0).is_ok());
        // Jun 30
        assert!(SqlDateTime::new(2025, 6, 30, 0, 0, 0, 0.0).is_ok());
        // Feb 28 (non-leap year)
        assert!(SqlDateTime::new(2025, 2, 28, 0, 0, 0, 0.0).is_ok());
    }

    #[test]
    fn new_feb_twenty_nine_leap_year_ok() {
        // 2024 is a leap year
        assert!(SqlDateTime::new(2024, 2, 29, 0, 0, 0, 0.0).is_ok());
    }

    // ── T013: US2 — from_ticks() ─────────────────────────────────────────────

    #[test]
    fn from_ticks_epoch() {
        let dt = SqlDateTime::from_ticks(0, 0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), 0);
        assert_eq!(dt.time_ticks().unwrap(), 0);
    }

    #[test]
    fn from_ticks_min_value() {
        let dt = SqlDateTime::from_ticks(-53690, 0).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), -53690);
    }

    #[test]
    fn from_ticks_max_value() {
        let dt = SqlDateTime::from_ticks(2958463, 25919999).unwrap();
        assert_eq!(dt.day_ticks().unwrap(), 2958463);
        assert_eq!(dt.time_ticks().unwrap(), 25919999);
    }

    #[test]
    fn from_ticks_day_too_low() {
        assert!(SqlDateTime::from_ticks(-53691, 0).is_err());
    }

    #[test]
    fn from_ticks_day_too_high() {
        assert!(SqlDateTime::from_ticks(2958464, 0).is_err());
    }

    #[test]
    fn from_ticks_time_negative() {
        assert!(SqlDateTime::from_ticks(0, -1).is_err());
    }

    #[test]
    fn from_ticks_time_too_high() {
        assert!(SqlDateTime::from_ticks(0, 25920000).is_err());
    }

    // ── T014: US2 — NULL and constants ───────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlDateTime::NULL.is_null());
    }

    #[test]
    fn null_value_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.value(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn null_day_ticks_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.day_ticks(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn null_time_ticks_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.time_ticks(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn min_value_day_ticks() {
        assert_eq!(SqlDateTime::MIN_VALUE.day_ticks().unwrap(), -53690);
    }

    #[test]
    fn max_value_time_ticks() {
        assert_eq!(SqlDateTime::MAX_VALUE.time_ticks().unwrap(), 25919999);
    }

    #[test]
    fn tick_rate_constants() {
        assert_eq!(SqlDateTime::TICKS_PER_SECOND, 300);
        assert_eq!(SqlDateTime::TICKS_PER_MINUTE, 18_000);
        assert_eq!(SqlDateTime::TICKS_PER_HOUR, 1_080_000);
        assert_eq!(SqlDateTime::TICKS_PER_DAY, 25_920_000);
    }

    // ── T015: US3 — Millisecond rounding ─────────────────────────────────────

    #[test]
    fn ms_rounding_zero() {
        // ms=0.0 → 0 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 0);
    }

    #[test]
    fn ms_rounding_3_33() {
        // ms=3.33 → (3.33*0.3+0.5)=1.499 → 1 tick
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 3.33).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 1);
    }

    #[test]
    fn ms_rounding_3_34() {
        // ms=3.34 → (3.34*0.3+0.5)=1.502 → 1 tick
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 3.34).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 1);
    }

    #[test]
    fn ms_rounding_500() {
        // ms=500.0 → (500*0.3+0.5)=150.5 → 150 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 500.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 150);
    }

    #[test]
    fn ms_rounding_997() {
        // ms=997.0 → (997*0.3+0.5)=299.6 → 299 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 997.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 299);
    }

    #[test]
    fn ms_rounding_6_67() {
        // ms=6.67 → (6.67*0.3+0.5)=2.501 → 2 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 6.67).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 2);
    }

    #[test]
    fn ms_rounding_10() {
        // ms=10.0 → (10*0.3+0.5)=3.5 → 3 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 10.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 3);
    }

    #[test]
    fn ms_rounding_100() {
        // ms=100.0 → (100*0.3+0.5)=30.5 → 30 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 100.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 30);
    }

    #[test]
    fn ms_rounding_333() {
        // ms=333.0 → (333*0.3+0.5)=100.4 → 100 ticks
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 333.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 100);
    }

    #[test]
    fn ms_rounding_with_time_components() {
        // 12:30:45 with 333ms → 12*1080000+30*18000+45*300+100 = 12960000+540000+13500+100 = 13513600
        let dt = SqlDateTime::new(2025, 1, 1, 12, 30, 45, 333.0).unwrap();
        assert_eq!(dt.time_ticks().unwrap(), 13_513_600);
    }

    // ── T016: US3 — Midnight rollover ────────────────────────────────────────

    #[test]
    fn midnight_rollover() {
        // 23:59:59 with ms that rounds up past MAX_TIME.
        // 23:59:59 base ticks = 23*1080000+59*18000+59*300 = 25919700
        // Need ms that makes total > 25919999.
        // ms_ticks >= 300 → ms ≈ 998.34+ :
        // (998.34*0.3+0.5) = 300.002 → 300 → total = 25920000 > MAX_TIME
        // But 998.34 < 1000.0 so it's valid.
        //
        // Use a date that can roll over (not Dec 31, 9999).
        let dt = SqlDateTime::new(2025, 1, 1, 23, 59, 59, 998.4).unwrap();
        // After rollover: day should increment, time should be 0
        let day_of_jan1 = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0)
            .unwrap()
            .day_ticks()
            .unwrap();
        assert_eq!(dt.day_ticks().unwrap(), day_of_jan1 + 1);
        assert_eq!(dt.time_ticks().unwrap(), 0);
    }

    #[test]
    fn midnight_rollover_max_value_overflow() {
        // 9999-12-31 23:59:59 with ms that causes rollover → day_ticks exceeds MAX_DAY
        assert!(SqlDateTime::new(9999, 12, 31, 23, 59, 59, 998.4).is_err());
    }

    #[test]
    fn no_rollover_at_max_time() {
        // 23:59:59.997 → 299 ticks → total = 25919999 = MAX_TIME → no rollover
        let dt = SqlDateTime::new(2025, 1, 1, 23, 59, 59, 997.0).unwrap();
        let day_of_jan1 = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0)
            .unwrap()
            .day_ticks()
            .unwrap();
        assert_eq!(dt.day_ticks().unwrap(), day_of_jan1);
        assert_eq!(dt.time_ticks().unwrap(), 25_919_999);
    }

    // ── T017: US4 — checked_add and checked_add_days ─────────────────────────

    #[test]
    fn checked_add_one_day() {
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_add_days(1).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() + 1);
        assert_eq!(result.time_ticks().unwrap(), dt.time_ticks().unwrap());
    }

    #[test]
    fn checked_add_negative_days() {
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_add_days(-1).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() - 1);
    }

    #[test]
    fn checked_add_time_causes_day_rollover() {
        // 23:00 + 2 hours → rolls over to next day, 01:00
        let dt = SqlDateTime::new(2025, 1, 15, 23, 0, 0, 0.0).unwrap();
        let result = dt.checked_add(0, 2 * SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() + 1);
        // 01:00 = 1 * TICKS_PER_HOUR
        assert_eq!(result.time_ticks().unwrap(), SqlDateTime::TICKS_PER_HOUR);
    }

    #[test]
    fn checked_add_day_and_time() {
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_add(1, SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() + 1);
        // 12:00 + 1 hour = 13:00
        assert_eq!(
            result.time_ticks().unwrap(),
            13 * SqlDateTime::TICKS_PER_HOUR
        );
    }

    #[test]
    fn checked_add_at_max_value_overflow() {
        let result = SqlDateTime::MAX_VALUE.checked_add_ticks(1);
        assert!(result.is_err());
    }

    #[test]
    fn checked_add_null_propagation() {
        let result = SqlDateTime::NULL.checked_add_days(1).unwrap();
        assert!(result.is_null());
    }

    // ── T018: US4 — checked_sub ──────────────────────────────────────────────

    #[test]
    fn checked_sub_one_day() {
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_sub(1, 0).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() - 1);
    }

    #[test]
    fn checked_sub_time_causes_day_rollback() {
        // 01:00 - 2 hours → rolls back to previous day, 23:00
        let dt = SqlDateTime::new(2025, 1, 15, 1, 0, 0, 0.0).unwrap();
        let result = dt.checked_sub(0, 2 * SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() - 1);
        assert_eq!(
            result.time_ticks().unwrap(),
            23 * SqlDateTime::TICKS_PER_HOUR
        );
    }

    #[test]
    fn checked_sub_at_min_value_underflow() {
        let result = SqlDateTime::MIN_VALUE.checked_sub(0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn checked_sub_null_propagation() {
        let result = SqlDateTime::NULL.checked_sub(1, 0).unwrap();
        assert!(result.is_null());
    }

    // ── T019: US4 — checked_add_ticks ────────────────────────────────────────

    #[test]
    fn checked_add_ticks_within_same_day() {
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_add_ticks(SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap());
        assert_eq!(
            result.time_ticks().unwrap(),
            13 * SqlDateTime::TICKS_PER_HOUR
        );
    }

    #[test]
    fn checked_add_ticks_causing_day_rollover() {
        // Add full day of ticks
        let dt = SqlDateTime::new(2025, 1, 15, 0, 0, 0, 0.0).unwrap();
        let result = dt.checked_add_ticks(SqlDateTime::TICKS_PER_DAY).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap() + 1);
        assert_eq!(result.time_ticks().unwrap(), 0);
    }

    #[test]
    fn checked_add_ticks_negative() {
        // Subtract 1 hour via negative ticks
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let result = dt.checked_add_ticks(-SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(result.day_ticks().unwrap(), dt.day_ticks().unwrap());
        assert_eq!(
            result.time_ticks().unwrap(),
            11 * SqlDateTime::TICKS_PER_HOUR
        );
    }

    #[test]
    fn checked_add_ticks_equivalence() {
        // checked_add_ticks(n) == checked_add(0, n)
        let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0).unwrap();
        let r1 = dt
            .checked_add_ticks(3 * SqlDateTime::TICKS_PER_HOUR)
            .unwrap();
        let r2 = dt.checked_add(0, 3 * SqlDateTime::TICKS_PER_HOUR).unwrap();
        assert_eq!(r1.value().unwrap(), r2.value().unwrap());
    }

    // ── T023: US5 — SQL comparison methods ───────────────────────────────────

    #[test]
    fn sql_equals_same_value() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(dt.sql_equals(&dt), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_different_value() {
        let a = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 1, 2, 12, 0, 0, 0.0).unwrap();
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_not_equals_different() {
        let a = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 1, 2, 12, 0, 0, 0.0).unwrap();
        assert_eq!(a.sql_not_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_not_equals_same() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(dt.sql_not_equals(&dt), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_earlier_date() {
        let a = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 1, 2, 12, 0, 0, 0.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_same_date_earlier_time() {
        let a = SqlDateTime::new(2025, 1, 1, 11, 0, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_equal_values() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(dt.sql_less_than(&dt), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_later_date() {
        let a = SqlDateTime::new(2025, 1, 2, 12, 0, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(dt.sql_less_than_or_equal(&dt), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert_eq!(dt.sql_greater_than_or_equal(&dt), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_comparison_null_lhs() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert!(SqlDateTime::NULL.sql_equals(&dt).is_null());
        assert!(SqlDateTime::NULL.sql_not_equals(&dt).is_null());
        assert!(SqlDateTime::NULL.sql_less_than(&dt).is_null());
        assert!(SqlDateTime::NULL.sql_greater_than(&dt).is_null());
        assert!(SqlDateTime::NULL.sql_less_than_or_equal(&dt).is_null());
        assert!(SqlDateTime::NULL.sql_greater_than_or_equal(&dt).is_null());
    }

    #[test]
    fn sql_comparison_null_rhs() {
        let dt = SqlDateTime::new(2025, 1, 1, 12, 0, 0, 0.0).unwrap();
        assert!(dt.sql_equals(&SqlDateTime::NULL).is_null());
        assert!(dt.sql_not_equals(&SqlDateTime::NULL).is_null());
        assert!(dt.sql_less_than(&SqlDateTime::NULL).is_null());
        assert!(dt.sql_greater_than(&SqlDateTime::NULL).is_null());
        assert!(dt.sql_less_than_or_equal(&SqlDateTime::NULL).is_null());
        assert!(dt.sql_greater_than_or_equal(&SqlDateTime::NULL).is_null());
    }

    // ── T025: US6 — Display ─────────────────────────────────────────────────

    #[test]
    fn display_representative_date() {
        let dt = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(format!("{}", dt), "2025-07-17 12:30:00.000");
    }

    #[test]
    fn display_epoch() {
        let dt = SqlDateTime::new(1900, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(format!("{}", dt), "1900-01-01 00:00:00.000");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlDateTime::NULL), "Null");
    }

    #[test]
    fn display_leading_zeros() {
        let dt = SqlDateTime::new(2025, 3, 5, 9, 7, 3, 0.0).unwrap();
        assert_eq!(format!("{}", dt), "2025-03-05 09:07:03.000");
    }

    #[test]
    fn display_with_milliseconds() {
        // ms=333 → ticks=100 → back to ms=(100*1000+150)/300=333
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 333.0).unwrap();
        assert_eq!(format!("{}", dt), "2025-01-01 00:00:00.333");
    }

    #[test]
    fn display_max_value() {
        assert_eq!(
            format!("{}", SqlDateTime::MAX_VALUE),
            "9999-12-31 23:59:59.997"
        );
    }

    #[test]
    fn display_min_value() {
        assert_eq!(
            format!("{}", SqlDateTime::MIN_VALUE),
            "1753-01-01 00:00:00.000"
        );
    }

    // ── T026: US6 — FromStr ──────────────────────────────────────────────────

    #[test]
    fn parse_null() {
        let dt: SqlDateTime = "Null".parse().unwrap();
        assert!(dt.is_null());
    }

    #[test]
    fn parse_null_case_insensitive() {
        let dt: SqlDateTime = "null".parse().unwrap();
        assert!(dt.is_null());
        let dt2: SqlDateTime = "NULL".parse().unwrap();
        assert!(dt2.is_null());
    }

    #[test]
    fn parse_full_with_space() {
        let dt: SqlDateTime = "2025-07-17 12:30:00.000".parse().unwrap();
        assert_eq!(
            dt.day_ticks().unwrap(),
            SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0)
                .unwrap()
                .day_ticks()
                .unwrap()
        );
    }

    #[test]
    fn parse_without_ms() {
        let dt: SqlDateTime = "2025-07-17 12:30:00".parse().unwrap();
        let expected = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(dt.value().unwrap(), expected.value().unwrap());
    }

    #[test]
    fn parse_with_t_separator() {
        let dt: SqlDateTime = "2025-07-17T12:30:00.000".parse().unwrap();
        let expected = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(dt.value().unwrap(), expected.value().unwrap());
    }

    #[test]
    fn parse_with_t_no_ms() {
        let dt: SqlDateTime = "2025-07-17T12:30:00".parse().unwrap();
        let expected = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(dt.value().unwrap(), expected.value().unwrap());
    }

    #[test]
    fn parse_date_only() {
        let dt: SqlDateTime = "2025-07-17".parse().unwrap();
        let expected = SqlDateTime::new(2025, 7, 17, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.value().unwrap(), expected.value().unwrap());
    }

    #[test]
    fn parse_invalid_string() {
        let result = "abc".parse::<SqlDateTime>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_out_of_range_year_low() {
        let result = "1752-01-01 00:00:00".parse::<SqlDateTime>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_out_of_range_year_high() {
        let result = "10000-01-01 00:00:00".parse::<SqlDateTime>();
        assert!(result.is_err());
    }

    #[test]
    fn display_fromstr_roundtrip() {
        let dt = SqlDateTime::new(2025, 7, 17, 14, 30, 45, 333.0).unwrap();
        let s = format!("{}", dt);
        let parsed: SqlDateTime = s.parse().unwrap();
        assert_eq!(dt.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn display_fromstr_roundtrip_min() {
        let s = format!("{}", SqlDateTime::MIN_VALUE);
        let parsed: SqlDateTime = s.parse().unwrap();
        assert_eq!(
            SqlDateTime::MIN_VALUE.value().unwrap(),
            parsed.value().unwrap()
        );
    }

    #[test]
    fn display_fromstr_roundtrip_max() {
        let s = format!("{}", SqlDateTime::MAX_VALUE);
        let parsed: SqlDateTime = s.parse().unwrap();
        assert_eq!(
            SqlDateTime::MAX_VALUE.value().unwrap(),
            parsed.value().unwrap()
        );
    }

    // ── T029: US7 — Leap year construction ───────────────────────────────────

    #[test]
    fn leap_year_2024_feb_29() {
        // 2024 is divisible by 4 → leap year
        assert!(SqlDateTime::new(2024, 2, 29, 0, 0, 0, 0.0).is_ok());
    }

    #[test]
    fn non_leap_year_2023_feb_29() {
        // 2023 is not divisible by 4
        assert!(SqlDateTime::new(2023, 2, 29, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn leap_year_2000_feb_29() {
        // 2000 is divisible by 400 → leap year
        assert!(SqlDateTime::new(2000, 2, 29, 0, 0, 0, 0.0).is_ok());
    }

    #[test]
    fn non_leap_year_1900_feb_29() {
        // 1900 is divisible by 100 but not 400 → not leap year
        assert!(SqlDateTime::new(1900, 2, 29, 0, 0, 0, 0.0).is_err());
    }

    #[test]
    fn arithmetic_across_leap_boundary_2024() {
        // Feb 28, 2024 + 1 day → Feb 29 (leap year)
        let dt = SqlDateTime::new(2024, 2, 28, 0, 0, 0, 0.0).unwrap();
        let next = dt.checked_add_days(1).unwrap();
        let feb29 = SqlDateTime::new(2024, 2, 29, 0, 0, 0, 0.0).unwrap();
        assert_eq!(next.value().unwrap(), feb29.value().unwrap());
    }

    #[test]
    fn arithmetic_across_leap_boundary_2023() {
        // Feb 28, 2023 + 1 day → Mar 1 (non-leap year)
        let dt = SqlDateTime::new(2023, 2, 28, 0, 0, 0, 0.0).unwrap();
        let next = dt.checked_add_days(1).unwrap();
        let mar1 = SqlDateTime::new(2023, 3, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(next.value().unwrap(), mar1.value().unwrap());
    }

    // ── T030: US8 — Date component extraction ───────────────────────────────

    #[test]
    fn year_representative() {
        let dt = SqlDateTime::new(2025, 7, 17, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.year().unwrap(), 2025);
    }

    #[test]
    fn month_representative() {
        let dt = SqlDateTime::new(2025, 7, 17, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.month().unwrap(), 7);
    }

    #[test]
    fn day_representative() {
        let dt = SqlDateTime::new(2025, 7, 17, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.day().unwrap(), 17);
    }

    #[test]
    fn ymd_epoch() {
        let dt = SqlDateTime::new(1900, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.year().unwrap(), 1900);
        assert_eq!(dt.month().unwrap(), 1);
        assert_eq!(dt.day().unwrap(), 1);
    }

    #[test]
    fn ymd_min_value() {
        assert_eq!(SqlDateTime::MIN_VALUE.year().unwrap(), 1753);
        assert_eq!(SqlDateTime::MIN_VALUE.month().unwrap(), 1);
        assert_eq!(SqlDateTime::MIN_VALUE.day().unwrap(), 1);
    }

    #[test]
    fn ymd_max_value() {
        assert_eq!(SqlDateTime::MAX_VALUE.year().unwrap(), 9999);
        assert_eq!(SqlDateTime::MAX_VALUE.month().unwrap(), 12);
        assert_eq!(SqlDateTime::MAX_VALUE.day().unwrap(), 31);
    }

    #[test]
    fn year_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.year(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn month_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.month(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn day_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.day(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T031: US8 — Time component extraction ───────────────────────────────

    #[test]
    fn hour_representative() {
        let dt = SqlDateTime::new(2025, 1, 1, 14, 30, 45, 0.0).unwrap();
        assert_eq!(dt.hour().unwrap(), 14);
    }

    #[test]
    fn minute_representative() {
        let dt = SqlDateTime::new(2025, 1, 1, 14, 30, 45, 0.0).unwrap();
        assert_eq!(dt.minute().unwrap(), 30);
    }

    #[test]
    fn second_representative() {
        let dt = SqlDateTime::new(2025, 1, 1, 14, 30, 45, 0.0).unwrap();
        assert_eq!(dt.second().unwrap(), 45);
    }

    #[test]
    fn hms_midnight() {
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_eq!(dt.hour().unwrap(), 0);
        assert_eq!(dt.minute().unwrap(), 0);
        assert_eq!(dt.second().unwrap(), 0);
    }

    #[test]
    fn hms_max_time() {
        let dt = SqlDateTime::new(2025, 1, 1, 23, 59, 59, 997.0).unwrap();
        assert_eq!(dt.hour().unwrap(), 23);
        assert_eq!(dt.minute().unwrap(), 59);
        assert_eq!(dt.second().unwrap(), 59);
    }

    #[test]
    fn hour_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.hour(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn minute_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.minute(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn second_null_returns_err() {
        assert!(matches!(
            SqlDateTime::NULL.second(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T034: PartialEq / Eq ─────────────────────────────────────────────────

    #[test]
    fn eq_same_values() {
        let a = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn eq_different_values() {
        let a = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 7, 18, 12, 30, 0, 0.0).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn eq_null_equals_null() {
        // Rust PartialEq: NULL == NULL is true (structural equality)
        assert_eq!(SqlDateTime::NULL, SqlDateTime::NULL);
    }

    #[test]
    fn eq_null_ne_non_null() {
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert_ne!(SqlDateTime::NULL, dt);
    }

    // ── T035: Hash ───────────────────────────────────────────────────────────

    #[test]
    fn hash_equal_values_hash_equal() {
        use std::collections::hash_map::DefaultHasher;
        let a = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn hash_null_consistent() {
        use std::collections::hash_map::DefaultHasher;
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        SqlDateTime::NULL.hash(&mut h1);
        SqlDateTime::NULL.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    // ── T036: PartialOrd / Ord ───────────────────────────────────────────────

    #[test]
    fn ord_null_less_than_non_null() {
        let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0).unwrap();
        assert!(SqlDateTime::NULL < dt);
    }

    #[test]
    fn ord_earlier_date_less_than_later() {
        let earlier = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0).unwrap();
        let later = SqlDateTime::new(2025, 1, 2, 0, 0, 0, 0.0).unwrap();
        assert!(earlier < later);
    }

    #[test]
    fn ord_same_date_earlier_time_less() {
        let earlier = SqlDateTime::new(2025, 1, 1, 10, 0, 0, 0.0).unwrap();
        let later = SqlDateTime::new(2025, 1, 1, 14, 0, 0, 0.0).unwrap();
        assert!(earlier < later);
    }

    #[test]
    fn ord_equal_values_compare_equal() {
        let a = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        let b = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ord_null_null_equal() {
        assert_eq!(SqlDateTime::NULL.cmp(&SqlDateTime::NULL), Ordering::Equal);
    }
}
