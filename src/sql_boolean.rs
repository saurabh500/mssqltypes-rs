// Licensed under the MIT License. See LICENSE file in the project root for full license information.

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

// ── T014: Not trait ──────────────────────────────────────────────────────────

impl Not for SqlBoolean {
    type Output = SqlBoolean;

    fn not(self) -> SqlBoolean {
        match self.m_value {
            X_TRUE => SqlBoolean::FALSE,
            X_FALSE => SqlBoolean::TRUE,
            _ => SqlBoolean::NULL,
        }
    }
}

// ── T015: BitAnd trait ───────────────────────────────────────────────────────

impl BitAnd for SqlBoolean {
    type Output = SqlBoolean;

    fn bitand(self, rhs: SqlBoolean) -> SqlBoolean {
        // FALSE short-circuit: if either is False, result is False
        if self.m_value == X_FALSE || rhs.m_value == X_FALSE {
            return SqlBoolean::FALSE;
        }
        // Both True → True
        if self.m_value == X_TRUE && rhs.m_value == X_TRUE {
            return SqlBoolean::TRUE;
        }
        // Otherwise (at least one Null, none False) → Null
        SqlBoolean::NULL
    }
}

// ── T016: BitOr trait ────────────────────────────────────────────────────────

impl BitOr for SqlBoolean {
    type Output = SqlBoolean;

    fn bitor(self, rhs: SqlBoolean) -> SqlBoolean {
        // TRUE short-circuit: if either is True, result is True
        if self.m_value == X_TRUE || rhs.m_value == X_TRUE {
            return SqlBoolean::TRUE;
        }
        // Both False → False
        if self.m_value == X_FALSE && rhs.m_value == X_FALSE {
            return SqlBoolean::FALSE;
        }
        // Otherwise (at least one Null, none True) → Null
        SqlBoolean::NULL
    }
}

// ── T017: BitXor trait ───────────────────────────────────────────────────────

impl BitXor for SqlBoolean {
    type Output = SqlBoolean;

    fn bitxor(self, rhs: SqlBoolean) -> SqlBoolean {
        // If either is Null, result is Null
        if self.m_value == X_NULL || rhs.m_value == X_NULL {
            return SqlBoolean::NULL;
        }
        // Otherwise XOR the boolean values
        if self.m_value != rhs.m_value {
            SqlBoolean::TRUE
        } else {
            SqlBoolean::FALSE
        }
    }
}

// ── T020: SQL comparison methods ─────────────────────────────────────────────

impl SqlBoolean {
    /// SQL equality: returns NULL if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value == other.m_value)
    }

    /// SQL inequality: returns NULL if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value != other.m_value)
    }

    /// SQL less than: False < True per m_value. Returns NULL if either is NULL.
    pub fn sql_less_than(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value < other.m_value)
    }

    /// SQL greater than. Returns NULL if either is NULL.
    pub fn sql_greater_than(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value > other.m_value)
    }

    /// SQL less than or equal. Returns NULL if either is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value <= other.m_value)
    }

    /// SQL greater than or equal. Returns NULL if either is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlBoolean) -> SqlBoolean {
        if self.is_null() || other.is_null() {
            return SqlBoolean::NULL;
        }
        SqlBoolean::new(self.m_value >= other.m_value)
    }
}

// ── T021: PartialEq, Eq, Hash ───────────────────────────────────────────────

impl PartialEq for SqlBoolean {
    fn eq(&self, other: &Self) -> bool {
        self.m_value == other.m_value
    }
}

impl Eq for SqlBoolean {}

impl Hash for SqlBoolean {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.m_value.hash(state);
    }
}

// ── T022: PartialOrd, Ord ───────────────────────────────────────────────────

impl PartialOrd for SqlBoolean {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlBoolean {
    fn cmp(&self, other: &Self) -> Ordering {
        // Null(0) < False(1) < True(2) — matches m_value ordering
        self.m_value.cmp(&other.m_value)
    }
}

// ── T025: Display ────────────────────────────────────────────────────────────

impl fmt::Display for SqlBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.m_value {
            X_TRUE => write!(f, "True"),
            X_FALSE => write!(f, "False"),
            _ => write!(f, "Null"),
        }
    }
}

// ── T026: FromStr ────────────────────────────────────────────────────────────

impl FromStr for SqlBoolean {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SqlTypeError::ParseError(
                "Cannot parse empty string as SqlBoolean".to_string(),
            ));
        }

        // Check for "null" (case-insensitive)
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(SqlBoolean::NULL);
        }

        // Check if it starts with a digit, '-', or '+'
        let first = trimmed.as_bytes()[0];
        if first.is_ascii_digit() || first == b'-' || first == b'+' {
            return match trimmed.parse::<i32>() {
                Ok(n) => Ok(SqlBoolean::from_int(n)),
                Err(_) => Err(SqlTypeError::ParseError(format!(
                    "Cannot parse '{s}' as SqlBoolean"
                ))),
            };
        }

        // Try boolean parse (case-insensitive)
        if trimmed.eq_ignore_ascii_case("true") {
            return Ok(SqlBoolean::TRUE);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Ok(SqlBoolean::FALSE);
        }

        Err(SqlTypeError::ParseError(format!(
            "Cannot parse '{s}' as SqlBoolean"
        )))
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

    // ── T010: NOT tests ──────────────────────────────────────────────────

    #[test]
    fn test_not_true() {
        assert!((!SqlBoolean::TRUE).is_false());
    }

    #[test]
    fn test_not_false() {
        assert!((!SqlBoolean::FALSE).is_true());
    }

    #[test]
    fn test_not_null() {
        assert!((!SqlBoolean::NULL).is_null());
    }

    // ── T011: AND truth table tests ──────────────────────────────────────

    #[test]
    fn test_and_true_true() {
        assert!((SqlBoolean::TRUE & SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_and_true_false() {
        assert!((SqlBoolean::TRUE & SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_and_true_null() {
        assert!((SqlBoolean::TRUE & SqlBoolean::NULL).is_null());
    }

    #[test]
    fn test_and_false_true() {
        assert!((SqlBoolean::FALSE & SqlBoolean::TRUE).is_false());
    }

    #[test]
    fn test_and_false_false() {
        assert!((SqlBoolean::FALSE & SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_and_false_null() {
        // Short-circuit: FALSE & NULL = FALSE
        assert!((SqlBoolean::FALSE & SqlBoolean::NULL).is_false());
    }

    #[test]
    fn test_and_null_true() {
        assert!((SqlBoolean::NULL & SqlBoolean::TRUE).is_null());
    }

    #[test]
    fn test_and_null_false() {
        // Short-circuit: NULL & FALSE = FALSE
        assert!((SqlBoolean::NULL & SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_and_null_null() {
        assert!((SqlBoolean::NULL & SqlBoolean::NULL).is_null());
    }

    // ── T012: OR truth table tests ───────────────────────────────────────

    #[test]
    fn test_or_true_true() {
        assert!((SqlBoolean::TRUE | SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_or_true_false() {
        assert!((SqlBoolean::TRUE | SqlBoolean::FALSE).is_true());
    }

    #[test]
    fn test_or_true_null() {
        // Short-circuit: TRUE | NULL = TRUE
        assert!((SqlBoolean::TRUE | SqlBoolean::NULL).is_true());
    }

    #[test]
    fn test_or_false_true() {
        assert!((SqlBoolean::FALSE | SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_or_false_false() {
        assert!((SqlBoolean::FALSE | SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_or_false_null() {
        assert!((SqlBoolean::FALSE | SqlBoolean::NULL).is_null());
    }

    #[test]
    fn test_or_null_true() {
        // Short-circuit: NULL | TRUE = TRUE
        assert!((SqlBoolean::NULL | SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_or_null_false() {
        assert!((SqlBoolean::NULL | SqlBoolean::FALSE).is_null());
    }

    #[test]
    fn test_or_null_null() {
        assert!((SqlBoolean::NULL | SqlBoolean::NULL).is_null());
    }

    // ── T013: XOR truth table tests ──────────────────────────────────────

    #[test]
    fn test_xor_true_true() {
        assert!((SqlBoolean::TRUE ^ SqlBoolean::TRUE).is_false());
    }

    #[test]
    fn test_xor_true_false() {
        assert!((SqlBoolean::TRUE ^ SqlBoolean::FALSE).is_true());
    }

    #[test]
    fn test_xor_true_null() {
        assert!((SqlBoolean::TRUE ^ SqlBoolean::NULL).is_null());
    }

    #[test]
    fn test_xor_false_true() {
        assert!((SqlBoolean::FALSE ^ SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_xor_false_false() {
        assert!((SqlBoolean::FALSE ^ SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_xor_false_null() {
        assert!((SqlBoolean::FALSE ^ SqlBoolean::NULL).is_null());
    }

    #[test]
    fn test_xor_null_true() {
        assert!((SqlBoolean::NULL ^ SqlBoolean::TRUE).is_null());
    }

    #[test]
    fn test_xor_null_false() {
        assert!((SqlBoolean::NULL ^ SqlBoolean::FALSE).is_null());
    }

    #[test]
    fn test_xor_null_null() {
        assert!((SqlBoolean::NULL ^ SqlBoolean::NULL).is_null());
    }

    // ── T018: SQL comparison tests ───────────────────────────────────────

    #[test]
    fn test_sql_equals_true_true() {
        assert!(SqlBoolean::TRUE.sql_equals(&SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_sql_equals_true_false() {
        assert!(SqlBoolean::TRUE.sql_equals(&SqlBoolean::FALSE).is_false());
    }

    #[test]
    fn test_sql_equals_null_any() {
        assert!(SqlBoolean::NULL.sql_equals(&SqlBoolean::TRUE).is_null());
        assert!(SqlBoolean::TRUE.sql_equals(&SqlBoolean::NULL).is_null());
        assert!(SqlBoolean::NULL.sql_equals(&SqlBoolean::NULL).is_null());
    }

    #[test]
    fn test_sql_not_equals_true_false() {
        assert!(
            SqlBoolean::TRUE
                .sql_not_equals(&SqlBoolean::FALSE)
                .is_true()
        );
    }

    #[test]
    fn test_sql_not_equals_true_true() {
        assert!(
            SqlBoolean::TRUE
                .sql_not_equals(&SqlBoolean::TRUE)
                .is_false()
        );
    }

    #[test]
    fn test_sql_less_than_false_true() {
        assert!(SqlBoolean::FALSE.sql_less_than(&SqlBoolean::TRUE).is_true());
    }

    #[test]
    fn test_sql_less_than_true_false() {
        assert!(
            SqlBoolean::TRUE
                .sql_less_than(&SqlBoolean::FALSE)
                .is_false()
        );
    }

    #[test]
    fn test_sql_greater_than_true_false() {
        assert!(
            SqlBoolean::TRUE
                .sql_greater_than(&SqlBoolean::FALSE)
                .is_true()
        );
    }

    #[test]
    fn test_sql_less_than_or_equal_true_true() {
        assert!(
            SqlBoolean::TRUE
                .sql_less_than_or_equal(&SqlBoolean::TRUE)
                .is_true()
        );
    }

    #[test]
    fn test_sql_greater_than_or_equal_false_false() {
        assert!(
            SqlBoolean::FALSE
                .sql_greater_than_or_equal(&SqlBoolean::FALSE)
                .is_true()
        );
    }

    #[test]
    fn test_sql_comparison_with_null() {
        assert!(SqlBoolean::NULL.sql_equals(&SqlBoolean::TRUE).is_null());
        assert!(SqlBoolean::NULL.sql_not_equals(&SqlBoolean::TRUE).is_null());
        assert!(SqlBoolean::NULL.sql_less_than(&SqlBoolean::TRUE).is_null());
        assert!(
            SqlBoolean::NULL
                .sql_greater_than(&SqlBoolean::TRUE)
                .is_null()
        );
        assert!(
            SqlBoolean::NULL
                .sql_less_than_or_equal(&SqlBoolean::TRUE)
                .is_null()
        );
        assert!(
            SqlBoolean::NULL
                .sql_greater_than_or_equal(&SqlBoolean::TRUE)
                .is_null()
        );
    }

    // ── T019: PartialEq, Eq, Hash, Ord tests ────────────────────────────

    #[test]
    fn test_partialeq_true_true() {
        assert_eq!(SqlBoolean::TRUE, SqlBoolean::TRUE);
    }

    #[test]
    fn test_partialeq_true_false() {
        assert_ne!(SqlBoolean::TRUE, SqlBoolean::FALSE);
    }

    #[test]
    fn test_partialeq_null_null() {
        assert_eq!(SqlBoolean::NULL, SqlBoolean::NULL);
    }

    #[test]
    fn test_partialeq_null_true() {
        assert_ne!(SqlBoolean::NULL, SqlBoolean::TRUE);
    }

    #[test]
    fn test_hash_true_consistent() {
        use std::collections::hash_map::DefaultHasher;
        let hash1 = {
            let mut h = DefaultHasher::new();
            SqlBoolean::TRUE.hash(&mut h);
            h.finish()
        };
        let hash2 = {
            let mut h = DefaultHasher::new();
            SqlBoolean::TRUE.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_null_is_zero() {
        use std::collections::hash_map::DefaultHasher;
        let hash_null = {
            let mut h = DefaultHasher::new();
            SqlBoolean::NULL.hash(&mut h);
            h.finish()
        };
        // Just verify it's deterministic — hash(0u8)
        let hash_zero = {
            let mut h = DefaultHasher::new();
            0u8.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_null, hash_zero);
    }

    #[test]
    fn test_ord_null_less_than_false() {
        assert!(SqlBoolean::NULL < SqlBoolean::FALSE);
    }

    #[test]
    fn test_ord_false_less_than_true() {
        assert!(SqlBoolean::FALSE < SqlBoolean::TRUE);
    }

    #[test]
    fn test_ord_null_equal_null() {
        assert_eq!(SqlBoolean::NULL.cmp(&SqlBoolean::NULL), Ordering::Equal);
    }

    #[test]
    fn test_sorting() {
        let mut values = [SqlBoolean::TRUE, SqlBoolean::NULL, SqlBoolean::FALSE];
        values.sort();
        assert!(values[0].is_null());
        assert!(values[1].is_false());
        assert!(values[2].is_true());
    }

    // ── T023: Display tests ──────────────────────────────────────────────

    #[test]
    fn test_display_true() {
        assert_eq!(format!("{}", SqlBoolean::TRUE), "True");
    }

    #[test]
    fn test_display_false() {
        assert_eq!(format!("{}", SqlBoolean::FALSE), "False");
    }

    #[test]
    fn test_display_null() {
        assert_eq!(format!("{}", SqlBoolean::NULL), "Null");
    }

    // ── T024: FromStr tests ──────────────────────────────────────────────

    #[test]
    fn test_parse_true_lowercase() {
        let result: SqlBoolean = "true".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_true_uppercase() {
        let result: SqlBoolean = "True".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_true_mixed_case() {
        let result: SqlBoolean = "TRUE".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_false_lowercase() {
        let result: SqlBoolean = "false".parse().unwrap();
        assert!(result.is_false());
    }

    #[test]
    fn test_parse_one() {
        let result: SqlBoolean = "1".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_zero() {
        let result: SqlBoolean = "0".parse().unwrap();
        assert!(result.is_false());
    }

    #[test]
    fn test_parse_positive_int() {
        let result: SqlBoolean = "42".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_negative_int() {
        let result: SqlBoolean = "-1".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_null_string() {
        let result: SqlBoolean = "Null".parse().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_parse_with_leading_whitespace() {
        let result: SqlBoolean = " true".parse().unwrap();
        assert!(result.is_true());
    }

    #[test]
    fn test_parse_invalid() {
        let result = "maybe".parse::<SqlBoolean>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn test_parse_empty() {
        let result = "".parse::<SqlBoolean>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }
}
