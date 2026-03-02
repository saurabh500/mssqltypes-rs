// ── T001: SqlSingle module ──────────────────────────────────────────────────

//! `SqlSingle` — a nullable 32-bit IEEE 754 floating-point type with SQL NULL
//! support, equivalent to C# `System.Data.SqlTypes.SqlSingle` / SQL Server `REAL`.
//!
//! Uses `Option<f32>` internally: `None` = SQL NULL, `Some(v)` = a finite `f32` value.
//! NaN and Infinity are rejected on construction and after every arithmetic operation.
//! All arithmetic returns `Result<SqlSingle, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_double::SqlDouble;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use crate::sql_int64::SqlInt64;
use crate::sql_money::SqlMoney;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A nullable 32-bit IEEE 754 floating-point value with SQL NULL support,
/// equivalent to C# `System.Data.SqlTypes.SqlSingle` / SQL Server `REAL`.
///
/// Uses `Option<f32>` internally: `None` = SQL NULL, `Some(v)` = a finite
/// `f32` value. NaN and Infinity are never stored — all construction and
/// arithmetic paths reject non-finite results.
#[derive(Copy, Clone, Debug)]
pub struct SqlSingle {
    value: Option<f32>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors & Accessors ──────────────────────────────────────────
// ── T011: is_null ───────────────────────────────────────────────────────────
// ── T012: value ─────────────────────────────────────────────────────────────

impl SqlSingle {
    /// SQL NULL.
    pub const NULL: SqlSingle = SqlSingle { value: None };

    /// Zero (0.0).
    pub const ZERO: SqlSingle = SqlSingle { value: Some(0.0) };

    /// Minimum finite `f32` value.
    pub const MIN_VALUE: SqlSingle = SqlSingle {
        value: Some(f32::MIN),
    };

    /// Maximum finite `f32` value.
    pub const MAX_VALUE: SqlSingle = SqlSingle {
        value: Some(f32::MAX),
    };

    // ── Constructors ────────────────────────────────────────────────────────

    /// Creates a new `SqlSingle` from a finite `f32` value.
    ///
    /// # Errors
    /// * `Overflow` — if `value` is NaN, Infinity, or negative Infinity
    pub fn new(value: f32) -> Result<Self, SqlTypeError> {
        if value.is_finite() {
            Ok(SqlSingle { value: Some(value) })
        } else {
            Err(SqlTypeError::Overflow)
        }
    }

    // ── Accessors ───────────────────────────────────────────────────────────

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `f32` value.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn value(&self) -> Result<f32, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<f32> ─────────────────────────────────────────────────────────

impl From<f32> for SqlSingle {
    /// Converts an `f32` to `SqlSingle`.
    ///
    /// # Panics
    /// Panics if `value` is NaN, Infinity, or negative Infinity.
    fn from(value: f32) -> Self {
        SqlSingle::new(value).expect("SqlSingle::from(f32) called with non-finite value")
    }
}

// ── T017-T018: Checked Arithmetic ───────────────────────────────────────────

impl SqlSingle {
    /// Checked addition. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlSingle::NULL)`.
    pub fn checked_add(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlSingle::NULL),
            (Some(a), Some(b)) => {
                let result = a + b;
                if result.is_finite() {
                    Ok(SqlSingle {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlSingle::NULL)`.
    pub fn checked_sub(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlSingle::NULL),
            (Some(a), Some(b)) => {
                let result = a - b;
                if result.is_finite() {
                    Ok(SqlSingle {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlSingle::NULL)`.
    pub fn checked_mul(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlSingle::NULL),
            (Some(a), Some(b)) => {
                let result = a * b;
                if result.is_finite() {
                    Ok(SqlSingle {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }

    /// Checked division. Returns `Err(DivideByZero)` if divisor is ±0.0.
    /// Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlSingle::NULL)`.
    pub fn checked_div(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlSingle::NULL),
            (Some(_), Some(0.0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => {
                let result = a / b;
                if result.is_finite() {
                    Ok(SqlSingle {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }
}

// ── T019: Operator Traits ───────────────────────────────────────────────────

impl Add for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn add(self, rhs: SqlSingle) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Add<&SqlSingle> for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn add(self, rhs: &SqlSingle) -> Self::Output {
        self.checked_add(*rhs)
    }
}

impl Add<SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn add(self, rhs: SqlSingle) -> Self::Output {
        (*self).checked_add(rhs)
    }
}

impl Add<&SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn add(self, rhs: &SqlSingle) -> Self::Output {
        (*self).checked_add(*rhs)
    }
}

impl Sub for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn sub(self, rhs: SqlSingle) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<&SqlSingle> for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn sub(self, rhs: &SqlSingle) -> Self::Output {
        self.checked_sub(*rhs)
    }
}

impl Sub<SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn sub(self, rhs: SqlSingle) -> Self::Output {
        (*self).checked_sub(rhs)
    }
}

impl Sub<&SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn sub(self, rhs: &SqlSingle) -> Self::Output {
        (*self).checked_sub(*rhs)
    }
}

impl Mul for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn mul(self, rhs: SqlSingle) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<&SqlSingle> for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn mul(self, rhs: &SqlSingle) -> Self::Output {
        self.checked_mul(*rhs)
    }
}

impl Mul<SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn mul(self, rhs: SqlSingle) -> Self::Output {
        (*self).checked_mul(rhs)
    }
}

impl Mul<&SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn mul(self, rhs: &SqlSingle) -> Self::Output {
        (*self).checked_mul(*rhs)
    }
}

impl Div for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn div(self, rhs: SqlSingle) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<&SqlSingle> for SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn div(self, rhs: &SqlSingle) -> Self::Output {
        self.checked_div(*rhs)
    }
}

impl Div<SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn div(self, rhs: SqlSingle) -> Self::Output {
        (*self).checked_div(rhs)
    }
}

impl Div<&SqlSingle> for &SqlSingle {
    type Output = Result<SqlSingle, SqlTypeError>;
    fn div(self, rhs: &SqlSingle) -> Self::Output {
        (*self).checked_div(*rhs)
    }
}

// ── T022: Negation (infallible) ─────────────────────────────────────────────

impl Neg for SqlSingle {
    type Output = SqlSingle;
    fn neg(self) -> Self::Output {
        SqlSingle {
            value: self.value.map(|v| -v),
        }
    }
}

impl Neg for &SqlSingle {
    type Output = SqlSingle;
    fn neg(self) -> Self::Output {
        SqlSingle {
            value: self.value.map(|v| -v),
        }
    }
}

// ── T024: SQL Comparisons ───────────────────────────────────────────────────

impl SqlSingle {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlSingle) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T027-T028: Display / FromStr ────────────────────────────────────────────

impl fmt::Display for SqlSingle {
    /// Displays the value using default `f32` formatting. NULL displays as `"Null"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            None => write!(f, "Null"),
            Some(v) => write!(f, "{v}"),
        }
    }
}

impl FromStr for SqlSingle {
    type Err = SqlTypeError;

    /// Parses a string into `SqlSingle`.
    /// `"Null"` (case-insensitive) → `SqlSingle::NULL`.
    /// NaN and Infinity strings are rejected.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlSingle::NULL);
        }

        let parsed: f32 = trimmed
            .parse()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid SqlSingle string: {s}")))?;

        if parsed.is_finite() {
            Ok(SqlSingle {
                value: Some(parsed),
            })
        } else {
            Err(SqlTypeError::Overflow)
        }
    }
}

// ── T035-T041: Conversions ──────────────────────────────────────────────────

impl SqlSingle {
    /// Converts from `SqlByte` (widening: `u8` → `f32`, always exact).
    /// NULL → NULL.
    pub fn from_sql_byte(v: SqlByte) -> SqlSingle {
        match v.value() {
            Ok(val) => SqlSingle {
                value: Some(val as f32),
            },
            Err(_) => SqlSingle::NULL,
        }
    }

    /// Converts from `SqlInt16` (widening: `i16` → `f32`, always exact).
    /// NULL → NULL.
    pub fn from_sql_int16(v: SqlInt16) -> SqlSingle {
        match v.value() {
            Ok(val) => SqlSingle {
                value: Some(val as f32),
            },
            Err(_) => SqlSingle::NULL,
        }
    }

    /// Converts from `SqlInt32` (widening: `i32` → `f32`, may lose precision for large values).
    /// NULL → NULL.
    pub fn from_sql_int32(v: SqlInt32) -> SqlSingle {
        match v.value() {
            Ok(val) => SqlSingle {
                value: Some(val as f32),
            },
            Err(_) => SqlSingle::NULL,
        }
    }

    /// Converts from `SqlInt64` (widening: `i64` → `f32`, may lose precision for large values).
    /// NULL → NULL.
    pub fn from_sql_int64(v: SqlInt64) -> SqlSingle {
        match v.value() {
            Ok(val) => SqlSingle {
                value: Some(val as f32),
            },
            Err(_) => SqlSingle::NULL,
        }
    }

    /// Converts from `SqlMoney` (widening: extract scaled `i64`, divide by 10,000.0 via f64 intermediate).
    /// NULL → NULL.
    pub fn from_sql_money(v: SqlMoney) -> SqlSingle {
        match v.scaled_value() {
            Ok(scaled) => SqlSingle {
                value: Some((scaled as f64 / 10_000.0) as f32),
            },
            Err(_) => SqlSingle::NULL,
        }
    }

    /// Converts from `SqlBoolean`. TRUE = 1.0, FALSE = 0.0, NULL = NULL.
    pub fn from_sql_boolean(v: SqlBoolean) -> SqlSingle {
        if v.is_null() {
            SqlSingle::NULL
        } else if v.is_true() {
            SqlSingle { value: Some(1.0) }
        } else {
            SqlSingle::ZERO
        }
    }

    /// Converts to `SqlDouble` (widening: `f32` → `f64`, lossless, always finite).
    /// NULL → NULL.
    pub fn to_sql_double(&self) -> SqlDouble {
        match self.value {
            None => SqlDouble::NULL,
            Some(v) => SqlDouble::new(v as f64).unwrap(),
        }
    }

    /// Converts from `SqlDouble` (narrowing: `f64` → `f32`, may overflow).
    /// Returns `Err(Overflow)` if the f64 value exceeds f32 finite range.
    /// NULL → NULL.
    pub fn from_sql_double(v: SqlDouble) -> Result<SqlSingle, SqlTypeError> {
        match v.value() {
            Ok(val) => {
                let narrowed = val as f32;
                if narrowed.is_finite() {
                    Ok(SqlSingle {
                        value: Some(narrowed),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
            Err(SqlTypeError::NullValue) => Ok(SqlSingle::NULL),
            Err(e) => Err(e),
        }
    }

    /// Converts to `SqlBoolean`. Zero → FALSE, non-zero → TRUE, NULL → NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0.0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }
}

// ── T045-T047: Rust Standard Traits ─────────────────────────────────────────

impl PartialEq for SqlSingle {
    fn eq(&self, other: &Self) -> bool {
        match (self.value, other.value) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for SqlSingle {}

impl Hash for SqlSingle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.value {
            None => 0u32.hash(state),
            Some(v) => {
                // Normalize -0.0 to 0.0 so that equal values hash identically
                let normalized = if v == 0.0 { 0.0_f32 } else { v };
                normalized.to_bits().hash(state);
            }
        }
    }
}

impl PartialOrd for SqlSingle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlSingle {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.value, other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of(val: SqlSingle) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 3: US1 — Create and Inspect Values
    // ═══════════════════════════════════════════════════════════════════════

    // ── T007: Tests for new() with valid values ─────────────────────────

    #[test]
    fn new_positive_value() {
        let s = SqlSingle::new(3.14).unwrap();
        assert_eq!(s.value().unwrap(), 3.14);
        assert!(!s.is_null());
    }

    #[test]
    fn new_negative_value() {
        let s = SqlSingle::new(-2.5).unwrap();
        assert_eq!(s.value().unwrap(), -2.5);
    }

    #[test]
    fn new_zero() {
        let s = SqlSingle::new(0.0).unwrap();
        assert_eq!(s.value().unwrap(), 0.0);
    }

    #[test]
    fn new_negative_zero() {
        let s = SqlSingle::new(-0.0).unwrap();
        assert_eq!(s.value().unwrap(), 0.0); // IEEE 754: -0.0 == 0.0
    }

    #[test]
    fn new_subnormal() {
        let subnormal = f32::MIN_POSITIVE * 0.5;
        let s = SqlSingle::new(subnormal).unwrap();
        assert_eq!(s.value().unwrap(), subnormal);
    }

    // ── T008: Tests for NaN/Infinity rejection ──────────────────────────

    #[test]
    fn new_nan_rejected() {
        let result = SqlSingle::new(f32::NAN);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn new_infinity_rejected() {
        let result = SqlSingle::new(f32::INFINITY);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn new_neg_infinity_rejected() {
        let result = SqlSingle::new(f32::NEG_INFINITY);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn from_f32_nan_panics() {
        let _ = SqlSingle::from(f32::NAN);
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn from_f32_infinity_panics() {
        let _ = SqlSingle::from(f32::INFINITY);
    }

    // ── T009: Tests for constants ───────────────────────────────────────

    #[test]
    fn constant_null() {
        assert!(SqlSingle::NULL.is_null());
    }

    #[test]
    fn constant_zero() {
        assert_eq!(SqlSingle::ZERO.value().unwrap(), 0.0);
    }

    #[test]
    fn constant_min_value() {
        assert_eq!(SqlSingle::MIN_VALUE.value().unwrap(), f32::MIN);
    }

    #[test]
    fn constant_max_value() {
        assert_eq!(SqlSingle::MAX_VALUE.value().unwrap(), f32::MAX);
    }

    // ── T010: Tests for value() and is_null() ───────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlSingle::NULL.is_null());
    }

    #[test]
    fn null_value_returns_err() {
        let result = SqlSingle::NULL.value();
        assert!(matches!(result, Err(SqlTypeError::NullValue)));
    }

    #[test]
    fn non_null_is_not_null() {
        let s = SqlSingle::new(42.0).unwrap();
        assert!(!s.is_null());
    }

    #[test]
    fn value_returns_stored_value() {
        let s = SqlSingle::new(99.99).unwrap();
        assert_eq!(s.value().unwrap(), 99.99);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 4: US2 — Arithmetic
    // ═══════════════════════════════════════════════════════════════════════

    // ── T013: Tests for checked_add ─────────────────────────────────────

    #[test]
    fn add_normal() {
        let a = SqlSingle::new(2.5).unwrap();
        let b = SqlSingle::new(3.5).unwrap();
        let result = a.checked_add(b).unwrap();
        assert_eq!(result.value().unwrap(), 6.0);
    }

    #[test]
    fn add_overflow() {
        let a = SqlSingle::new(f32::MAX).unwrap();
        let b = SqlSingle::new(f32::MAX).unwrap();
        let result = a.checked_add(b);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_null_left() {
        let result = SqlSingle::NULL
            .checked_add(SqlSingle::new(1.0).unwrap())
            .unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn add_null_right() {
        let result = SqlSingle::new(1.0)
            .unwrap()
            .checked_add(SqlSingle::NULL)
            .unwrap();
        assert!(result.is_null());
    }

    // ── T014: Tests for checked_sub ─────────────────────────────────────

    #[test]
    fn sub_normal() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(3.0).unwrap();
        let result = a.checked_sub(b).unwrap();
        assert_eq!(result.value().unwrap(), 7.0);
    }

    #[test]
    fn sub_overflow() {
        let a = SqlSingle::new(-f32::MAX).unwrap();
        let b = SqlSingle::new(f32::MAX).unwrap();
        let result = a.checked_sub(b);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn sub_null_propagation() {
        let result = SqlSingle::NULL
            .checked_sub(SqlSingle::new(1.0).unwrap())
            .unwrap();
        assert!(result.is_null());
    }

    // ── T015: Tests for checked_mul ─────────────────────────────────────

    #[test]
    fn mul_normal() {
        let a = SqlSingle::new(4.0).unwrap();
        let b = SqlSingle::new(2.5).unwrap();
        let result = a.checked_mul(b).unwrap();
        assert_eq!(result.value().unwrap(), 10.0);
    }

    #[test]
    fn mul_overflow() {
        let a = SqlSingle::new(f32::MAX).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = a.checked_mul(b);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn mul_null_propagation() {
        let result = SqlSingle::new(5.0)
            .unwrap()
            .checked_mul(SqlSingle::NULL)
            .unwrap();
        assert!(result.is_null());
    }

    // ── T016: Tests for checked_div ─────────────────────────────────────

    #[test]
    fn div_normal() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(4.0).unwrap();
        let result = a.checked_div(b).unwrap();
        assert_eq!(result.value().unwrap(), 2.5);
    }

    #[test]
    fn div_by_zero() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_zero_by_zero() {
        let a = SqlSingle::new(0.0).unwrap();
        let b = SqlSingle::new(0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_null_propagation() {
        let result = SqlSingle::NULL
            .checked_div(SqlSingle::new(1.0).unwrap())
            .unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn div_by_negative_zero() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(-0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    // ── T020: Tests for operator traits ──────────────────────────────────

    #[test]
    fn add_operator_owned_owned() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (a + b).unwrap();
        assert_eq!(result.value().unwrap(), 3.0);
    }

    #[test]
    fn add_operator_owned_ref() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (a + &b).unwrap();
        assert_eq!(result.value().unwrap(), 3.0);
    }

    #[test]
    fn add_operator_ref_owned() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (&a + b).unwrap();
        assert_eq!(result.value().unwrap(), 3.0);
    }

    #[test]
    fn add_operator_ref_ref() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (&a + &b).unwrap();
        assert_eq!(result.value().unwrap(), 3.0);
    }

    #[test]
    fn sub_operator() {
        let a = SqlSingle::new(5.0).unwrap();
        let b = SqlSingle::new(3.0).unwrap();
        let result = (a - b).unwrap();
        assert_eq!(result.value().unwrap(), 2.0);
    }

    #[test]
    fn mul_operator() {
        let a = SqlSingle::new(3.0).unwrap();
        let b = SqlSingle::new(4.0).unwrap();
        let result = (a * b).unwrap();
        assert_eq!(result.value().unwrap(), 12.0);
    }

    #[test]
    fn div_operator() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (a / b).unwrap();
        assert_eq!(result.value().unwrap(), 5.0);
    }

    #[test]
    fn sub_operator_ref_ref() {
        let a = SqlSingle::new(5.0).unwrap();
        let b = SqlSingle::new(3.0).unwrap();
        let result = (&a - &b).unwrap();
        assert_eq!(result.value().unwrap(), 2.0);
    }

    #[test]
    fn mul_operator_ref_ref() {
        let a = SqlSingle::new(3.0).unwrap();
        let b = SqlSingle::new(4.0).unwrap();
        let result = (&a * &b).unwrap();
        assert_eq!(result.value().unwrap(), 12.0);
    }

    #[test]
    fn div_operator_ref_ref() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        let result = (&a / &b).unwrap();
        assert_eq!(result.value().unwrap(), 5.0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 5: US3 — Negation
    // ═══════════════════════════════════════════════════════════════════════

    // ── T021: Tests for negation ────────────────────────────────────────

    #[test]
    fn neg_positive() {
        let s = SqlSingle::new(5.0).unwrap();
        let result = -s;
        assert_eq!(result.value().unwrap(), -5.0);
    }

    #[test]
    fn neg_negative() {
        let s = SqlSingle::new(-3.14).unwrap();
        let result = -s;
        assert_eq!(result.value().unwrap(), 3.14);
    }

    #[test]
    fn neg_zero() {
        let s = SqlSingle::new(0.0).unwrap();
        let result = -s;
        // IEEE 754: -0.0 is valid
        assert!(result.value().unwrap().is_sign_negative());
    }

    #[test]
    fn neg_null() {
        let result = -SqlSingle::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn neg_ref() {
        let s = SqlSingle::new(5.0).unwrap();
        let result = -&s;
        assert_eq!(result.value().unwrap(), -5.0);
    }

    #[test]
    fn neg_max() {
        let s = SqlSingle::MAX_VALUE;
        let result = -s;
        assert_eq!(result.value().unwrap(), -f32::MAX);
    }

    #[test]
    fn neg_min() {
        let s = SqlSingle::MIN_VALUE;
        let result = -s;
        assert_eq!(result.value().unwrap(), f32::MAX);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 6: US4 — SQL Comparisons
    // ═══════════════════════════════════════════════════════════════════════

    // ── T023: Tests for SQL comparisons ─────────────────────────────────

    #[test]
    fn sql_equals_equal_values() {
        let a = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_equals(&a), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_unequal_values() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_equals_null() {
        let a = SqlSingle::new(1.0).unwrap();
        assert!(a.sql_equals(&SqlSingle::NULL).is_null());
    }

    #[test]
    fn sql_not_equals_true() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_not_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_not_equals_false() {
        let a = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_not_equals(&a), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_true() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_false() {
        let a = SqlSingle::new(2.0).unwrap();
        let b = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_null() {
        let a = SqlSingle::new(1.0).unwrap();
        assert!(a.sql_less_than(&SqlSingle::NULL).is_null());
    }

    #[test]
    fn sql_greater_than_true() {
        let a = SqlSingle::new(2.0).unwrap();
        let b = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_false() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let a = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_less_than_or_equal(&a), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_less() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let a = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_greater_than_or_equal(&a), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_greater() {
        let a = SqlSingle::new(2.0).unwrap();
        let b = SqlSingle::new(1.0).unwrap();
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_comparison_null_null() {
        assert!(SqlSingle::NULL.sql_equals(&SqlSingle::NULL).is_null());
        assert!(SqlSingle::NULL.sql_less_than(&SqlSingle::NULL).is_null());
        assert!(SqlSingle::NULL.sql_greater_than(&SqlSingle::NULL).is_null());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 7: US5 — Display and Parsing
    // ═══════════════════════════════════════════════════════════════════════

    // ── T025: Tests for Display ─────────────────────────────────────────

    #[test]
    fn display_value() {
        let s = SqlSingle::new(3.14).unwrap();
        assert_eq!(format!("{s}"), "3.14");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlSingle::NULL), "Null");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", SqlSingle::ZERO), "0");
    }

    #[test]
    fn display_integer_value() {
        let s = SqlSingle::new(42.0).unwrap();
        assert_eq!(format!("{s}"), "42");
    }

    // ── T026: Tests for FromStr ──────────────────────────────────────────

    #[test]
    fn parse_valid() {
        let s: SqlSingle = "3.14".parse().unwrap();
        assert_eq!(s.value().unwrap(), 3.14);
    }

    #[test]
    fn parse_null() {
        let s: SqlSingle = "Null".parse().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn parse_null_case_insensitive() {
        let s: SqlSingle = "NULL".parse().unwrap();
        assert!(s.is_null());
        let s2: SqlSingle = "null".parse().unwrap();
        assert!(s2.is_null());
    }

    #[test]
    fn parse_invalid() {
        let result = "abc".parse::<SqlSingle>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_nan_rejected() {
        let result = "NaN".parse::<SqlSingle>();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn parse_infinity_rejected() {
        let result = "inf".parse::<SqlSingle>();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn parse_neg_infinity_rejected() {
        let result = "-inf".parse::<SqlSingle>();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn parse_overflow_value() {
        // f32::MAX is ~3.4028235e38; anything bigger overflows to Infinity
        let result = "3.5e38".parse::<SqlSingle>();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn parse_whitespace_trimmed() {
        let s: SqlSingle = "  42.0  ".parse().unwrap();
        assert_eq!(s.value().unwrap(), 42.0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 8: US6 — Conversions
    // ═══════════════════════════════════════════════════════════════════════

    // ── T029: Tests for integer conversions ──────────────────────────────

    #[test]
    fn from_sql_byte() {
        let b = SqlByte::new(42);
        let s = SqlSingle::from_sql_byte(b);
        assert_eq!(s.value().unwrap(), 42.0);
    }

    #[test]
    fn from_sql_byte_null() {
        let s = SqlSingle::from_sql_byte(SqlByte::NULL);
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_int16() {
        let i = SqlInt16::new(1000);
        let s = SqlSingle::from_sql_int16(i);
        assert_eq!(s.value().unwrap(), 1000.0);
    }

    #[test]
    fn from_sql_int16_null() {
        let s = SqlSingle::from_sql_int16(SqlInt16::NULL);
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_int32() {
        let i = SqlInt32::new(100_000);
        let s = SqlSingle::from_sql_int32(i);
        assert_eq!(s.value().unwrap(), 100_000.0);
    }

    #[test]
    fn from_sql_int32_null() {
        let s = SqlSingle::from_sql_int32(SqlInt32::NULL);
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_int32_large_loses_precision() {
        // i32::MAX = 2147483647, but f32 can't represent it exactly
        let i = SqlInt32::new(i32::MAX);
        let s = SqlSingle::from_sql_int32(i);
        // The result is a valid f32 (not NaN/Infinity) but may differ
        assert!(!s.is_null());
        assert!(s.value().unwrap().is_finite());
    }

    #[test]
    fn from_sql_int64() {
        let i = SqlInt64::new(1_000_000);
        let s = SqlSingle::from_sql_int64(i);
        assert_eq!(s.value().unwrap(), 1_000_000.0);
    }

    #[test]
    fn from_sql_int64_null() {
        let s = SqlSingle::from_sql_int64(SqlInt64::NULL);
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_int64_large_loses_precision() {
        let i = SqlInt64::new(i64::MAX);
        let s = SqlSingle::from_sql_int64(i);
        assert!(!s.is_null());
        assert!(s.value().unwrap().is_finite());
    }

    // ── T030: Tests for from_sql_boolean ─────────────────────────────────

    #[test]
    fn from_sql_boolean_true() {
        let s = SqlSingle::from_sql_boolean(SqlBoolean::TRUE);
        assert_eq!(s.value().unwrap(), 1.0);
    }

    #[test]
    fn from_sql_boolean_false() {
        let s = SqlSingle::from_sql_boolean(SqlBoolean::FALSE);
        assert_eq!(s.value().unwrap(), 0.0);
    }

    #[test]
    fn from_sql_boolean_null() {
        let s = SqlSingle::from_sql_boolean(SqlBoolean::NULL);
        assert!(s.is_null());
    }

    // ── T031: Tests for from_sql_money ───────────────────────────────────

    #[test]
    fn from_sql_money_value() {
        let m = SqlMoney::from_scaled(425_000); // 42.50 (scaled by 10_000)
        let s = SqlSingle::from_sql_money(m);
        assert_eq!(s.value().unwrap(), 42.5);
    }

    #[test]
    fn from_sql_money_null() {
        let s = SqlSingle::from_sql_money(SqlMoney::NULL);
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_money_zero() {
        let m = SqlMoney::from_i32(0);
        let s = SqlSingle::from_sql_money(m);
        assert_eq!(s.value().unwrap(), 0.0);
    }

    // ── T032: Tests for to_sql_double ────────────────────────────────────

    #[test]
    fn to_sql_double_value() {
        let s = SqlSingle::new(3.14).unwrap();
        let d = s.to_sql_double();
        assert!(!d.is_null());
        // f32→f64 widening is lossless
        assert_eq!(d.value().unwrap(), 3.14_f32 as f64);
    }

    #[test]
    fn to_sql_double_null() {
        let d = SqlSingle::NULL.to_sql_double();
        assert!(d.is_null());
    }

    #[test]
    fn to_sql_double_max() {
        let s = SqlSingle::MAX_VALUE;
        let d = s.to_sql_double();
        assert!(!d.is_null());
        assert!(d.value().unwrap().is_finite());
    }

    // ── T033: Tests for from_sql_double ──────────────────────────────────

    #[test]
    fn from_sql_double_normal() {
        let d = SqlDouble::new(3.14).unwrap();
        let s = SqlSingle::from_sql_double(d).unwrap();
        assert_eq!(s.value().unwrap(), 3.14_f64 as f32);
    }

    #[test]
    fn from_sql_double_null() {
        let s = SqlSingle::from_sql_double(SqlDouble::NULL).unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn from_sql_double_overflow() {
        // f64::MAX exceeds f32 range → becomes Infinity when narrowed
        let d = SqlDouble::new(f64::MAX).unwrap();
        let result = SqlSingle::from_sql_double(d);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn from_sql_double_within_range() {
        let d = SqlDouble::new(42.0).unwrap();
        let s = SqlSingle::from_sql_double(d).unwrap();
        assert_eq!(s.value().unwrap(), 42.0);
    }

    // ── T034: Tests for to_sql_boolean ───────────────────────────────────

    #[test]
    fn to_sql_boolean_non_zero() {
        let s = SqlSingle::new(42.0).unwrap();
        assert_eq!(s.to_sql_boolean(), SqlBoolean::TRUE);
    }

    #[test]
    fn to_sql_boolean_zero() {
        let s = SqlSingle::ZERO;
        assert_eq!(s.to_sql_boolean(), SqlBoolean::FALSE);
    }

    #[test]
    fn to_sql_boolean_null() {
        assert_eq!(SqlSingle::NULL.to_sql_boolean(), SqlBoolean::NULL);
    }

    #[test]
    fn to_sql_boolean_negative() {
        let s = SqlSingle::new(-1.0).unwrap();
        assert_eq!(s.to_sql_boolean(), SqlBoolean::TRUE);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Phase 9: Polish — Eq/Hash/Ord
    // ═══════════════════════════════════════════════════════════════════════

    // ── T042: Tests for PartialEq/Eq ────────────────────────────────────

    #[test]
    fn eq_equal_values() {
        let a = SqlSingle::new(3.14).unwrap();
        let b = SqlSingle::new(3.14).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn eq_unequal_values() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn eq_null_null() {
        assert_eq!(SqlSingle::NULL, SqlSingle::NULL);
    }

    #[test]
    fn eq_null_value() {
        assert_ne!(SqlSingle::NULL, SqlSingle::ZERO);
    }

    #[test]
    fn eq_neg_zero_eq_zero() {
        let a = SqlSingle::new(0.0).unwrap();
        let b = SqlSingle::new(-0.0).unwrap();
        assert_eq!(a, b); // IEEE 754: 0.0 == -0.0
    }

    // ── T043: Tests for Hash ────────────────────────────────────────────

    #[test]
    fn hash_equal_values_hash_equal() {
        let a = SqlSingle::new(3.14).unwrap();
        let b = SqlSingle::new(3.14).unwrap();
        assert_eq!(hash_of(a), hash_of(b));
    }

    #[test]
    fn hash_neg_zero_same_as_zero() {
        let a = SqlSingle::new(0.0).unwrap();
        let b = SqlSingle::new(-0.0).unwrap();
        assert_eq!(hash_of(a), hash_of(b));
    }

    #[test]
    fn hash_null_is_consistent() {
        assert_eq!(hash_of(SqlSingle::NULL), hash_of(SqlSingle::NULL));
    }

    #[test]
    fn hash_different_values_differ() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        // Not guaranteed, but very likely for these values
        assert_ne!(hash_of(a), hash_of(b));
    }

    // ── T044: Tests for PartialOrd/Ord ──────────────────────────────────

    #[test]
    fn ord_null_less_than_value() {
        assert!(SqlSingle::NULL < SqlSingle::ZERO);
    }

    #[test]
    fn ord_null_equal_null() {
        assert_eq!(SqlSingle::NULL.cmp(&SqlSingle::NULL), Ordering::Equal);
    }

    #[test]
    fn ord_value_greater_than_null() {
        assert!(SqlSingle::ZERO > SqlSingle::NULL);
    }

    #[test]
    fn ord_values_ordered() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn ord_equal_values() {
        let a = SqlSingle::new(3.14).unwrap();
        let b = SqlSingle::new(3.14).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ord_negative_less_than_positive() {
        let neg = SqlSingle::new(-1.0).unwrap();
        let pos = SqlSingle::new(1.0).unwrap();
        assert!(neg < pos);
    }

    // ── T050: Quickstart validation tests ───────────────────────────────

    #[test]
    fn quickstart_create_and_inspect() {
        let x = SqlSingle::new(3.14).unwrap();
        assert_eq!(x.value().unwrap(), 3.14);
        assert!(!x.is_null());

        assert!(SqlSingle::NULL.is_null());
        assert!(SqlSingle::NULL.value().is_err());
        assert!(SqlSingle::new(f32::NAN).is_err());
    }

    #[test]
    fn quickstart_arithmetic() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(3.0).unwrap();
        let sum = (a + b).unwrap();
        assert_eq!(sum.value().unwrap(), 13.0);

        let result = (a + SqlSingle::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn quickstart_sql_comparisons() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
        assert!(a.sql_equals(&SqlSingle::NULL).is_null());
    }

    #[test]
    fn quickstart_display_and_parse() {
        let x = SqlSingle::new(3.14).unwrap();
        assert_eq!(format!("{x}"), "3.14");
        assert_eq!(format!("{}", SqlSingle::NULL), "Null");

        let parsed: SqlSingle = "3.14".parse().unwrap();
        assert_eq!(parsed.value().unwrap(), 3.14);
    }

    #[test]
    fn quickstart_to_sql_double_widening() {
        let x = SqlSingle::new(3.14).unwrap();
        let d = x.to_sql_double();
        assert!(!d.is_null());

        assert!(SqlSingle::NULL.to_sql_double().is_null());
    }
}
