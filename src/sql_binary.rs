// ── SqlBinary module ─────────────────────────────────────────────────────────

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Add;

/// A nullable variable-length byte sequence equivalent to C#
/// `System.Data.SqlTypes.SqlBinary` / SQL Server `BINARY` / `VARBINARY`.
///
/// Uses `Option<Vec<u8>>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// Empty binary (`vec![]`) is a valid value, distinct from NULL.
/// Comparisons use trailing-zero-padded semantics matching C#'s
/// `PerformCompareByte`.
#[derive(Clone, Debug)]
pub struct SqlBinary {
    value: Option<Vec<u8>>,
}

// ── Constants and constructors ──────────────────────────────────────────────

impl SqlBinary {
    /// SQL NULL.
    pub const NULL: SqlBinary = SqlBinary { value: None };

    /// Creates a new `SqlBinary` taking ownership of a byte vector.
    pub fn new(v: Vec<u8>) -> Self {
        SqlBinary { value: Some(v) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the byte slice, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<&[u8], SqlTypeError> {
        self.value.as_deref().ok_or(SqlTypeError::NullValue)
    }

    /// Returns the byte count, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn len(&self) -> Result<usize, SqlTypeError> {
        self.value
            .as_ref()
            .map(|v| v.len())
            .ok_or(SqlTypeError::NullValue)
    }

    /// Returns `true` if length is 0, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn is_empty(&self) -> Result<bool, SqlTypeError> {
        self.value
            .as_ref()
            .map(|v| v.is_empty())
            .ok_or(SqlTypeError::NullValue)
    }

    /// Returns the byte at `index`, or `Err(SqlTypeError::OutOfRange)` if
    /// out-of-bounds, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn get(&self, index: usize) -> Result<u8, SqlTypeError> {
        match &self.value {
            None => Err(SqlTypeError::NullValue),
            Some(v) => v
                .get(index)
                .copied()
                .ok_or(SqlTypeError::OutOfRange(format!(
                    "Index {} is out of range for SqlBinary of length {}",
                    index,
                    v.len()
                ))),
        }
    }
}

// ── From conversions ────────────────────────────────────────────────────────

impl From<&[u8]> for SqlBinary {
    fn from(v: &[u8]) -> Self {
        SqlBinary::new(v.to_vec())
    }
}

impl From<Vec<u8>> for SqlBinary {
    fn from(v: Vec<u8>) -> Self {
        SqlBinary::new(v)
    }
}

// ── Concatenation (Add) ─────────────────────────────────────────────────────

impl Add for SqlBinary {
    type Output = SqlBinary;

    fn add(self, rhs: SqlBinary) -> SqlBinary {
        match (&self.value, &rhs.value) {
            (Some(a), Some(b)) => {
                let mut result = Vec::with_capacity(a.len() + b.len());
                result.extend_from_slice(a);
                result.extend_from_slice(b);
                SqlBinary::new(result)
            }
            _ => SqlBinary::NULL,
        }
    }
}

// ── Trailing-zero-padded comparison helper ───────────────────────────────────

/// Compare two byte slices using trailing-zero-padded semantics.
/// Shorter arrays are logically padded with zeros to match the longer length.
/// This is a direct port of C#'s `PerformCompareByte`.
fn compare_bytes(a: &[u8], b: &[u8]) -> Ordering {
    let min_len = a.len().min(b.len());

    // Compare the common prefix byte-by-byte
    for i in 0..min_len {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }

    // Prefix matched — check the tail of the longer array
    if a.len() > b.len() {
        // Check if remaining bytes in `a` are all zero
        for &byte in &a[min_len..] {
            if byte != 0 {
                return Ordering::Greater;
            }
        }
    } else if b.len() > a.len() {
        // Check if remaining bytes in `b` are all zero
        for &byte in &b[min_len..] {
            if byte != 0 {
                return Ordering::Less;
            }
        }
    }

    Ordering::Equal
}

// ── SQL comparison methods ──────────────────────────────────────────────────

impl SqlBinary {
    /// SQL equals — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) == Ordering::Equal),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL not-equals — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) != Ordering::Equal),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) == Ordering::Less),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) == Ordering::Greater),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL less-than-or-equal — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) != Ordering::Greater),
            _ => SqlBoolean::NULL,
        }
    }

    /// SQL greater-than-or-equal — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlBinary) -> SqlBoolean {
        match (&self.value, &other.value) {
            (Some(a), Some(b)) => SqlBoolean::from(compare_bytes(a, b) != Ordering::Less),
            _ => SqlBoolean::NULL,
        }
    }
}

// ── Display ─────────────────────────────────────────────────────────────────

impl fmt::Display for SqlBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            None => write!(f, "Null"),
            Some(v) => {
                for byte in v {
                    write!(f, "{:02x}", byte)?;
                }
                Ok(())
            }
        }
    }
}

// ── PartialEq / Eq ─────────────────────────────────────────────────────────

impl PartialEq for SqlBinary {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value, &other.value) {
            (None, None) => true,
            (Some(a), Some(b)) => compare_bytes(a, b) == Ordering::Equal,
            _ => false,
        }
    }
}

impl Eq for SqlBinary {}

// ── Hash ────────────────────────────────────────────────────────────────────

impl Hash for SqlBinary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.value {
            None => {
                // NULL hashes as empty slice for consistency
                let empty: &[u8] = &[];
                empty.hash(state);
            }
            Some(v) => {
                // Trim trailing zeros to ensure Hash/Eq consistency
                // [1,2,0,0] must hash the same as [1,2]
                let trimmed = trim_trailing_zeros(v);
                trimmed.hash(state);
            }
        }
    }
}

/// Trim trailing zero bytes from a slice, returning a subslice.
fn trim_trailing_zeros(v: &[u8]) -> &[u8] {
    let mut end = v.len();
    while end > 0 && v[end - 1] == 0 {
        end -= 1;
    }
    &v[..end]
}

// ── PartialOrd / Ord ────────────────────────────────────────────────────────

impl PartialOrd for SqlBinary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlBinary {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.value, &other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => compare_bytes(a, b),
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── US1: Create and inspect binary values ───────────────────────────

    // T007: Tests for new(), is_null(), value()
    #[test]
    fn new_creates_non_null() {
        let bin = SqlBinary::new(vec![1, 2, 3]);
        assert!(!bin.is_null());
        assert_eq!(bin.value().unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn null_constant_is_null() {
        assert!(SqlBinary::NULL.is_null());
    }

    #[test]
    fn null_value_returns_err() {
        let result = SqlBinary::NULL.value();
        assert!(matches!(result, Err(SqlTypeError::NullValue)));
    }

    #[test]
    fn empty_binary_is_not_null() {
        let bin = SqlBinary::new(vec![]);
        assert!(!bin.is_null());
        assert_eq!(bin.value().unwrap(), &[] as &[u8]);
    }

    // T008: Tests for len(), is_empty()
    #[test]
    fn len_returns_byte_count() {
        let bin = SqlBinary::new(vec![1, 2, 3]);
        assert_eq!(bin.len().unwrap(), 3);
    }

    #[test]
    fn len_empty_returns_zero() {
        let bin = SqlBinary::new(vec![]);
        assert_eq!(bin.len().unwrap(), 0);
    }

    #[test]
    fn len_null_returns_err() {
        assert!(matches!(
            SqlBinary::NULL.len(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn is_empty_true_for_empty() {
        let bin = SqlBinary::new(vec![]);
        assert_eq!(bin.is_empty().unwrap(), true);
    }

    #[test]
    fn is_empty_false_for_non_empty() {
        let bin = SqlBinary::new(vec![1, 2, 3]);
        assert_eq!(bin.is_empty().unwrap(), false);
    }

    #[test]
    fn is_empty_null_returns_err() {
        assert!(matches!(
            SqlBinary::NULL.is_empty(),
            Err(SqlTypeError::NullValue)
        ));
    }

    // T009: Tests for get()
    #[test]
    fn get_valid_indices() {
        let bin = SqlBinary::new(vec![10, 20, 30]);
        assert_eq!(bin.get(0).unwrap(), 10);
        assert_eq!(bin.get(1).unwrap(), 20);
        assert_eq!(bin.get(2).unwrap(), 30);
    }

    #[test]
    fn get_out_of_bounds_returns_err() {
        let bin = SqlBinary::new(vec![10, 20, 30]);
        assert!(matches!(bin.get(5), Err(SqlTypeError::OutOfRange(_))));
    }

    #[test]
    fn get_null_returns_err() {
        assert!(matches!(
            SqlBinary::NULL.get(0),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn get_empty_binary_returns_err() {
        let bin = SqlBinary::new(vec![]);
        assert!(matches!(bin.get(0), Err(SqlTypeError::OutOfRange(_))));
    }

    // T010: Tests for From<&[u8]> and From<Vec<u8>>
    #[test]
    fn from_slice_creates_non_null() {
        let a: SqlBinary = (&[10u8, 20, 30][..]).into();
        assert!(!a.is_null());
        assert_eq!(a.value().unwrap(), &[10, 20, 30]);
    }

    #[test]
    fn from_vec_creates_non_null() {
        let v = vec![0xAB, 0xCD];
        let b: SqlBinary = v.into();
        assert!(!b.is_null());
        assert_eq!(b.value().unwrap(), &[0xAB, 0xCD]);
    }

    #[test]
    fn from_empty_slice() {
        let a: SqlBinary = (&[][..]).into();
        assert!(!a.is_null());
        assert_eq!(a.len().unwrap(), 0);
    }

    // ── US2: Concatenation ──────────────────────────────────────────────

    // T012: Tests for Add operator
    #[test]
    fn add_two_binaries() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![3, 4]);
        let result = a + b;
        assert_eq!(result.value().unwrap(), &[1, 2, 3, 4]);
    }

    #[test]
    fn add_left_null_returns_null() {
        let result = SqlBinary::NULL + SqlBinary::new(vec![3, 4]);
        assert!(result.is_null());
    }

    #[test]
    fn add_right_null_returns_null() {
        let result = SqlBinary::new(vec![1, 2]) + SqlBinary::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn add_both_null_returns_null() {
        let result = SqlBinary::NULL + SqlBinary::NULL;
        assert!(result.is_null());
    }

    #[test]
    fn add_empty_left() {
        let result = SqlBinary::new(vec![]) + SqlBinary::new(vec![1, 2]);
        assert_eq!(result.value().unwrap(), &[1, 2]);
    }

    #[test]
    fn add_empty_right() {
        let result = SqlBinary::new(vec![1, 2]) + SqlBinary::new(vec![]);
        assert_eq!(result.value().unwrap(), &[1, 2]);
    }

    #[test]
    fn add_both_empty() {
        let result = SqlBinary::new(vec![]) + SqlBinary::new(vec![]);
        assert_eq!(result.value().unwrap(), &[] as &[u8]);
    }

    // ── US3: Comparison with trailing-zero padding ──────────────────────

    // T014: Trailing-zero-padded comparison edge cases
    #[test]
    fn compare_equal_with_trailing_zeros() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2, 0, 0]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_single_zero_equals_empty() {
        let a = SqlBinary::new(vec![0]);
        let b = SqlBinary::new(vec![]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_both_empty() {
        let a = SqlBinary::new(vec![]);
        let b = SqlBinary::new(vec![]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_extra_nonzero_byte_is_greater() {
        let a = SqlBinary::new(vec![1, 2, 1]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_byte_less_than() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_trailing_zero_in_middle() {
        let a = SqlBinary::new(vec![1, 2, 0]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn compare_all_zeros_equals_empty() {
        let a = SqlBinary::new(vec![0, 0, 0]);
        let b = SqlBinary::new(vec![]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    // T015: Tests for 6 SQL comparison methods
    #[test]
    fn sql_equals_equal_values() {
        let a = SqlBinary::new(vec![1, 2, 3]);
        let b = SqlBinary::new(vec![1, 2, 3]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_equals_different_values() {
        let a = SqlBinary::new(vec![1, 2, 3]);
        let b = SqlBinary::new(vec![1, 2, 4]);
        assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_not_equals_different() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![3, 4]);
        assert_eq!(a.sql_not_equals(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_not_equals_equal() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_not_equals(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_true() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_false() {
        let a = SqlBinary::new(vec![1, 3]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_less_than(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_true() {
        let a = SqlBinary::new(vec![1, 3]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_false() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_eq!(a.sql_greater_than(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_less_than_or_equal_equal() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_less() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_less_than_or_equal_greater() {
        let a = SqlBinary::new(vec![1, 3]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_less_than_or_equal(&b), SqlBoolean::FALSE);
    }

    #[test]
    fn sql_greater_than_or_equal_equal() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_greater() {
        let a = SqlBinary::new(vec![1, 3]);
        let b = SqlBinary::new(vec![1, 2]);
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::TRUE);
    }

    #[test]
    fn sql_greater_than_or_equal_less() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_eq!(a.sql_greater_than_or_equal(&b), SqlBoolean::FALSE);
    }

    // NULL propagation for all 6 comparison methods
    #[test]
    fn sql_equals_null_left() {
        assert!(
            SqlBinary::NULL
                .sql_equals(&SqlBinary::new(vec![1]))
                .is_null()
        );
    }

    #[test]
    fn sql_equals_null_right() {
        assert!(
            SqlBinary::new(vec![1])
                .sql_equals(&SqlBinary::NULL)
                .is_null()
        );
    }

    #[test]
    fn sql_equals_both_null() {
        assert!(SqlBinary::NULL.sql_equals(&SqlBinary::NULL).is_null());
    }

    #[test]
    fn sql_not_equals_null() {
        assert!(
            SqlBinary::NULL
                .sql_not_equals(&SqlBinary::new(vec![1]))
                .is_null()
        );
    }

    #[test]
    fn sql_less_than_null() {
        assert!(
            SqlBinary::NULL
                .sql_less_than(&SqlBinary::new(vec![1]))
                .is_null()
        );
    }

    #[test]
    fn sql_greater_than_null() {
        assert!(
            SqlBinary::new(vec![1])
                .sql_greater_than(&SqlBinary::NULL)
                .is_null()
        );
    }

    #[test]
    fn sql_less_than_or_equal_null() {
        assert!(
            SqlBinary::NULL
                .sql_less_than_or_equal(&SqlBinary::new(vec![1]))
                .is_null()
        );
    }

    #[test]
    fn sql_greater_than_or_equal_null() {
        assert!(
            SqlBinary::NULL
                .sql_greater_than_or_equal(&SqlBinary::new(vec![1]))
                .is_null()
        );
    }

    // ── US4: Display ────────────────────────────────────────────────────

    // T018: Tests for Display
    #[test]
    fn display_hex() {
        let bin = SqlBinary::new(vec![0x0A, 0xFF]);
        assert_eq!(format!("{}", bin), "0aff");
    }

    #[test]
    fn display_single_zero_byte() {
        let bin = SqlBinary::new(vec![0x00]);
        assert_eq!(format!("{}", bin), "00");
    }

    #[test]
    fn display_full_range() {
        let bin = SqlBinary::new(vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        assert_eq!(format!("{}", bin), "0123456789abcdef");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlBinary::NULL), "Null");
    }

    #[test]
    fn display_empty() {
        let bin = SqlBinary::new(vec![]);
        assert_eq!(format!("{}", bin), "");
    }

    // ── Polish: PartialEq / Eq ──────────────────────────────────────────

    // T020: Tests for PartialEq/Eq
    #[test]
    fn eq_equal_values() {
        let a = SqlBinary::new(vec![1, 2, 3]);
        let b = SqlBinary::new(vec![1, 2, 3]);
        assert_eq!(a, b);
    }

    #[test]
    fn eq_trailing_zeros() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2, 0, 0]);
        assert_eq!(a, b);
    }

    #[test]
    fn eq_null_null() {
        assert_eq!(SqlBinary::NULL, SqlBinary::NULL);
    }

    #[test]
    fn eq_empty_empty() {
        let a = SqlBinary::new(vec![]);
        let b = SqlBinary::new(vec![]);
        assert_eq!(a, b);
    }

    #[test]
    fn ne_different_values() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert_ne!(a, b);
    }

    #[test]
    fn ne_null_vs_value() {
        assert_ne!(SqlBinary::NULL, SqlBinary::new(vec![1]));
    }

    // T021: Tests for Hash
    #[test]
    fn hash_equal_values_hash_equal() {
        use std::collections::hash_map::DefaultHasher;

        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2, 0, 0]);

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn hash_null_consistent() {
        use std::collections::hash_map::DefaultHasher;

        let a = SqlBinary::NULL;
        let b = SqlBinary::NULL;

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn hash_zero_and_empty_equal() {
        use std::collections::hash_map::DefaultHasher;

        let a = SqlBinary::new(vec![0]);
        let b = SqlBinary::new(vec![]);

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    // T022: Tests for PartialOrd/Ord
    #[test]
    fn ord_null_less_than_value() {
        assert!(SqlBinary::NULL < SqlBinary::new(vec![1]));
    }

    #[test]
    fn ord_null_less_than_empty() {
        assert!(SqlBinary::NULL < SqlBinary::new(vec![]));
    }

    #[test]
    fn ord_null_equal_null() {
        assert_eq!(SqlBinary::NULL.cmp(&SqlBinary::NULL), Ordering::Equal);
    }

    #[test]
    fn ord_trailing_zero_equal() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 2, 0, 0]);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ord_less_than() {
        let a = SqlBinary::new(vec![1, 2]);
        let b = SqlBinary::new(vec![1, 3]);
        assert!(a < b);
    }

    #[test]
    fn ord_greater_than() {
        let a = SqlBinary::new(vec![1, 2, 1]);
        let b = SqlBinary::new(vec![1, 2]);
        assert!(a > b);
    }

    #[test]
    fn ord_empty_less_than_nonzero() {
        let a = SqlBinary::new(vec![]);
        let b = SqlBinary::new(vec![1]);
        assert!(a < b);
    }

    #[test]
    fn ord_value_greater_than_null() {
        assert!(SqlBinary::new(vec![]) > SqlBinary::NULL);
    }
}
