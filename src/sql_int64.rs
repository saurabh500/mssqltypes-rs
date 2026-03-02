// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// ── T001: SqlInt64 module ─────────────────────────────────────────────────────

//! `SqlInt64` — a signed 64-bit integer with SQL NULL support, equivalent to
//! C# `System.Data.SqlTypes.SqlInt64` / SQL Server `BIGINT`.
//!
//! Uses `Option<i64>` internally: `None` = SQL NULL, `Some(v)` = a value.
//! All arithmetic returns `Result<SqlInt64, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A signed 64-bit integer (−9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
/// with SQL NULL support, equivalent to C# `System.Data.SqlTypes.SqlInt64` / SQL
/// Server `BIGINT`.
///
/// Uses `Option<i64>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// All arithmetic returns `Result<SqlInt64, SqlTypeError>` with overflow detection
/// using Rust's `checked_*` methods. Comparisons return `SqlBoolean` for
/// three-valued NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlInt64 {
    value: Option<i64>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors and accessors ────────────────────────────────────────

impl SqlInt64 {
    /// SQL NULL.
    pub const NULL: SqlInt64 = SqlInt64 { value: None };
    /// Zero (0).
    pub const ZERO: SqlInt64 = SqlInt64 { value: Some(0) };
    /// Minimum value (−9,223,372,036,854,775,808).
    pub const MIN_VALUE: SqlInt64 = SqlInt64 {
        value: Some(i64::MIN),
    };
    /// Maximum value (9,223,372,036,854,775,807).
    pub const MAX_VALUE: SqlInt64 = SqlInt64 {
        value: Some(i64::MAX),
    };

    /// Creates a new `SqlInt64` from an `i64` value.
    pub fn new(v: i64) -> Self {
        SqlInt64 { value: Some(v) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `i64`, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<i64, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<i64> ─────────────────────────────────────────────────────────

impl From<i64> for SqlInt64 {
    fn from(v: i64) -> Self {
        SqlInt64::new(v)
    }
}

// ── T016: Checked arithmetic (add, sub) ─────────────────────────────────────
// ── T017: Checked arithmetic (mul) ──────────────────────────────────────────
// ── T018: Checked arithmetic (div) ──────────────────────────────────────────
// ── T019: Checked arithmetic (rem) ──────────────────────────────────────────
// ── T020: Checked arithmetic (neg) ──────────────────────────────────────────

impl SqlInt64 {
    /// Checked addition. Returns `Err(Overflow)` if result overflows i64.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_add(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt64::NULL),
            (Some(a), Some(b)) => a
                .checked_add(b)
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if result overflows i64.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_sub(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt64::NULL),
            (Some(a), Some(b)) => a
                .checked_sub(b)
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if result overflows i64.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_mul(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt64::NULL),
            (Some(a), Some(b)) => a
                .checked_mul(b)
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked division. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE / -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_div(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt64::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_div(b)
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked remainder. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE % -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_rem(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt64::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_rem(b)
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked negation. Returns `Err(Overflow)` if value is MIN_VALUE.
    /// NULL propagation: if operand is NULL, returns `Ok(SqlInt64::NULL)`.
    pub fn checked_neg(self) -> Result<SqlInt64, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt64::NULL),
            Some(v) => v
                .checked_neg()
                .map(SqlInt64::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }
}

// ── T021: Operator traits ───────────────────────────────────────────────────

impl Add for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn add(self, rhs: SqlInt64) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn sub(self, rhs: SqlInt64) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Mul for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn mul(self, rhs: SqlInt64) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Div for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn div(self, rhs: SqlInt64) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Rem for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn rem(self, rhs: SqlInt64) -> Self::Output {
        self.checked_rem(rhs)
    }
}

impl Neg for SqlInt64 {
    type Output = Result<SqlInt64, SqlTypeError>;

    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

// ── T024: Bitwise operations ────────────────────────────────────────────────
// ── T025: Not / ones_complement ─────────────────────────────────────────────

impl SqlInt64 {
    /// Returns the ones' complement (~value). NULL → NULL.
    pub fn ones_complement(self) -> SqlInt64 {
        match self.value {
            None => SqlInt64::NULL,
            Some(v) => SqlInt64::new(!v),
        }
    }
}

impl BitAnd for SqlInt64 {
    type Output = SqlInt64;

    fn bitand(self, rhs: SqlInt64) -> SqlInt64 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt64::new(a & b),
            _ => SqlInt64::NULL,
        }
    }
}

impl BitOr for SqlInt64 {
    type Output = SqlInt64;

    fn bitor(self, rhs: SqlInt64) -> SqlInt64 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt64::new(a | b),
            _ => SqlInt64::NULL,
        }
    }
}

impl BitXor for SqlInt64 {
    type Output = SqlInt64;

    fn bitxor(self, rhs: SqlInt64) -> SqlInt64 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt64::new(a ^ b),
            _ => SqlInt64::NULL,
        }
    }
}

impl Not for SqlInt64 {
    type Output = SqlInt64;

    fn not(self) -> SqlInt64 {
        self.ones_complement()
    }
}

// ── T027: SQL comparison methods ────────────────────────────────────────────

impl SqlInt64 {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt64) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T043: PartialEq, Eq, Hash ───────────────────────────────────────────────
// ── T044: PartialOrd, Ord ───────────────────────────────────────────────────

impl PartialEq for SqlInt64 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlInt64 {}

impl Hash for SqlInt64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NULL hashes as 0i64 for consistency
        match self.value {
            Some(v) => v.hash(state),
            None => 0i64.hash(state),
        }
    }
}

impl PartialOrd for SqlInt64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlInt64 {
    fn cmp(&self, other: &Self) -> Ordering {
        // NULL < any non-null value
        match (self.value, other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}

// ── T030: Display ───────────────────────────────────────────────────────────
// ── T031: FromStr ───────────────────────────────────────────────────────────

impl fmt::Display for SqlInt64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "Null"),
        }
    }
}

impl FromStr for SqlInt64 {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SqlTypeError::ParseError(
                "Cannot parse empty string as SqlInt64".to_string(),
            ));
        }

        // Check for "null" (case-insensitive)
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlInt64::NULL);
        }

        // Parse as i64
        match trimmed.parse::<i64>() {
            Ok(v) => Ok(SqlInt64::new(v)),
            Err(e) => Err(SqlTypeError::ParseError(format!(
                "Cannot parse '{s}' as SqlInt64: {e}"
            ))),
        }
    }
}

// ── T036: From<SqlBoolean> ──────────────────────────────────────────────────
// ── T037: to_sql_int32 ─────────────────────────────────────────────────────
// ── T038: to_sql_int16 ─────────────────────────────────────────────────────
// ── T039: to_sql_byte ──────────────────────────────────────────────────────

impl From<SqlBoolean> for SqlInt64 {
    fn from(b: SqlBoolean) -> Self {
        if b.is_null() {
            SqlInt64::NULL
        } else {
            match b.value() {
                Ok(true) => SqlInt64::new(1),
                Ok(false) => SqlInt64::new(0),
                Err(_) => SqlInt64::NULL,
            }
        }
    }
}

impl SqlInt64 {
    /// Converts to `SqlInt32`: NULL→NULL, otherwise checks range −2,147,483,648..=2,147,483,647.
    /// Returns `Err(Overflow)` if value is outside i32 range.
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt32::NULL),
            Some(v) => {
                if v < i32::MIN as i64 || v > i32::MAX as i64 {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlInt32::new(v as i32))
                }
            }
        }
    }

    /// Converts to `SqlInt16`: NULL→NULL, otherwise checks range −32,768..=32,767.
    /// Returns `Err(Overflow)` if value is outside i16 range.
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt16::NULL),
            Some(v) => {
                if v < i16::MIN as i64 || v > i16::MAX as i64 {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlInt16::new(v as i16))
                }
            }
        }
    }

    /// Converts to `SqlByte`: NULL→NULL, otherwise checks range 0..=255.
    /// Returns `Err(Overflow)` if value is negative or > 255.
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError> {
        match self.value {
            None => Ok(SqlByte::NULL),
            Some(v) => {
                if !(0..=255).contains(&v) {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlByte::new(v as u8))
                }
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of(val: &SqlInt64) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ── T007: Tests for new(), is_null(), value() ───────────────────────────

    #[test]
    fn new_positive_value() {
        let v = SqlInt64::new(9_000_000_000);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 9_000_000_000);
    }

    #[test]
    fn new_negative_value() {
        let v = SqlInt64::new(-9_000_000_000);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), -9_000_000_000);
    }

    #[test]
    fn new_zero() {
        let v = SqlInt64::new(0);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn null_access_returns_err() {
        let v = SqlInt64::NULL;
        assert!(v.is_null());
        assert!(matches!(v.value(), Err(SqlTypeError::NullValue)));
    }

    // ── T008: Tests for constants ───────────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlInt64::NULL.is_null());
    }

    #[test]
    fn zero_value() {
        assert_eq!(SqlInt64::ZERO.value().unwrap(), 0);
    }

    #[test]
    fn min_value() {
        assert_eq!(
            SqlInt64::MIN_VALUE.value().unwrap(),
            -9_223_372_036_854_775_808
        );
    }

    #[test]
    fn max_value() {
        assert_eq!(
            SqlInt64::MAX_VALUE.value().unwrap(),
            9_223_372_036_854_775_807
        );
    }

    // ── T009: Tests for From<i64> ───────────────────────────────────────────

    #[test]
    fn from_i64_positive() {
        let v: SqlInt64 = 42i64.into();
        assert_eq!(v.value().unwrap(), 42);
    }

    #[test]
    fn from_i64_min() {
        let v = SqlInt64::from(i64::MIN);
        assert_eq!(v.value().unwrap(), i64::MIN);
    }

    #[test]
    fn from_i64_max() {
        let v = SqlInt64::from(i64::MAX);
        assert_eq!(v.value().unwrap(), i64::MAX);
    }

    // ── T010: Tests for checked_add ─────────────────────────────────────────

    #[test]
    fn add_normal() {
        let result = SqlInt64::new(100).checked_add(SqlInt64::new(200));
        assert_eq!(result.unwrap().value().unwrap(), 300);
    }

    #[test]
    fn add_overflow() {
        let result = SqlInt64::new(i64::MAX).checked_add(SqlInt64::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_underflow() {
        let result = SqlInt64::new(i64::MIN).checked_add(SqlInt64::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_null_lhs() {
        let result = SqlInt64::NULL.checked_add(SqlInt64::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_null_rhs() {
        let result = SqlInt64::new(1).checked_add(SqlInt64::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_operator() {
        let result = SqlInt64::new(100) + SqlInt64::new(200);
        assert_eq!(result.unwrap().value().unwrap(), 300);
    }

    // ── T011: Tests for checked_sub ─────────────────────────────────────────

    #[test]
    fn sub_normal() {
        let result = SqlInt64::new(300).checked_sub(SqlInt64::new(100));
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn sub_overflow() {
        let result = SqlInt64::new(i64::MIN).checked_sub(SqlInt64::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn sub_null_propagation() {
        let result = SqlInt64::NULL.checked_sub(SqlInt64::new(1));
        assert!(result.unwrap().is_null());
        let result2 = SqlInt64::new(1).checked_sub(SqlInt64::NULL);
        assert!(result2.unwrap().is_null());
    }

    #[test]
    fn sub_operator() {
        let result = SqlInt64::new(300) - SqlInt64::new(100);
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    // ── T012: Tests for checked_mul ─────────────────────────────────────────

    #[test]
    fn mul_normal() {
        let result = SqlInt64::new(100).checked_mul(SqlInt64::new(200));
        assert_eq!(result.unwrap().value().unwrap(), 20_000);
    }

    #[test]
    fn mul_overflow_large() {
        let result = SqlInt64::new(5_000_000_000).checked_mul(SqlInt64::new(5_000_000_000));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn mul_overflow_max_times_2() {
        let result = SqlInt64::new(i64::MAX).checked_mul(SqlInt64::new(2));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn mul_null_propagation() {
        let result = SqlInt64::NULL.checked_mul(SqlInt64::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn mul_operator() {
        let result = SqlInt64::new(10) * SqlInt64::new(20);
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    // ── T013: Tests for checked_div ─────────────────────────────────────────

    #[test]
    fn div_normal() {
        let result = SqlInt64::new(100).checked_div(SqlInt64::new(10));
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    #[test]
    fn div_by_zero() {
        let result = SqlInt64::new(10).checked_div(SqlInt64::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_min_by_neg_one() {
        let result = SqlInt64::new(i64::MIN).checked_div(SqlInt64::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn div_null_propagation() {
        let result = SqlInt64::NULL.checked_div(SqlInt64::new(1));
        assert!(result.unwrap().is_null());
        let result2 = SqlInt64::new(1).checked_div(SqlInt64::NULL);
        assert!(result2.unwrap().is_null());
    }

    #[test]
    fn div_operator() {
        let result = SqlInt64::new(100) / SqlInt64::new(10);
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    // ── T014: Tests for checked_rem ─────────────────────────────────────────

    #[test]
    fn rem_normal() {
        let result = SqlInt64::new(7).checked_rem(SqlInt64::new(3));
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    #[test]
    fn rem_by_zero() {
        let result = SqlInt64::new(10).checked_rem(SqlInt64::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn rem_min_by_neg_one() {
        let result = SqlInt64::new(i64::MIN).checked_rem(SqlInt64::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn rem_null_propagation() {
        let result = SqlInt64::NULL.checked_rem(SqlInt64::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn rem_operator() {
        let result = SqlInt64::new(7) % SqlInt64::new(3);
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    // ── T015: Tests for checked_neg ─────────────────────────────────────────

    #[test]
    fn neg_normal() {
        let result = SqlInt64::new(42).checked_neg();
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn neg_min_value_overflow() {
        let result = SqlInt64::new(i64::MIN).checked_neg();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn neg_null_returns_null() {
        let result = SqlInt64::NULL.checked_neg();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn neg_operator() {
        let result = -SqlInt64::new(42);
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn neg_operator_min_overflow() {
        let result = -SqlInt64::new(i64::MIN);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T022: Tests for BitAnd, BitOr, BitXor ───────────────────────────────

    #[test]
    fn bitand_normal() {
        let result = SqlInt64::new(0xFF00) & SqlInt64::new(0x0FF0);
        assert_eq!(result.value().unwrap(), 0x0F00);
    }

    #[test]
    fn bitor_normal() {
        let result = SqlInt64::new(0xFF00) | SqlInt64::new(0x00FF);
        assert_eq!(result.value().unwrap(), 0xFFFF);
    }

    #[test]
    fn bitxor_normal() {
        let result = SqlInt64::new(0xFF) ^ SqlInt64::new(0x0F);
        assert_eq!(result.value().unwrap(), 0xF0);
    }

    #[test]
    fn bitand_negative_values() {
        let result = SqlInt64::new(-1) & SqlInt64::new(0xFF);
        assert_eq!(result.value().unwrap(), 0xFF);
    }

    #[test]
    fn bitand_null_propagation() {
        let result = SqlInt64::NULL & SqlInt64::new(0xFF);
        assert!(result.is_null());
    }

    #[test]
    fn bitor_null_propagation() {
        let result = SqlInt64::new(0xFF) | SqlInt64::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn bitxor_null_propagation() {
        let result = SqlInt64::NULL ^ SqlInt64::NULL;
        assert!(result.is_null());
    }

    // ── T023: Tests for Not / ones_complement ───────────────────────────────

    #[test]
    fn not_zero() {
        let result = !SqlInt64::new(0);
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn not_neg_one() {
        let result = !SqlInt64::new(-1);
        assert_eq!(result.value().unwrap(), 0);
    }

    #[test]
    fn ones_complement_method() {
        let result = SqlInt64::new(0).ones_complement();
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn not_null_returns_null() {
        let result = !SqlInt64::NULL;
        assert!(result.is_null());
    }

    // ── T026: Tests for SQL comparison methods ──────────────────────────────

    #[test]
    fn sql_equals_true() {
        let cmp = SqlInt64::new(100).sql_equals(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_false() {
        let cmp = SqlInt64::new(100).sql_equals(&SqlInt64::new(200));
        assert_eq!(cmp, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_not_equals_true() {
        let cmp = SqlInt64::new(100).sql_not_equals(&SqlInt64::new(200));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_true() {
        let cmp = SqlInt64::new(100).sql_less_than(&SqlInt64::new(200));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_false() {
        let cmp = SqlInt64::new(200).sql_less_than(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_true() {
        let cmp = SqlInt64::new(200).sql_greater_than(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let cmp = SqlInt64::new(100).sql_less_than_or_equal(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_less() {
        let cmp = SqlInt64::new(100).sql_less_than_or_equal(&SqlInt64::new(200));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let cmp = SqlInt64::new(100).sql_greater_than_or_equal(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_greater() {
        let cmp = SqlInt64::new(200).sql_greater_than_or_equal(&SqlInt64::new(100));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_null_lhs() {
        let cmp = SqlInt64::NULL.sql_equals(&SqlInt64::new(100));
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_equals_null_rhs() {
        let cmp = SqlInt64::new(100).sql_equals(&SqlInt64::NULL);
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_less_than_null() {
        let cmp = SqlInt64::new(100).sql_less_than(&SqlInt64::NULL);
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_greater_than_null() {
        let cmp = SqlInt64::NULL.sql_greater_than(&SqlInt64::new(100));
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_not_equals_null() {
        let cmp = SqlInt64::NULL.sql_not_equals(&SqlInt64::NULL);
        assert!(cmp.is_null());
    }

    // ── T028: Tests for Display ─────────────────────────────────────────────

    #[test]
    fn display_positive() {
        assert_eq!(format!("{}", SqlInt64::new(9_000_000_000)), "9000000000");
    }

    #[test]
    fn display_negative() {
        assert_eq!(format!("{}", SqlInt64::new(-100)), "-100");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", SqlInt64::ZERO), "0");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlInt64::NULL), "Null");
    }

    // ── T029: Tests for FromStr ─────────────────────────────────────────────

    #[test]
    fn parse_valid_positive() {
        let v: SqlInt64 = "9000000000".parse().unwrap();
        assert_eq!(v.value().unwrap(), 9_000_000_000);
    }

    #[test]
    fn parse_valid_negative() {
        let v: SqlInt64 = "-100".parse().unwrap();
        assert_eq!(v.value().unwrap(), -100);
    }

    #[test]
    fn parse_null_string() {
        let v: SqlInt64 = "Null".parse().unwrap();
        assert!(v.is_null());
    }

    #[test]
    fn parse_null_case_insensitive() {
        let v: SqlInt64 = "null".parse().unwrap();
        assert!(v.is_null());
        let v2: SqlInt64 = "NULL".parse().unwrap();
        assert!(v2.is_null());
    }

    #[test]
    fn parse_out_of_range() {
        let result = "99999999999999999999".parse::<SqlInt64>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_non_numeric() {
        let result = "abc".parse::<SqlInt64>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_empty_string() {
        let result = "".parse::<SqlInt64>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn display_fromstr_roundtrip() {
        let original = SqlInt64::new(9_000_000_000);
        let s = format!("{original}");
        let parsed: SqlInt64 = s.parse().unwrap();
        assert_eq!(original, parsed);
    }

    // ── T032: Tests for From<SqlBoolean> ────────────────────────────────────

    #[test]
    fn from_sql_boolean_true() {
        let v = SqlInt64::from(SqlBoolean::TRUE);
        assert_eq!(v.value().unwrap(), 1);
    }

    #[test]
    fn from_sql_boolean_false() {
        let v = SqlInt64::from(SqlBoolean::FALSE);
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_boolean_null() {
        let v = SqlInt64::from(SqlBoolean::NULL);
        assert!(v.is_null());
    }

    // ── T033: Tests for to_sql_int32 ────────────────────────────────────────

    #[test]
    fn to_sql_int32_in_range() {
        let result = SqlInt64::new(100).to_sql_int32();
        assert_eq!(result.unwrap().value().unwrap(), 100);
    }

    #[test]
    fn to_sql_int32_overflow() {
        let result = SqlInt64::new(3_000_000_000).to_sql_int32();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int32_underflow() {
        let result = SqlInt64::new(-3_000_000_000).to_sql_int32();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int32_null() {
        let result = SqlInt64::NULL.to_sql_int32();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn to_sql_int32_boundary_max() {
        let result = SqlInt64::new(i32::MAX as i64).to_sql_int32();
        assert_eq!(result.unwrap().value().unwrap(), i32::MAX);
    }

    #[test]
    fn to_sql_int32_boundary_min() {
        let result = SqlInt64::new(i32::MIN as i64).to_sql_int32();
        assert_eq!(result.unwrap().value().unwrap(), i32::MIN);
    }

    #[test]
    fn to_sql_int32_just_over_max() {
        let result = SqlInt64::new(i32::MAX as i64 + 1).to_sql_int32();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int32_just_under_min() {
        let result = SqlInt64::new(i32::MIN as i64 - 1).to_sql_int32();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T034: Tests for to_sql_int16 ────────────────────────────────────────

    #[test]
    fn to_sql_int16_in_range() {
        let result = SqlInt64::new(100).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), 100);
    }

    #[test]
    fn to_sql_int16_overflow() {
        let result = SqlInt64::new(100_000).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_underflow() {
        let result = SqlInt64::new(-100_000).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_null() {
        let result = SqlInt64::NULL.to_sql_int16();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn to_sql_int16_boundary_max() {
        let result = SqlInt64::new(i16::MAX as i64).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), i16::MAX);
    }

    #[test]
    fn to_sql_int16_boundary_min() {
        let result = SqlInt64::new(i16::MIN as i64).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), i16::MIN);
    }

    #[test]
    fn to_sql_int16_just_over_max() {
        let result = SqlInt64::new(i16::MAX as i64 + 1).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_just_under_min() {
        let result = SqlInt64::new(i16::MIN as i64 - 1).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T035: Tests for to_sql_byte ─────────────────────────────────────────

    #[test]
    fn to_sql_byte_in_range() {
        let result = SqlInt64::new(200).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn to_sql_byte_overflow() {
        let result = SqlInt64::new(300).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_negative() {
        let result = SqlInt64::new(-1).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_null() {
        let result = SqlInt64::NULL.to_sql_byte();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn to_sql_byte_zero() {
        let result = SqlInt64::new(0).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 0);
    }

    #[test]
    fn to_sql_byte_max_valid() {
        let result = SqlInt64::new(255).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 255);
    }

    #[test]
    fn to_sql_byte_just_over_max() {
        let result = SqlInt64::new(256).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T040: Tests for PartialEq / Eq ──────────────────────────────────────

    #[test]
    fn eq_same_value() {
        assert_eq!(SqlInt64::new(42), SqlInt64::new(42));
    }

    #[test]
    fn eq_different_value() {
        assert_ne!(SqlInt64::new(42), SqlInt64::new(43));
    }

    #[test]
    fn eq_null_null() {
        assert_eq!(SqlInt64::NULL, SqlInt64::NULL);
    }

    #[test]
    fn eq_null_non_null() {
        assert_ne!(SqlInt64::NULL, SqlInt64::new(0));
    }

    // ── T041: Tests for Hash ────────────────────────────────────────────────

    #[test]
    fn hash_equal_values() {
        assert_eq!(hash_of(&SqlInt64::new(42)), hash_of(&SqlInt64::new(42)));
    }

    #[test]
    fn hash_null_consistent() {
        assert_eq!(hash_of(&SqlInt64::NULL), hash_of(&SqlInt64::NULL));
    }

    // ── T042: Tests for PartialOrd / Ord ────────────────────────────────────

    #[test]
    fn ord_null_less_than_any() {
        assert!(SqlInt64::NULL < SqlInt64::new(i64::MIN));
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlInt64::MIN_VALUE < SqlInt64::MAX_VALUE);
    }

    #[test]
    fn ord_negative_less_than_positive() {
        assert!(SqlInt64::new(-1) < SqlInt64::new(1));
    }

    #[test]
    fn ord_equal_values() {
        assert_eq!(SqlInt64::new(42).cmp(&SqlInt64::new(42)), Ordering::Equal);
    }

    #[test]
    fn ord_null_null_equal() {
        assert_eq!(SqlInt64::NULL.cmp(&SqlInt64::NULL), Ordering::Equal);
    }
}
