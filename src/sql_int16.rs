// ── T001: SqlInt16 module ─────────────────────────────────────────────────────

//! `SqlInt16` — a signed 16-bit integer with SQL NULL support, equivalent to
//! C# `System.Data.SqlTypes.SqlInt16` / SQL Server `SMALLINT`.
//!
//! Uses `Option<i16>` internally: `None` = SQL NULL, `Some(v)` = a value.
//! All arithmetic returns `Result<SqlInt16, SqlTypeError>` with overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub};
use std::str::FromStr;

// ── T003: Struct definition ─────────────────────────────────────────────────

/// A signed 16-bit integer (−32,768 to 32,767) with SQL NULL support, equivalent
/// to C# `System.Data.SqlTypes.SqlInt16` / SQL Server `SMALLINT`.
///
/// Uses `Option<i16>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// All arithmetic returns `Result<SqlInt16, SqlTypeError>` with overflow detection
/// using Rust's `checked_*` methods. Comparisons return `SqlBoolean` for
/// three-valued NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlInt16 {
    value: Option<i16>,
}

// ── T004: Constants ─────────────────────────────────────────────────────────
// ── T005: Constructors and accessors ────────────────────────────────────────

impl SqlInt16 {
    /// SQL NULL.
    pub const NULL: SqlInt16 = SqlInt16 { value: None };
    /// Zero (0).
    pub const ZERO: SqlInt16 = SqlInt16 { value: Some(0) };
    /// Minimum value (−32,768).
    pub const MIN_VALUE: SqlInt16 = SqlInt16 {
        value: Some(i16::MIN),
    };
    /// Maximum value (32,767).
    pub const MAX_VALUE: SqlInt16 = SqlInt16 {
        value: Some(i16::MAX),
    };

    /// Creates a new `SqlInt16` from an `i16` value.
    pub fn new(v: i16) -> Self {
        SqlInt16 { value: Some(v) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `i16`, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<i16, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<i16> ─────────────────────────────────────────────────────────

impl From<i16> for SqlInt16 {
    fn from(v: i16) -> Self {
        SqlInt16::new(v)
    }
}

// ── T016: Checked arithmetic (add, sub) ─────────────────────────────────────
// ── T017: Checked arithmetic (mul) ──────────────────────────────────────────
// ── T018: Checked arithmetic (div) ──────────────────────────────────────────
// ── T019: Checked arithmetic (rem) ──────────────────────────────────────────
// ── T020: Checked arithmetic (neg) ──────────────────────────────────────────

impl SqlInt16 {
    /// Checked addition. Returns `Err(Overflow)` if result overflows i16.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_add(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt16::NULL),
            (Some(a), Some(b)) => a
                .checked_add(b)
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if result overflows i16.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_sub(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt16::NULL),
            (Some(a), Some(b)) => a
                .checked_sub(b)
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if result overflows i16.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_mul(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt16::NULL),
            (Some(a), Some(b)) => a
                .checked_mul(b)
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked division. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE / -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_div(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt16::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_div(b)
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked remainder. Returns `Err(DivideByZero)` if divisor is zero,
    /// `Err(Overflow)` if MIN_VALUE % -1.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_rem(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlInt16::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => a
                .checked_rem(b)
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }

    /// Checked negation. Returns `Err(Overflow)` if value is MIN_VALUE.
    /// NULL propagation: if operand is NULL, returns `Ok(SqlInt16::NULL)`.
    pub fn checked_neg(self) -> Result<SqlInt16, SqlTypeError> {
        match self.value {
            None => Ok(SqlInt16::NULL),
            Some(v) => v
                .checked_neg()
                .map(SqlInt16::new)
                .ok_or(SqlTypeError::Overflow),
        }
    }
}

// ── T021: Operator traits ───────────────────────────────────────────────────

impl Add for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn add(self, rhs: SqlInt16) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn sub(self, rhs: SqlInt16) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Mul for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn mul(self, rhs: SqlInt16) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Div for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn div(self, rhs: SqlInt16) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Rem for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn rem(self, rhs: SqlInt16) -> Self::Output {
        self.checked_rem(rhs)
    }
}

impl Neg for SqlInt16 {
    type Output = Result<SqlInt16, SqlTypeError>;

    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

// ── T025: Bitwise operations ────────────────────────────────────────────────
// ── T026: Not / ones_complement ─────────────────────────────────────────────

impl SqlInt16 {
    /// Returns the ones' complement (~value). NULL → NULL.
    pub fn ones_complement(self) -> SqlInt16 {
        match self.value {
            None => SqlInt16::NULL,
            Some(v) => SqlInt16::new(!v),
        }
    }
}

impl BitAnd for SqlInt16 {
    type Output = SqlInt16;

    fn bitand(self, rhs: SqlInt16) -> SqlInt16 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt16::new(a & b),
            _ => SqlInt16::NULL,
        }
    }
}

impl BitOr for SqlInt16 {
    type Output = SqlInt16;

    fn bitor(self, rhs: SqlInt16) -> SqlInt16 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt16::new(a | b),
            _ => SqlInt16::NULL,
        }
    }
}

impl BitXor for SqlInt16 {
    type Output = SqlInt16;

    fn bitxor(self, rhs: SqlInt16) -> SqlInt16 {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlInt16::new(a ^ b),
            _ => SqlInt16::NULL,
        }
    }
}

impl Not for SqlInt16 {
    type Output = SqlInt16;

    fn not(self) -> SqlInt16 {
        self.ones_complement()
    }
}

// ── T027: SQL comparison methods ────────────────────────────────────────────

impl SqlInt16 {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt16) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T034: PartialEq, Eq, Hash ───────────────────────────────────────────────
// ── T035: PartialOrd, Ord ───────────────────────────────────────────────────

impl PartialEq for SqlInt16 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlInt16 {}

impl Hash for SqlInt16 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NULL hashes as 0i16 for consistency
        match self.value {
            Some(v) => v.hash(state),
            None => 0i16.hash(state),
        }
    }
}

impl PartialOrd for SqlInt16 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlInt16 {
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

// ── T036: Display ───────────────────────────────────────────────────────────
// ── T037: FromStr ───────────────────────────────────────────────────────────

impl fmt::Display for SqlInt16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "Null"),
        }
    }
}

impl FromStr for SqlInt16 {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SqlTypeError::ParseError(
                "Cannot parse empty string as SqlInt16".to_string(),
            ));
        }

        // Check for "null" (case-insensitive)
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlInt16::NULL);
        }

        // Parse as i16
        match trimmed.parse::<i16>() {
            Ok(v) => Ok(SqlInt16::new(v)),
            Err(e) => Err(SqlTypeError::ParseError(format!(
                "Cannot parse '{s}' as SqlInt16: {e}"
            ))),
        }
    }
}

// ── T038: Widening conversions (From<SqlBoolean>, From<SqlByte>) ────────────
// ── T039: Narrowing conversions (to_sql_boolean, to_sql_byte) ───────────────

impl From<SqlBoolean> for SqlInt16 {
    fn from(b: SqlBoolean) -> Self {
        if b.is_null() {
            SqlInt16::NULL
        } else {
            match b.value() {
                Ok(true) => SqlInt16::new(1),
                Ok(false) => SqlInt16::new(0),
                Err(_) => SqlInt16::NULL,
            }
        }
    }
}

impl From<SqlByte> for SqlInt16 {
    fn from(b: SqlByte) -> Self {
        if b.is_null() {
            SqlInt16::NULL
        } else {
            match b.value() {
                Ok(v) => SqlInt16::new(v as i16),
                Err(_) => SqlInt16::NULL,
            }
        }
    }
}

impl SqlInt16 {
    /// Converts to `SqlBoolean`: NULL→NULL, 0→FALSE, non-zero→TRUE.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// Converts to `SqlByte`: NULL→NULL, otherwise checks range 0–255.
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

    fn hash_of(val: &SqlInt16) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ── T007: Tests for new(), is_null(), value() ───────────────────────────

    #[test]
    fn new_positive_value() {
        let v = SqlInt16::new(1000);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 1000);
    }

    #[test]
    fn new_negative_value() {
        let v = SqlInt16::new(-500);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), -500);
    }

    #[test]
    fn new_zero() {
        let v = SqlInt16::new(0);
        assert!(!v.is_null());
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn null_access_returns_err() {
        let v = SqlInt16::NULL;
        assert!(v.is_null());
        assert!(matches!(v.value(), Err(SqlTypeError::NullValue)));
    }

    // ── T008: Tests for constants ───────────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlInt16::NULL.is_null());
    }

    #[test]
    fn zero_value() {
        assert_eq!(SqlInt16::ZERO.value().unwrap(), 0);
    }

    #[test]
    fn min_value() {
        assert_eq!(SqlInt16::MIN_VALUE.value().unwrap(), -32768);
        assert_eq!(SqlInt16::MIN_VALUE.value().unwrap(), i16::MIN);
    }

    #[test]
    fn max_value() {
        assert_eq!(SqlInt16::MAX_VALUE.value().unwrap(), 32767);
        assert_eq!(SqlInt16::MAX_VALUE.value().unwrap(), i16::MAX);
    }

    // ── T009: Tests for From<i16> ───────────────────────────────────────────

    #[test]
    fn from_i16_positive() {
        let v = SqlInt16::from(42);
        assert_eq!(v.value().unwrap(), 42);
    }

    #[test]
    fn from_i16_min() {
        let v = SqlInt16::from(i16::MIN);
        assert_eq!(v.value().unwrap(), i16::MIN);
    }

    #[test]
    fn from_i16_max() {
        let v = SqlInt16::from(i16::MAX);
        assert_eq!(v.value().unwrap(), i16::MAX);
    }

    #[test]
    fn from_i16_zero() {
        let v = SqlInt16::from(0i16);
        assert_eq!(v.value().unwrap(), 0);
    }

    // ── T010: Tests for checked_add ─────────────────────────────────────────

    #[test]
    fn checked_add_normal() {
        let result = SqlInt16::new(100).checked_add(SqlInt16::new(200));
        assert_eq!(result.unwrap().value().unwrap(), 300);
    }

    #[test]
    fn checked_add_negative() {
        let result = SqlInt16::new(-100).checked_add(SqlInt16::new(-200));
        assert_eq!(result.unwrap().value().unwrap(), -300);
    }

    #[test]
    fn checked_add_overflow_max_plus_one() {
        let result = SqlInt16::new(i16::MAX).checked_add(SqlInt16::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_add_underflow_min_minus_one() {
        let result = SqlInt16::new(i16::MIN).checked_add(SqlInt16::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_add_null_lhs() {
        let result = SqlInt16::NULL.checked_add(SqlInt16::new(1));
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn checked_add_null_rhs() {
        let result = SqlInt16::new(1).checked_add(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn checked_add_both_null() {
        let result = SqlInt16::NULL.checked_add(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
    }

    // ── T011: Tests for checked_sub ─────────────────────────────────────────

    #[test]
    fn checked_sub_normal() {
        let result = SqlInt16::new(300).checked_sub(SqlInt16::new(100));
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn checked_sub_overflow_min_minus_one() {
        let result = SqlInt16::new(i16::MIN).checked_sub(SqlInt16::new(1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_sub_overflow_max_direction() {
        let result = SqlInt16::new(i16::MAX).checked_sub(SqlInt16::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_sub_null_propagation() {
        let result = SqlInt16::new(1).checked_sub(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
        let result = SqlInt16::NULL.checked_sub(SqlInt16::new(1));
        assert!(result.unwrap().is_null());
    }

    // ── T012: Tests for checked_mul ─────────────────────────────────────────

    #[test]
    fn checked_mul_normal() {
        let result = SqlInt16::new(10).checked_mul(SqlInt16::new(20));
        assert_eq!(result.unwrap().value().unwrap(), 200);
    }

    #[test]
    fn checked_mul_overflow_positive() {
        let result = SqlInt16::new(100).checked_mul(SqlInt16::new(400));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_mul_overflow_negative() {
        let result = SqlInt16::new(-100).checked_mul(SqlInt16::new(400));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_mul_null_propagation() {
        let result = SqlInt16::new(1).checked_mul(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
        let result = SqlInt16::NULL.checked_mul(SqlInt16::new(1));
        assert!(result.unwrap().is_null());
    }

    // ── T013: Tests for checked_div ─────────────────────────────────────────

    #[test]
    fn checked_div_normal() {
        let result = SqlInt16::new(100).checked_div(SqlInt16::new(10));
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    #[test]
    fn checked_div_by_zero() {
        let result = SqlInt16::new(100).checked_div(SqlInt16::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn checked_div_min_by_neg_one() {
        let result = SqlInt16::new(i16::MIN).checked_div(SqlInt16::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_div_null_propagation() {
        let result = SqlInt16::new(1).checked_div(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
        let result = SqlInt16::NULL.checked_div(SqlInt16::new(1));
        assert!(result.unwrap().is_null());
    }

    // ── T014: Tests for checked_rem ─────────────────────────────────────────

    #[test]
    fn checked_rem_normal() {
        let result = SqlInt16::new(7).checked_rem(SqlInt16::new(3));
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    #[test]
    fn checked_rem_negative() {
        let result = SqlInt16::new(-7).checked_rem(SqlInt16::new(3));
        assert_eq!(result.unwrap().value().unwrap(), -1);
    }

    #[test]
    fn checked_rem_by_zero() {
        let result = SqlInt16::new(7).checked_rem(SqlInt16::new(0));
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn checked_rem_min_by_neg_one() {
        let result = SqlInt16::new(i16::MIN).checked_rem(SqlInt16::new(-1));
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_rem_null_propagation() {
        let result = SqlInt16::new(7).checked_rem(SqlInt16::NULL);
        assert!(result.unwrap().is_null());
        let result = SqlInt16::NULL.checked_rem(SqlInt16::new(3));
        assert!(result.unwrap().is_null());
    }

    // ── T015: Tests for checked_neg ─────────────────────────────────────────

    #[test]
    fn checked_neg_normal() {
        let result = SqlInt16::new(42).checked_neg();
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn checked_neg_negative() {
        let result = SqlInt16::new(-100).checked_neg();
        assert_eq!(result.unwrap().value().unwrap(), 100);
    }

    #[test]
    fn checked_neg_min_value_overflow() {
        let result = SqlInt16::new(i16::MIN).checked_neg();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn checked_neg_null_returns_null() {
        let result = SqlInt16::NULL.checked_neg();
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn checked_neg_zero() {
        let result = SqlInt16::new(0).checked_neg();
        assert_eq!(result.unwrap().value().unwrap(), 0);
    }

    // ── Operator trait tests ────────────────────────────────────────────────

    #[test]
    fn add_operator() {
        let result = SqlInt16::new(10) + SqlInt16::new(20);
        assert_eq!(result.unwrap().value().unwrap(), 30);
    }

    #[test]
    fn sub_operator() {
        let result = SqlInt16::new(30) - SqlInt16::new(10);
        assert_eq!(result.unwrap().value().unwrap(), 20);
    }

    #[test]
    fn mul_operator() {
        let result = SqlInt16::new(5) * SqlInt16::new(6);
        assert_eq!(result.unwrap().value().unwrap(), 30);
    }

    #[test]
    fn div_operator() {
        let result = SqlInt16::new(100) / SqlInt16::new(10);
        assert_eq!(result.unwrap().value().unwrap(), 10);
    }

    #[test]
    fn rem_operator() {
        let result = SqlInt16::new(7) % SqlInt16::new(3);
        assert_eq!(result.unwrap().value().unwrap(), 1);
    }

    #[test]
    fn neg_operator() {
        let result = -SqlInt16::new(42);
        assert_eq!(result.unwrap().value().unwrap(), -42);
    }

    #[test]
    fn neg_operator_overflow() {
        let result = -SqlInt16::new(i16::MIN);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── T022: Tests for BitAnd, BitOr, BitXor ──────────────────────────────

    #[test]
    fn bitand_normal() {
        let result = SqlInt16::new(0xFF) & SqlInt16::new(0x0F);
        assert_eq!(result.value().unwrap(), 0x0F);
    }

    #[test]
    fn bitor_normal() {
        let result = SqlInt16::new(0xF0) | SqlInt16::new(0x0F);
        assert_eq!(result.value().unwrap(), 0xFF);
    }

    #[test]
    fn bitxor_normal() {
        let result = SqlInt16::new(0xFF) ^ SqlInt16::new(0x0F);
        assert_eq!(result.value().unwrap(), 0xF0);
    }

    #[test]
    fn bitand_negative() {
        let result = SqlInt16::new(-1) & SqlInt16::new(0x00FF);
        assert_eq!(result.value().unwrap(), 0x00FF);
    }

    #[test]
    fn bitor_negative() {
        // -1 is all bits set in i16 (0xFFFF)
        let result = SqlInt16::new(-1) | SqlInt16::new(0);
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn bitxor_negative() {
        let result = SqlInt16::new(-1) ^ SqlInt16::new(-1);
        assert_eq!(result.value().unwrap(), 0);
    }

    #[test]
    fn bitand_null_propagation() {
        let result = SqlInt16::new(0xFF) & SqlInt16::NULL;
        assert!(result.is_null());
        let result = SqlInt16::NULL & SqlInt16::new(0xFF);
        assert!(result.is_null());
    }

    #[test]
    fn bitor_null_propagation() {
        let result = SqlInt16::new(42) | SqlInt16::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn bitxor_null_propagation() {
        let result = SqlInt16::NULL ^ SqlInt16::new(42);
        assert!(result.is_null());
    }

    // ── T023: Tests for Not (ones complement) ───────────────────────────────

    #[test]
    fn not_zero() {
        let result = !SqlInt16::new(0);
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn not_min_equals_max() {
        let result = !SqlInt16::MIN_VALUE;
        assert_eq!(result.value().unwrap(), i16::MAX);
    }

    #[test]
    fn not_max_equals_min() {
        let result = !SqlInt16::MAX_VALUE;
        assert_eq!(result.value().unwrap(), i16::MIN);
    }

    #[test]
    fn not_null_returns_null() {
        let result = !SqlInt16::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn ones_complement_method() {
        let result = SqlInt16::new(0).ones_complement();
        assert_eq!(result.value().unwrap(), -1);
    }

    #[test]
    fn ones_complement_null() {
        let result = SqlInt16::NULL.ones_complement();
        assert!(result.is_null());
    }

    // ── T024: Tests for SQL comparison methods ──────────────────────────────

    #[test]
    fn sql_equals_true() {
        let result = SqlInt16::new(42).sql_equals(&SqlInt16::new(42));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_false() {
        let result = SqlInt16::new(42).sql_equals(&SqlInt16::new(43));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_equals_null_lhs() {
        let result = SqlInt16::NULL.sql_equals(&SqlInt16::new(42));
        assert!(result.is_null());
    }

    #[test]
    fn sql_equals_null_rhs() {
        let result = SqlInt16::new(42).sql_equals(&SqlInt16::NULL);
        assert!(result.is_null());
    }

    #[test]
    fn sql_equals_both_null() {
        let result = SqlInt16::NULL.sql_equals(&SqlInt16::NULL);
        assert!(result.is_null());
    }

    #[test]
    fn sql_not_equals_true() {
        let result = SqlInt16::new(42).sql_not_equals(&SqlInt16::new(43));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_not_equals_false() {
        let result = SqlInt16::new(42).sql_not_equals(&SqlInt16::new(42));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_not_equals_null() {
        let result = SqlInt16::new(42).sql_not_equals(&SqlInt16::NULL);
        assert!(result.is_null());
    }

    #[test]
    fn sql_less_than_true() {
        let result = SqlInt16::new(10).sql_less_than(&SqlInt16::new(20));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_false() {
        let result = SqlInt16::new(20).sql_less_than(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_equal_values() {
        let result = SqlInt16::new(10).sql_less_than(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_null() {
        let result = SqlInt16::new(10).sql_less_than(&SqlInt16::NULL);
        assert!(result.is_null());
    }

    #[test]
    fn sql_greater_than_true() {
        let result = SqlInt16::new(20).sql_greater_than(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_false() {
        let result = SqlInt16::new(10).sql_greater_than(&SqlInt16::new(20));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_null() {
        let result = SqlInt16::NULL.sql_greater_than(&SqlInt16::new(10));
        assert!(result.is_null());
    }

    #[test]
    fn sql_less_than_or_equal_true_less() {
        let result = SqlInt16::new(10).sql_less_than_or_equal(&SqlInt16::new(20));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_true_equal() {
        let result = SqlInt16::new(10).sql_less_than_or_equal(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_false() {
        let result = SqlInt16::new(20).sql_less_than_or_equal(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_or_equal_null() {
        let result = SqlInt16::new(10).sql_less_than_or_equal(&SqlInt16::NULL);
        assert!(result.is_null());
    }

    #[test]
    fn sql_greater_than_or_equal_true_greater() {
        let result = SqlInt16::new(20).sql_greater_than_or_equal(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_true_equal() {
        let result = SqlInt16::new(10).sql_greater_than_or_equal(&SqlInt16::new(10));
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_false() {
        let result = SqlInt16::new(10).sql_greater_than_or_equal(&SqlInt16::new(20));
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_or_equal_null() {
        let result = SqlInt16::NULL.sql_greater_than_or_equal(&SqlInt16::new(10));
        assert!(result.is_null());
    }

    // ── T028: Tests for PartialEq / Eq ──────────────────────────────────────

    #[test]
    fn partial_eq_same_value() {
        assert_eq!(SqlInt16::new(42), SqlInt16::new(42));
    }

    #[test]
    fn partial_eq_different_values() {
        assert_ne!(SqlInt16::new(42), SqlInt16::new(43));
    }

    #[test]
    fn partial_eq_null_null() {
        // Rust semantics: NULL == NULL in PartialEq
        assert_eq!(SqlInt16::NULL, SqlInt16::NULL);
    }

    #[test]
    fn partial_eq_null_value() {
        assert_ne!(SqlInt16::NULL, SqlInt16::new(42));
    }

    #[test]
    fn partial_eq_negative() {
        assert_eq!(SqlInt16::new(-100), SqlInt16::new(-100));
        assert_ne!(SqlInt16::new(-100), SqlInt16::new(100));
    }

    // ── T029: Tests for Hash ────────────────────────────────────────────────

    #[test]
    fn hash_equal_values_hash_equal() {
        assert_eq!(hash_of(&SqlInt16::new(42)), hash_of(&SqlInt16::new(42)));
    }

    #[test]
    fn hash_different_values_may_differ() {
        // Not guaranteed, but very likely for small distinct values
        assert_ne!(hash_of(&SqlInt16::new(1)), hash_of(&SqlInt16::new(2)));
    }

    #[test]
    fn hash_null_consistent() {
        assert_eq!(hash_of(&SqlInt16::NULL), hash_of(&SqlInt16::NULL));
    }

    // ── T030: Tests for PartialOrd / Ord ────────────────────────────────────

    #[test]
    fn ord_null_less_than_value() {
        assert!(SqlInt16::NULL < SqlInt16::new(i16::MIN));
    }

    #[test]
    fn ord_null_equals_null() {
        assert_eq!(SqlInt16::NULL.cmp(&SqlInt16::NULL), Ordering::Equal);
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlInt16::MIN_VALUE < SqlInt16::MAX_VALUE);
    }

    #[test]
    fn ord_negative_less_than_positive() {
        assert!(SqlInt16::new(-1) < SqlInt16::new(1));
    }

    #[test]
    fn ord_value_greater_than_null() {
        assert!(SqlInt16::new(0) > SqlInt16::NULL);
    }

    // ── T031: Tests for Display ─────────────────────────────────────────────

    #[test]
    fn display_positive() {
        assert_eq!(SqlInt16::new(1234).to_string(), "1234");
    }

    #[test]
    fn display_negative() {
        assert_eq!(SqlInt16::new(-1234).to_string(), "-1234");
    }

    #[test]
    fn display_zero() {
        assert_eq!(SqlInt16::new(0).to_string(), "0");
    }

    #[test]
    fn display_null() {
        assert_eq!(SqlInt16::NULL.to_string(), "Null");
    }

    #[test]
    fn display_min_value() {
        assert_eq!(SqlInt16::MIN_VALUE.to_string(), "-32768");
    }

    #[test]
    fn display_max_value() {
        assert_eq!(SqlInt16::MAX_VALUE.to_string(), "32767");
    }

    // ── T032: Tests for FromStr ─────────────────────────────────────────────

    #[test]
    fn from_str_positive() {
        let v: SqlInt16 = "1234".parse().unwrap();
        assert_eq!(v.value().unwrap(), 1234);
    }

    #[test]
    fn from_str_negative() {
        let v: SqlInt16 = "-1234".parse().unwrap();
        assert_eq!(v.value().unwrap(), -1234);
    }

    #[test]
    fn from_str_zero() {
        let v: SqlInt16 = "0".parse().unwrap();
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn from_str_null() {
        let v: SqlInt16 = "Null".parse().unwrap();
        assert!(v.is_null());
    }

    #[test]
    fn from_str_null_case_insensitive() {
        let v: SqlInt16 = "null".parse().unwrap();
        assert!(v.is_null());
        let v: SqlInt16 = "NULL".parse().unwrap();
        assert!(v.is_null());
    }

    #[test]
    fn from_str_out_of_range() {
        let result: Result<SqlInt16, _> = "40000".parse();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn from_str_non_numeric() {
        let result: Result<SqlInt16, _> = "abc".parse();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn from_str_empty() {
        let result: Result<SqlInt16, _> = "".parse();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn from_str_whitespace_trimmed() {
        let v: SqlInt16 = "  42  ".parse().unwrap();
        assert_eq!(v.value().unwrap(), 42);
    }

    #[test]
    fn from_str_min_value() {
        let v: SqlInt16 = "-32768".parse().unwrap();
        assert_eq!(v.value().unwrap(), i16::MIN);
    }

    #[test]
    fn from_str_max_value() {
        let v: SqlInt16 = "32767".parse().unwrap();
        assert_eq!(v.value().unwrap(), i16::MAX);
    }

    // ── T033: Tests for conversions ─────────────────────────────────────────

    #[test]
    fn from_sql_boolean_null() {
        let v = SqlInt16::from(SqlBoolean::NULL);
        assert!(v.is_null());
    }

    #[test]
    fn from_sql_boolean_false() {
        let v = SqlInt16::from(SqlBoolean::FALSE);
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_boolean_true() {
        let v = SqlInt16::from(SqlBoolean::TRUE);
        assert_eq!(v.value().unwrap(), 1);
    }

    #[test]
    fn from_sql_byte_null() {
        let v = SqlInt16::from(SqlByte::NULL);
        assert!(v.is_null());
    }

    #[test]
    fn from_sql_byte_zero() {
        let v = SqlInt16::from(SqlByte::new(0));
        assert_eq!(v.value().unwrap(), 0);
    }

    #[test]
    fn from_sql_byte_max() {
        let v = SqlInt16::from(SqlByte::new(255));
        assert_eq!(v.value().unwrap(), 255);
    }

    #[test]
    fn to_sql_boolean_null() {
        let result = SqlInt16::NULL.to_sql_boolean();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_boolean_zero_is_false() {
        let result = SqlInt16::new(0).to_sql_boolean();
        assert_eq!(result, SqlBoolean::FALSE);
    }

    #[test]
    fn to_sql_boolean_nonzero_is_true() {
        let result = SqlInt16::new(42).to_sql_boolean();
        assert_eq!(result, SqlBoolean::TRUE);
        let result = SqlInt16::new(-1).to_sql_boolean();
        assert_eq!(result, SqlBoolean::TRUE);
    }

    #[test]
    fn to_sql_byte_null() {
        let result = SqlInt16::NULL.to_sql_byte().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_byte_in_range() {
        let result = SqlInt16::new(100).to_sql_byte().unwrap();
        assert_eq!(result.value().unwrap(), 100);
    }

    #[test]
    fn to_sql_byte_zero() {
        let result = SqlInt16::new(0).to_sql_byte().unwrap();
        assert_eq!(result.value().unwrap(), 0);
    }

    #[test]
    fn to_sql_byte_max_byte() {
        let result = SqlInt16::new(255).to_sql_byte().unwrap();
        assert_eq!(result.value().unwrap(), 255);
    }

    #[test]
    fn to_sql_byte_negative_overflows() {
        let result = SqlInt16::new(-1).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_over_255_overflows() {
        let result = SqlInt16::new(256).to_sql_byte();
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    // ── Copy/Clone/Debug trait tests ────────────────────────────────────────

    #[test]
    fn copy_semantics() {
        let a = SqlInt16::new(42);
        let b = a; // Copy
        assert_eq!(a.value().unwrap(), b.value().unwrap());
    }

    #[test]
    fn clone_semantics() {
        let a = SqlInt16::new(42);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format() {
        let v = SqlInt16::new(42);
        let debug = format!("{:?}", v);
        assert!(debug.contains("42"));
    }

    #[test]
    fn debug_format_null() {
        let v = SqlInt16::NULL;
        let debug = format!("{:?}", v);
        assert!(debug.contains("None"));
    }

    // ── Quickstart smoke tests (T041) ───────────────────────────────────────

    #[test]
    fn quickstart_create_values() {
        let a = SqlInt16::new(100);
        let b = SqlInt16::new(-200);
        let null = SqlInt16::NULL;
        assert_eq!(a.value().unwrap(), 100);
        assert_eq!(b.value().unwrap(), -200);
        assert!(null.is_null());
    }

    #[test]
    fn quickstart_constants() {
        assert_eq!(SqlInt16::ZERO.value().unwrap(), 0);
        assert_eq!(SqlInt16::MIN_VALUE.value().unwrap(), -32768);
        assert_eq!(SqlInt16::MAX_VALUE.value().unwrap(), 32767);
        assert!(SqlInt16::NULL.is_null());
    }

    #[test]
    fn quickstart_arithmetic() {
        let sum = (SqlInt16::new(100) + SqlInt16::new(200)).unwrap();
        assert_eq!(sum.value().unwrap(), 300);

        let overflow = SqlInt16::new(i16::MAX) + SqlInt16::new(1);
        assert!(overflow.is_err());

        let null_sum = (SqlInt16::new(42) + SqlInt16::NULL).unwrap();
        assert!(null_sum.is_null());

        let div_zero = SqlInt16::new(10) / SqlInt16::new(0);
        assert!(div_zero.is_err());

        let min_div = SqlInt16::new(i16::MIN) / SqlInt16::new(-1);
        assert!(min_div.is_err());
    }

    #[test]
    fn quickstart_bitwise() {
        let and = SqlInt16::new(0xFF) & SqlInt16::new(0x0F);
        assert_eq!(and.value().unwrap(), 0x0F);

        let not = !SqlInt16::new(0);
        assert_eq!(not.value().unwrap(), -1);

        let null_or = SqlInt16::new(42) | SqlInt16::NULL;
        assert!(null_or.is_null());
    }

    #[test]
    fn quickstart_comparisons() {
        let cmp = SqlInt16::new(10).sql_less_than(&SqlInt16::new(20));
        assert_eq!(cmp, SqlBoolean::TRUE);

        let null_cmp = SqlInt16::new(10).sql_equals(&SqlInt16::NULL);
        assert!(null_cmp.is_null());
    }

    #[test]
    fn quickstart_conversions() {
        let a: SqlInt16 = 42i16.into();
        assert_eq!(a.value().unwrap(), 42);

        let b: SqlInt16 = SqlByte::new(255).into();
        assert_eq!(b.value().unwrap(), 255);

        let c: SqlInt16 = SqlBoolean::TRUE.into();
        assert_eq!(c.value().unwrap(), 1);

        let d = SqlInt16::new(100).to_sql_byte().unwrap();
        assert_eq!(d.value().unwrap(), 100);

        let e = SqlInt16::new(-1).to_sql_byte();
        assert!(e.is_err());

        let f = SqlInt16::new(0).to_sql_boolean();
        assert_eq!(f, SqlBoolean::FALSE);
    }

    #[test]
    fn quickstart_display_parse() {
        assert_eq!(SqlInt16::new(-1234).to_string(), "-1234");
        assert_eq!(SqlInt16::NULL.to_string(), "Null");

        let parsed: SqlInt16 = "-1234".parse().unwrap();
        assert_eq!(parsed.value().unwrap(), -1234);

        let null_parsed: SqlInt16 = "Null".parse().unwrap();
        assert!(null_parsed.is_null());
    }
}
