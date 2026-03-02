// ── T001: SqlDouble module ──────────────────────────────────────────────────

//! `SqlDouble` — a nullable 64-bit IEEE 754 floating-point type with SQL NULL
//! support, equivalent to C# `System.Data.SqlTypes.SqlDouble` / SQL Server `FLOAT`.
//!
//! Uses `Option<f64>` internally: `None` = SQL NULL, `Some(v)` = a finite `f64` value.
//! NaN and Infinity are rejected on construction and after every arithmetic operation.
//! All arithmetic returns `Result<SqlDouble, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use crate::sql_int64::SqlInt64;
use crate::sql_money::SqlMoney;
use crate::sql_single::SqlSingle;
use crate::sql_string::SqlString;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A nullable 64-bit IEEE 754 floating-point value with SQL NULL support,
/// equivalent to C# `System.Data.SqlTypes.SqlDouble` / SQL Server `FLOAT`.
///
/// Uses `Option<f64>` internally: `None` = SQL NULL, `Some(v)` = a finite
/// `f64` value. NaN and Infinity are never stored — all construction and
/// arithmetic paths reject non-finite results.
#[derive(Copy, Clone, Debug)]
pub struct SqlDouble {
    value: Option<f64>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors & Accessors ──────────────────────────────────────────

impl SqlDouble {
    /// SQL NULL.
    pub const NULL: SqlDouble = SqlDouble { value: None };

    /// Zero (0.0).
    pub const ZERO: SqlDouble = SqlDouble { value: Some(0.0) };

    /// Minimum finite `f64` value.
    pub const MIN_VALUE: SqlDouble = SqlDouble {
        value: Some(f64::MIN),
    };

    /// Maximum finite `f64` value.
    pub const MAX_VALUE: SqlDouble = SqlDouble {
        value: Some(f64::MAX),
    };

    // ── Constructors ────────────────────────────────────────────────────────

    /// Creates a new `SqlDouble` from a finite `f64` value.
    ///
    /// # Errors
    /// * `Overflow` — if `value` is NaN, Infinity, or negative Infinity
    pub fn new(value: f64) -> Result<Self, SqlTypeError> {
        if value.is_finite() {
            Ok(SqlDouble { value: Some(value) })
        } else {
            Err(SqlTypeError::Overflow)
        }
    }

    // ── Accessors ───────────────────────────────────────────────────────────

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `f64` value.
    ///
    /// # Errors
    /// * `NullValue` — if this value is SQL NULL
    pub fn value(&self) -> Result<f64, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<f64> ─────────────────────────────────────────────────────────

impl From<f64> for SqlDouble {
    /// Converts an `f64` to `SqlDouble`.
    ///
    /// # Panics
    /// Panics if `value` is NaN, Infinity, or negative Infinity.
    fn from(value: f64) -> Self {
        SqlDouble::new(value).expect("SqlDouble::from(f64) called with non-finite value")
    }
}

// ── Checked Arithmetic ──────────────────────────────────────────────────────

impl SqlDouble {
    /// Checked addition. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlDouble::NULL)`.
    pub fn checked_add(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlDouble::NULL),
            (Some(a), Some(b)) => {
                let result = a + b;
                if result.is_finite() {
                    Ok(SqlDouble {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlDouble::NULL)`.
    pub fn checked_sub(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlDouble::NULL),
            (Some(a), Some(b)) => {
                let result = a - b;
                if result.is_finite() {
                    Ok(SqlDouble {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if the result is not finite.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlDouble::NULL)`.
    pub fn checked_mul(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlDouble::NULL),
            (Some(a), Some(b)) => {
                let result = a * b;
                if result.is_finite() {
                    Ok(SqlDouble {
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
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlDouble::NULL)`.
    pub fn checked_div(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlDouble::NULL),
            (Some(_), Some(0.0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => {
                let result = a / b;
                if result.is_finite() {
                    Ok(SqlDouble {
                        value: Some(result),
                    })
                } else {
                    Err(SqlTypeError::Overflow)
                }
            }
        }
    }
}

// ── Operator Traits ─────────────────────────────────────────────────────────

impl Add for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn add(self, rhs: SqlDouble) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Add<&SqlDouble> for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn add(self, rhs: &SqlDouble) -> Self::Output {
        self.checked_add(*rhs)
    }
}

impl Add<SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn add(self, rhs: SqlDouble) -> Self::Output {
        (*self).checked_add(rhs)
    }
}

impl Add<&SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn add(self, rhs: &SqlDouble) -> Self::Output {
        (*self).checked_add(*rhs)
    }
}

impl Sub for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn sub(self, rhs: SqlDouble) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Sub<&SqlDouble> for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn sub(self, rhs: &SqlDouble) -> Self::Output {
        self.checked_sub(*rhs)
    }
}

impl Sub<SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn sub(self, rhs: SqlDouble) -> Self::Output {
        (*self).checked_sub(rhs)
    }
}

impl Sub<&SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn sub(self, rhs: &SqlDouble) -> Self::Output {
        (*self).checked_sub(*rhs)
    }
}

impl Mul for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn mul(self, rhs: SqlDouble) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Mul<&SqlDouble> for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn mul(self, rhs: &SqlDouble) -> Self::Output {
        self.checked_mul(*rhs)
    }
}

impl Mul<SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn mul(self, rhs: SqlDouble) -> Self::Output {
        (*self).checked_mul(rhs)
    }
}

impl Mul<&SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn mul(self, rhs: &SqlDouble) -> Self::Output {
        (*self).checked_mul(*rhs)
    }
}

impl Div for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn div(self, rhs: SqlDouble) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Div<&SqlDouble> for SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn div(self, rhs: &SqlDouble) -> Self::Output {
        self.checked_div(*rhs)
    }
}

impl Div<SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn div(self, rhs: SqlDouble) -> Self::Output {
        (*self).checked_div(rhs)
    }
}

impl Div<&SqlDouble> for &SqlDouble {
    type Output = Result<SqlDouble, SqlTypeError>;
    fn div(self, rhs: &SqlDouble) -> Self::Output {
        (*self).checked_div(*rhs)
    }
}

// ── Negation (infallible) ───────────────────────────────────────────────────

impl Neg for SqlDouble {
    type Output = SqlDouble;
    fn neg(self) -> Self::Output {
        SqlDouble {
            value: self.value.map(|v| -v),
        }
    }
}

impl Neg for &SqlDouble {
    type Output = SqlDouble;
    fn neg(self) -> Self::Output {
        SqlDouble {
            value: self.value.map(|v| -v),
        }
    }
}

// ── SQL Comparisons ─────────────────────────────────────────────────────────

impl SqlDouble {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than-or-equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlDouble) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── Display / FromStr ───────────────────────────────────────────────────────

impl fmt::Display for SqlDouble {
    /// Displays the value using default `f64` formatting. NULL displays as `"Null"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            None => write!(f, "Null"),
            Some(v) => write!(f, "{v}"),
        }
    }
}

impl FromStr for SqlDouble {
    type Err = SqlTypeError;

    /// Parses a string into `SqlDouble`.
    /// `"Null"` (case-insensitive) → `SqlDouble::NULL`.
    /// NaN and Infinity strings are rejected.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlDouble::NULL);
        }

        let parsed: f64 = trimmed
            .parse()
            .map_err(|_| SqlTypeError::ParseError(format!("Invalid SqlDouble string: {s}")))?;

        if parsed.is_finite() {
            Ok(SqlDouble {
                value: Some(parsed),
            })
        } else {
            Err(SqlTypeError::Overflow)
        }
    }
}

// ── Conversions: Widening INTO SqlDouble ─────────────────────────────────────

impl SqlDouble {
    /// Converts from `SqlByte` (widening: `u8` → `f64`, always exact).
    /// NULL → NULL.
    pub fn from_sql_byte(v: SqlByte) -> SqlDouble {
        match v.value() {
            Ok(val) => SqlDouble {
                value: Some(val as f64),
            },
            Err(_) => SqlDouble::NULL,
        }
    }

    /// Converts from `SqlInt16` (widening: `i16` → `f64`, always exact).
    /// NULL → NULL.
    pub fn from_sql_int16(v: SqlInt16) -> SqlDouble {
        match v.value() {
            Ok(val) => SqlDouble {
                value: Some(val as f64),
            },
            Err(_) => SqlDouble::NULL,
        }
    }

    /// Converts from `SqlInt32` (widening: `i32` → `f64`, always exact).
    /// NULL → NULL.
    pub fn from_sql_int32(v: SqlInt32) -> SqlDouble {
        match v.value() {
            Ok(val) => SqlDouble {
                value: Some(val as f64),
            },
            Err(_) => SqlDouble::NULL,
        }
    }

    /// Converts from `SqlInt64` (widening: `i64` → `f64`, may lose precision for large values).
    /// NULL → NULL.
    pub fn from_sql_int64(v: SqlInt64) -> SqlDouble {
        match v.value() {
            Ok(val) => SqlDouble {
                value: Some(val as f64),
            },
            Err(_) => SqlDouble::NULL,
        }
    }

    /// Converts from `SqlMoney` (widening: extract scaled `i64`, divide by 10,000.0).
    /// NULL → NULL.
    pub fn from_sql_money(v: SqlMoney) -> SqlDouble {
        match v.scaled_value() {
            Ok(scaled) => SqlDouble {
                value: Some(scaled as f64 / 10_000.0),
            },
            Err(_) => SqlDouble::NULL,
        }
    }

    /// Converts from `SqlBoolean`. TRUE = 1.0, FALSE = 0.0, NULL = NULL.
    pub fn from_sql_boolean(v: SqlBoolean) -> SqlDouble {
        if v.is_null() {
            SqlDouble::NULL
        } else if v.is_true() {
            SqlDouble { value: Some(1.0) }
        } else {
            SqlDouble::ZERO
        }
    }

    /// Widens `SqlSingle` (f32) to `SqlDouble` (f64). NULL → NULL.
    pub fn from_sql_single(v: SqlSingle) -> SqlDouble {
        if v.is_null() {
            SqlDouble::NULL
        } else {
            match v.value() {
                Ok(f) => SqlDouble {
                    value: Some(f64::from(f)),
                },
                Err(_) => SqlDouble::NULL,
            }
        }
    }
}

// ── Conversions: OUT of SqlDouble ────────────────────────────────────────────

impl SqlDouble {
    /// Converts to `SqlBoolean`. Zero → FALSE, non-zero → TRUE, NULL → NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0.0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// Narrows to `SqlSingle` (f32). Returns `Err(Overflow)` if the f64 value
    /// is finite but out of f32 range (result becomes infinite). NULL → `Ok(SqlSingle::NULL)`.
    pub fn to_sql_single(&self) -> Result<SqlSingle, SqlTypeError> {
        match self.value {
            None => Ok(SqlSingle::NULL),
            Some(v) => {
                let narrowed = v as f32;
                if narrowed.is_infinite() && v.is_finite() {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlSingle::new(narrowed)?)
                }
            }
        }
    }
}

impl SqlDouble {
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

impl PartialEq for SqlDouble {
    fn eq(&self, other: &Self) -> bool {
        match (self.value, other.value) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for SqlDouble {}

impl Hash for SqlDouble {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.value {
            None => 0u64.hash(state),
            Some(v) => {
                // Normalize -0.0 to 0.0 so that equal values hash identically
                let normalized = if v == 0.0 { 0.0 } else { v };
                normalized.to_bits().hash(state);
            }
        }
    }
}

impl PartialOrd for SqlDouble {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlDouble {
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

    fn hash_of(val: SqlDouble) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ── T007: Tests for new(), is_null(), value() ───────────────────────────

    #[test]
    fn new_positive_value() {
        let d = SqlDouble::new(3.14159265358979).unwrap();
        assert_eq!(d.value().unwrap(), 3.14159265358979);
        assert!(!d.is_null());
    }

    #[test]
    fn new_negative_value() {
        let d = SqlDouble::new(-2.718281828).unwrap();
        assert_eq!(d.value().unwrap(), -2.718281828);
    }

    #[test]
    fn new_zero() {
        let d = SqlDouble::new(0.0).unwrap();
        assert_eq!(d.value().unwrap(), 0.0);
    }

    #[test]
    fn null_is_null() {
        assert!(SqlDouble::NULL.is_null());
    }

    #[test]
    fn null_value_returns_err() {
        let result = SqlDouble::NULL.value();
        assert!(matches!(result, Err(SqlTypeError::NullValue)));
    }

    // ── T008: Tests for constants ───────────────────────────────────────────

    #[test]
    fn constant_null() {
        assert!(SqlDouble::NULL.is_null());
    }

    #[test]
    fn constant_zero() {
        assert_eq!(SqlDouble::ZERO.value().unwrap(), 0.0);
    }

    #[test]
    fn constant_min_value() {
        assert_eq!(SqlDouble::MIN_VALUE.value().unwrap(), f64::MIN);
    }

    #[test]
    fn constant_max_value() {
        assert_eq!(SqlDouble::MAX_VALUE.value().unwrap(), f64::MAX);
    }

    // ── T009: Tests for NaN/Infinity rejection ──────────────────────────────

    #[test]
    fn new_nan_rejected() {
        let result = SqlDouble::new(f64::NAN);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn new_infinity_rejected() {
        let result = SqlDouble::new(f64::INFINITY);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn new_neg_infinity_rejected() {
        let result = SqlDouble::new(f64::NEG_INFINITY);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T010: Tests for From<f64> ───────────────────────────────────────────

    #[test]
    fn from_f64_valid() {
        let d = SqlDouble::from(42.0);
        assert_eq!(d.value().unwrap(), 42.0);
    }

    #[test]
    fn from_f64_min() {
        let d = SqlDouble::from(f64::MIN);
        assert_eq!(d.value().unwrap(), f64::MIN);
    }

    #[test]
    fn from_f64_max() {
        let d = SqlDouble::from(f64::MAX);
        assert_eq!(d.value().unwrap(), f64::MAX);
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn from_f64_nan_panics() {
        let _ = SqlDouble::from(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn from_f64_infinity_panics() {
        let _ = SqlDouble::from(f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "non-finite")]
    fn from_f64_neg_infinity_panics() {
        let _ = SqlDouble::from(f64::NEG_INFINITY);
    }

    // ── T011: Tests for checked_add ─────────────────────────────────────────

    #[test]
    fn add_normal() {
        let a = SqlDouble::new(2.5).unwrap();
        let b = SqlDouble::new(3.5).unwrap();
        let result = a.checked_add(b).unwrap();
        assert_eq!(result.value().unwrap(), 6.0);
    }

    #[test]
    fn add_overflow() {
        let result = SqlDouble::MAX_VALUE.checked_add(SqlDouble::MAX_VALUE);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_null_lhs() {
        let result = SqlDouble::NULL.checked_add(SqlDouble::new(1.0).unwrap());
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_null_rhs() {
        let result = SqlDouble::new(1.0).unwrap().checked_add(SqlDouble::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_operator() {
        let a = SqlDouble::new(2.5).unwrap();
        let b = SqlDouble::new(3.5).unwrap();
        let result = (a + b).unwrap();
        assert_eq!(result.value().unwrap(), 6.0);
    }

    #[test]
    fn add_ref_operators() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_eq!((&a + b).unwrap().value().unwrap(), 3.0);
        assert_eq!((a + &b).unwrap().value().unwrap(), 3.0);
        assert_eq!((&a + &b).unwrap().value().unwrap(), 3.0);
    }

    // ── T012: Tests for checked_sub ─────────────────────────────────────────

    #[test]
    fn sub_normal() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(3.0).unwrap();
        let result = a.checked_sub(b).unwrap();
        assert_eq!(result.value().unwrap(), 7.0);
    }

    #[test]
    fn sub_overflow() {
        let a = SqlDouble::new(-f64::MAX).unwrap();
        let b = SqlDouble::MAX_VALUE;
        let result = a.checked_sub(b);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn sub_null_propagation() {
        let result = SqlDouble::NULL.checked_sub(SqlDouble::new(1.0).unwrap());
        assert!(result.unwrap().is_null());
        let result = SqlDouble::new(1.0).unwrap().checked_sub(SqlDouble::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn sub_operator() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(3.0).unwrap();
        let result = (a - b).unwrap();
        assert_eq!(result.value().unwrap(), 7.0);
    }

    #[test]
    fn sub_ref_operators() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(3.0).unwrap();
        assert_eq!((&a - b).unwrap().value().unwrap(), 7.0);
        assert_eq!((a - &b).unwrap().value().unwrap(), 7.0);
        assert_eq!((&a - &b).unwrap().value().unwrap(), 7.0);
    }

    // ── T013: Tests for checked_mul ─────────────────────────────────────────

    #[test]
    fn mul_normal() {
        let a = SqlDouble::new(4.0).unwrap();
        let b = SqlDouble::new(2.5).unwrap();
        let result = a.checked_mul(b).unwrap();
        assert_eq!(result.value().unwrap(), 10.0);
    }

    #[test]
    fn mul_overflow() {
        let a = SqlDouble::MAX_VALUE;
        let b = SqlDouble::new(2.0).unwrap();
        let result = a.checked_mul(b);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn mul_null_propagation() {
        let result = SqlDouble::NULL.checked_mul(SqlDouble::new(2.0).unwrap());
        assert!(result.unwrap().is_null());
        let result = SqlDouble::new(2.0).unwrap().checked_mul(SqlDouble::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn mul_operator() {
        let a = SqlDouble::new(4.0).unwrap();
        let b = SqlDouble::new(2.5).unwrap();
        let result = (a * b).unwrap();
        assert_eq!(result.value().unwrap(), 10.0);
    }

    #[test]
    fn mul_ref_operators() {
        let a = SqlDouble::new(4.0).unwrap();
        let b = SqlDouble::new(2.5).unwrap();
        assert_eq!((&a * b).unwrap().value().unwrap(), 10.0);
        assert_eq!((a * &b).unwrap().value().unwrap(), 10.0);
        assert_eq!((&a * &b).unwrap().value().unwrap(), 10.0);
    }

    // ── T014: Tests for checked_div ─────────────────────────────────────────

    #[test]
    fn div_normal() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(4.0).unwrap();
        let result = a.checked_div(b).unwrap();
        assert_eq!(result.value().unwrap(), 2.5);
    }

    #[test]
    fn div_by_zero() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_zero_by_zero() {
        let a = SqlDouble::new(0.0).unwrap();
        let b = SqlDouble::new(0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_by_neg_zero() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(-0.0).unwrap();
        let result = a.checked_div(b);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_null_propagation() {
        let result = SqlDouble::NULL.checked_div(SqlDouble::new(1.0).unwrap());
        assert!(result.unwrap().is_null());
        let result = SqlDouble::new(1.0).unwrap().checked_div(SqlDouble::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn div_operator() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(4.0).unwrap();
        let result = (a / b).unwrap();
        assert_eq!(result.value().unwrap(), 2.5);
    }

    #[test]
    fn div_ref_operators() {
        let a = SqlDouble::new(10.0).unwrap();
        let b = SqlDouble::new(4.0).unwrap();
        assert_eq!((&a / b).unwrap().value().unwrap(), 2.5);
        assert_eq!((a / &b).unwrap().value().unwrap(), 2.5);
        assert_eq!((&a / &b).unwrap().value().unwrap(), 2.5);
    }

    // ── T019: Tests for Neg ─────────────────────────────────────────────────

    #[test]
    fn neg_positive() {
        let d = SqlDouble::new(5.0).unwrap();
        let result = -d;
        assert_eq!(result.value().unwrap(), -5.0);
    }

    #[test]
    fn neg_negative() {
        let d = SqlDouble::new(-3.14).unwrap();
        let result = -d;
        assert_eq!(result.value().unwrap(), 3.14);
    }

    #[test]
    fn neg_zero_gives_neg_zero() {
        let d = SqlDouble::new(0.0).unwrap();
        let result = -d;
        let v = result.value().unwrap();
        assert!(v.is_sign_negative());
        assert_eq!(v, 0.0); // IEEE 754: -0.0 == 0.0
    }

    #[test]
    fn neg_null_returns_null() {
        let result = -SqlDouble::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn neg_ref() {
        let d = SqlDouble::new(5.0).unwrap();
        let result = -&d;
        assert_eq!(result.value().unwrap(), -5.0);
    }

    // ── T021: Tests for SQL comparisons ─────────────────────────────────────

    #[test]
    fn sql_equals_equal_values() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(1.0).unwrap();
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_different_values() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_equals_null_operand() {
        let a = SqlDouble::new(1.0).unwrap();
        assert!(a.sql_equals(&SqlDouble::NULL).is_null());
        assert!(SqlDouble::NULL.sql_equals(&a).is_null());
        assert!(SqlDouble::NULL.sql_equals(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn sql_not_equals() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_eq!(a.sql_not_equals(&b), SqlBoolean::TRUE);
        assert_eq!(a.sql_not_equals(&a), SqlBoolean::FALSE);
        assert!(a.sql_not_equals(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn sql_less_than() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
        assert_eq!(b.sql_less_than(&a), SqlBoolean::FALSE);
        assert_eq!(a.sql_less_than(&a), SqlBoolean::FALSE);
        assert!(a.sql_less_than(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn sql_greater_than() {
        let a = SqlDouble::new(2.0).unwrap();
        let b = SqlDouble::new(1.0).unwrap();
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
        assert_eq!(b.sql_greater_than(&a), SqlBoolean::FALSE);
        assert!(a.sql_greater_than(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn sql_less_than_or_equal() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::TRUE);
        assert_eq!(a.sql_less_than_or_equal(&a), SqlBoolean::TRUE);
        assert_eq!(b.sql_less_than_or_equal(&a), SqlBoolean::FALSE);
        assert!(a.sql_less_than_or_equal(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn sql_greater_than_or_equal() {
        let a = SqlDouble::new(2.0).unwrap();
        let b = SqlDouble::new(1.0).unwrap();
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::TRUE);
        assert_eq!(a.sql_greater_than_or_equal(&a), SqlBoolean::TRUE);
        assert_eq!(b.sql_greater_than_or_equal(&a), SqlBoolean::FALSE);
        assert!(a.sql_greater_than_or_equal(&SqlDouble::NULL).is_null());
    }

    // ── T023: Tests for Display ─────────────────────────────────────────────

    #[test]
    fn display_positive() {
        let d = SqlDouble::new(3.14159265358979).unwrap();
        assert_eq!(format!("{d}"), "3.14159265358979");
    }

    #[test]
    fn display_negative() {
        let d = SqlDouble::new(-2.718).unwrap();
        assert_eq!(format!("{d}"), "-2.718");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", SqlDouble::ZERO), "0");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlDouble::NULL), "Null");
    }

    // ── T024: Tests for FromStr ─────────────────────────────────────────────

    #[test]
    fn parse_valid_number() {
        let d: SqlDouble = "3.14159265358979".parse().unwrap();
        assert_eq!(d.value().unwrap(), 3.14159265358979);
    }

    #[test]
    fn parse_null_case_insensitive() {
        let d: SqlDouble = "Null".parse().unwrap();
        assert!(d.is_null());
        let d: SqlDouble = "null".parse().unwrap();
        assert!(d.is_null());
        let d: SqlDouble = "NULL".parse().unwrap();
        assert!(d.is_null());
    }

    #[test]
    fn parse_nan_rejected() {
        let result = "NaN".parse::<SqlDouble>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_infinity_rejected() {
        let result = "inf".parse::<SqlDouble>();
        assert!(result.is_err());
        let result = "infinity".parse::<SqlDouble>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_neg_infinity_rejected() {
        let result = "-inf".parse::<SqlDouble>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_string() {
        let result = "abc".parse::<SqlDouble>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_negative() {
        let d: SqlDouble = "-42.5".parse().unwrap();
        assert_eq!(d.value().unwrap(), -42.5);
    }

    #[test]
    fn parse_integer() {
        let d: SqlDouble = "100".parse().unwrap();
        assert_eq!(d.value().unwrap(), 100.0);
    }

    // ── T027: Tests for from_sql_byte, from_sql_int16, from_sql_int32 ──────

    #[test]
    fn from_sql_byte_value() {
        let d = SqlDouble::from_sql_byte(SqlByte::new(42));
        assert_eq!(d.value().unwrap(), 42.0);
    }

    #[test]
    fn from_sql_byte_null() {
        let d = SqlDouble::from_sql_byte(SqlByte::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_int16_value() {
        let d = SqlDouble::from_sql_int16(SqlInt16::new(1000));
        assert_eq!(d.value().unwrap(), 1000.0);
    }

    #[test]
    fn from_sql_int16_null() {
        let d = SqlDouble::from_sql_int16(SqlInt16::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_int32_value() {
        let d = SqlDouble::from_sql_int32(SqlInt32::new(100_000));
        assert_eq!(d.value().unwrap(), 100_000.0);
    }

    #[test]
    fn from_sql_int32_null() {
        let d = SqlDouble::from_sql_int32(SqlInt32::NULL);
        assert!(d.is_null());
    }

    // ── T028: Tests for from_sql_int64 ──────────────────────────────────────

    #[test]
    fn from_sql_int64_value() {
        let d = SqlDouble::from_sql_int64(SqlInt64::new(1_000_000_000));
        assert_eq!(d.value().unwrap(), 1_000_000_000.0);
    }

    #[test]
    fn from_sql_int64_large_value_precision_loss() {
        // i64::MAX as f64 loses low-order bits but is still finite
        let d = SqlDouble::from_sql_int64(SqlInt64::new(i64::MAX));
        let v = d.value().unwrap();
        assert!(v.is_finite());
        // Cannot be exact due to f64 precision limits for large i64
        assert!(v > 0.0);
    }

    #[test]
    fn from_sql_int64_null() {
        let d = SqlDouble::from_sql_int64(SqlInt64::NULL);
        assert!(d.is_null());
    }

    // ── T029: Tests for from_sql_money ──────────────────────────────────────

    #[test]
    fn from_sql_money_value() {
        let m = SqlMoney::from_f64(42.5).unwrap();
        let d = SqlDouble::from_sql_money(m);
        assert_eq!(d.value().unwrap(), 42.5);
    }

    #[test]
    fn from_sql_money_negative() {
        let m = SqlMoney::from_f64(-100.1234).unwrap();
        let d = SqlDouble::from_sql_money(m);
        let v = d.value().unwrap();
        assert!((v - (-100.1234)).abs() < 1e-10);
    }

    #[test]
    fn from_sql_money_null() {
        let d = SqlDouble::from_sql_money(SqlMoney::NULL);
        assert!(d.is_null());
    }

    // ── T030: Tests for from_sql_boolean ────────────────────────────────────

    #[test]
    fn from_sql_boolean_true() {
        let d = SqlDouble::from_sql_boolean(SqlBoolean::TRUE);
        assert_eq!(d.value().unwrap(), 1.0);
    }

    #[test]
    fn from_sql_boolean_false() {
        let d = SqlDouble::from_sql_boolean(SqlBoolean::FALSE);
        assert_eq!(d.value().unwrap(), 0.0);
    }

    #[test]
    fn from_sql_boolean_null() {
        let d = SqlDouble::from_sql_boolean(SqlBoolean::NULL);
        assert!(d.is_null());
    }

    // ── T031: Tests for to_sql_boolean ──────────────────────────────────────

    #[test]
    fn to_sql_boolean_nonzero() {
        let d = SqlDouble::new(42.0).unwrap();
        assert_eq!(d.to_sql_boolean(), SqlBoolean::TRUE);
    }

    #[test]
    fn to_sql_boolean_zero() {
        let d = SqlDouble::new(0.0).unwrap();
        assert_eq!(d.to_sql_boolean(), SqlBoolean::FALSE);
    }

    #[test]
    fn to_sql_boolean_negative() {
        let d = SqlDouble::new(-1.5).unwrap();
        assert_eq!(d.to_sql_boolean(), SqlBoolean::TRUE);
    }

    #[test]
    fn to_sql_boolean_null() {
        assert_eq!(SqlDouble::NULL.to_sql_boolean(), SqlBoolean::NULL);
    }

    // ── T037: Tests for PartialEq/Eq ────────────────────────────────────────

    #[test]
    fn eq_equal_values() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(1.0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn eq_different_values() {
        let a = SqlDouble::new(1.0).unwrap();
        let b = SqlDouble::new(2.0).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn eq_null_equals_null() {
        assert_eq!(SqlDouble::NULL, SqlDouble::NULL);
    }

    #[test]
    fn eq_null_ne_value() {
        assert_ne!(SqlDouble::NULL, SqlDouble::new(1.0).unwrap());
    }

    #[test]
    fn eq_neg_zero_equals_zero() {
        let a = SqlDouble::new(0.0).unwrap();
        let b = SqlDouble::new(-0.0).unwrap();
        assert_eq!(a, b); // IEEE 754: 0.0 == -0.0
    }

    // ── T038: Tests for Hash ────────────────────────────────────────────────

    #[test]
    fn hash_equal_values_equal() {
        let a = SqlDouble::new(42.0).unwrap();
        let b = SqlDouble::new(42.0).unwrap();
        assert_eq!(hash_of(a), hash_of(b));
    }

    #[test]
    fn hash_zero_neg_zero_equal() {
        let a = SqlDouble::new(0.0).unwrap();
        let b = SqlDouble::new(-0.0).unwrap();
        assert_eq!(hash_of(a), hash_of(b));
    }

    #[test]
    fn hash_null_consistent() {
        assert_eq!(hash_of(SqlDouble::NULL), hash_of(SqlDouble::NULL));
    }

    // ── T039: Tests for PartialOrd/Ord ──────────────────────────────────────

    #[test]
    fn ord_null_less_than_value() {
        assert!(SqlDouble::NULL < SqlDouble::new(0.0).unwrap());
    }

    #[test]
    fn ord_negative_less_than_positive() {
        let neg = SqlDouble::new(-1.0).unwrap();
        let pos = SqlDouble::new(1.0).unwrap();
        assert!(neg < pos);
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlDouble::MIN_VALUE < SqlDouble::MAX_VALUE);
    }

    #[test]
    fn ord_equal_values() {
        let a = SqlDouble::new(5.0).unwrap();
        let b = SqlDouble::new(5.0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ord_null_equals_null() {
        assert_eq!(SqlDouble::NULL.cmp(&SqlDouble::NULL), Ordering::Equal);
    }

    // ── T044: Quickstart validation ─────────────────────────────────────────

    #[test]
    fn quickstart_create_values() {
        let pi = SqlDouble::new(3.14159265358979).unwrap();
        assert_eq!(pi.value().unwrap(), 3.14159265358979);

        let neg = SqlDouble::new(-2.718281828).unwrap();
        assert_eq!(neg.value().unwrap(), -2.718281828);

        let zero = SqlDouble::new(0.0).unwrap();
        assert_eq!(zero.value().unwrap(), 0.0);

        assert_eq!(SqlDouble::ZERO.value().unwrap(), 0.0);
        assert_eq!(SqlDouble::MIN_VALUE.value().unwrap(), f64::MIN);
        assert_eq!(SqlDouble::MAX_VALUE.value().unwrap(), f64::MAX);

        let null = SqlDouble::NULL;
        assert!(null.is_null());
        assert!(null.value().is_err());
    }

    #[test]
    fn quickstart_nan_infinity_rejected() {
        assert!(SqlDouble::new(f64::NAN).is_err());
        assert!(SqlDouble::new(f64::INFINITY).is_err());
        assert!(SqlDouble::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn quickstart_arithmetic() {
        let a = SqlDouble::new(2.5).unwrap();
        let b = SqlDouble::new(3.5).unwrap();
        let sum = (a + b).unwrap();
        assert_eq!(sum.value().unwrap(), 6.0);

        let diff = (SqlDouble::new(10.0).unwrap() - SqlDouble::new(3.0).unwrap()).unwrap();
        assert_eq!(diff.value().unwrap(), 7.0);

        let product = (SqlDouble::new(4.0).unwrap() * SqlDouble::new(2.5).unwrap()).unwrap();
        assert_eq!(product.value().unwrap(), 10.0);

        let quotient = (SqlDouble::new(10.0).unwrap() / SqlDouble::new(4.0).unwrap()).unwrap();
        assert_eq!(quotient.value().unwrap(), 2.5);

        let neg = -SqlDouble::new(5.0).unwrap();
        assert_eq!(neg.value().unwrap(), -5.0);

        let null_result = (SqlDouble::new(1.0).unwrap() + SqlDouble::NULL).unwrap();
        assert!(null_result.is_null());
    }

    #[test]
    fn quickstart_overflow_div_zero() {
        let overflow = SqlDouble::MAX_VALUE + SqlDouble::MAX_VALUE;
        assert!(overflow.is_err());

        let div_zero = SqlDouble::new(1.0).unwrap() / SqlDouble::new(0.0).unwrap();
        assert!(div_zero.is_err());

        let nan_div = SqlDouble::new(0.0).unwrap() / SqlDouble::new(0.0).unwrap();
        assert!(nan_div.is_err());
    }

    #[test]
    fn quickstart_comparisons() {
        let x = SqlDouble::new(1.0).unwrap();
        let y = SqlDouble::new(2.0).unwrap();
        assert_eq!(x.sql_less_than(&y), SqlBoolean::TRUE);
        assert_eq!(y.sql_greater_than(&x), SqlBoolean::TRUE);
        assert_eq!(x.sql_equals(&x), SqlBoolean::TRUE);
        assert_eq!(x.sql_not_equals(&y), SqlBoolean::TRUE);
        assert!(x.sql_equals(&SqlDouble::NULL).is_null());
    }

    #[test]
    fn quickstart_display_parse() {
        let val = SqlDouble::new(3.14159265358979).unwrap();
        assert_eq!(format!("{val}"), "3.14159265358979");
        assert_eq!(format!("{}", SqlDouble::NULL), "Null");

        let parsed: SqlDouble = "3.14".parse().unwrap();
        assert_eq!(parsed.value().unwrap(), 3.14);

        let null: SqlDouble = "Null".parse().unwrap();
        assert!(null.is_null());

        assert!("abc".parse::<SqlDouble>().is_err());
        assert!("NaN".parse::<SqlDouble>().is_err());
    }

    #[test]
    fn quickstart_conversions() {
        let from_byte = SqlDouble::from_sql_byte(SqlByte::new(42));
        assert_eq!(from_byte.value().unwrap(), 42.0);

        let from_i32 = SqlDouble::from_sql_int32(SqlInt32::new(100_000));
        assert_eq!(from_i32.value().unwrap(), 100_000.0);

        let from_i64 = SqlDouble::from_sql_int64(SqlInt64::new(1_000_000_000));
        assert_eq!(from_i64.value().unwrap(), 1_000_000_000.0);

        let from_true = SqlDouble::from_sql_boolean(SqlBoolean::TRUE);
        assert_eq!(from_true.value().unwrap(), 1.0);

        let from_false = SqlDouble::from_sql_boolean(SqlBoolean::FALSE);
        assert_eq!(from_false.value().unwrap(), 0.0);

        let from_null = SqlDouble::from_sql_byte(SqlByte::NULL);
        assert!(from_null.is_null());

        let to_bool = SqlDouble::new(42.0).unwrap().to_sql_boolean();
        assert_eq!(to_bool, SqlBoolean::TRUE);

        let to_bool_zero = SqlDouble::new(0.0).unwrap().to_sql_boolean();
        assert_eq!(to_bool_zero, SqlBoolean::FALSE);
    }

    // ── Edge case tests ─────────────────────────────────────────────────────

    #[test]
    fn subnormal_value_accepted() {
        let subnormal = f64::MIN_POSITIVE * 0.5;
        let d = SqlDouble::new(subnormal).unwrap();
        assert_eq!(d.value().unwrap(), subnormal);
    }

    #[test]
    fn neg_max_is_min() {
        let d = -SqlDouble::MAX_VALUE;
        assert_eq!(d.value().unwrap(), -f64::MAX);
    }

    #[test]
    fn neg_min_is_max() {
        let d = -SqlDouble::MIN_VALUE;
        assert_eq!(d.value().unwrap(), f64::MAX);
    }

    #[test]
    fn display_fromstr_roundtrip() {
        let values = [0.0, 1.0, -1.0, 3.14, f64::MIN_POSITIVE, 1e100, -1e100];
        for &v in &values {
            let d = SqlDouble::new(v).unwrap();
            let s = format!("{d}");
            let parsed: SqlDouble = s.parse().unwrap();
            assert_eq!(d, parsed, "roundtrip failed for {v}");
        }
    }

    // ── from_sql_single() tests ──────────────────────────────────────────

    #[test]
    fn from_sql_single_normal() {
        let s = SqlSingle::new(3.14).unwrap();
        let d = SqlDouble::from_sql_single(s);
        assert!(!d.is_null());
        assert!((d.value().unwrap() - 3.14f64).abs() < 0.001);
    }

    #[test]
    fn from_sql_single_zero() {
        let s = SqlSingle::new(0.0).unwrap();
        let d = SqlDouble::from_sql_single(s);
        assert_eq!(d.value().unwrap(), 0.0);
    }

    #[test]
    fn from_sql_single_null() {
        let d = SqlDouble::from_sql_single(SqlSingle::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_single_max() {
        let s = SqlSingle::new(f32::MAX).unwrap();
        let d = SqlDouble::from_sql_single(s);
        assert_eq!(d.value().unwrap(), f32::MAX as f64);
    }

    // ── to_sql_single() tests ───────────────────────────────────────────

    #[test]
    fn to_sql_single_normal() {
        let d = SqlDouble::new(3.14).unwrap();
        let s = d.to_sql_single().unwrap();
        assert!((s.value().unwrap() - 3.14f32).abs() < 0.001);
    }

    #[test]
    fn to_sql_single_overflow() {
        let d = SqlDouble::new(1e300).unwrap();
        let result = d.to_sql_single();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_single_null() {
        let d = SqlDouble::NULL;
        let s = d.to_sql_single().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn to_sql_single_f32_max_roundtrip() {
        let d = SqlDouble::new(f32::MAX as f64).unwrap();
        let s = d.to_sql_single().unwrap();
        assert_eq!(s.value().unwrap(), f32::MAX);
    }

    #[test]
    fn to_sql_single_negative_overflow() {
        let d = SqlDouble::new(-1e300).unwrap();
        let result = d.to_sql_single();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── to_sql_string() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_string_positive() {
        let s = SqlDouble::new(3.14159).unwrap().to_sql_string();
        assert_eq!(s.value().unwrap(), "3.14159");
    }

    #[test]
    fn to_sql_string_negative() {
        let s = SqlDouble::new(-2.5).unwrap().to_sql_string();
        assert_eq!(s.value().unwrap(), "-2.5");
    }

    #[test]
    fn to_sql_string_zero() {
        let s = SqlDouble::new(0.0).unwrap().to_sql_string();
        assert_eq!(s.value().unwrap(), "0");
    }

    #[test]
    fn to_sql_string_null() {
        let s = SqlDouble::NULL.to_sql_string();
        assert!(s.is_null());
    }
}
