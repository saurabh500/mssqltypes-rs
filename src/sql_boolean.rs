use crate::error::SqlTypeError;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::str::FromStr;

// Internal state constants matching C# SqlBoolean layout
const X_NULL: u8 = 0;
const X_FALSE: u8 = 1;
const X_TRUE: u8 = 2;

/// A three-state boolean type representing SQL Server's BIT type with full NULL support.
///
/// Uses a `u8` internal representation (`0=Null, 1=False, 2=True`) matching the
/// C# `System.Data.SqlTypes.SqlBoolean` layout.
#[derive(Copy, Clone, Debug)]
pub struct SqlBoolean {
    m_value: u8,
}

// ── T006: Constants ──────────────────────────────────────────────────────────

impl SqlBoolean {
    /// A `SqlBoolean` representing SQL NULL.
    pub const NULL: SqlBoolean = SqlBoolean { m_value: X_NULL };
    /// A `SqlBoolean` representing SQL TRUE.
    pub const TRUE: SqlBoolean = SqlBoolean { m_value: X_TRUE };
    /// A `SqlBoolean` representing SQL FALSE.
    pub const FALSE: SqlBoolean = SqlBoolean { m_value: X_FALSE };
    /// Alias for `FALSE` (numeric zero).
    pub const ZERO: SqlBoolean = SqlBoolean { m_value: X_FALSE };
    /// Alias for `TRUE` (numeric one).
    pub const ONE: SqlBoolean = SqlBoolean { m_value: X_TRUE };
}

// ── T007: Constructors and inspectors ────────────────────────────────────────

impl SqlBoolean {
    /// Creates a new `SqlBoolean` from a Rust `bool`.
    pub fn new(value: bool) -> Self {
        if value {
            SqlBoolean { m_value: X_TRUE }
        } else {
            SqlBoolean { m_value: X_FALSE }
        }
    }

    /// Creates a `SqlBoolean` from an integer. Zero → FALSE, non-zero → TRUE.
    pub fn from_int(value: i32) -> Self {
        if value == 0 {
            SqlBoolean { m_value: X_FALSE }
        } else {
            SqlBoolean { m_value: X_TRUE }
        }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.m_value == X_NULL
    }

    /// Returns `true` if this value is SQL TRUE.
    pub fn is_true(&self) -> bool {
        self.m_value == X_TRUE
    }

    /// Returns `true` if this value is SQL FALSE.
    pub fn is_false(&self) -> bool {
        self.m_value == X_FALSE
    }
}

// ── T008: Value access ───────────────────────────────────────────────────────

impl SqlBoolean {
    /// Returns the underlying `bool` value, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<bool, SqlTypeError> {
        match self.m_value {
            X_TRUE => Ok(true),
            X_FALSE => Ok(false),
            _ => Err(SqlTypeError::NullValue),
        }
    }

    /// Returns `1` for TRUE, `0` for FALSE, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn byte_value(&self) -> Result<u8, SqlTypeError> {
        match self.m_value {
            X_TRUE => Ok(1),
            X_FALSE => Ok(0),
            _ => Err(SqlTypeError::NullValue),
        }
    }
}

// ── T009: From<bool> ─────────────────────────────────────────────────────────

impl From<bool> for SqlBoolean {
    fn from(value: bool) -> Self {
        SqlBoolean::new(value)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── T003: Constants and construction tests ───────────────────────────

    #[test]
    fn test_true_constant_is_true() {
        assert!(SqlBoolean::TRUE.is_true());
    }

    #[test]
    fn test_false_constant_is_false() {
        assert!(SqlBoolean::FALSE.is_false());
    }

    #[test]
    fn test_null_constant_is_null() {
        assert!(SqlBoolean::NULL.is_null());
    }

    #[test]
    fn test_zero_equals_false() {
        assert!(SqlBoolean::ZERO.is_false());
        assert!(!SqlBoolean::ZERO.is_true());
        assert!(!SqlBoolean::ZERO.is_null());
    }

    #[test]
    fn test_one_equals_true() {
        assert!(SqlBoolean::ONE.is_true());
        assert!(!SqlBoolean::ONE.is_false());
        assert!(!SqlBoolean::ONE.is_null());
    }

    #[test]
    fn test_new_true() {
        assert!(SqlBoolean::new(true).is_true());
    }

    #[test]
    fn test_new_false() {
        assert!(SqlBoolean::new(false).is_false());
    }

    #[test]
    fn test_from_int_zero() {
        assert!(SqlBoolean::from_int(0).is_false());
    }

    #[test]
    fn test_from_int_positive() {
        assert!(SqlBoolean::from_int(42).is_true());
    }

    #[test]
    fn test_from_int_negative() {
        assert!(SqlBoolean::from_int(-1).is_true());
    }

    #[test]
    fn test_from_bool_trait() {
        assert!(SqlBoolean::from(true).is_true());
        assert!(SqlBoolean::from(false).is_false());
    }

    // ── T004: Value access tests ─────────────────────────────────────────

    #[test]
    fn test_value_true() {
        assert_eq!(SqlBoolean::TRUE.value(), Ok(true));
    }

    #[test]
    fn test_value_false() {
        assert_eq!(SqlBoolean::FALSE.value(), Ok(false));
    }

    #[test]
    fn test_value_null_returns_error() {
        assert_eq!(SqlBoolean::NULL.value(), Err(SqlTypeError::NullValue));
    }

    #[test]
    fn test_byte_value_true() {
        assert_eq!(SqlBoolean::TRUE.byte_value(), Ok(1));
    }

    #[test]
    fn test_byte_value_false() {
        assert_eq!(SqlBoolean::FALSE.byte_value(), Ok(0));
    }

    #[test]
    fn test_byte_value_null_returns_error() {
        assert_eq!(SqlBoolean::NULL.byte_value(), Err(SqlTypeError::NullValue));
    }

    // ── T005: Copy/Clone/Debug tests ─────────────────────────────────────

    #[test]
    fn test_copy_semantics() {
        let a = SqlBoolean::TRUE;
        let b = a; // Copy
        assert!(a.is_true());
        assert!(b.is_true());
    }

    #[test]
    fn test_debug_format() {
        let debug_str = format!("{:?}", SqlBoolean::TRUE);
        assert!(debug_str.contains("SqlBoolean"));
    }
}
