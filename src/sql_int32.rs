// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// ── T001: SqlInt32 module ─────────────────────────────────────────────────────

//! `SqlInt32` — a signed 32-bit integer with SQL NULL support, equivalent to
//! C# `System.Data.SqlTypes.SqlInt32` / SQL Server `INT`.
//!
//! Uses `Option<i32>` internally: `None` = SQL NULL, `Some(v)` = a value.
//! All arithmetic returns `Result<SqlInt32, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_int16::SqlInt16;
use crate::sql_string::SqlString;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A signed 32-bit integer (−2,147,483,648 to 2,147,483,647) with SQL NULL support,
/// equivalent to C# `System.Data.SqlTypes.SqlInt32` / SQL Server `INT`.
///
/// Uses `Option<i32>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// All arithmetic returns `Result<SqlInt32, SqlTypeError>` with overflow detection
/// using Rust's `checked_*` methods. Comparisons return `SqlBoolean` for
/// three-valued NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlInt32 {
    value: Option<i32>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors and accessors ────────────────────────────────────────

impl SqlInt32 {
    /// SQL NULL.
    pub const NULL: SqlInt32 = SqlInt32 { value: None };
    /// Zero (0).
    pub const ZERO: SqlInt32 = SqlInt32 { value: Some(0) };
    /// Minimum value (−2,147,483,648).
    pub const MIN_VALUE: SqlInt32 = SqlInt32 {
        value: Some(i32::MIN),
    };
    /// Maximum value (2,147,483,647).
    pub const MAX_VALUE: SqlInt32 = SqlInt32 {
        value: Some(i32::MAX),
    };

    /// Creates a new `SqlInt32` from an `i32` value.
    pub fn new(v: i32) -> Self {
        SqlInt32 { value: Some(v) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `i32`, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<i32, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<i32> ─────────────────────────────────────────────────────────

impl From<i32> for SqlInt32 {
    fn from(v: i32) -> Self {
        SqlInt32::new(v)
    }
}

// ── T016: Checked arithmetic (add, sub) ─────────────────────────────────────
// ── T017: Checked arithmetic (mul) ──────────────────────────────────────────
// ── T018: Checked arithmetic (div) ──────────────────────────────────────────
// ── T019: Checked arithmetic (rem) ──────────────────────────────────────────
// ── T020: Checked arithmetic (neg) ──────────────────────────────────────────

impl SqlInt32 {
    /// Checked addition. Returns `Err(Overflow)` if result overflows i32.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_add(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt32::NULL),
            (Some(a), Some(b)) => a
                .checked_add(b)
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if result overflows i32.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_sub(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt32::NULL),
            (Some(a), Some(b)) => a
                .checked_sub(b)
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if result overflows i32.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_mul(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt32::NULL),
            (Some(a), Some(b)) => a
                .checked_mul(b)
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked division. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE / -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_div(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt32::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_div(b)
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked remainder. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE % -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_rem(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt32::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_rem(b)
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked negation. Returns `Err(Overflow)` if value is MIN_VALUE.
    /// NULL propagation: if operand is NULL, returns `Ok(SqlInt32::NULL)`.
    pub fn checked_neg(self) -> Result<SqlInt32, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt32::NULL),
            Some(v) => v
                .checked_neg()
                .map(SqlInt32::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }
}

// ── T021: Operator traits ───────────────────────────────────────────────────

impl Add for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn add(self, rhs: SqlInt32) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn sub(self, rhs: SqlInt32) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Mul for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn mul(self, rhs: SqlInt32) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Div for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn div(self, rhs: SqlInt32) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Rem for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn rem(self, rhs: SqlInt32) -> Self::Output {
        self.checked_rem(rhs)
    }
}

impl Neg for SqlInt32 {
    type Output = Result<SqlInt32, SqlTypeError>;

    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

// ── T024: Bitwise operations ────────────────────────────────────────────────
// ── T025: Not / ones_complement ─────────────────────────────────────────────

impl SqlInt32 {
    /// Returns the ones' complement (~value). NULL → NULL.
    pub fn ones_complement(self) -> SqlInt32 {
        match self.value {
            None => SqlInt32::NULL,
            Some(v) => SqlInt32::new(!v),
        }
    }
}

impl BitAnd for SqlInt32 {
    type Output = SqlInt32;

    fn bitand(self, rhs: SqlInt32) -> SqlInt32 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt32::new(a & b),
            _ => SqlInt32::NULL,
        }
    }
}

impl BitOr for SqlInt32 {
    type Output = SqlInt32;

    fn bitor(self, rhs: SqlInt32) -> SqlInt32 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt32::new(a | b),
            _ => SqlInt32::NULL,
        }
    }
}

impl BitXor for SqlInt32 {
    type Output = SqlInt32;

    fn bitxor(self, rhs: SqlInt32) -> SqlInt32 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt32::new(a ^ b),
            _ => SqlInt32::NULL,
        }
    }
}

impl Not for SqlInt32 {
    type Output = SqlInt32;

    fn not(self) -> SqlInt32 {
        self.ones_complement()
    }
}

// ── T027: SQL comparison methods ────────────────────────────────────────────

impl SqlInt32 {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt32) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T041: PartialEq, Eq, Hash ───────────────────────────────────────────────
// ── T042: PartialOrd, Ord ───────────────────────────────────────────────────

impl PartialEq for SqlInt32 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlInt32 {}

impl Hash for SqlInt32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NULL hashes as 0i32 for consistency
        match self.value {
            Some(v) => v.hash(state),
            None => 0i32.hash(state),
        }
    }
}

impl PartialOrd for SqlInt32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlInt32 {
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

impl fmt::Display for SqlInt32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "Null"),
        }
    }
}

impl FromStr for SqlInt32 {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SqlTypeError::ParseError(
                "Cannot parse empty string as SqlInt32".to_string(),
            ));
        }

        // Check for "null" (case-insensitive)
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlInt32::NULL);
        }

        // Parse as i32
        match trimmed.parse::<i32>() {
            Ok(v) => Ok(SqlInt32::new(v)),
            Err(e) => Err(SqlTypeError::ParseError(format!(
                "Cannot parse '{s}' as SqlInt32: {e}"
            ))),
        }
    }
}

// ── T035: From<SqlBoolean> ──────────────────────────────────────────────────
// ── T036: to_sql_int16 ─────────────────────────────────────────────────────
// ── T037: to_sql_byte ──────────────────────────────────────────────────────

impl From<SqlBoolean> for SqlInt32 {
    fn from(b: SqlBoolean) -> Self {
        if b.is_null() {
            SqlInt32::NULL
        } else {
            match b.value() {
                Ok(true) => SqlInt32::new(1),
                Ok(false) => SqlInt32::new(0),
                Err(_) => SqlInt32::NULL,
            }
        }
    }
}

impl From<SqlByte> for SqlInt32 {
    fn from(b: SqlByte) -> Self {
        if b.is_null() {
            SqlInt32::NULL
        } else {
            match b.value() {
                Ok(v) => SqlInt32::new(i32::from(v)),
                Err(_) => SqlInt32::NULL,
            }
        }
    }
}

impl From<SqlInt16> for SqlInt32 {
    fn from(s: SqlInt16) -> Self {
        if s.is_null() {
            SqlInt32::NULL
        } else {
            match s.value() {
                Ok(v) => SqlInt32::new(i32::from(v)),
                Err(_) => SqlInt32::NULL,
            }
        }
    }
}

impl SqlInt32 {
    /// Converts to `SqlBoolean`: NULL→NULL, zero→FALSE, non-zero→TRUE.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// Converts to `SqlInt16`: NULL→NULL, otherwise checks range −32,768..=32,767.
    /// Returns `Err(Overflow)` if value is outside i16 range.
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt16::NULL),
            Some(v) => {
                if v < i16::MIN as i32 || v > i16::MAX as i32 {
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

impl SqlInt32 {
    /// Converts to `SqlString` via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString {
        if self.is_null() {
            SqlString::NULL
        } else {
            SqlString::new(&format!("{self}"))
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    fn hash_of(val: &SqlInt32) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ── T007: Tests for new(), is_null(), value() ───────────────────────────

    #[test]
    fn new_positive_value() {
        let v = SqlInt32::new(100_000);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 100_000);
    }

    #[test]
    fn new_negative_value() {
        let v = SqlInt32::new(-200_000);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), -200_000);
    }

    #[test]
    fn new_zero() {
        let v = SqlInt32::new(0);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn null_access_returns_err() {
        let v = SqlInt32::NULL;
        assert!(v.is_null());
        assert!(matches!(v.value(), Err(SqlTypeError::NullValue)));
    }

    // ── T008: Tests for constants ───────────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlInt32::NULL.is_null());
    }

    #[test]
    fn zero_value() {
        assert_eq!(SqlInt32::ZERO.value().unwrap(), 0);
    }

    #[test]
    fn min_value() {
        assert_eq!(SqlInt32::MIN_VALUE.value().unwrap(), -2_147_483_648);
    }

    #[test]
    fn max_value() {
        assert_eq!(SqlInt32::MAX_VALUE.value().unwrap(), 2_147_483_647);
    }

    // ── T009: Tests for From<i32> ───────────────────────────────────────────

    #[test]
    fn from_i32_positive() {
        let v: SqlInt32 = 42i32.into();
        assert_eq!(v.value().unwrap(), 42);
    }

    #[test]
    fn from_i32_min() {
        let v = SqlInt32::from(i32::MIN);
        assert_eq!(v.value().unwrap(), i32::MIN);
    }

    #[test]
    fn from_i32_max() {
        let v = SqlInt32::from(i32::MAX);
        assert_eq!(v.value().unwrap(), i32::MAX);
    }

    // ── T010: Tests for checked_add ─────────────────────────────────────────

    #[test]
    fn add_normal() {
        let result = SqlInt32::new(100).checked_add(SqlInt32::new(200));
        assert_eq!(result.unwrap().value().unwrap(), 300);
    }

    #[test]
    fn add_overflow() {
        let result = SqlInt32::new(i32::MAX).checked_add(SqlInt32::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_underflow() {
        let result = SqlInt32::new(i32::MIN).checked_add(SqlInt32::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn add_null_lhs() {
        let result = SqlInt32::NULL.checked_add(SqlInt32::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_null_rhs() {
        let result = SqlInt32::new(1).checked_add(SqlInt32::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn add_operator() {
        let result = SqlInt32::new(100) + SqlInt32::new(200);
        assert_eq!(result.unwrap().value().unwrap(), 300);
    }

    // ── T011: Tests for checked_sub ─────────────────────────────────────────

    #[test]
    fn sub_normal() {
        let result = SqlInt32::new(300).checked_sub(SqlInt32::new(100));
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn sub_overflow() {
        let result = SqlInt32::new(i32::MIN).checked_sub(SqlInt32::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn sub_null_propagation() {
        let result = SqlInt32::NULL.checked_sub(SqlInt32::new(1));
        assert!(result.unwrap().is_null());
        let result2 = SqlInt32::new(1).checked_sub(SqlInt32::NULL);
        assert!(result2.unwrap().is_null());
    }

    #[test]
    fn sub_operator() {
        let result = SqlInt32::new(300) - SqlInt32::new(100);
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    // ── T012: Tests for checked_mul ─────────────────────────────────────────

    #[test]
    fn mul_normal() {
        let result = SqlInt32::new(100).checked_mul(SqlInt32::new(200));
        assert_eq!(result.unwrap().value().unwrap(), 20_000);
    }

    #[test]
    fn mul_overflow() {
        let result = SqlInt32::new(100_000).checked_mul(SqlInt32::new(100_000));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn mul_null_propagation() {
        let result = SqlInt32::NULL.checked_mul(SqlInt32::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn mul_operator() {
        let result = SqlInt32::new(10) * SqlInt32::new(20);
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    // ── T013: Tests for checked_div ─────────────────────────────────────────

    #[test]
    fn div_normal() {
        let result = SqlInt32::new(100).checked_div(SqlInt32::new(10));
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    #[test]
    fn div_by_zero() {
        let result = SqlInt32::new(10).checked_div(SqlInt32::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn div_min_by_neg_one() {
        let result = SqlInt32::new(i32::MIN).checked_div(SqlInt32::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn div_null_propagation() {
        let result = SqlInt32::NULL.checked_div(SqlInt32::new(1));
        assert!(result.unwrap().is_null());
        let result2 = SqlInt32::new(1).checked_div(SqlInt32::NULL);
        assert!(result2.unwrap().is_null());
    }

    #[test]
    fn div_operator() {
        let result = SqlInt32::new(100) / SqlInt32::new(10);
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    // ── T014: Tests for checked_rem ─────────────────────────────────────────

    #[test]
    fn rem_normal() {
        let result = SqlInt32::new(7).checked_rem(SqlInt32::new(3));
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    #[test]
    fn rem_by_zero() {
        let result = SqlInt32::new(10).checked_rem(SqlInt32::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn rem_min_by_neg_one() {
        let result = SqlInt32::new(i32::MIN).checked_rem(SqlInt32::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn rem_null_propagation() {
        let result = SqlInt32::NULL.checked_rem(SqlInt32::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn rem_operator() {
        let result = SqlInt32::new(7) % SqlInt32::new(3);
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    // ── T015: Tests for checked_neg ─────────────────────────────────────────

    #[test]
    fn neg_normal() {
        let result = SqlInt32::new(42).checked_neg();
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn neg_min_value_overflow() {
        let result = SqlInt32::new(i32::MIN).checked_neg();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn neg_null_returns_null() {
        let result = SqlInt32::NULL.checked_neg();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn neg_operator() {
        let result = -SqlInt32::new(42);
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn neg_operator_min_overflow() {
        let result = -SqlInt32::new(i32::MIN);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T022: Tests for BitAnd, BitOr, BitXor ───────────────────────────────

    #[test]
    fn bitand_normal() {
        let result = SqlInt32::new(0xFF00) & SqlInt32::new(0x0FF0);
        assert_eq!(result.value().unwrap(), 0x0F00);
    }

    #[test]
    fn bitor_normal() {
        let result = SqlInt32::new(0xFF00) | SqlInt32::new(0x00FF);
        assert_eq!(result.value().unwrap(), 0xFFFF);
    }

    #[test]
    fn bitxor_normal() {
        let result = SqlInt32::new(0xFF) ^ SqlInt32::new(0x0F);
        assert_eq!(result.value().unwrap(), 0xF0);
    }

    #[test]
    fn bitand_negative_values() {
        let result = SqlInt32::new(-1) & SqlInt32::new(0xFF);
        assert_eq!(result.value().unwrap(), 0xFF);
    }

    #[test]
    fn bitand_null_propagation() {
        let result = SqlInt32::NULL & SqlInt32::new(0xFF);
        assert!(result.is_null());
    }

    #[test]
    fn bitor_null_propagation() {
        let result = SqlInt32::new(0xFF) | SqlInt32::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn bitxor_null_propagation() {
        let result = SqlInt32::NULL ^ SqlInt32::NULL;
        assert!(result.is_null());
    }

    // ── T023: Tests for Not / ones_complement ───────────────────────────────

    #[test]
    fn not_zero() {
        let result = !SqlInt32::new(0);
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn not_neg_one() {
        let result = !SqlInt32::new(-1);
        assert_eq!(result.value().unwrap(), 0);
    }

    #[test]
    fn ones_complement_method() {
        let result = SqlInt32::new(0).ones_complement();
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn not_null_returns_null() {
        let result = !SqlInt32::NULL;
        assert!(result.is_null());
    }

    // ── T026: Tests for SQL comparison methods ──────────────────────────────

    #[test]
    fn sql_equals_true() {
        let cmp = SqlInt32::new(10).sql_equals(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_false() {
        let cmp = SqlInt32::new(10).sql_equals(&SqlInt32::new(20));
        assert_eq!(cmp, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_not_equals_true() {
        let cmp = SqlInt32::new(10).sql_not_equals(&SqlInt32::new(20));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_true() {
        let cmp = SqlInt32::new(10).sql_less_than(&SqlInt32::new(20));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_false() {
        let cmp = SqlInt32::new(20).sql_less_than(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_true() {
        let cmp = SqlInt32::new(20).sql_greater_than(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let cmp = SqlInt32::new(10).sql_less_than_or_equal(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_less() {
        let cmp = SqlInt32::new(10).sql_less_than_or_equal(&SqlInt32::new(20));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let cmp = SqlInt32::new(10).sql_greater_than_or_equal(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_greater() {
        let cmp = SqlInt32::new(20).sql_greater_than_or_equal(&SqlInt32::new(10));
        assert_eq!(cmp, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_null_lhs() {
        let cmp = SqlInt32::NULL.sql_equals(&SqlInt32::new(10));
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_equals_null_rhs() {
        let cmp = SqlInt32::new(10).sql_equals(&SqlInt32::NULL);
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_less_than_null() {
        let cmp = SqlInt32::new(10).sql_less_than(&SqlInt32::NULL);
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_greater_than_null() {
        let cmp = SqlInt32::NULL.sql_greater_than(&SqlInt32::new(10));
        assert!(cmp.is_null());
    }

    #[test]
    fn sql_not_equals_null() {
        let cmp = SqlInt32::NULL.sql_not_equals(&SqlInt32::NULL);
        assert!(cmp.is_null());
    }

    // ── T028: Tests for Display ─────────────────────────────────────────────

    #[test]
    fn display_positive() {
        assert_eq!(format!("{}", SqlInt32::new(42)), "42");
    }

    #[test]
    fn display_negative() {
        assert_eq!(format!("{}", SqlInt32::new(-100)), "-100");
    }

    #[test]
    fn display_zero() {
        assert_eq!(format!("{}", SqlInt32::ZERO), "0");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlInt32::NULL), "Null");
    }

    // ── T029: Tests for FromStr ─────────────────────────────────────────────

    #[test]
    fn parse_valid_positive() {
        let v: SqlInt32 = "42".parse().unwrap();
        assert_eq!(v.value().unwrap(), 42);
    }

    #[test]
    fn parse_valid_negative() {
        let v: SqlInt32 = "-100".parse().unwrap();
        assert_eq!(v.value().unwrap(), -100);
    }

    #[test]
    fn parse_null_string() {
        let v: SqlInt32 = "Null".parse().unwrap();
        assert!(v.is_null());
    }

    #[test]
    fn parse_null_case_insensitive() {
        let v: SqlInt32 = "null".parse().unwrap();
        assert!(v.is_null());
        let v2: SqlInt32 = "NULL".parse().unwrap();
        assert!(v2.is_null());
    }

    #[test]
    fn parse_out_of_range() {
        let result = "99999999999".parse::<SqlInt32>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_non_numeric() {
        let result = "abc".parse::<SqlInt32>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_empty_string() {
        let result = "".parse::<SqlInt32>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn display_fromstr_roundtrip() {
        let original = SqlInt32::new(12345);
        let s = format!("{original}");
        let parsed: SqlInt32 = s.parse().unwrap();
        assert_eq!(original, parsed);
    }

    // ── T032: Tests for From<SqlBoolean> ────────────────────────────────────

    #[test]
    fn from_sql_boolean_true() {
        let v = SqlInt32::from(SqlBoolean::TRUE);
        assert_eq!(v.value().unwrap(), 1);
    }

    #[test]
    fn from_sql_boolean_false() {
        let v = SqlInt32::from(SqlBoolean::FALSE);
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_boolean_null() {
        let v = SqlInt32::from(SqlBoolean::NULL);
        assert!(v.is_null());
    }

    // ── T033: Tests for to_sql_int16 ────────────────────────────────────────

    #[test]
    fn to_sql_int16_in_range() {
        let result = SqlInt32::new(100).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), 100);
    }

    #[test]
    fn to_sql_int16_overflow() {
        let result = SqlInt32::new(100_000).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_underflow() {
        let result = SqlInt32::new(-100_000).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_null() {
        let result = SqlInt32::NULL.to_sql_int16();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn to_sql_int16_boundary_max() {
        let result = SqlInt32::new(i16::MAX as i32).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), i16::MAX);
    }

    #[test]
    fn to_sql_int16_boundary_min() {
        let result = SqlInt32::new(i16::MIN as i32).to_sql_int16();
        assert_eq!(result.unwrap().value().unwrap(), i16::MIN);
    }

    #[test]
    fn to_sql_int16_just_over_max() {
        let result = SqlInt32::new(i16::MAX as i32 + 1).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_just_under_min() {
        let result = SqlInt32::new(i16::MIN as i32 - 1).to_sql_int16();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T034: Tests for to_sql_byte ─────────────────────────────────────────

    #[test]
    fn to_sql_byte_in_range() {
        let result = SqlInt32::new(200).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn to_sql_byte_overflow() {
        let result = SqlInt32::new(300).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_negative() {
        let result = SqlInt32::new(-1).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_null() {
        let result = SqlInt32::NULL.to_sql_byte();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn to_sql_byte_zero() {
        let result = SqlInt32::new(0).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 0);
    }

    #[test]
    fn to_sql_byte_max_valid() {
        let result = SqlInt32::new(255).to_sql_byte();
        assert_eq!(result.unwrap().value().unwrap(), 255);
    }

    #[test]
    fn to_sql_byte_just_over_max() {
        let result = SqlInt32::new(256).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T038: Tests for PartialEq / Eq ──────────────────────────────────────

    #[test]
    fn eq_same_value() {
        assert_eq!(SqlInt32::new(42), SqlInt32::new(42));
    }

    #[test]
    fn eq_different_value() {
        assert_ne!(SqlInt32::new(42), SqlInt32::new(43));
    }

    #[test]
    fn eq_null_null() {
        assert_eq!(SqlInt32::NULL, SqlInt32::NULL);
    }

    #[test]
    fn eq_null_non_null() {
        assert_ne!(SqlInt32::NULL, SqlInt32::new(0));
    }

    // ── T039: Tests for Hash ────────────────────────────────────────────────

    #[test]
    fn hash_equal_values() {
        assert_eq!(hash_of(&SqlInt32::new(42)), hash_of(&SqlInt32::new(42)));
    }

    #[test]
    fn hash_null_consistent() {
        assert_eq!(hash_of(&SqlInt32::NULL), hash_of(&SqlInt32::NULL));
    }

    // ── T040: Tests for PartialOrd / Ord ────────────────────────────────────

    #[test]
    fn ord_null_less_than_any() {
        assert!(SqlInt32::NULL < SqlInt32::new(i32::MIN));
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlInt32::MIN_VALUE < SqlInt32::MAX_VALUE);
    }

    #[test]
    fn ord_negative_less_than_positive() {
        assert!(SqlInt32::new(-1) < SqlInt32::new(1));
    }

    #[test]
    fn ord_equal_values() {
        assert_eq!(SqlInt32::new(42).cmp(&SqlInt32::new(42)), Ordering::Equal);
    }

    #[test]
    fn ord_null_null_equal() {
        assert_eq!(SqlInt32::NULL.cmp(&SqlInt32::NULL), Ordering::Equal);
    }

    // ── From<SqlByte> for SqlInt32 tests ─────────────────────────────────────

    #[test]
    fn from_sql_byte_normal() {
        let b = SqlByte::new(100);
        let i: SqlInt32 = SqlInt32::from(b);
        assert_eq!(i.value().unwrap(), 100);
    }

    #[test]
    fn from_sql_byte_zero() {
        let b = SqlByte::new(0);
        let i: SqlInt32 = SqlInt32::from(b);
        assert_eq!(i.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_byte_max() {
        let b = SqlByte::new(u8::MAX);
        let i: SqlInt32 = SqlInt32::from(b);
        assert_eq!(i.value().unwrap(), 255);
    }

    #[test]
    fn from_sql_byte_null() {
        let b = SqlByte::NULL;
        let i: SqlInt32 = SqlInt32::from(b);
        assert!(i.is_null());
    }

    // ── From<SqlInt16> for SqlInt32 tests ────────────────────────────────────

    #[test]
    fn from_sql_int16_normal() {
        let s = SqlInt16::new(1000);
        let i: SqlInt32 = SqlInt32::from(s);
        assert_eq!(i.value().unwrap(), 1000);
    }

    #[test]
    fn from_sql_int16_zero() {
        let s = SqlInt16::new(0);
        let i: SqlInt32 = SqlInt32::from(s);
        assert_eq!(i.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_int16_max() {
        let s = SqlInt16::new(i16::MAX);
        let i: SqlInt32 = SqlInt32::from(s);
        assert_eq!(i.value().unwrap(), i16::MAX as i32);
    }

    #[test]
    fn from_sql_int16_min() {
        let s = SqlInt16::new(i16::MIN);
        let i: SqlInt32 = SqlInt32::from(s);
        assert_eq!(i.value().unwrap(), i16::MIN as i32);
    }

    #[test]
    fn from_sql_int16_null() {
        let s = SqlInt16::NULL;
        let i: SqlInt32 = SqlInt32::from(s);
        assert!(i.is_null());
    }

    // ── to_sql_boolean() tests ──────────────────────────────────────────────

    #[test]
    fn to_sql_boolean_zero_is_false() {
        let v = SqlInt32::new(0);
        let b = v.to_sql_boolean();
        assert!(!b.is_null());
        assert_eq!(b.value().unwrap(), false);
    }

    #[test]
    fn to_sql_boolean_positive_is_true() {
        let v = SqlInt32::new(42);
        let b = v.to_sql_boolean();
        assert_eq!(b.value().unwrap(), true);
    }

    #[test]
    fn to_sql_boolean_negative_is_true() {
        let v = SqlInt32::new(-1);
        let b = v.to_sql_boolean();
        assert_eq!(b.value().unwrap(), true);
    }

    #[test]
    fn to_sql_boolean_max_is_true() {
        let v = SqlInt32::new(i32::MAX);
        let b = v.to_sql_boolean();
        assert_eq!(b.value().unwrap(), true);
    }

    #[test]
    fn to_sql_boolean_null_is_null() {
        let v = SqlInt32::NULL;
        let b = v.to_sql_boolean();
        assert!(b.is_null());
    }

    // ── to_sql_string() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_string_positive() {
        let s = SqlInt32::new(123456).to_sql_string();
        assert_eq!(s.value().unwrap(), "123456");
    }

    #[test]
    fn to_sql_string_negative() {
        let s = SqlInt32::new(-789).to_sql_string();
        assert_eq!(s.value().unwrap(), "-789");
    }

    #[test]
    fn to_sql_string_zero() {
        let s = SqlInt32::new(0).to_sql_string();
        assert_eq!(s.value().unwrap(), "0");
    }

    #[test]
    fn to_sql_string_null() {
        let s = SqlInt32::NULL.to_sql_string();
        assert!(s.is_null());
    }
}
