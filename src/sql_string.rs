//! `SqlString` — a nullable string type with configurable comparison options.
//!
//! Equivalent to C# `System.Data.SqlTypes.SqlString` / SQL Server `NVARCHAR`.
//! Stores `Option<String>` plus `SqlCompareOptions`. Unlike numeric SQL types,
//! `SqlString` is `Clone` but NOT `Copy` (heap-allocated `String`).

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_compare_options::SqlCompareOptions;
use crate::sql_datetime::SqlDateTime;
use crate::sql_decimal::SqlDecimal;
use crate::sql_double::SqlDouble;
use crate::sql_guid::SqlGuid;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use crate::sql_int64::SqlInt64;
use crate::sql_money::SqlMoney;
use crate::sql_single::SqlSingle;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::str::FromStr;

/// A nullable string value representing SQL Server `NVARCHAR` / `VARCHAR`
/// with configurable comparison options.
///
/// Internal representation: `Option<String>` + `SqlCompareOptions`.
/// `Clone` but NOT `Copy` (contains heap-allocated `String`).
#[derive(Clone, Debug)]
pub struct SqlString {
    value: Option<String>,
    compare_options: SqlCompareOptions,
}

// ── Constants ────────────────────────────────────────────────────────────────

impl SqlString {
    /// A `SqlString` representing SQL NULL with default `IgnoreCase` options.
    pub const NULL: SqlString = SqlString {
        value: None,
        compare_options: SqlCompareOptions::IgnoreCase,
    };
}

// ── Constructors & Accessors ─────────────────────────────────────────────────

impl SqlString {
    /// Creates a new `SqlString` with default `IgnoreCase` compare options.
    pub fn new(s: &str) -> Self {
        SqlString {
            value: Some(s.to_string()),
            compare_options: SqlCompareOptions::IgnoreCase,
        }
    }

    /// Creates a new `SqlString` with explicit compare options.
    pub fn with_options(s: &str, options: SqlCompareOptions) -> Self {
        SqlString {
            value: Some(s.to_string()),
            compare_options: options,
        }
    }

    /// Returns `true` if this is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the string value, or `Err(NullValue)` if NULL.
    pub fn value(&self) -> Result<&str, SqlTypeError> {
        self.value.as_deref().ok_or(SqlTypeError::NullValue)
    }

    /// Returns the byte length of the string, or `Err(NullValue)` if NULL.
    pub fn len(&self) -> Result<usize, SqlTypeError> {
        match &self.value {
            Some(s) => Ok(s.len()),
            None => Err(SqlTypeError::NullValue),
        }
    }

    /// Returns `true` if the string is empty (zero bytes), or `Err(NullValue)` if NULL.
    pub fn is_empty(&self) -> Result<bool, SqlTypeError> {
        match &self.value {
            Some(s) => Ok(s.is_empty()),
            None => Err(SqlTypeError::NullValue),
        }
    }

    /// Returns the compare options for this instance.
    pub fn compare_options(&self) -> SqlCompareOptions {
        self.compare_options
    }
}

// ── Concatenation (Add) ──────────────────────────────────────────────────────

impl Add for SqlString {
    type Output = SqlString;

    /// Concatenates two `SqlString` values. NULL propagation: if either
    /// operand is NULL, the result is NULL. The result inherits the left
    /// operand's compare options.
    fn add(self, rhs: SqlString) -> SqlString {
        match (&self.value, &rhs.value) {
            (Some(l), Some(r)) => SqlString {
                value: Some(format!("{}{}", l, r)),
                compare_options: self.compare_options,
            },
            _ => SqlString {
                value: None,
                compare_options: self.compare_options,
            },
        }
    }
}

// ── SQL Comparisons ──────────────────────────────────────────────────────────

impl SqlString {
    /// Compares two non-NULL string values using `self.compare_options`.
    /// Trailing spaces are trimmed before comparison.
    /// Returns `None` if either operand is NULL.
    fn compare_strings(&self, other: &SqlString) -> Option<Ordering> {
        let l = self.value.as_deref()?;
        let r = other.value.as_deref()?;
        let l = l.trim_end();
        let r = r.trim_end();

        let ord = match self.compare_options {
            SqlCompareOptions::None => l.cmp(r),
            SqlCompareOptions::IgnoreCase => {
                let ll = l.to_ascii_lowercase();
                let rl = r.to_ascii_lowercase();
                ll.cmp(&rl)
            }
            SqlCompareOptions::BinarySort | SqlCompareOptions::BinarySort2 => {
                l.as_bytes().cmp(r.as_bytes())
            }
        };
        Some(ord)
    }

    /// Returns `SqlBoolean::TRUE` if the two strings are equal according to
    /// the left operand's compare options. NULL if either is NULL.
    pub fn sql_equals(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Equal) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
            None => SqlBoolean::NULL,
        }
    }

    /// Returns `SqlBoolean::TRUE` if the two strings are NOT equal.
    pub fn sql_not_equals(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Equal) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
            None => SqlBoolean::NULL,
        }
    }

    /// Returns `SqlBoolean::TRUE` if `self < other`.
    pub fn sql_less_than(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Less) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
            None => SqlBoolean::NULL,
        }
    }

    /// Returns `SqlBoolean::TRUE` if `self > other`.
    pub fn sql_greater_than(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Greater) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
            None => SqlBoolean::NULL,
        }
    }

    /// Returns `SqlBoolean::TRUE` if `self <= other`.
    pub fn sql_less_than_or_equal(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Greater) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
            None => SqlBoolean::NULL,
        }
    }

    /// Returns `SqlBoolean::TRUE` if `self >= other`.
    pub fn sql_greater_than_or_equal(&self, other: &SqlString) -> SqlBoolean {
        match self.compare_strings(other) {
            Some(Ordering::Less) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
            None => SqlBoolean::NULL,
        }
    }
}

// ── Display & FromStr ────────────────────────────────────────────────────────

impl fmt::Display for SqlString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "Null"),
        }
    }
}

impl FromStr for SqlString {
    type Err = SqlTypeError;

    /// Parses a string into a `SqlString`.
    /// "Null" (case-insensitive) returns `SqlString::NULL`.
    /// All other strings return `SqlString::new(input)`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("null") {
            Ok(SqlString::NULL)
        } else {
            Ok(SqlString::new(s))
        }
    }
}

// ── From conversions ─────────────────────────────────────────────────────────

impl From<&str> for SqlString {
    fn from(s: &str) -> Self {
        SqlString::new(s)
    }
}

impl From<String> for SqlString {
    fn from(s: String) -> Self {
        SqlString {
            value: Some(s),
            compare_options: SqlCompareOptions::IgnoreCase,
        }
    }
}

// ── PartialEq / Eq ──────────────────────────────────────────────────────────

impl PartialEq for SqlString {
    /// Case-insensitive ASCII comparison with trailing-space trimming.
    /// NULL == NULL is `true` (Rust trait semantics, not SQL semantics).
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => a.trim_end().eq_ignore_ascii_case(b.trim_end()),
            (None, None) => true,
            _ => false,
        }
    }
}

impl Eq for SqlString {}

// ── Hash ─────────────────────────────────────────────────────────────────────

impl Hash for SqlString {
    /// Hash of lowercased + trailing-space-trimmed value.
    /// NULL hashes as empty string.
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.value {
            Some(s) => {
                let trimmed = s.trim_end().to_ascii_lowercase();
                trimmed.hash(state);
            }
            None => "".hash(state),
        }
    }
}

// ── PartialOrd / Ord ─────────────────────────────────────────────────────────

impl PartialOrd for SqlString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlString {
    /// Case-insensitive ordering. NULL < any non-NULL.
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.value, &other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => {
                let la = a.trim_end().to_ascii_lowercase();
                let lb = b.trim_end().to_ascii_lowercase();
                la.cmp(&lb)
            }
        }
    }
}

// ── Cross-Type Parsing Conversions ───────────────────────────────────────────

impl SqlString {
    /// Parses this string as `SqlBoolean`. NULL → `Ok(SqlBoolean::NULL)`.
    pub fn to_sql_boolean(&self) -> Result<SqlBoolean, SqlTypeError> {
        match &self.value {
            None => Ok(SqlBoolean::NULL),
            Some(s) => s.parse::<SqlBoolean>(),
        }
    }

    /// Parses this string as `SqlByte`. NULL → `Ok(SqlByte::NULL)`.
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError> {
        match &self.value {
            None => Ok(SqlByte::NULL),
            Some(s) => s.parse::<SqlByte>(),
        }
    }

    /// Parses this string as `SqlInt16`. NULL → `Ok(SqlInt16::NULL)`.
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError> {
        match &self.value {
            None => Ok(SqlInt16::NULL),
            Some(s) => s.parse::<SqlInt16>(),
        }
    }

    /// Parses this string as `SqlInt32`. NULL → `Ok(SqlInt32::NULL)`.
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError> {
        match &self.value {
            None => Ok(SqlInt32::NULL),
            Some(s) => s.parse::<SqlInt32>(),
        }
    }

    /// Parses this string as `SqlInt64`. NULL → `Ok(SqlInt64::NULL)`.
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError> {
        match &self.value {
            None => Ok(SqlInt64::NULL),
            Some(s) => s.parse::<SqlInt64>(),
        }
    }

    /// Parses this string as `SqlSingle`. NULL → `Ok(SqlSingle::NULL)`.
    pub fn to_sql_single(&self) -> Result<SqlSingle, SqlTypeError> {
        match &self.value {
            None => Ok(SqlSingle::NULL),
            Some(s) => s.parse::<SqlSingle>(),
        }
    }

    /// Parses this string as `SqlDouble`. NULL → `Ok(SqlDouble::NULL)`.
    pub fn to_sql_double(&self) -> Result<SqlDouble, SqlTypeError> {
        match &self.value {
            None => Ok(SqlDouble::NULL),
            Some(s) => s.parse::<SqlDouble>(),
        }
    }

    /// Parses this string as `SqlDecimal`. NULL → `Ok(SqlDecimal::NULL)`.
    pub fn to_sql_decimal(&self) -> Result<SqlDecimal, SqlTypeError> {
        match &self.value {
            None => Ok(SqlDecimal::NULL),
            Some(s) => s.parse::<SqlDecimal>(),
        }
    }

    /// Parses this string as `SqlMoney`. NULL → `Ok(SqlMoney::NULL)`.
    pub fn to_sql_money(&self) -> Result<SqlMoney, SqlTypeError> {
        match &self.value {
            None => Ok(SqlMoney::NULL),
            Some(s) => s.parse::<SqlMoney>(),
        }
    }

    /// Parses this string as `SqlDateTime`. NULL → `Ok(SqlDateTime::NULL)`.
    pub fn to_sql_date_time(&self) -> Result<SqlDateTime, SqlTypeError> {
        match &self.value {
            None => Ok(SqlDateTime::NULL),
            Some(s) => s.parse::<SqlDateTime>(),
        }
    }

    /// Parses this string as `SqlGuid`. NULL → `Ok(SqlGuid::NULL)`.
    pub fn to_sql_guid(&self) -> Result<SqlGuid, SqlTypeError> {
        match &self.value {
            None => Ok(SqlGuid::NULL),
            Some(s) => s.parse::<SqlGuid>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T009: new(), is_null(), value() ──────────────────────────────────────

    #[test]
    fn new_hello_returns_hello() {
        let s = SqlString::new("hello");
        assert_eq!(s.value().unwrap(), "hello");
    }

    #[test]
    fn null_value_returns_err() {
        let s = SqlString::NULL;
        assert!(matches!(s.value(), Err(SqlTypeError::NullValue)));
    }

    #[test]
    fn empty_string_is_not_null() {
        let s = SqlString::new("");
        assert!(!s.is_null());
        assert_eq!(s.value().unwrap(), "");
    }

    #[test]
    fn is_null_returns_false_for_non_null() {
        let s = SqlString::new("hello");
        assert!(!s.is_null());
    }

    #[test]
    fn is_null_returns_true_for_null() {
        assert!(SqlString::NULL.is_null());
    }

    // ── T010: len() ─────────────────────────────────────────────────────────

    #[test]
    fn len_hello_returns_5() {
        assert_eq!(SqlString::new("hello").len().unwrap(), 5);
    }

    #[test]
    fn len_empty_returns_0() {
        assert_eq!(SqlString::new("").len().unwrap(), 0);
    }

    #[test]
    fn len_multibyte_utf8_returns_byte_len() {
        // 🦀 is 4 bytes in UTF-8
        assert_eq!(SqlString::new("🦀").len().unwrap(), 4);
    }

    #[test]
    fn len_null_returns_err() {
        assert!(matches!(
            SqlString::NULL.len(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn is_empty_true_for_empty_string() {
        assert!(SqlString::new("").is_empty().unwrap());
    }

    #[test]
    fn is_empty_false_for_non_empty() {
        assert!(!SqlString::new("hello").is_empty().unwrap());
    }

    #[test]
    fn is_empty_null_returns_err() {
        assert!(matches!(
            SqlString::NULL.is_empty(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // ── T011: with_options() ─────────────────────────────────────────────────

    #[test]
    fn with_options_none_stores_none() {
        let s = SqlString::with_options("test", SqlCompareOptions::None);
        assert_eq!(s.compare_options(), SqlCompareOptions::None);
        assert_eq!(s.value().unwrap(), "test");
    }

    #[test]
    fn with_options_ignore_case() {
        let s = SqlString::with_options("test", SqlCompareOptions::IgnoreCase);
        assert_eq!(s.compare_options(), SqlCompareOptions::IgnoreCase);
    }

    #[test]
    fn with_options_binary_sort() {
        let s = SqlString::with_options("test", SqlCompareOptions::BinarySort);
        assert_eq!(s.compare_options(), SqlCompareOptions::BinarySort);
    }

    #[test]
    fn with_options_binary_sort2() {
        let s = SqlString::with_options("test", SqlCompareOptions::BinarySort2);
        assert_eq!(s.compare_options(), SqlCompareOptions::BinarySort2);
    }

    // ── T012: default options ────────────────────────────────────────────────

    #[test]
    fn new_has_ignore_case_options() {
        assert_eq!(
            SqlString::new("hello").compare_options(),
            SqlCompareOptions::IgnoreCase
        );
    }

    #[test]
    fn null_has_ignore_case_options() {
        assert_eq!(
            SqlString::NULL.compare_options(),
            SqlCompareOptions::IgnoreCase
        );
    }

    // ── T013: Add operator basic ─────────────────────────────────────────────

    #[test]
    fn add_hello_world() {
        let result = SqlString::new("hello") + SqlString::new(" world");
        assert_eq!(result.value().unwrap(), "hello world");
    }

    #[test]
    fn add_empty_plus_hello() {
        let result = SqlString::new("") + SqlString::new("hello");
        assert_eq!(result.value().unwrap(), "hello");
    }

    #[test]
    fn add_hello_plus_empty() {
        let result = SqlString::new("hello") + SqlString::new("");
        assert_eq!(result.value().unwrap(), "hello");
    }

    #[test]
    fn add_empty_plus_empty() {
        let result = SqlString::new("") + SqlString::new("");
        assert_eq!(result.value().unwrap(), "");
    }

    // ── T014: Add NULL propagation ───────────────────────────────────────────

    #[test]
    fn add_non_null_plus_null_returns_null() {
        let result = SqlString::new("hello") + SqlString::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn add_null_plus_non_null_returns_null() {
        let result = SqlString::NULL + SqlString::new("hello");
        assert!(result.is_null());
    }

    #[test]
    fn add_null_plus_null_returns_null() {
        let result = SqlString::NULL + SqlString::NULL;
        assert!(result.is_null());
    }

    // ── T015: Add options inheritance ────────────────────────────────────────

    #[test]
    fn add_inherits_left_operand_options_ignore_case_plus_binary() {
        let left = SqlString::with_options("hello", SqlCompareOptions::IgnoreCase);
        let right = SqlString::with_options(" world", SqlCompareOptions::BinarySort);
        let result = left + right;
        assert_eq!(result.compare_options(), SqlCompareOptions::IgnoreCase);
    }

    #[test]
    fn add_inherits_left_operand_options_binary_plus_ignore_case() {
        let left = SqlString::with_options("hello", SqlCompareOptions::BinarySort);
        let right = SqlString::with_options(" world", SqlCompareOptions::IgnoreCase);
        let result = left + right;
        assert_eq!(result.compare_options(), SqlCompareOptions::BinarySort);
    }

    #[test]
    fn add_null_result_inherits_left_options() {
        let left = SqlString::with_options("hello", SqlCompareOptions::None);
        let result = left + SqlString::NULL;
        assert!(result.is_null());
        assert_eq!(result.compare_options(), SqlCompareOptions::None);
    }

    // ── T017: IgnoreCase comparisons ─────────────────────────────────────────

    #[test]
    fn ignore_case_abc_eq_abc_upper() {
        let a = SqlString::new("ABC");
        let b = SqlString::new("abc");
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_apple_lt_banana() {
        let a = SqlString::new("apple");
        let b = SqlString::new("banana");
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_hello_eq_hello_mixed() {
        let a = SqlString::new("Hello");
        let b = SqlString::new("hello");
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_trailing_spaces_ignored() {
        let a = SqlString::new("hello");
        let b = SqlString::new("hello   ");
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_not_equal_different_strings() {
        let a = SqlString::new("hello");
        let b = SqlString::new("world");
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn ignore_case_greater_than() {
        let a = SqlString::new("banana");
        let b = SqlString::new("apple");
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_less_than_or_equal_same() {
        let a = SqlString::new("Hello");
        let b = SqlString::new("hello");
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ignore_case_greater_than_or_equal_same() {
        let a = SqlString::new("hello");
        let b = SqlString::new("HELLO");
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::TRUE);
    }

    // ── T018: BinarySort comparisons ─────────────────────────────────────────

    #[test]
    fn binary_sort_abc_neq_abc_upper() {
        let a = SqlString::with_options("ABC", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("abc", SqlCompareOptions::BinarySort);
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn binary_sort_upper_a_lt_lower_a() {
        // 'A' (0x41) < 'a' (0x61) in byte comparison
        let a = SqlString::with_options("A", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("a", SqlCompareOptions::BinarySort);
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn binary_sort_same_string_eq() {
        let a = SqlString::with_options("abc", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("abc", SqlCompareOptions::BinarySort);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn binary_sort2_same_as_binary_sort() {
        let a = SqlString::with_options("ABC", SqlCompareOptions::BinarySort2);
        let b = SqlString::with_options("abc", SqlCompareOptions::BinarySort2);
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn binary_sort_trailing_spaces_trimmed() {
        let a = SqlString::with_options("hello", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("hello  ", SqlCompareOptions::BinarySort);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    // ── T019: None (ordinal) comparisons ─────────────────────────────────────

    #[test]
    fn ordinal_hello_neq_hello_upper() {
        let a = SqlString::with_options("Hello", SqlCompareOptions::None);
        let b = SqlString::with_options("hello", SqlCompareOptions::None);
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn ordinal_same_string_eq() {
        let a = SqlString::with_options("abc", SqlCompareOptions::None);
        let b = SqlString::with_options("abc", SqlCompareOptions::None);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ordinal_a_lt_b() {
        let a = SqlString::with_options("A", SqlCompareOptions::None);
        let b = SqlString::with_options("B", SqlCompareOptions::None);
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn ordinal_trailing_spaces_trimmed() {
        let a = SqlString::with_options("hello", SqlCompareOptions::None);
        let b = SqlString::with_options("hello   ", SqlCompareOptions::None);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    // ── T020: NULL propagation across all 6 comparison methods ───────────────

    #[test]
    fn sql_equals_null_left() {
        assert!(
            SqlString::NULL
                .sql_equals(&SqlString::new("hello"))
                .is_null()
        );
    }

    #[test]
    fn sql_equals_null_right() {
        assert!(
            SqlString::new("hello")
                .sql_equals(&SqlString::NULL)
                .is_null()
        );
    }

    #[test]
    fn sql_equals_both_null() {
        assert!(SqlString::NULL.sql_equals(&SqlString::NULL).is_null());
    }

    #[test]
    fn sql_not_equals_null() {
        assert!(
            SqlString::NULL
                .sql_not_equals(&SqlString::new("hello"))
                .is_null()
        );
        assert!(
            SqlString::new("hello")
                .sql_not_equals(&SqlString::NULL)
                .is_null()
        );
        assert!(SqlString::NULL.sql_not_equals(&SqlString::NULL).is_null());
    }

    #[test]
    fn sql_less_than_null() {
        assert!(
            SqlString::NULL
                .sql_less_than(&SqlString::new("hello"))
                .is_null()
        );
        assert!(
            SqlString::new("hello")
                .sql_less_than(&SqlString::NULL)
                .is_null()
        );
        assert!(SqlString::NULL.sql_less_than(&SqlString::NULL).is_null());
    }

    #[test]
    fn sql_greater_than_null() {
        assert!(
            SqlString::NULL
                .sql_greater_than(&SqlString::new("hello"))
                .is_null()
        );
        assert!(
            SqlString::new("hello")
                .sql_greater_than(&SqlString::NULL)
                .is_null()
        );
        assert!(SqlString::NULL.sql_greater_than(&SqlString::NULL).is_null());
    }

    #[test]
    fn sql_less_than_or_equal_null() {
        assert!(
            SqlString::NULL
                .sql_less_than_or_equal(&SqlString::new("hello"))
                .is_null()
        );
        assert!(
            SqlString::new("hello")
                .sql_less_than_or_equal(&SqlString::NULL)
                .is_null()
        );
        assert!(
            SqlString::NULL
                .sql_less_than_or_equal(&SqlString::NULL)
                .is_null()
        );
    }

    #[test]
    fn sql_greater_than_or_equal_null() {
        assert!(
            SqlString::NULL
                .sql_greater_than_or_equal(&SqlString::new("hello"))
                .is_null()
        );
        assert!(
            SqlString::new("hello")
                .sql_greater_than_or_equal(&SqlString::NULL)
                .is_null()
        );
        assert!(
            SqlString::NULL
                .sql_greater_than_or_equal(&SqlString::NULL)
                .is_null()
        );
    }

    // ── T021: Left-operand-options-govern ────────────────────────────────────

    #[test]
    fn left_ignore_case_governs_with_binary_right() {
        // Left is IgnoreCase → should compare case-insensitively
        let left = SqlString::with_options("hello", SqlCompareOptions::IgnoreCase);
        let right = SqlString::with_options("HELLO", SqlCompareOptions::BinarySort);
        assert_eq!(left.sql_equals(&right), SqlBoolean::TRUE);
    }

    #[test]
    fn left_binary_governs_with_ignore_case_right() {
        // Left is BinarySort → should compare case-sensitively (bytes)
        let left = SqlString::with_options("hello", SqlCompareOptions::BinarySort);
        let right = SqlString::with_options("HELLO", SqlCompareOptions::IgnoreCase);
        assert_eq!(left.sql_equals(&right), SqlBoolean::FALSE);
    }

    #[test]
    fn left_none_governs_ordinal() {
        // Left is None (ordinal, case-sensitive) → "Hello" != "hello"
        let left = SqlString::with_options("Hello", SqlCompareOptions::None);
        let right = SqlString::new("hello"); // IgnoreCase
        assert_eq!(left.sql_equals(&right), SqlBoolean::FALSE);
    }

    // ── T024: End-to-end constructor-to-comparison ───────────────────────────

    #[test]
    fn e2e_none_option_case_sensitive() {
        let a = SqlString::with_options("hello", SqlCompareOptions::None);
        let b = SqlString::new("HELLO");
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn e2e_ignore_case_option() {
        let a = SqlString::with_options("hello", SqlCompareOptions::IgnoreCase);
        let b = SqlString::new("HELLO");
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn e2e_binary_sort_option() {
        let a = SqlString::with_options("hello", SqlCompareOptions::BinarySort);
        let b = SqlString::new("HELLO");
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn e2e_binary_sort2_same_as_binary_sort() {
        let a = SqlString::with_options("hello", SqlCompareOptions::BinarySort2);
        let b = SqlString::new("HELLO");
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn e2e_default_new_is_case_insensitive() {
        // new() defaults to IgnoreCase
        let a = SqlString::new("Hello");
        let b = SqlString::new("hello");
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    // ── T025: Display ────────────────────────────────────────────────────────

    #[test]
    fn display_hello() {
        assert_eq!(format!("{}", SqlString::new("hello")), "hello");
    }

    #[test]
    fn display_empty_string() {
        assert_eq!(format!("{}", SqlString::new("")), "");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlString::NULL), "Null");
    }

    // ── T026: FromStr ────────────────────────────────────────────────────────

    #[test]
    fn parse_hello() {
        let s: SqlString = "hello".parse().unwrap();
        assert_eq!(s.value().unwrap(), "hello");
    }

    #[test]
    fn parse_null_lowercase() {
        let s: SqlString = "null".parse().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn parse_null_uppercase() {
        let s: SqlString = "NULL".parse().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn parse_null_mixed_case() {
        let s: SqlString = "nUlL".parse().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn parse_null_title_case() {
        let s: SqlString = "Null".parse().unwrap();
        assert!(s.is_null());
    }

    #[test]
    fn parsed_value_has_ignore_case_options() {
        let s: SqlString = "hello".parse().unwrap();
        assert_eq!(s.compare_options(), SqlCompareOptions::IgnoreCase);
    }

    // ── T029: From<&str> and From<String> ────────────────────────────────────

    #[test]
    fn from_str_ref() {
        let s: SqlString = "hello".into();
        assert_eq!(s.value().unwrap(), "hello");
        assert_eq!(s.compare_options(), SqlCompareOptions::IgnoreCase);
    }

    #[test]
    fn from_string_owned() {
        let s: SqlString = String::from("world").into();
        assert_eq!(s.value().unwrap(), "world");
        assert_eq!(s.compare_options(), SqlCompareOptions::IgnoreCase);
    }

    // ── T030: PartialEq / Eq ─────────────────────────────────────────────────

    #[test]
    fn eq_case_insensitive() {
        assert_eq!(SqlString::new("Hello"), SqlString::new("hello"));
    }

    #[test]
    fn eq_trailing_space_trimmed() {
        assert_eq!(SqlString::new("hello"), SqlString::new("hello   "));
    }

    #[test]
    fn eq_null_equals_null() {
        assert_eq!(SqlString::NULL, SqlString::NULL);
    }

    #[test]
    fn neq_null_vs_non_null() {
        assert_ne!(SqlString::NULL, SqlString::new("hello"));
        assert_ne!(SqlString::new("hello"), SqlString::NULL);
    }

    #[test]
    fn eq_different_options_same_value() {
        let a = SqlString::with_options("hello", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("hello", SqlCompareOptions::IgnoreCase);
        assert_eq!(a, b);
    }

    #[test]
    fn eq_different_options_case_different() {
        // PartialEq always uses case-insensitive comparison
        let a = SqlString::with_options("Hello", SqlCompareOptions::BinarySort);
        let b = SqlString::with_options("hello", SqlCompareOptions::None);
        assert_eq!(a, b);
    }

    // ── T031: Hash ───────────────────────────────────────────────────────────

    #[test]
    fn hash_equal_for_case_variants() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(SqlString::new("Hello"));
        assert!(set.contains(&SqlString::new("hello")));
        assert!(set.contains(&SqlString::new("HELLO")));
    }

    #[test]
    fn hash_null_consistent() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let hash = |s: &SqlString| {
            let mut h = DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash(&SqlString::NULL), hash(&SqlString::NULL));
    }

    #[test]
    fn hash_trailing_spaces_same() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let hash = |s: &SqlString| {
            let mut h = DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        };
        assert_eq!(
            hash(&SqlString::new("hello")),
            hash(&SqlString::new("hello   "))
        );
    }

    // ── T032: PartialOrd / Ord ───────────────────────────────────────────────

    #[test]
    fn ord_null_lt_non_null() {
        assert!(SqlString::NULL < SqlString::new("hello"));
    }

    #[test]
    fn ord_non_null_gt_null() {
        assert!(SqlString::new("hello") > SqlString::NULL);
    }

    #[test]
    fn ord_null_eq_null() {
        assert_eq!(
            SqlString::NULL.cmp(&SqlString::NULL),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn ord_case_insensitive() {
        assert!(SqlString::new("apple") < SqlString::new("Banana"));
    }

    #[test]
    fn ord_equal_values_different_case() {
        assert_eq!(
            SqlString::new("Hello").cmp(&SqlString::new("hello")),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn ord_trailing_spaces() {
        assert_eq!(
            SqlString::new("hello").cmp(&SqlString::new("hello   ")),
            std::cmp::Ordering::Equal
        );
    }

    // ── to_sql_boolean() tests ───────────────────────────────────────────────

    #[test]
    fn to_sql_boolean_true() {
        let s = SqlString::new("True");
        let b = s.to_sql_boolean().unwrap();
        assert_eq!(b, SqlBoolean::TRUE);
    }

    #[test]
    fn to_sql_boolean_false() {
        let s = SqlString::new("false");
        let b = s.to_sql_boolean().unwrap();
        assert_eq!(b, SqlBoolean::FALSE);
    }

    #[test]
    fn to_sql_boolean_invalid() {
        let s = SqlString::new("maybe");
        assert!(matches!(
            s.to_sql_boolean(),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    #[test]
    fn to_sql_boolean_null() {
        let b = SqlString::NULL.to_sql_boolean().unwrap();
        assert!(b.is_null());
    }

    // ── to_sql_byte() tests ─────────────────────────────────────────────────

    #[test]
    fn to_sql_byte_valid() {
        let s = SqlString::new("200");
        let b = s.to_sql_byte().unwrap();
        assert_eq!(b.value().unwrap(), 200);
    }

    #[test]
    fn to_sql_byte_invalid() {
        let s = SqlString::new("abc");
        assert!(s.to_sql_byte().is_err());
    }

    #[test]
    fn to_sql_byte_overflow() {
        let s = SqlString::new("300");
        assert!(s.to_sql_byte().is_err());
    }

    #[test]
    fn to_sql_byte_null() {
        let b = SqlString::NULL.to_sql_byte().unwrap();
        assert!(b.is_null());
    }

    // ── to_sql_int16() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_int16_valid() {
        let s = SqlString::new("1234");
        let v = s.to_sql_int16().unwrap();
        assert_eq!(v.value().unwrap(), 1234);
    }

    #[test]
    fn to_sql_int16_negative() {
        let s = SqlString::new("-5678");
        let v = s.to_sql_int16().unwrap();
        assert_eq!(v.value().unwrap(), -5678);
    }

    #[test]
    fn to_sql_int16_invalid() {
        let s = SqlString::new("not_a_number");
        assert!(s.to_sql_int16().is_err());
    }

    #[test]
    fn to_sql_int16_null() {
        let v = SqlString::NULL.to_sql_int16().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_int32() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_int32_valid() {
        let s = SqlString::new("123456");
        let v = s.to_sql_int32().unwrap();
        assert_eq!(v.value().unwrap(), 123456);
    }

    #[test]
    fn to_sql_int32_invalid() {
        let s = SqlString::new("xyz");
        assert!(s.to_sql_int32().is_err());
    }

    #[test]
    fn to_sql_int32_null() {
        let v = SqlString::NULL.to_sql_int32().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_int64() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_int64_valid() {
        let s = SqlString::new("9876543210");
        let v = s.to_sql_int64().unwrap();
        assert_eq!(v.value().unwrap(), 9876543210);
    }

    #[test]
    fn to_sql_int64_invalid() {
        let s = SqlString::new("bad");
        assert!(s.to_sql_int64().is_err());
    }

    #[test]
    fn to_sql_int64_null() {
        let v = SqlString::NULL.to_sql_int64().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_single() tests ───────────────────────────────────────────────

    #[test]
    fn to_sql_single_valid() {
        let s = SqlString::new("3.14");
        let v = s.to_sql_single().unwrap();
        assert_eq!(v.value().unwrap(), 3.14_f32);
    }

    #[test]
    fn to_sql_single_invalid() {
        let s = SqlString::new("not_float");
        assert!(s.to_sql_single().is_err());
    }

    #[test]
    fn to_sql_single_null() {
        let v = SqlString::NULL.to_sql_single().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_double() tests ───────────────────────────────────────────────

    #[test]
    fn to_sql_double_valid() {
        let s = SqlString::new("3.14159");
        let v = s.to_sql_double().unwrap();
        assert_eq!(v.value().unwrap(), 3.14159_f64);
    }

    #[test]
    fn to_sql_double_invalid() {
        let s = SqlString::new("not_double");
        assert!(s.to_sql_double().is_err());
    }

    #[test]
    fn to_sql_double_null() {
        let v = SqlString::NULL.to_sql_double().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_decimal() tests ──────────────────────────────────────────────

    #[test]
    fn to_sql_decimal_valid() {
        let s = SqlString::new("123.45");
        let v = s.to_sql_decimal().unwrap();
        assert!(!v.is_null());
        assert_eq!(format!("{v}"), "123.45");
    }

    #[test]
    fn to_sql_decimal_invalid() {
        let s = SqlString::new("not_decimal");
        assert!(s.to_sql_decimal().is_err());
    }

    #[test]
    fn to_sql_decimal_null() {
        let v = SqlString::NULL.to_sql_decimal().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_money() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_money_valid() {
        let s = SqlString::new("100.50");
        let v = s.to_sql_money().unwrap();
        assert!(!v.is_null());
    }

    #[test]
    fn to_sql_money_invalid() {
        let s = SqlString::new("not_money");
        assert!(s.to_sql_money().is_err());
    }

    #[test]
    fn to_sql_money_null() {
        let v = SqlString::NULL.to_sql_money().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_date_time() tests ────────────────────────────────────────────

    #[test]
    fn to_sql_date_time_valid() {
        let s = SqlString::new("2025-01-15 10:30:00.000");
        let v = s.to_sql_date_time().unwrap();
        assert!(!v.is_null());
    }

    #[test]
    fn to_sql_date_time_invalid() {
        let s = SqlString::new("not_a_date");
        assert!(s.to_sql_date_time().is_err());
    }

    #[test]
    fn to_sql_date_time_null() {
        let v = SqlString::NULL.to_sql_date_time().unwrap();
        assert!(v.is_null());
    }

    // ── to_sql_guid() tests ─────────────────────────────────────────────────

    #[test]
    fn to_sql_guid_valid() {
        let s = SqlString::new("6f9619ff-8b86-d011-b42d-00cf4fc964ff");
        let v = s.to_sql_guid().unwrap();
        assert!(!v.is_null());
    }

    #[test]
    fn to_sql_guid_invalid() {
        let s = SqlString::new("not-a-guid");
        assert!(s.to_sql_guid().is_err());
    }

    #[test]
    fn to_sql_guid_null() {
        let v = SqlString::NULL.to_sql_guid().unwrap();
        assert!(v.is_null());
    }

    // ── Round-trip tests ────────────────────────────────────────────────────

    #[test]
    fn roundtrip_int32() {
        let original = SqlInt32::new(42);
        let s = original.to_sql_string();
        let back = s.to_sql_int32().unwrap();
        assert_eq!(back.value().unwrap(), 42);
    }

    #[test]
    fn roundtrip_boolean() {
        let original = SqlBoolean::TRUE;
        let s = original.to_sql_string();
        let back = s.to_sql_boolean().unwrap();
        assert!(back.is_true());
    }
}
