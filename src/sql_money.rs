// ── T001: SqlMoney module ──────────────────────────────────────────────────────

//! `SqlMoney` — a fixed-point currency type with 4 decimal places and SQL NULL
//! support, equivalent to C# `System.Data.SqlTypes.SqlMoney` / SQL Server `MONEY`.
//!
//! Uses `Option<i64>` internally: `None` = SQL NULL, `Some(v)` = monetary value × 10,000.
//! Range: −922,337,203,685,477.5808 to 922,337,203,685,477.5807.
//! All arithmetic returns `Result<SqlMoney, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_double::SqlDouble;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use crate::sql_int64::SqlInt64;
use crate::sql_single::SqlSingle;
use crate::sql_string::SqlString;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A fixed-point currency value (−922,337,203,685,477.5808 to
/// 922,337,203,685,477.5807) with 4 decimal places and SQL NULL support,
/// equivalent to C# `System.Data.SqlTypes.SqlMoney` / SQL Server `MONEY`.
///
/// Uses `Option<i64>` internally: `None` = SQL NULL, `Some(v)` = monetary
/// value × 10,000. All arithmetic returns `Result<SqlMoney, SqlTypeError>`
/// with overflow detection. Comparisons return `SqlBoolean` for three-valued
/// NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlMoney {
    value: Option<i64>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors ─────────────────────────────────────────────────────
// ── T006: Accessors ────────────────────────────────────────────────────────

impl SqlMoney {
    /// Internal scale factor: all values stored as `actual_value × 10,000`.
    const SCALE: i64 = 10_000;

    /// SQL NULL.
    pub const NULL: SqlMoney = SqlMoney { value: None };

    /// Zero (0.0000).
    pub const ZERO: SqlMoney = SqlMoney { value: Some(0) };

    /// Minimum representable value: −922,337,203,685,477.5808 (internal: `i64::MIN`).
    pub const MIN_VALUE: SqlMoney = SqlMoney {
        value: Some(i64::MIN),
    };

    /// Maximum representable value: 922,337,203,685,477.5807 (internal: `i64::MAX`).
    pub const MAX_VALUE: SqlMoney = SqlMoney {
        value: Some(i64::MAX),
    };

    // ── Constructors ────────────────────────────────────────────────────────

    /// Create from `i32`. Always succeeds — `i32 × 10,000` fits in `i64`.
    pub fn from_i32(v: i32) -> Self {
        SqlMoney {
            value: Some(v as i64 * Self::SCALE),
        }
    }

    /// Create from `i64`. Range-checked: `value × 10,000` must fit in `i64`.
    ///
    /// # Errors
    /// * `Overflow` — if `value × 10,000` overflows `i64`
    pub fn from_i64(v: i64) -> Result<Self, SqlTypeError> {
        v.checked_mul(Self::SCALE)
            .map(|scaled| SqlMoney {
                value: Some(scaled),
            })
            .ok_or(SqlTypeError::Overflow)
    }

    /// Create from `f64`. Reject NaN/Infinity, round to 4 decimal places,
    /// range-check against `i64`.
    ///
    /// # Errors
    /// * `OutOfRange` — if `v` is NaN or Infinity
    /// * `Overflow` — if the rounded scaled value doesn't fit in `i64`
    pub fn from_f64(v: f64) -> Result<Self, SqlTypeError> {
        if !v.is_finite() {
            return Err(SqlTypeError::OutOfRange(
                "SqlMoney cannot be NaN or Infinity".to_string(),
            ));
        }
        let scaled = (v * Self::SCALE as f64).round();
        // Check range: i64::MIN as f64 is exactly representable (-2^63),
        // i64::MAX as f64 rounds up to 2^63 so we use < (not <=).
        if scaled < i64::MIN as f64 || scaled >= (i64::MAX as f64 + 1.0) {
            return Err(SqlTypeError::Overflow);
        }
        Ok(SqlMoney {
            value: Some(scaled as i64),
        })
    }

    /// Create from a raw scaled value (no scaling applied, no validation).
    /// The value is stored directly as the internal tick representation.
    pub fn from_scaled(v: i64) -> Self {
        SqlMoney { value: Some(v) }
    }

    // ── Accessors ───────────────────────────────────────────────────────────

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the raw internal scaled value (ticks).
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn scaled_value(&self) -> Result<i64, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }

    /// Convert to `i64`, rounding half-away-from-zero.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn to_i64(&self) -> Result<i64, SqlTypeError> {
        let v = self.value.ok_or(SqlTypeError::NullValue)?;
        // Round-half-away-from-zero matching C# SqlMoney.ToInt64():
        // Divide by 1000 to keep one extra digit, check remainder, round.
        let div_1000 = v / 1000;
        let remainder = (div_1000 % 10).abs();
        let mut result = div_1000 / 10;
        if remainder >= 5 {
            if v >= 0 {
                result += 1;
            } else {
                result -= 1;
            }
        }
        Ok(result)
    }

    /// Convert to `i32`, rounding half-away-from-zero then range-checking.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    /// * `Overflow` — if the rounded value doesn't fit in `i32`
    pub fn to_i32(&self) -> Result<i32, SqlTypeError> {
        let v = self.to_i64()?;
        i32::try_from(v).map_err(|_| SqlTypeError::Overflow)
    }

    /// Convert to `f64`.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn to_f64(&self) -> Result<f64, SqlTypeError> {
        let v = self.value.ok_or(SqlTypeError::NullValue)?;
        Ok(v as f64 / Self::SCALE as f64)
    }
}

// ── Checked Arithmetic ──────────────────────────────────────────────────────

impl SqlMoney {
    /// Checked addition. Returns `Err(Overflow)` if the result overflows `i64`.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlMoney::NULL)`.
    pub fn checked_add(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlMoney::NULL),
            (Some(a), Some(b)) => a
                .checked_add(b)
                .map(|v| SqlMoney { value: Some(v) })
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if the result overflows `i64`.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlMoney::NULL)`.
    pub fn checked_sub(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlMoney::NULL),
            (Some(a), Some(b)) => a
                .checked_sub(b)
                .map(|v| SqlMoney { value: Some(v) })
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked multiplication using i128 intermediate.
    /// `(a_ticks × b_ticks) / SCALE` with rounding.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlMoney::NULL)`.
    pub fn checked_mul(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlMoney::NULL),
            (Some(a), Some(b)) => {
                let product = (a as i128) * (b as i128);
                let scale = Self::SCALE as i128;
                let quotient = product / scale;
                let remainder = product % scale;
                // Round-half-away-from-zero
                let rounded = if remainder.abs() * 2 >= scale.abs() {
                    if product >= 0 {
                        quotient + 1
                    } else {
                        quotient - 1
                    }
                } else {
                    quotient
                };
                i64::try_from(rounded)
                    .map(|v| SqlMoney { value: Some(v) })
                    .map_err(|_| SqlTypeError::Overflow)
            }
        }
    }

    /// Checked division using i128 intermediate.
    /// `(a_ticks × SCALE) / b_ticks` with rounding.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlMoney::NULL)`.
    ///
    /// # Errors
    /// * `DivideByZero` — if `rhs` internal value is zero
    /// * `Overflow` — if the result doesn't fit in `i64`
    pub fn checked_div(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlMoney::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => {
                let dividend = (a as i128) * (Self::SCALE as i128);
                let divisor = b as i128;
                let quotient = dividend / divisor;
                let remainder = dividend % divisor;
                // Round-half-away-from-zero
                let rounded = if (remainder.abs() * 2) >= divisor.abs() {
                    if (dividend >= 0) == (divisor >= 0) {
                        quotient + 1
                    } else {
                        quotient - 1
                    }
                } else {
                    quotient
                };
                i64::try_from(rounded)
                    .map(|v| SqlMoney { value: Some(v) })
                    .map_err(|_| SqlTypeError::Overflow)
            }
        }
    }

    /// Checked negation. Returns `Err(Overflow)` if value is `i64::MIN`.
    /// NULL propagation: if NULL, returns `Ok(SqlMoney::NULL)`.
    pub fn checked_neg(self) -> Result<SqlMoney, SqlTypeError> {
        match self.value {
            None => Ok(SqlMoney::NULL),
            Some(v) => v
                .checked_neg()
                .map(|neg| SqlMoney { value: Some(neg) })
                .ok_or(SqlTypeError::Overflow),
        }
    }
}

// ── Operator Traits ─────────────────────────────────────────────────────────

impl Add for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn add(self, rhs: SqlMoney) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Add<&SqlMoney> for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn add(self, rhs: &SqlMoney) -> Self::Output {
        self.checked_add(*rhs)
    }
}

impl Add<SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn add(self, rhs: SqlMoney) -> Self::Output {
        (*self).checked_add(rhs)
    }
}

impl Add<&SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn add(self, rhs: &SqlMoney) -> Self::Output {
        (*self).checked_add(*rhs)
    }
}

impl Sub for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn sub(self, rhs: SqlMoney) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<&SqlMoney> for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn sub(self, rhs: &SqlMoney) -> Self::Output {
        self.checked_sub(*rhs)
    }
}

impl Sub<SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn sub(self, rhs: SqlMoney) -> Self::Output {
        (*self).checked_sub(rhs)
    }
}

impl Sub<&SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn sub(self, rhs: &SqlMoney) -> Self::Output {
        (*self).checked_sub(*rhs)
    }
}

impl Mul for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn mul(self, rhs: SqlMoney) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<&SqlMoney> for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn mul(self, rhs: &SqlMoney) -> Self::Output {
        self.checked_mul(*rhs)
    }
}

impl Mul<SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn mul(self, rhs: SqlMoney) -> Self::Output {
        (*self).checked_mul(rhs)
    }
}

impl Mul<&SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn mul(self, rhs: &SqlMoney) -> Self::Output {
        (*self).checked_mul(*rhs)
    }
}

impl Div for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn div(self, rhs: SqlMoney) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<&SqlMoney> for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn div(self, rhs: &SqlMoney) -> Self::Output {
        self.checked_div(*rhs)
    }
}

impl Div<SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn div(self, rhs: SqlMoney) -> Self::Output {
        (*self).checked_div(rhs)
    }
}

impl Div<&SqlMoney> for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn div(self, rhs: &SqlMoney) -> Self::Output {
        (*self).checked_div(*rhs)
    }
}

impl Neg for SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

impl Neg for &SqlMoney {
    type Output = Result<SqlMoney, SqlTypeError>;
    fn neg(self) -> Self::Output {
        (*self).checked_neg()
    }
}

// ── SQL Comparisons ─────────────────────────────────────────────────────────

impl SqlMoney {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlMoney) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── Display / FromStr ───────────────────────────────────────────────────────

impl fmt::Display for SqlMoney {
    /// Display format `"#0.00##"`: minimum 2 decimal places, maximum 4,
    /// trimming trailing zeros beyond the 2nd place. NULL displays as `"Null"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            None => write!(f, "Null"),
            Some(v) => {
                let sign = if v < 0 { "-" } else { "" };
                let abs_v = (v as i128).unsigned_abs(); // avoid i64::MIN overflow
                let integer_part = abs_v / Self::SCALE as u128;
                let frac = abs_v % Self::SCALE as u128;
                // Format fractional part: always 4 digits, then trim trailing
                // zeros to a minimum of 2.
                let frac_str = format!("{frac:04}");
                let trimmed = frac_str.trim_end_matches('0');
                let frac_display = if trimmed.len() < 2 {
                    &frac_str[..2]
                } else {
                    trimmed
                };
                write!(f, "{sign}{integer_part}.{frac_display}")
            }
        }
    }
}

impl FromStr for SqlMoney {
    type Err = SqlTypeError;

    /// Parse a decimal string into `SqlMoney`.
    /// `"Null"` (case-insensitive) → `SqlMoney::NULL`.
    /// Supports optional sign, integer and fractional parts.
    /// Values with more than 4 decimal places are rounded.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlMoney::NULL);
        }

        // Determine sign
        let (negative, num_str) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = trimmed.strip_prefix('+') {
            (false, rest)
        } else {
            (false, trimmed)
        };

        if num_str.is_empty() {
            return Err(SqlTypeError::ParseError("Empty numeric string".to_string()));
        }

        // Split into integer and fractional parts
        let (int_str, frac_str) = if let Some((i, f)) = num_str.split_once('.') {
            (i, f)
        } else {
            (num_str, "")
        };

        // Validate: digits only
        if !int_str.chars().all(|c| c.is_ascii_digit())
            || (int_str.is_empty() && frac_str.is_empty())
        {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid SqlMoney string: {s}"
            )));
        }
        if !frac_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid SqlMoney string: {s}"
            )));
        }

        // Parse integer part
        let int_val: i128 = if int_str.is_empty() {
            0
        } else {
            int_str
                .parse::<i128>()
                .map_err(|_| SqlTypeError::ParseError(format!("Invalid SqlMoney string: {s}")))?
        };

        // Parse fractional part, pad/truncate to 5 digits for rounding
        let frac_5: i128 = if frac_str.is_empty() {
            0
        } else if frac_str.len() <= 4 {
            // Pad to 4 digits: "45" -> 4500
            let padded = format!("{frac_str:0<4}");
            padded.parse::<i128>().unwrap() * 10 // to 5 digits for rounding
        } else {
            // Take first 5 digits for rounding
            let five = &frac_str[..5];
            five.parse::<i128>()
                .map_err(|_| SqlTypeError::ParseError(format!("Invalid SqlMoney string: {s}")))?
        };

        // frac_5 is in units of 0.00001 (5 decimal places)
        // Round to 4 decimal places: round-half-away-from-zero on the 5th digit
        let frac_4 = (frac_5 + 5) / 10; // e.g., 45678 → (45678+5)/10 = 45683/10 = 4568

        // Combine: scaled = int_val * 10_000 + frac_4
        let scaled = int_val * Self::SCALE as i128 + frac_4;

        let signed = if negative { -scaled } else { scaled };

        i64::try_from(signed)
            .map(|v| SqlMoney { value: Some(v) })
            .map_err(|_| SqlTypeError::Overflow)
    }
}

// ── Conversions: Widening INTO SqlMoney ─────────────────────────────────────

impl From<SqlBoolean> for SqlMoney {
    fn from(v: SqlBoolean) -> Self {
        if v.is_null() {
            SqlMoney::NULL
        } else if v.is_true() {
            SqlMoney::from_i32(1)
        } else {
            SqlMoney::ZERO
        }
    }
}

impl From<SqlByte> for SqlMoney {
    fn from(v: SqlByte) -> Self {
        match v.value() {
            Ok(val) => SqlMoney {
                value: Some(val as i64 * SqlMoney::SCALE),
            },
            Err(_) => SqlMoney::NULL,
        }
    }
}

impl From<SqlInt16> for SqlMoney {
    fn from(v: SqlInt16) -> Self {
        match v.value() {
            Ok(val) => SqlMoney {
                value: Some(val as i64 * SqlMoney::SCALE),
            },
            Err(_) => SqlMoney::NULL,
        }
    }
}

impl From<SqlInt32> for SqlMoney {
    fn from(v: SqlInt32) -> Self {
        match v.value() {
            Ok(val) => SqlMoney {
                value: Some(val as i64 * SqlMoney::SCALE),
            },
            Err(_) => SqlMoney::NULL,
        }
    }
}

// ── Conversions: Fallible INTO SqlMoney ─────────────────────────────────────

impl SqlMoney {
    /// Create from `SqlInt64`, range-checked: `value × 10,000` must fit in `i64`.
    /// NULL → `Ok(SqlMoney::NULL)`.
    pub fn from_sql_int64(v: SqlInt64) -> Result<Self, SqlTypeError> {
        match v.value() {
            Err(_) => Ok(SqlMoney::NULL),
            Ok(val) => SqlMoney::from_i64(val),
        }
    }
}

// ── Conversions: OUT of SqlMoney ────────────────────────────────────────────

impl SqlMoney {
    /// Convert to `SqlInt64`, rounding half-away-from-zero.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError> {
        self.to_i64().map(SqlInt64::new)
    }

    /// Convert to `SqlInt32`, rounding half-away-from-zero then range-checking.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    /// * `Overflow` — if the rounded value doesn't fit in `i32`
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError> {
        self.to_i32().map(SqlInt32::new)
    }

    /// Convert to `SqlInt16`, rounding half-away-from-zero then range-checking.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    /// * `Overflow` — if the rounded value doesn't fit in `i16`
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError> {
        let v = self.to_i64()?;
        i16::try_from(v)
            .map(SqlInt16::new)
            .map_err(|_| SqlTypeError::Overflow)
    }

    /// Convert to `SqlByte`, rounding half-away-from-zero then range-checking.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    /// * `Overflow` — if the rounded value doesn't fit in `u8`
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError> {
        let v = self.to_i64()?;
        u8::try_from(v)
            .map(SqlByte::new)
            .map_err(|_| SqlTypeError::Overflow)
    }

    /// Convert to `SqlBoolean`. Zero → FALSE, non-zero → TRUE, NULL → NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// Convert to `SqlDecimal` with exact representation and scale = 4.
    /// NULL → `SqlDecimal::NULL`.
    pub fn to_sql_decimal(&self) -> crate::sql_decimal::SqlDecimal {
        use crate::sql_decimal::SqlDecimal;
        match self.value {
            None => SqlDecimal::NULL,
            Some(v) => {
                let positive = v >= 0;
                let abs_val = (v as i128).unsigned_abs();
                let data1 = abs_val as u32;
                let data2 = (abs_val >> 32) as u32;
                // Maximum precision needed: i64::MAX = 9_223_372_036_854_775_807
                // which is 19 digits, so precision = 19. Scale = 4 since
                // internal value is already × 10,000.
                SqlDecimal::new(19, 4, positive, data1, data2, 0, 0)
                    .expect("SqlMoney value always fits in SqlDecimal(19,4)")
            }
        }
    }
}

impl SqlMoney {
    /// Creates `SqlMoney` from `SqlSingle`. NULL → `Ok(NULL)`.
    /// Returns `Err(Overflow)` if the value overflows the money range.
    pub fn from_sql_single(v: SqlSingle) -> Result<SqlMoney, SqlTypeError> {
        if v.is_null() {
            return Ok(SqlMoney::NULL);
        }
        SqlMoney::from_f64(v.value().unwrap() as f64)
    }

    /// Creates `SqlMoney` from `SqlDouble`. NULL → `Ok(NULL)`.
    /// Returns `Err(Overflow)` if the value overflows the money range.
    pub fn from_sql_double(v: SqlDouble) -> Result<SqlMoney, SqlTypeError> {
        if v.is_null() {
            return Ok(SqlMoney::NULL);
        }
        SqlMoney::from_f64(v.value().unwrap())
    }

    /// Converts to `SqlSingle`. NULL → NULL.
    pub fn to_sql_single(&self) -> SqlSingle {
        if self.is_null() {
            SqlSingle::NULL
        } else {
            let raw = self.scaled_value().unwrap();
            let f = raw as f64 / 10_000.0;
            SqlSingle::new(f as f32).unwrap_or(SqlSingle::NULL)
        }
    }

    /// Converts to `SqlDouble`. NULL → NULL.
    pub fn to_sql_double(&self) -> SqlDouble {
        if self.is_null() {
            SqlDouble::NULL
        } else {
            let raw = self.scaled_value().unwrap();
            let f = raw as f64 / 10_000.0;
            SqlDouble::new(f).unwrap_or(SqlDouble::NULL)
        }
    }
}

impl SqlMoney {
    /// Converts to `SqlString` via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString {
        if self.is_null() {
            SqlString::NULL
        } else {
            SqlString::new(&format!("{self}"))
        }
    }
}

// ── Rust Standard Traits ────────────────────────────────────────────────────

impl PartialEq for SqlMoney {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlMoney {}

impl Hash for SqlMoney {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.value {
            None => 0i64.hash(state),
            Some(v) => v.hash(state),
        }
    }
}

impl PartialOrd for SqlMoney {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlMoney {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.value, other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── T007: Tests for from_i32() ──────────────────────────────────────────

    #[test]
    fn from_i32_positive() {
        let m = SqlMoney::from_i32(100);
        assert_eq!(m.scaled_value().unwrap(), 1_000_000);
    }

    #[test]
    fn from_i32_negative() {
        let m = SqlMoney::from_i32(-50);
        assert_eq!(m.scaled_value().unwrap(), -500_000);
    }

    #[test]
    fn from_i32_zero() {
        let m = SqlMoney::from_i32(0);
        assert_eq!(m.scaled_value().unwrap(), 0);
    }

    #[test]
    fn from_i32_max() {
        let m = SqlMoney::from_i32(i32::MAX);
        assert_eq!(m.scaled_value().unwrap(), i32::MAX as i64 * 10_000);
    }

    #[test]
    fn from_i32_min() {
        let m = SqlMoney::from_i32(i32::MIN);
        assert_eq!(m.scaled_value().unwrap(), i32::MIN as i64 * 10_000);
    }

    // ── T008: Tests for from_i64() ──────────────────────────────────────────

    #[test]
    fn from_i64_in_range() {
        let m = SqlMoney::from_i64(922_337_203_685_477).unwrap();
        assert_eq!(m.scaled_value().unwrap(), 9_223_372_036_854_770_000);
    }

    #[test]
    fn from_i64_overflow_positive() {
        // i64::MAX / 10_000 = 922_337_203_685_477 (max valid)
        // 922_337_203_685_478 would overflow
        assert!(matches!(
            SqlMoney::from_i64(922_337_203_685_478),
            Err(SqlTypeError::Overflow)
        ));
    }

    #[test]
    fn from_i64_overflow_negative() {
        // i64::MIN / 10_000 = -922_337_203_685_477 (but i64::MIN % 10_000 != 0)
        // -922_337_203_685_478 would overflow
        assert!(matches!(
            SqlMoney::from_i64(-922_337_203_685_478),
            Err(SqlTypeError::Overflow)
        ));
    }

    #[test]
    fn from_i64_boundary_max() {
        let max_val = i64::MAX / 10_000; // 922_337_203_685_477
        let m = SqlMoney::from_i64(max_val).unwrap();
        assert_eq!(m.scaled_value().unwrap(), max_val * 10_000);
    }

    #[test]
    fn from_i64_boundary_min() {
        let min_val = i64::MIN / 10_000; // -922_337_203_685_477 (rounds toward zero)
        let m = SqlMoney::from_i64(min_val).unwrap();
        assert_eq!(m.scaled_value().unwrap(), min_val * 10_000);
    }

    // ── T009: Tests for from_f64() ──────────────────────────────────────────

    #[test]
    fn from_f64_exact_4dp() {
        let m = SqlMoney::from_f64(123.4567).unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_234_567);
    }

    #[test]
    fn from_f64_rounding_beyond_4dp() {
        // 123.45678 → 123.4568 (rounded to 4dp)
        let m = SqlMoney::from_f64(123.45678).unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_234_568);
    }

    #[test]
    fn from_f64_nan() {
        assert!(matches!(
            SqlMoney::from_f64(f64::NAN),
            Err(SqlTypeError::OutOfRange(_))
        ));
    }

    #[test]
    fn from_f64_infinity() {
        assert!(matches!(
            SqlMoney::from_f64(f64::INFINITY),
            Err(SqlTypeError::OutOfRange(_))
        ));
    }

    #[test]
    fn from_f64_neg_infinity() {
        assert!(matches!(
            SqlMoney::from_f64(f64::NEG_INFINITY),
            Err(SqlTypeError::OutOfRange(_))
        ));
    }

    #[test]
    fn from_f64_range_overflow() {
        // A value that exceeds i64 range when scaled
        assert!(matches!(
            SqlMoney::from_f64(1e18),
            Err(SqlTypeError::Overflow)
        ));
    }

    // ── T010: Tests for from_scaled() ───────────────────────────────────────

    #[test]
    fn from_scaled_any_value() {
        let m = SqlMoney::from_scaled(42);
        assert_eq!(m.scaled_value().unwrap(), 42);
    }

    #[test]
    fn from_scaled_i64_max() {
        let m = SqlMoney::from_scaled(i64::MAX);
        assert_eq!(m.scaled_value().unwrap(), i64::MAX);
    }

    #[test]
    fn from_scaled_i64_min() {
        let m = SqlMoney::from_scaled(i64::MIN);
        assert_eq!(m.scaled_value().unwrap(), i64::MIN);
    }

    #[test]
    fn from_scaled_zero() {
        let m = SqlMoney::from_scaled(0);
        assert_eq!(m.scaled_value().unwrap(), 0);
    }

    // ── T011: Tests for constants ───────────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlMoney::NULL.is_null());
    }

    #[test]
    fn zero_value() {
        assert_eq!(SqlMoney::ZERO.scaled_value().unwrap(), 0);
    }

    #[test]
    fn min_value() {
        assert_eq!(SqlMoney::MIN_VALUE.scaled_value().unwrap(), i64::MIN);
    }

    #[test]
    fn max_value() {
        assert_eq!(SqlMoney::MAX_VALUE.scaled_value().unwrap(), i64::MAX);
    }

    // ── T012: Tests for is_null(), scaled_value() ───────────────────────────

    #[test]
    fn is_null_non_null() {
        assert!(!SqlMoney::from_i32(42).is_null());
    }

    #[test]
    fn is_null_null() {
        assert!(SqlMoney::NULL.is_null());
    }

    #[test]
    fn scaled_value_on_null() {
        assert!(matches!(
            SqlMoney::NULL.scaled_value(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn scaled_value_on_value() {
        let m = SqlMoney::from_i32(100);
        assert_eq!(m.scaled_value().unwrap(), 1_000_000);
    }

    // ── T013: Tests for to_i64(), to_i32(), to_f64() ────────────────────────

    #[test]
    fn to_i64_exact() {
        let m = SqlMoney::from_i32(42);
        assert_eq!(m.to_i64().unwrap(), 42);
    }

    #[test]
    fn to_i64_round_up() {
        // 42.5 → 43 (round-half-away-from-zero)
        let m = SqlMoney::from_scaled(425_000); // 42.5000
        assert_eq!(m.to_i64().unwrap(), 43);
    }

    #[test]
    fn to_i64_round_up_negative() {
        // -42.5 → -43 (round-half-away-from-zero)
        let m = SqlMoney::from_scaled(-425_000);
        assert_eq!(m.to_i64().unwrap(), -43);
    }

    #[test]
    fn to_i64_round_down() {
        // 42.4 → 42
        let m = SqlMoney::from_scaled(424_000);
        assert_eq!(m.to_i64().unwrap(), 42);
    }

    #[test]
    fn to_i64_null() {
        assert!(matches!(
            SqlMoney::NULL.to_i64(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn to_i32_in_range() {
        let m = SqlMoney::from_i32(42);
        assert_eq!(m.to_i32().unwrap(), 42);
    }

    #[test]
    fn to_i32_overflow() {
        // Value too large for i32 after rounding
        let m = SqlMoney::from_i64(i32::MAX as i64 + 1).unwrap();
        assert!(matches!(m.to_i32(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_i32_null() {
        assert!(matches!(
            SqlMoney::NULL.to_i32(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn to_f64_value() {
        let m = SqlMoney::from_i32(42);
        let f = m.to_f64().unwrap();
        assert!((f - 42.0).abs() < 1e-10);
    }

    #[test]
    fn to_f64_fractional() {
        let m = SqlMoney::from_scaled(1_234_567); // 123.4567
        let f = m.to_f64().unwrap();
        assert!((f - 123.4567).abs() < 1e-10);
    }

    #[test]
    fn to_f64_null() {
        assert!(matches!(
            SqlMoney::NULL.to_f64(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T014: Tests for checked_add ─────────────────────────────────────────

    #[test]
    fn checked_add_normal() {
        let a = SqlMoney::from_scaled(1_000_000); // 100.0000
        let b = SqlMoney::from_scaled(502_500); // 50.2500
        let r = a.checked_add(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 1_502_500); // 150.2500
    }

    #[test]
    fn checked_add_exact_no_rounding() {
        // Verify add is exact i64 arithmetic
        let a = SqlMoney::from_scaled(1);
        let b = SqlMoney::from_scaled(2);
        assert_eq!(a.checked_add(b).unwrap().scaled_value().unwrap(), 3);
    }

    #[test]
    fn checked_add_overflow() {
        let a = SqlMoney::MAX_VALUE;
        let b = SqlMoney::from_scaled(1);
        assert!(matches!(a.checked_add(b), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_add_null_left() {
        let r = SqlMoney::NULL.checked_add(SqlMoney::from_i32(1)).unwrap();
        assert!(r.is_null());
    }

    #[test]
    fn checked_add_null_right() {
        let r = SqlMoney::from_i32(1).checked_add(SqlMoney::NULL).unwrap();
        assert!(r.is_null());
    }

    // ── T015: Tests for checked_sub ─────────────────────────────────────────

    #[test]
    fn checked_sub_normal() {
        let a = SqlMoney::from_scaled(1_000_000); // 100.0000
        let b = SqlMoney::from_scaled(2_000_000); // 200.0000
        let r = a.checked_sub(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), -1_000_000); // -100.0000
    }

    #[test]
    fn checked_sub_underflow() {
        let a = SqlMoney::MIN_VALUE;
        let b = SqlMoney::from_scaled(1);
        assert!(matches!(a.checked_sub(b), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_sub_null() {
        let r = SqlMoney::NULL.checked_sub(SqlMoney::from_i32(1)).unwrap();
        assert!(r.is_null());
    }

    // ── T016: Tests for checked_mul ─────────────────────────────────────────

    #[test]
    fn checked_mul_normal() {
        let a = SqlMoney::from_scaled(1_000_000); // 100.0000
        let b = SqlMoney::from_scaled(25_000); // 2.5000
        let r = a.checked_mul(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 2_500_000); // 250.0000
    }

    #[test]
    fn checked_mul_by_zero() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::ZERO;
        let r = a.checked_mul(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 0);
    }

    #[test]
    fn checked_mul_overflow() {
        let a = SqlMoney::MAX_VALUE;
        let b = SqlMoney::from_scaled(20_000); // 2.0000
        assert!(matches!(a.checked_mul(b), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_mul_null() {
        let r = SqlMoney::NULL.checked_mul(SqlMoney::from_i32(1)).unwrap();
        assert!(r.is_null());
    }

    // ── T017: Tests for checked_div ─────────────────────────────────────────

    #[test]
    fn checked_div_normal() {
        let a = SqlMoney::from_scaled(1_000_000); // 100.0000
        let b = SqlMoney::from_scaled(30_000); // 3.0000
        let r = a.checked_div(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 333_333); // 33.3333
    }

    #[test]
    fn checked_div_by_zero() {
        let a = SqlMoney::from_i32(100);
        let b = SqlMoney::ZERO;
        assert!(matches!(a.checked_div(b), Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn checked_div_null() {
        let r = SqlMoney::NULL.checked_div(SqlMoney::from_i32(1)).unwrap();
        assert!(r.is_null());
    }

    #[test]
    fn checked_div_exact() {
        let a = SqlMoney::from_scaled(1_000_000); // 100.0000
        let b = SqlMoney::from_scaled(50_000); // 5.0000
        let r = a.checked_div(b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 200_000); // 20.0000
    }

    // ── T018: Tests for checked_neg ─────────────────────────────────────────

    #[test]
    fn checked_neg_normal() {
        let m = SqlMoney::from_scaled(-1_000_000); // -100.0000
        let r = m.checked_neg().unwrap();
        assert_eq!(r.scaled_value().unwrap(), 1_000_000); // 100.0000
    }

    #[test]
    fn checked_neg_min_value_overflow() {
        // i64::MIN cannot be negated
        let m = SqlMoney::MIN_VALUE;
        assert!(matches!(m.checked_neg(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_neg_null() {
        let r = SqlMoney::NULL.checked_neg().unwrap();
        assert!(r.is_null());
    }

    #[test]
    fn checked_neg_zero() {
        let r = SqlMoney::ZERO.checked_neg().unwrap();
        assert_eq!(r.scaled_value().unwrap(), 0);
    }

    // ── T023: Tests for operator traits ─────────────────────────────────────

    #[test]
    fn add_operator() {
        let a = SqlMoney::from_i32(100);
        let b = SqlMoney::from_i32(50);
        let r = (a + b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 1_500_000);
    }

    #[test]
    fn sub_operator() {
        let a = SqlMoney::from_i32(100);
        let b = SqlMoney::from_i32(200);
        let r = (a - b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), -1_000_000);
    }

    #[test]
    fn mul_operator() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(25_000);
        let r = (a * b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 2_500_000);
    }

    #[test]
    fn div_operator() {
        let a = SqlMoney::from_i32(100);
        let b = SqlMoney::from_scaled(50_000); // 5.0
        let r = (a / b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 200_000); // 20.0
    }

    #[test]
    fn neg_operator() {
        let m = SqlMoney::from_i32(100);
        let r = (-m).unwrap();
        assert_eq!(r.scaled_value().unwrap(), -1_000_000);
    }

    #[test]
    fn add_borrowed() {
        let a = SqlMoney::from_i32(10);
        let b = SqlMoney::from_i32(20);
        let r = (&a + &b).unwrap();
        assert_eq!(r.scaled_value().unwrap(), 300_000);
    }

    #[test]
    fn neg_borrowed() {
        let m = SqlMoney::from_i32(5);
        let r = (-&m).unwrap();
        assert_eq!(r.scaled_value().unwrap(), -50_000);
    }

    // ── T024: Tests for SQL comparison methods ──────────────────────────────

    #[test]
    fn sql_equals_true() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(1_000_000);
        assert!(a.sql_equals(&b).is_true());
    }

    #[test]
    fn sql_equals_false() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(2_000_000);
        assert!(a.sql_equals(&b).is_false());
    }

    #[test]
    fn sql_not_equals_true() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(2_000_000);
        assert!(a.sql_not_equals(&b).is_true());
    }

    #[test]
    fn sql_less_than_true() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(2_000_000);
        assert!(a.sql_less_than(&b).is_true());
    }

    #[test]
    fn sql_greater_than_true() {
        let a = SqlMoney::from_scaled(2_000_000);
        let b = SqlMoney::from_scaled(1_000_000);
        assert!(a.sql_greater_than(&b).is_true());
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(1_000_000);
        assert!(a.sql_less_than_or_equal(&b).is_true());
    }

    #[test]
    fn sql_less_than_or_equal_less() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(2_000_000);
        assert!(a.sql_less_than_or_equal(&b).is_true());
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let a = SqlMoney::from_scaled(1_000_000);
        let b = SqlMoney::from_scaled(1_000_000);
        assert!(a.sql_greater_than_or_equal(&b).is_true());
    }

    #[test]
    fn sql_greater_than_or_equal_greater() {
        let a = SqlMoney::from_scaled(2_000_000);
        let b = SqlMoney::from_scaled(1_000_000);
        assert!(a.sql_greater_than_or_equal(&b).is_true());
    }

    #[test]
    fn sql_comparison_null_left() {
        let a = SqlMoney::NULL;
        let b = SqlMoney::from_i32(100);
        assert!(a.sql_equals(&b).is_null());
        assert!(a.sql_not_equals(&b).is_null());
        assert!(a.sql_less_than(&b).is_null());
        assert!(a.sql_greater_than(&b).is_null());
        assert!(a.sql_less_than_or_equal(&b).is_null());
        assert!(a.sql_greater_than_or_equal(&b).is_null());
    }

    #[test]
    fn sql_comparison_null_right() {
        let a = SqlMoney::from_i32(100);
        let b = SqlMoney::NULL;
        assert!(a.sql_equals(&b).is_null());
        assert!(a.sql_not_equals(&b).is_null());
        assert!(a.sql_less_than(&b).is_null());
        assert!(a.sql_greater_than(&b).is_null());
        assert!(a.sql_less_than_or_equal(&b).is_null());
        assert!(a.sql_greater_than_or_equal(&b).is_null());
    }

    #[test]
    fn sql_comparison_both_null() {
        let a = SqlMoney::NULL;
        let b = SqlMoney::NULL;
        assert!(a.sql_equals(&b).is_null());
    }

    // ── T026: Tests for Display ─────────────────────────────────────────────

    #[test]
    fn display_all_4dp() {
        let m = SqlMoney::from_scaled(1_234_567); // 123.4567
        assert_eq!(format!("{m}"), "123.4567");
    }

    #[test]
    fn display_trim_to_2dp() {
        let m = SqlMoney::from_scaled(1_234_500); // 123.4500
        assert_eq!(format!("{m}"), "123.45");
    }

    #[test]
    fn display_min_2dp() {
        let m = SqlMoney::from_i32(100); // 100.0000
        assert_eq!(format!("{m}"), "100.00");
    }

    #[test]
    fn display_3dp() {
        let m = SqlMoney::from_scaled(1_234_560); // 123.4560
        assert_eq!(format!("{m}"), "123.456");
    }

    #[test]
    fn display_negative() {
        let m = SqlMoney::from_scaled(-501_000); // -50.1000
        assert_eq!(format!("{m}"), "-50.10");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlMoney::NULL), "Null");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", SqlMoney::ZERO), "0.00");
    }

    #[test]
    fn display_min_value() {
        // i64::MIN = -9223372036854775808, / 10000 = -922337203685477.5808
        let s = format!("{}", SqlMoney::MIN_VALUE);
        assert_eq!(s, "-922337203685477.5808");
    }

    #[test]
    fn display_max_value() {
        // i64::MAX = 9223372036854775807, / 10000 = 922337203685477.5807
        let s = format!("{}", SqlMoney::MAX_VALUE);
        assert_eq!(s, "922337203685477.5807");
    }

    // ── T027: Tests for FromStr ─────────────────────────────────────────────

    #[test]
    fn from_str_valid_decimal() {
        let m: SqlMoney = "123.4567".parse().unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_234_567);
    }

    #[test]
    fn from_str_negative() {
        let m: SqlMoney = "-50.10".parse().unwrap();
        assert_eq!(m.scaled_value().unwrap(), -501_000);
    }

    #[test]
    fn from_str_integer() {
        let m: SqlMoney = "100".parse().unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_000_000);
    }

    #[test]
    fn from_str_null() {
        let m: SqlMoney = "Null".parse().unwrap();
        assert!(m.is_null());
    }

    #[test]
    fn from_str_null_lowercase() {
        let m: SqlMoney = "null".parse().unwrap();
        assert!(m.is_null());
    }

    #[test]
    fn from_str_invalid() {
        let r = "abc".parse::<SqlMoney>();
        assert!(matches!(r, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn from_str_more_than_4dp_rounds() {
        // 123.45678 → rounds to 123.4568
        let m: SqlMoney = "123.45678".parse().unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_234_568);
    }

    #[test]
    fn from_str_range_overflow() {
        let r = "999999999999999999".parse::<SqlMoney>();
        assert!(matches!(r, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn from_str_with_whitespace() {
        let m: SqlMoney = "  42.50  ".parse().unwrap();
        assert_eq!(m.scaled_value().unwrap(), 425_000);
    }

    #[test]
    fn from_str_display_roundtrip() {
        let original = SqlMoney::from_scaled(1_234_567);
        let s = format!("{original}");
        let parsed: SqlMoney = s.parse().unwrap();
        assert_eq!(
            original.scaled_value().unwrap(),
            parsed.scaled_value().unwrap()
        );
    }

    // ── T030: Tests for widening From conversions ───────────────────────────

    #[test]
    fn from_sql_boolean_null() {
        let m = SqlMoney::from(SqlBoolean::NULL);
        assert!(m.is_null());
    }

    #[test]
    fn from_sql_boolean_false() {
        let m = SqlMoney::from(SqlBoolean::FALSE);
        assert_eq!(m.scaled_value().unwrap(), 0);
    }

    #[test]
    fn from_sql_boolean_true() {
        let m = SqlMoney::from(SqlBoolean::TRUE);
        assert_eq!(m.scaled_value().unwrap(), 10_000);
    }

    #[test]
    fn from_sql_byte_null() {
        let m = SqlMoney::from(SqlByte::NULL);
        assert!(m.is_null());
    }

    #[test]
    fn from_sql_byte_value() {
        let m = SqlMoney::from(SqlByte::new(255));
        assert_eq!(m.scaled_value().unwrap(), 2_550_000);
    }

    #[test]
    fn from_sql_int16_null() {
        let m = SqlMoney::from(SqlInt16::NULL);
        assert!(m.is_null());
    }

    #[test]
    fn from_sql_int16_value() {
        let m = SqlMoney::from(SqlInt16::new(1000));
        assert_eq!(m.scaled_value().unwrap(), 10_000_000);
    }

    #[test]
    fn from_sql_int32_null() {
        let m = SqlMoney::from(SqlInt32::NULL);
        assert!(m.is_null());
    }

    #[test]
    fn from_sql_int32_value() {
        let m = SqlMoney::from(SqlInt32::new(42));
        assert_eq!(m.scaled_value().unwrap(), 420_000);
    }

    #[test]
    fn from_sql_int32_max() {
        let m = SqlMoney::from(SqlInt32::new(i32::MAX));
        assert_eq!(m.scaled_value().unwrap(), i32::MAX as i64 * 10_000);
    }

    #[test]
    fn from_sql_int32_min() {
        let m = SqlMoney::from(SqlInt32::new(i32::MIN));
        assert_eq!(m.scaled_value().unwrap(), i32::MIN as i64 * 10_000);
    }

    // ── T031: Tests for from_sql_int64() ────────────────────────────────────

    #[test]
    fn from_sql_int64_in_range() {
        let m = SqlMoney::from_sql_int64(SqlInt64::new(100)).unwrap();
        assert_eq!(m.scaled_value().unwrap(), 1_000_000);
    }

    #[test]
    fn from_sql_int64_overflow() {
        let r = SqlMoney::from_sql_int64(SqlInt64::new(i64::MAX));
        assert!(matches!(r, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn from_sql_int64_null() {
        let m = SqlMoney::from_sql_int64(SqlInt64::NULL).unwrap();
        assert!(m.is_null());
    }

    // ── T032: Tests for to_sql_int64() ──────────────────────────────────────

    #[test]
    fn to_sql_int64_round() {
        let m = SqlMoney::from_scaled(429_999); // 42.9999
        let r = m.to_sql_int64().unwrap();
        assert_eq!(r.value().unwrap(), 43);
    }

    #[test]
    fn to_sql_int64_round_negative() {
        let m = SqlMoney::from_scaled(-425_000); // -42.5000
        let r = m.to_sql_int64().unwrap();
        assert_eq!(r.value().unwrap(), -43);
    }

    #[test]
    fn to_sql_int64_null() {
        assert!(matches!(
            SqlMoney::NULL.to_sql_int64(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T033: Tests for to_sql_int32() ──────────────────────────────────────

    #[test]
    fn to_sql_int32_in_range() {
        let m = SqlMoney::from_i32(42);
        let r = m.to_sql_int32().unwrap();
        assert_eq!(r.value().unwrap(), 42);
    }

    #[test]
    fn to_sql_int32_rounding() {
        let m = SqlMoney::from_scaled(425_000); // 42.5
        let r = m.to_sql_int32().unwrap();
        assert_eq!(r.value().unwrap(), 43);
    }

    #[test]
    fn to_sql_int32_overflow() {
        let m = SqlMoney::from_i64(i32::MAX as i64 + 1).unwrap();
        assert!(matches!(m.to_sql_int32(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int32_null() {
        assert!(matches!(
            SqlMoney::NULL.to_sql_int32(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T034: Tests for to_sql_int16(), to_sql_byte() ───────────────────────

    #[test]
    fn to_sql_int16_in_range() {
        let m = SqlMoney::from_i32(100);
        let r = m.to_sql_int16().unwrap();
        assert_eq!(r.value().unwrap(), 100);
    }

    #[test]
    fn to_sql_int16_overflow() {
        let m = SqlMoney::from_i32(100_000);
        assert!(matches!(m.to_sql_int16(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_null() {
        assert!(matches!(
            SqlMoney::NULL.to_sql_int16(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn to_sql_byte_in_range() {
        let m = SqlMoney::from_i32(200);
        let r = m.to_sql_byte().unwrap();
        assert_eq!(r.value().unwrap(), 200);
    }

    #[test]
    fn to_sql_byte_overflow() {
        let m = SqlMoney::from_i32(300);
        assert!(matches!(m.to_sql_byte(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_negative() {
        let m = SqlMoney::from_i32(-1);
        assert!(matches!(m.to_sql_byte(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_null() {
        assert!(matches!(
            SqlMoney::NULL.to_sql_byte(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T035: Tests for to_sql_boolean() ────────────────────────────────────

    #[test]
    fn to_sql_boolean_zero() {
        assert!(SqlMoney::ZERO.to_sql_boolean().is_false());
    }

    #[test]
    fn to_sql_boolean_nonzero() {
        assert!(SqlMoney::from_i32(1).to_sql_boolean().is_true());
    }

    #[test]
    fn to_sql_boolean_null() {
        assert!(SqlMoney::NULL.to_sql_boolean().is_null());
    }

    // ── T036: Tests for to_sql_decimal() ────────────────────────────────────

    #[test]
    fn to_sql_decimal_value() {
        let m = SqlMoney::from_scaled(425_000); // 42.5000
        let d = m.to_sql_decimal();
        assert!(!d.is_null());
        assert_eq!(d.scale().unwrap(), 4);
        // 42.5000 in scale=4 → mantissa = 425000
        let data = d.data().unwrap();
        assert_eq!(data[0], 425_000);
        assert_eq!(data[1], 0);
    }

    #[test]
    fn to_sql_decimal_null() {
        let d = SqlMoney::NULL.to_sql_decimal();
        assert!(d.is_null());
    }

    #[test]
    fn to_sql_decimal_negative() {
        let m = SqlMoney::from_scaled(-1_000_000); // -100.0000
        let d = m.to_sql_decimal();
        assert!(!d.is_null());
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.scale().unwrap(), 4);
    }

    // ── T042: Tests for PartialEq/Eq ────────────────────────────────────────

    #[test]
    fn eq_equal_values() {
        assert_eq!(SqlMoney::from_i32(100), SqlMoney::from_i32(100));
    }

    #[test]
    fn eq_not_equal() {
        assert_ne!(SqlMoney::from_i32(100), SqlMoney::from_i32(200));
    }

    #[test]
    fn eq_null_null() {
        assert_eq!(SqlMoney::NULL, SqlMoney::NULL);
    }

    #[test]
    fn eq_null_not_null() {
        assert_ne!(SqlMoney::NULL, SqlMoney::from_i32(100));
    }

    // ── T043: Tests for Hash ────────────────────────────────────────────────

    #[test]
    fn hash_equal_values() {
        use std::collections::hash_map::DefaultHasher;
        let hash = |m: SqlMoney| {
            let mut h = DefaultHasher::new();
            m.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(SqlMoney::from_i32(42)), hash(SqlMoney::from_i32(42)));
    }

    #[test]
    fn hash_null_consistent() {
        use std::collections::hash_map::DefaultHasher;
        let hash = |m: SqlMoney| {
            let mut h = DefaultHasher::new();
            m.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(SqlMoney::NULL), hash(SqlMoney::NULL));
    }

    // ── T044: Tests for PartialOrd/Ord ──────────────────────────────────────

    #[test]
    fn ord_null_less_than_value() {
        assert!(SqlMoney::NULL < SqlMoney::from_i32(0));
    }

    #[test]
    fn ord_null_less_than_min_value() {
        assert!(SqlMoney::NULL < SqlMoney::MIN_VALUE);
    }

    #[test]
    fn ord_negative_less_than_positive() {
        assert!(SqlMoney::from_i32(-100) < SqlMoney::from_i32(100));
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlMoney::MIN_VALUE < SqlMoney::MAX_VALUE);
    }

    #[test]
    fn ord_equal_values() {
        assert_eq!(
            SqlMoney::from_i32(42).cmp(&SqlMoney::from_i32(42)),
            Ordering::Equal
        );
    }

    // ── to_sql_string() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_string_positive() {
        let m = SqlMoney::from_i32(100);
        let s = m.to_sql_string();
        assert_eq!(s.value().unwrap(), "100.00");
    }

    #[test]
    fn to_sql_string_negative() {
        let m = SqlMoney::from_i32(-50);
        let s = m.to_sql_string();
        assert_eq!(s.value().unwrap(), "-50.00");
    }

    #[test]
    fn to_sql_string_zero() {
        let m = SqlMoney::from_i32(0);
        let s = m.to_sql_string();
        assert_eq!(s.value().unwrap(), "0.00");
    }

    #[test]
    fn to_sql_string_null() {
        let s = SqlMoney::NULL.to_sql_string();
        assert!(s.is_null());
    }

    // ── from_sql_single/double, to_sql_single/double tests ────────────────

    #[test]
    fn from_sql_single_normal() {
        let result = SqlMoney::from_sql_single(SqlSingle::new(100.5).unwrap()).unwrap();
        assert!(!result.is_null());
        let f = result.scaled_value().unwrap() as f64 / 10_000.0;
        assert!((f - 100.5).abs() < 0.001);
    }

    #[test]
    fn from_sql_single_null() {
        let result = SqlMoney::from_sql_single(SqlSingle::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn from_sql_double_normal() {
        let result = SqlMoney::from_sql_double(SqlDouble::new(100.5).unwrap()).unwrap();
        assert!(!result.is_null());
    }

    #[test]
    fn from_sql_double_null() {
        let result = SqlMoney::from_sql_double(SqlDouble::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_single_normal() {
        let m = SqlMoney::from_i32(100);
        let s = m.to_sql_single();
        assert!(!s.is_null());
        assert!((s.value().unwrap() - 100.0).abs() < 0.01);
    }

    #[test]
    fn to_sql_single_null() {
        assert!(SqlMoney::NULL.to_sql_single().is_null());
    }

    #[test]
    fn to_sql_double_normal() {
        let m = SqlMoney::from_i32(100);
        let d = m.to_sql_double();
        assert!(!d.is_null());
        assert!((d.value().unwrap() - 100.0).abs() < 0.001);
    }

    #[test]
    fn to_sql_double_null() {
        assert!(SqlMoney::NULL.to_sql_double().is_null());
    }
}
