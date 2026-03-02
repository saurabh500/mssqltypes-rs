// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// ── T001: SqlByte module ─────────────────────────────────────────────────────

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_string::SqlString;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Sub};
use std::str::FromStr;

/// Overflow bitmask — mirrors C#'s `~0xff`. Any bits set outside 0xFF indicate
/// overflow (positive >255) or underflow (negative result).
const I_BIT_NOT_BYTE_MAX: i32 = !0xFF;

/// An unsigned 8-bit integer (0–255) with SQL NULL support, equivalent to
/// C# `System.Data.SqlTypes.SqlByte` / SQL Server `TINYINT`.
///
/// Uses `Option<u8>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// All arithmetic returns `Result<SqlByte, SqlTypeError>` with overflow detection.
/// Comparisons return `SqlBoolean` for three-valued NULL logic.
#[derive(Copy, Clone, Debug)]
pub struct SqlByte {
    value: Option<u8>,
}

// ── T005: Constants and constructors ────────────────────────────────────────

impl SqlByte {
    /// SQL NULL.
    pub const NULL: SqlByte = SqlByte { value: None };
    /// Zero (0).
    pub const ZERO: SqlByte = SqlByte { value: Some(0) };
    /// Minimum value (0).
    pub const MIN_VALUE: SqlByte = SqlByte { value: Some(0) };
    /// Maximum value (255).
    pub const MAX_VALUE: SqlByte = SqlByte {
        value: Some(u8::MAX),
    };

    /// Creates a new `SqlByte` from a `u8` value.
    pub fn new(v: u8) -> Self {
        SqlByte { value: Some(v) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the inner `u8`, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<u8, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}

// ── T006: From<u8> ──────────────────────────────────────────────────────────

impl From<u8> for SqlByte {
    fn from(v: u8) -> Self {
        SqlByte::new(v)
    }
}

// ── T011: Checked arithmetic (add, sub, mul) ────────────────────────────────

impl SqlByte {
    /// Checked addition. Returns `Err(Overflow)` if result > 255.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlByte::NULL)`.
    pub fn checked_add(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlByte::NULL),
            (Some(a), Some(b)) => {
                let result = a as i32 + b as i32;
                if (result & I_BIT_NOT_BYTE_MAX) != 0 {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlByte::new(result as u8))
                }
            }
        }
    }

    /// Checked subtraction. Returns `Err(Overflow)` if result < 0.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlByte::NULL)`.
    pub fn checked_sub(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlByte::NULL),
            (Some(a), Some(b)) => {
                let result = a as i32 - b as i32;
                if (result & I_BIT_NOT_BYTE_MAX) != 0 {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlByte::new(result as u8))
                }
            }
        }
    }

    /// Checked multiplication. Returns `Err(Overflow)` if result > 255.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlByte::NULL)`.
    pub fn checked_mul(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlByte::NULL),
            (Some(a), Some(b)) => {
                let result = a as i32 * b as i32;
                if (result & I_BIT_NOT_BYTE_MAX) != 0 {
                    Err(SqlTypeError::Overflow)
                } else {
                    Ok(SqlByte::new(result as u8))
                }
            }
        }
    }
}

// ── T012: Checked arithmetic (div, rem) ─────────────────────────────────────

impl SqlByte {
    /// Checked division. Returns `Err(DivideByZero)` if divisor is zero.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlByte::NULL)`.
    pub fn checked_div(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlByte::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => Ok(SqlByte::new(a / b)),
        }
    }

    /// Checked remainder. Returns `Err(DivideByZero)` if divisor is zero.
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlByte::NULL)`.
    pub fn checked_rem(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError> {
        match (self.value, rhs.value) {
            (None, _) | (_, None) => Ok(SqlByte::NULL),
            (Some(_), Some(0)) => Err(SqlTypeError::DivideByZero),
            (Some(a), Some(b)) => Ok(SqlByte::new(a % b)),
        }
    }
}

// ── T013: Operator traits (Add, Sub, Mul, Div, Rem) ─────────────────────────

impl Add for SqlByte {
    type Output = Result<SqlByte, SqlTypeError>;

    fn add(self, rhs: SqlByte) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for SqlByte {
    type Output = Result<SqlByte, SqlTypeError>;

    fn sub(self, rhs: SqlByte) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Mul for SqlByte {
    type Output = Result<SqlByte, SqlTypeError>;

    fn mul(self, rhs: SqlByte) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Div for SqlByte {
    type Output = Result<SqlByte, SqlTypeError>;

    fn div(self, rhs: SqlByte) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Rem for SqlByte {
    type Output = Result<SqlByte, SqlTypeError>;

    fn rem(self, rhs: SqlByte) -> Self::Output {
        self.checked_rem(rhs)
    }
}

// ── T015: Bitwise operations ────────────────────────────────────────────────

impl SqlByte {
    /// Returns the ones complement (~value). NULL → NULL.
    pub fn ones_complement(self) -> SqlByte {
        match self.value {
            None => SqlByte::NULL,
            Some(v) => SqlByte::new(!v),
        }
    }
}

impl BitAnd for SqlByte {
    type Output = SqlByte;

    fn bitand(self, rhs: SqlByte) -> SqlByte {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlByte::new(a & b),
            _ => SqlByte::NULL,
        }
    }
}

impl BitOr for SqlByte {
    type Output = SqlByte;

    fn bitor(self, rhs: SqlByte) -> SqlByte {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlByte::new(a | b),
            _ => SqlByte::NULL,
        }
    }
}

impl BitXor for SqlByte {
    type Output = SqlByte;

    fn bitxor(self, rhs: SqlByte) -> SqlByte {
        match (self.value, rhs.value) {
            (Some(a), Some(b)) => SqlByte::new(a ^ b),
            _ => SqlByte::NULL,
        }
    }
}

impl Not for SqlByte {
    type Output = SqlByte;

    fn not(self) -> SqlByte {
        self.ones_complement()
    }
}

// ── T020: SQL comparison methods ────────────────────────────────────────────

impl SqlByte {
    /// SQL equality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a == b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL inequality. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a != b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a < b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a > b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a <= b),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater than or equal. Returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlByte) -> SqlBoolean {
        match (self.value, other.value) {
            (Some(a), Some(b)) => SqlBoolean::new(a >= b),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T021: PartialEq, Eq, Hash, PartialOrd, Ord ─────────────────────────────

impl PartialEq for SqlByte {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlByte {}

impl Hash for SqlByte {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NULL hashes as 0u8 for consistency
        match self.value {
            Some(v) => v.hash(state),
            None => 0u8.hash(state),
        }
    }
}

impl PartialOrd for SqlByte {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlByte {
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

// ── T022: Display and FromStr ───────────────────────────────────────────────

impl fmt::Display for SqlByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(v) => write!(f, "{v}"),
            None => write!(f, "Null"),
        }
    }
}

impl FromStr for SqlByte {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SqlTypeError::ParseError(
                "Cannot parse empty string as SqlByte".to_string(),
            ));
        }

        // Check for "null" (case-insensitive)
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlByte::NULL);
        }

        // Parse as u8
        match trimmed.parse::<u8>() {
            Ok(v) => Ok(SqlByte::new(v)),
            Err(e) => Err(SqlTypeError::ParseError(format!(
                "Cannot parse '{s}' as SqlByte: {e}"
            ))),
        }
    }
}

// ── T023: Type conversions ──────────────────────────────────────────────────

impl SqlByte {
    /// Converts to `SqlBoolean`: NULL→NULL, 0→FALSE, non-zero→TRUE.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.value {
            None => SqlBoolean::NULL,
            Some(0) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }
}

impl From<SqlBoolean> for SqlByte {
    fn from(b: SqlBoolean) -> Self {
        if b.is_null() {
            SqlByte::NULL
        } else {
            match b.value() {
                Ok(true) => SqlByte::new(1),
                Ok(false) => SqlByte::new(0),
                Err(_) => SqlByte::NULL,
            }
        }
    }
}

impl SqlByte {
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

    fn hash_of(val: &SqlByte) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    // ── T003: Constant and construction tests ───────────────────────────

    #[test]
    fn test_new_value() {
        assert_eq!(SqlByte::new(42).value(), Ok(42));
    }

    #[test]
    fn test_null_is_null() {
        assert!(SqlByte::NULL.is_null());
    }

    #[test]
    fn test_null_value_returns_error() {
        assert_eq!(SqlByte::NULL.value(), Err(SqlTypeError::NullValue));
    }

    #[test]
    fn test_zero_constant() {
        assert_eq!(SqlByte::ZERO.value(), Ok(0));
    }

    #[test]
    fn test_min_value() {
        assert_eq!(SqlByte::MIN_VALUE.value(), Ok(0));
    }

    #[test]
    fn test_max_value() {
        assert_eq!(SqlByte::MAX_VALUE.value(), Ok(255));
    }

    #[test]
    fn test_non_null_is_not_null() {
        assert!(!SqlByte::new(100).is_null());
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(SqlByte::from(42u8).value(), Ok(42));
    }

    // ── T004: Copy/Clone/Debug tests ────────────────────────────────────

    #[test]
    fn test_copy_semantics() {
        let a = SqlByte::new(42);
        let b = a; // Copy
        assert_eq!(a.value(), Ok(42));
        assert_eq!(b.value(), Ok(42));
    }

    #[test]
    fn test_debug_format() {
        let debug = format!("{:?}", SqlByte::new(42));
        assert!(debug.contains("SqlByte"));
    }

    // ── T007: Addition tests ────────────────────────────────────────────

    #[test]
    fn test_add_normal() {
        let result = (SqlByte::new(10) + SqlByte::new(20)).unwrap();
        assert_eq!(result.value(), Ok(30));
    }

    #[test]
    fn test_add_overflow() {
        let result = SqlByte::new(200) + SqlByte::new(100);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_add_boundary_max() {
        let result = SqlByte::new(255) + SqlByte::new(1);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_add_zero() {
        let result = (SqlByte::new(42) + SqlByte::new(0)).unwrap();
        assert_eq!(result.value(), Ok(42));
    }

    #[test]
    fn test_add_null_left() {
        let result = (SqlByte::NULL + SqlByte::new(10)).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_add_null_right() {
        let result = (SqlByte::new(10) + SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_add_null_both() {
        let result = (SqlByte::NULL + SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    // ── T008: Subtraction tests ─────────────────────────────────────────

    #[test]
    fn test_sub_normal() {
        let result = (SqlByte::new(20) - SqlByte::new(10)).unwrap();
        assert_eq!(result.value(), Ok(10));
    }

    #[test]
    fn test_sub_negative_overflow() {
        let result = SqlByte::new(5) - SqlByte::new(10);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_sub_boundary_zero() {
        let result = SqlByte::new(0) - SqlByte::new(1);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_sub_to_zero() {
        let result = (SqlByte::new(10) - SqlByte::new(10)).unwrap();
        assert_eq!(result.value(), Ok(0));
    }

    #[test]
    fn test_sub_null() {
        let result = (SqlByte::new(10) - SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    // ── T009: Multiplication tests ──────────────────────────────────────

    #[test]
    fn test_mul_normal() {
        let result = (SqlByte::new(10) * SqlByte::new(5)).unwrap();
        assert_eq!(result.value(), Ok(50));
    }

    #[test]
    fn test_mul_overflow() {
        let result = SqlByte::new(15) * SqlByte::new(20);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_mul_boundary() {
        let result = SqlByte::new(128) * SqlByte::new(2);
        assert!(matches!(result, Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn test_mul_by_zero() {
        let result = (SqlByte::new(255) * SqlByte::new(0)).unwrap();
        assert_eq!(result.value(), Ok(0));
    }

    #[test]
    fn test_mul_by_one() {
        let result = (SqlByte::new(42) * SqlByte::new(1)).unwrap();
        assert_eq!(result.value(), Ok(42));
    }

    #[test]
    fn test_mul_null() {
        let result = (SqlByte::new(10) * SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    // ── T010: Division and remainder tests ──────────────────────────────

    #[test]
    fn test_div_normal() {
        let result = (SqlByte::new(20) / SqlByte::new(5)).unwrap();
        assert_eq!(result.value(), Ok(4));
    }

    #[test]
    fn test_div_by_zero() {
        let result = SqlByte::new(10) / SqlByte::new(0);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn test_div_truncates() {
        let result = (SqlByte::new(10) / SqlByte::new(3)).unwrap();
        assert_eq!(result.value(), Ok(3));
    }

    #[test]
    fn test_div_null() {
        let result = (SqlByte::new(10) / SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_rem_normal() {
        let result = (SqlByte::new(10) % SqlByte::new(3)).unwrap();
        assert_eq!(result.value(), Ok(1));
    }

    #[test]
    fn test_rem_by_zero() {
        let result = SqlByte::new(10) % SqlByte::new(0);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn test_rem_null() {
        let result = (SqlByte::new(10) % SqlByte::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_rem_even() {
        let result = (SqlByte::new(10) % SqlByte::new(5)).unwrap();
        assert_eq!(result.value(), Ok(0));
    }

    // ── T014: Bitwise operation tests ───────────────────────────────────

    #[test]
    fn test_bitand() {
        let result = SqlByte::new(0xFF) & SqlByte::new(0x0F);
        assert_eq!(result.value(), Ok(0x0F));
    }

    #[test]
    fn test_bitor() {
        let result = SqlByte::new(0xF0) | SqlByte::new(0x0F);
        assert_eq!(result.value(), Ok(0xFF));
    }

    #[test]
    fn test_bitxor() {
        let result = SqlByte::new(0xFF) ^ SqlByte::new(0x0F);
        assert_eq!(result.value(), Ok(0xF0));
    }

    #[test]
    fn test_not() {
        let result = !SqlByte::new(0x0F);
        assert_eq!(result.value(), Ok(0xF0));
    }

    #[test]
    fn test_ones_complement() {
        let result = SqlByte::new(0x0F).ones_complement();
        assert_eq!(result.value(), Ok(0xF0));
    }

    #[test]
    fn test_bitand_null() {
        let result = SqlByte::new(0xFF) & SqlByte::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn test_bitor_null() {
        let result = SqlByte::NULL | SqlByte::new(0x0F);
        assert!(result.is_null());
    }

    #[test]
    fn test_bitxor_null() {
        let result = SqlByte::NULL ^ SqlByte::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn test_not_null() {
        let result = !SqlByte::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn test_not_zero() {
        let result = !SqlByte::new(0x00);
        assert_eq!(result.value(), Ok(0xFF));
    }

    #[test]
    fn test_not_max() {
        let result = !SqlByte::new(0xFF);
        assert_eq!(result.value(), Ok(0x00));
    }

    // ── T016: SQL comparison tests ──────────────────────────────────────

    #[test]
    fn test_sql_equals_same() {
        assert!(SqlByte::new(10).sql_equals(&SqlByte::new(10)).is_true());
    }

    #[test]
    fn test_sql_equals_different() {
        assert!(SqlByte::new(10).sql_equals(&SqlByte::new(20)).is_false());
    }

    #[test]
    fn test_sql_equals_null() {
        assert!(SqlByte::new(10).sql_equals(&SqlByte::NULL).is_null());
    }

    #[test]
    fn test_sql_not_equals() {
        assert!(SqlByte::new(10).sql_not_equals(&SqlByte::new(20)).is_true());
    }

    #[test]
    fn test_sql_less_than() {
        assert!(SqlByte::new(10).sql_less_than(&SqlByte::new(20)).is_true());
    }

    #[test]
    fn test_sql_less_than_with_equal_values() {
        assert!(SqlByte::new(10).sql_less_than(&SqlByte::new(10)).is_false());
    }

    #[test]
    fn test_sql_greater_than() {
        assert!(
            SqlByte::new(20)
                .sql_greater_than(&SqlByte::new(10))
                .is_true()
        );
    }

    #[test]
    fn test_sql_less_than_or_equal() {
        assert!(
            SqlByte::new(10)
                .sql_less_than_or_equal(&SqlByte::new(10))
                .is_true()
        );
    }

    #[test]
    fn test_sql_greater_than_or_equal() {
        assert!(
            SqlByte::new(10)
                .sql_greater_than_or_equal(&SqlByte::new(20))
                .is_false()
        );
    }

    #[test]
    fn test_sql_comparison_null_propagation() {
        let v = SqlByte::new(10);
        let n = SqlByte::NULL;

        assert!(v.sql_equals(&n).is_null());
        assert!(n.sql_equals(&v).is_null());
        assert!(v.sql_not_equals(&n).is_null());
        assert!(v.sql_less_than(&n).is_null());
        assert!(v.sql_greater_than(&n).is_null());
        assert!(v.sql_less_than_or_equal(&n).is_null());
        assert!(v.sql_greater_than_or_equal(&n).is_null());
    }

    // ── T017: PartialEq, Eq, Hash, Ord tests ───────────────────────────

    #[test]
    fn test_partialeq_same() {
        assert_eq!(SqlByte::new(42), SqlByte::new(42));
    }

    #[test]
    fn test_partialeq_different() {
        assert_ne!(SqlByte::new(42), SqlByte::new(43));
    }

    #[test]
    fn test_partialeq_null_null() {
        assert_eq!(SqlByte::NULL, SqlByte::NULL);
    }

    #[test]
    fn test_partialeq_null_value() {
        assert_ne!(SqlByte::NULL, SqlByte::new(0));
    }

    #[test]
    fn test_hash_consistent() {
        assert_eq!(hash_of(&SqlByte::new(42)), hash_of(&SqlByte::new(42)));
    }

    #[test]
    fn test_hash_null_is_zero() {
        // NULL hashes the same as 0u8
        let null_hash = hash_of(&SqlByte::NULL);
        let zero_hash = hash_of(&SqlByte::new(0));
        assert_eq!(null_hash, zero_hash);
    }

    #[test]
    fn test_ord_null_less_than_value() {
        assert!(SqlByte::NULL < SqlByte::new(0));
    }

    #[test]
    fn test_ord_values() {
        assert!(SqlByte::new(10) < SqlByte::new(20));
    }

    #[test]
    fn test_sorting() {
        let mut v = vec![SqlByte::new(20), SqlByte::NULL, SqlByte::new(10)];
        v.sort();
        assert!(v[0].is_null());
        assert_eq!(v[1].value(), Ok(10));
        assert_eq!(v[2].value(), Ok(20));
    }

    // ── T018: Display and FromStr tests ─────────────────────────────────

    #[test]
    fn test_display_value() {
        assert_eq!(format!("{}", SqlByte::new(42)), "42");
    }

    #[test]
    fn test_display_zero() {
        assert_eq!(format!("{}", SqlByte::ZERO), "0");
    }

    #[test]
    fn test_display_max() {
        assert_eq!(format!("{}", SqlByte::MAX_VALUE), "255");
    }

    #[test]
    fn test_display_null() {
        assert_eq!(format!("{}", SqlByte::NULL), "Null");
    }

    #[test]
    fn test_parse_valid() {
        let result: SqlByte = "123".parse().unwrap();
        assert_eq!(result.value(), Ok(123));
    }

    #[test]
    fn test_parse_zero() {
        let result: SqlByte = "0".parse().unwrap();
        assert_eq!(result.value(), Ok(0));
    }

    #[test]
    fn test_parse_max() {
        let result: SqlByte = "255".parse().unwrap();
        assert_eq!(result.value(), Ok(255));
    }

    #[test]
    fn test_parse_null() {
        let result: SqlByte = "Null".parse().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_parse_null_case_insensitive() {
        let result: SqlByte = "NULL".parse().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_parse_overflow() {
        let result = "256".parse::<SqlByte>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn test_parse_negative() {
        let result = "-1".parse::<SqlByte>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn test_parse_invalid() {
        let result = "abc".parse::<SqlByte>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn test_parse_empty() {
        let result = "".parse::<SqlByte>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn test_parse_whitespace() {
        let result: SqlByte = " 42 ".parse().unwrap();
        assert_eq!(result.value(), Ok(42));
    }

    #[test]
    fn test_display_parse_roundtrip() {
        let original = SqlByte::new(42);
        let parsed: SqlByte = original.to_string().parse().unwrap();
        assert_eq!(original, parsed);
    }

    // ── T019: Conversion tests ──────────────────────────────────────────

    #[test]
    fn test_to_sql_boolean_nonzero() {
        assert!(SqlByte::new(42).to_sql_boolean().is_true());
    }

    #[test]
    fn test_to_sql_boolean_zero() {
        assert!(SqlByte::new(0).to_sql_boolean().is_false());
    }

    #[test]
    fn test_to_sql_boolean_null() {
        assert!(SqlByte::NULL.to_sql_boolean().is_null());
    }

    #[test]
    fn test_from_sql_boolean_true() {
        assert_eq!(SqlByte::from(SqlBoolean::TRUE).value(), Ok(1));
    }

    #[test]
    fn test_from_sql_boolean_false() {
        assert_eq!(SqlByte::from(SqlBoolean::FALSE).value(), Ok(0));
    }

    #[test]
    fn test_from_sql_boolean_null() {
        assert!(SqlByte::from(SqlBoolean::NULL).is_null());
    }

    // ── to_sql_string() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_string_normal() {
        let s = SqlByte::new(42).to_sql_string();
        assert_eq!(s.value().unwrap(), "42");
    }

    #[test]
    fn to_sql_string_zero() {
        let s = SqlByte::new(0).to_sql_string();
        assert_eq!(s.value().unwrap(), "0");
    }

    #[test]
    fn to_sql_string_max() {
        let s = SqlByte::new(255).to_sql_string();
        assert_eq!(s.value().unwrap(), "255");
    }

    #[test]
    fn to_sql_string_null() {
        let s = SqlByte::NULL.to_sql_string();
        assert!(s.is_null());
    }
}
