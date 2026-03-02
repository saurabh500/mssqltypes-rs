// Licensed under the MIT License. See LICENSE file in the project root for full license information.

// ── SqlGuid module ───────────────────────────────────────────────────────────

use crate::error::SqlTypeError;
use crate::sql_binary::SqlBinary;
use crate::sql_boolean::SqlBoolean;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// A nullable 128-bit GUID equivalent to C#
/// `System.Data.SqlTypes.SqlGuid` / SQL Server `UNIQUEIDENTIFIER`.
///
/// Uses `Option<[u8; 16]>` internally: `None` = SQL NULL, `Some(bytes)` = a value.
/// All-zeros GUID (`[0u8; 16]`) is a valid value, distinct from NULL.
///
/// **SQL Server comparison ordering**: GUIDs are compared using the non-standard
/// byte order `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` — node bytes first,
/// then clock sequence, time high, time mid, and time low last.
///
/// **Equality vs Ordering**: `PartialEq`/`Eq` use natural byte equality (matching
/// C# `Equals`). `PartialOrd`/`Ord` use SQL Server byte ordering (matching C#
/// `CompareTo`). SQL comparison methods (`sql_equals`, etc.) use SQL byte ordering
/// with NULL propagation.
#[derive(Clone, Debug)]
pub struct SqlGuid {
    value: Option<[u8; 16]>,
}

impl Copy for SqlGuid {}

// ── PartialEq, Eq (natural byte equality) ──────────────────────────────────

impl PartialEq for SqlGuid {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SqlGuid {}

// ── Hash (consistent with Eq — uses natural byte order) ─────────────────────

impl Hash for SqlGuid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

// ── PartialOrd, Ord (SQL Server byte ordering; NULL < any non-NULL) ─────────

impl PartialOrd for SqlGuid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlGuid {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.value, &other.value) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => {
                for &i in &Self::SQL_GUID_ORDER {
                    match a[i].cmp(&b[i]) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                Ordering::Equal
            }
        }
    }
}

// ── Constants and constructors ──────────────────────────────────────────────

impl SqlGuid {
    /// SQL NULL.
    pub const NULL: SqlGuid = SqlGuid { value: None };

    /// SQL Server's non-standard byte comparison order.
    /// Bytes 10–15 (node) first, then 8–9 (clock_seq), 6–7 (time_hi),
    /// 4–5 (time_mid), 0–3 (time_low) last.
    const SQL_GUID_ORDER: [usize; 16] = [10, 11, 12, 13, 14, 15, 8, 9, 6, 7, 4, 5, 0, 1, 2, 3];

    /// Creates a new `SqlGuid` from a 16-byte array.
    pub fn new(bytes: [u8; 16]) -> Self {
        SqlGuid { value: Some(bytes) }
    }

    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Returns the 16-byte array, or `Err(SqlTypeError::NullValue)` if NULL.
    pub fn value(&self) -> Result<[u8; 16], SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }

    /// Returns the 16-byte array, or `Err(SqlTypeError::NullValue)` if NULL.
    /// Alias for `value()` matching C# `ToByteArray()`.
    pub fn to_byte_array(&self) -> Result<[u8; 16], SqlTypeError> {
        self.value()
    }
}

// ── From conversions ────────────────────────────────────────────────────────

impl From<[u8; 16]> for SqlGuid {
    fn from(bytes: [u8; 16]) -> Self {
        SqlGuid::new(bytes)
    }
}

// ── SQL comparison methods ──────────────────────────────────────────────────

// ── Display & FromStr ───────────────────────────────────────────────────────

impl fmt::Display for SqlGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = match &self.value {
            None => return write!(f, "Null"),
            Some(b) => b,
        };
        // .NET mixed-endian: data1 (bytes 0-3) reversed, data2 (bytes 4-5) reversed,
        // data3 (bytes 6-7) reversed, clock_seq (bytes 8-9) as-is, node (bytes 10-15) as-is.
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[3],
            bytes[2],
            bytes[1],
            bytes[0], // data1 reversed
            bytes[5],
            bytes[4], // data2 reversed
            bytes[7],
            bytes[6], // data3 reversed
            bytes[8],
            bytes[9], // clock_seq as-is
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15], // node as-is
        )
    }
}

impl FromStr for SqlGuid {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // "Null" check (case-insensitive)
        if s.eq_ignore_ascii_case("null") {
            return Ok(SqlGuid::NULL);
        }

        // Strip hyphens for uniform processing
        let hex: String = s.chars().filter(|c| *c != '-').collect();

        // Must be exactly 32 hex chars (either 32 bare or 36 with 4 hyphens)
        if hex.len() != 32 {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid GUID string: expected 32 or 36 characters, got {}",
                s.len()
            )));
        }
        // Also validate original length is 32 or 36
        if s.len() != 32 && s.len() != 36 {
            return Err(SqlTypeError::ParseError(format!(
                "Invalid GUID string: expected 32 or 36 characters, got {}",
                s.len()
            )));
        }

        // Parse hex pairs
        let parse_byte = |offset: usize| -> Result<u8, SqlTypeError> {
            u8::from_str_radix(&hex[offset..offset + 2], 16).map_err(|_| {
                SqlTypeError::ParseError(format!("Invalid hex in GUID at offset {offset}"))
            })
        };

        // Parse all 16 bytes from the hex string, then apply .NET mixed-endian layout:
        // Group 1 (data1): hex[0..8]  → bytes[3,2,1,0] (reverse into positions 0-3)
        // Group 2 (data2): hex[8..12] → bytes[5,4] (reverse into positions 4-5)
        // Group 3 (data3): hex[12..16] → bytes[7,6] (reverse into positions 6-7)
        // Group 4 (clock_seq): hex[16..20] → bytes[8,9] (as-is)
        // Group 5 (node): hex[20..32] → bytes[10..15] (as-is)
        let mut bytes = [0u8; 16];
        bytes[3] = parse_byte(0)?;
        bytes[2] = parse_byte(2)?;
        bytes[1] = parse_byte(4)?;
        bytes[0] = parse_byte(6)?;
        bytes[5] = parse_byte(8)?;
        bytes[4] = parse_byte(10)?;
        bytes[7] = parse_byte(12)?;
        bytes[6] = parse_byte(14)?;
        bytes[8] = parse_byte(16)?;
        bytes[9] = parse_byte(18)?;
        bytes[10] = parse_byte(20)?;
        bytes[11] = parse_byte(22)?;
        bytes[12] = parse_byte(24)?;
        bytes[13] = parse_byte(26)?;
        bytes[14] = parse_byte(28)?;
        bytes[15] = parse_byte(30)?;

        Ok(SqlGuid::new(bytes))
    }
}

// ── SQL comparison methods ──────────────────────────────────────────────────

impl SqlGuid {
    /// Compares two non-null GUIDs using SQL Server's byte ordering.
    /// Returns `None` if either operand is NULL.
    fn sql_compare(&self, other: &SqlGuid) -> Option<Ordering> {
        let a = self.value.as_ref()?;
        let b = other.value.as_ref()?;
        for &i in &Self::SQL_GUID_ORDER {
            match a[i].cmp(&b[i]) {
                Ordering::Equal => continue,
                ord => return Some(ord),
            }
        }
        Some(Ordering::Equal)
    }

    /// SQL equality — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_equals(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Equal) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
        }
    }

    /// SQL inequality — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_not_equals(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Equal) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// SQL less-than — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Less) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
        }
    }

    /// SQL greater-than — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Greater) => SqlBoolean::TRUE,
            Some(_) => SqlBoolean::FALSE,
        }
    }

    /// SQL less-than-or-equal — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_less_than_or_equal(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Greater) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }

    /// SQL greater-than-or-equal — returns `SqlBoolean::NULL` if either operand is NULL.
    pub fn sql_greater_than_or_equal(&self, other: &SqlGuid) -> SqlBoolean {
        match self.sql_compare(other) {
            None => SqlBoolean::NULL,
            Some(Ordering::Less) => SqlBoolean::FALSE,
            Some(_) => SqlBoolean::TRUE,
        }
    }
}

// ── SqlBinary conversions ───────────────────────────────────────────────────

impl SqlGuid {
    /// Converts to `SqlBinary`. NULL returns `SqlBinary::NULL`.
    pub fn to_sql_binary(&self) -> SqlBinary {
        match &self.value {
            None => SqlBinary::NULL,
            Some(bytes) => SqlBinary::new(bytes.to_vec()),
        }
    }

    /// Converts from `SqlBinary`. Requires exactly 16 bytes.
    /// NULL `SqlBinary` returns `Ok(SqlGuid::NULL)`.
    pub fn from_sql_binary(binary: &SqlBinary) -> Result<SqlGuid, SqlTypeError> {
        if binary.is_null() {
            return Ok(SqlGuid::NULL);
        }
        let bytes = binary.value()?;
        if bytes.len() != 16 {
            return Err(SqlTypeError::ParseError(format!(
                "SqlBinary must be exactly 16 bytes for SqlGuid, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(SqlGuid::new(arr))
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Sample GUID bytes for testing
    const SAMPLE_BYTES: [u8; 16] = [
        0xFF, 0x19, 0x96, 0x6F, // time_low (LE for 6F9619FF)
        0x86, 0x8B, // time_mid (LE for 8B86)
        0x11, 0xD0, // time_hi (LE for D011)
        0xB4, 0x2D, // clock_seq (BE)
        0x00, 0xCF, 0x4F, 0xC9, 0x64, 0xFF, // node (BE)
    ];

    // ── US1: Create and inspect ─────────────────────────────────────────────

    #[test]
    fn new_returns_non_null_guid() {
        let guid = SqlGuid::new(SAMPLE_BYTES);
        assert!(!guid.is_null());
    }

    #[test]
    fn new_value_returns_correct_bytes() {
        let guid = SqlGuid::new(SAMPLE_BYTES);
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn null_is_null() {
        assert!(SqlGuid::NULL.is_null());
    }

    #[test]
    fn null_value_returns_err() {
        assert!(matches!(
            SqlGuid::NULL.value(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn all_zeros_guid_is_valid_not_null() {
        let guid = SqlGuid::new([0u8; 16]);
        assert!(!guid.is_null());
        assert_eq!(guid.value().unwrap(), [0u8; 16]);
    }

    #[test]
    fn to_byte_array_returns_same_as_value() {
        let guid = SqlGuid::new(SAMPLE_BYTES);
        assert_eq!(guid.to_byte_array().unwrap(), guid.value().unwrap());
    }

    #[test]
    fn to_byte_array_null_returns_err() {
        assert!(matches!(
            SqlGuid::NULL.to_byte_array(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn from_byte_array_creates_non_null() {
        let guid = SqlGuid::from(SAMPLE_BYTES);
        assert!(!guid.is_null());
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_all_zeros_creates_valid_guid() {
        let guid = SqlGuid::from([0u8; 16]);
        assert!(!guid.is_null());
    }

    // ── US2: SQL Server comparison ordering ─────────────────────────────────

    // Helper: create a GUID with all zeros except one byte set
    fn guid_with_byte(index: usize, val: u8) -> SqlGuid {
        let mut bytes = [0u8; 16];
        bytes[index] = val;
        SqlGuid::new(bytes)
    }

    // T009: sql_equals and sql_not_equals

    #[test]
    fn sql_equals_identical_returns_true() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_equals(&a).value().unwrap());
    }

    #[test]
    fn sql_equals_different_returns_false() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new([0u8; 16]);
        assert!(!a.sql_equals(&b).value().unwrap());
    }

    #[test]
    fn sql_not_equals_different_returns_true() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new([0u8; 16]);
        assert!(a.sql_not_equals(&b).value().unwrap());
    }

    #[test]
    fn sql_not_equals_identical_returns_false() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(!a.sql_not_equals(&a).value().unwrap());
    }

    #[test]
    fn sql_equals_null_left_returns_null() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(SqlGuid::NULL.sql_equals(&a).is_null());
    }

    #[test]
    fn sql_equals_null_right_returns_null() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_equals(&SqlGuid::NULL).is_null());
    }

    #[test]
    fn sql_equals_both_null_returns_null() {
        assert!(SqlGuid::NULL.sql_equals(&SqlGuid::NULL).is_null());
    }

    // T010: sql_less_than and sql_greater_than

    #[test]
    fn sql_less_than_byte10_determines_order() {
        // Byte 10 is highest priority in SQL ordering
        let a = guid_with_byte(10, 1);
        let b = guid_with_byte(10, 2);
        assert!(a.sql_less_than(&b).value().unwrap());
        assert!(b.sql_greater_than(&a).value().unwrap());
    }

    #[test]
    fn sql_less_than_byte10_wins_over_byte0() {
        // Byte 10 (high priority) vs byte 0 (low priority)
        let mut a_bytes = [0u8; 16];
        a_bytes[0] = 255; // High value in low-priority byte
        a_bytes[10] = 1; // Low value in high-priority byte
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[0] = 0; // Low value in low-priority byte
        b_bytes[10] = 2; // High value in high-priority byte
        let b = SqlGuid::new(b_bytes);

        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_less_than_byte3_last_group() {
        // Byte 3 is in the last group (time_low, lowest priority)
        let a = guid_with_byte(3, 1);
        let b = guid_with_byte(3, 2);
        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_less_than_null_propagation() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_less_than(&SqlGuid::NULL).is_null());
        assert!(SqlGuid::NULL.sql_less_than(&a).is_null());
        assert!(SqlGuid::NULL.sql_less_than(&SqlGuid::NULL).is_null());
    }

    #[test]
    fn sql_greater_than_null_propagation() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_greater_than(&SqlGuid::NULL).is_null());
        assert!(SqlGuid::NULL.sql_greater_than(&a).is_null());
    }

    // T011: sql_less_than_or_equal and sql_greater_than_or_equal

    #[test]
    fn sql_less_than_or_equal_equal_returns_true() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_less_than_or_equal(&a).value().unwrap());
    }

    #[test]
    fn sql_less_than_or_equal_less_returns_true() {
        let a = guid_with_byte(10, 1);
        let b = guid_with_byte(10, 2);
        assert!(a.sql_less_than_or_equal(&b).value().unwrap());
    }

    #[test]
    fn sql_less_than_or_equal_greater_returns_false() {
        let a = guid_with_byte(10, 2);
        let b = guid_with_byte(10, 1);
        assert!(!a.sql_less_than_or_equal(&b).value().unwrap());
    }

    #[test]
    fn sql_greater_than_or_equal_equal_returns_true() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_greater_than_or_equal(&a).value().unwrap());
    }

    #[test]
    fn sql_less_than_or_equal_null_propagation() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_less_than_or_equal(&SqlGuid::NULL).is_null());
        assert!(SqlGuid::NULL.sql_less_than_or_equal(&a).is_null());
    }

    // T012: SQL_GUID_ORDER byte-group boundary verification

    #[test]
    fn sql_order_node_bytes_first() {
        // Bytes 10-15 (node) should be compared first
        for i in 10..=15 {
            let a = guid_with_byte(i, 1);
            let b = guid_with_byte(i, 2);
            assert!(
                a.sql_less_than(&b).value().unwrap(),
                "Byte {i} should determine order"
            );
        }
    }

    #[test]
    fn sql_order_clock_seq_before_time() {
        // Byte 8 (clock_seq) should beat byte 6 (time_hi)
        let mut a_bytes = [0u8; 16];
        a_bytes[8] = 1;
        a_bytes[6] = 255;
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[8] = 2;
        b_bytes[6] = 0;
        let b = SqlGuid::new(b_bytes);

        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_order_time_hi_before_time_mid() {
        // Byte 6 (time_hi) should beat byte 4 (time_mid)
        let mut a_bytes = [0u8; 16];
        a_bytes[6] = 1;
        a_bytes[4] = 255;
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[6] = 2;
        b_bytes[4] = 0;
        let b = SqlGuid::new(b_bytes);

        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_order_time_mid_before_time_low() {
        // Byte 4 (time_mid) should beat byte 0 (time_low)
        let mut a_bytes = [0u8; 16];
        a_bytes[4] = 1;
        a_bytes[0] = 255;
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[4] = 2;
        b_bytes[0] = 0;
        let b = SqlGuid::new(b_bytes);

        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_order_full_priority_chain() {
        // Verify the complete priority: 10 > 8 > 6 > 4 > 0
        // A GUID with byte 10=1 should be less than one with byte 10=2,
        // regardless of all lower-priority bytes being 255
        let mut a_bytes = [0xFFu8; 16];
        a_bytes[10] = 1;
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[10] = 2;
        let b = SqlGuid::new(b_bytes);

        assert!(a.sql_less_than(&b).value().unwrap());
    }

    #[test]
    fn sql_order_equal_guids() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new(SAMPLE_BYTES);
        assert!(a.sql_equals(&b).value().unwrap());
        assert!(!a.sql_less_than(&b).value().unwrap());
        assert!(!a.sql_greater_than(&b).value().unwrap());
    }

    // ── US3: Display and parsing ────────────────────────────────────────────

    // T015: Display tests

    #[test]
    fn display_known_guid() {
        let guid = SqlGuid::new(SAMPLE_BYTES);
        // .NET mixed-endian: bytes[0..4] reversed, [4..6] reversed, [6..8] reversed,
        // [8..10] as-is, [10..16] as-is
        assert_eq!(guid.to_string(), "6f9619ff-8b86-d011-b42d-00cf4fc964ff");
    }

    #[test]
    fn display_null_shows_null() {
        assert_eq!(SqlGuid::NULL.to_string(), "Null");
    }

    #[test]
    fn display_all_zeros() {
        let guid = SqlGuid::new([0u8; 16]);
        assert_eq!(guid.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn display_all_ff() {
        let guid = SqlGuid::new([0xFFu8; 16]);
        assert_eq!(guid.to_string(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    // T016: FromStr tests

    #[test]
    fn from_str_hyphenated_lowercase() {
        let guid: SqlGuid = "6f9619ff-8b86-d011-b42d-00cf4fc964ff".parse().unwrap();
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_str_hyphenated_uppercase() {
        let guid: SqlGuid = "6F9619FF-8B86-D011-B42D-00CF4FC964FF".parse().unwrap();
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_str_hyphenated_mixed_case() {
        let guid: SqlGuid = "6f9619FF-8B86-d011-B42D-00cf4FC964ff".parse().unwrap();
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_str_bare_hex() {
        let guid: SqlGuid = "6f9619ff8b86d011b42d00cf4fc964ff".parse().unwrap();
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_str_null_variants() {
        let n1: SqlGuid = "Null".parse().unwrap();
        assert!(n1.is_null());
        let n2: SqlGuid = "null".parse().unwrap();
        assert!(n2.is_null());
        let n3: SqlGuid = "NULL".parse().unwrap();
        assert!(n3.is_null());
        let n4: SqlGuid = "nUlL".parse().unwrap();
        assert!(n4.is_null());
    }

    #[test]
    fn from_str_invalid_length() {
        assert!(matches!(
            "abc".parse::<SqlGuid>(),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    #[test]
    fn from_str_non_hex_chars() {
        assert!(matches!(
            "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz".parse::<SqlGuid>(),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    #[test]
    fn from_str_wrong_length_35_chars() {
        // 35 chars — neither 32 nor 36
        assert!(matches!(
            "6f9619ff-8b86-d011-b42d-00cf4fc964f".parse::<SqlGuid>(),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    // T017: Round-trip fidelity tests

    #[test]
    fn roundtrip_sample_guid() {
        let original = SqlGuid::new(SAMPLE_BYTES);
        let s = original.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert_eq!(original.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn roundtrip_all_zeros() {
        let original = SqlGuid::new([0u8; 16]);
        let s = original.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert_eq!(original.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn roundtrip_all_ff() {
        let original = SqlGuid::new([0xFFu8; 16]);
        let s = original.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert_eq!(original.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn roundtrip_sequential_bytes() {
        let bytes: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let original = SqlGuid::new(bytes);
        let s = original.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert_eq!(original.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn roundtrip_alternating_bytes() {
        let bytes: [u8; 16] = [
            0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55,
            0xAA, 0x55,
        ];
        let original = SqlGuid::new(bytes);
        let s = original.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert_eq!(original.value().unwrap(), parsed.value().unwrap());
    }

    #[test]
    fn roundtrip_null() {
        let s = SqlGuid::NULL.to_string();
        let parsed: SqlGuid = s.parse().unwrap();
        assert!(parsed.is_null());
    }

    // ── US4: SqlBinary conversions ──────────────────────────────────────────

    // T020: to_sql_binary tests

    #[test]
    fn to_sql_binary_non_null() {
        let guid = SqlGuid::new(SAMPLE_BYTES);
        let binary = guid.to_sql_binary();
        assert!(!binary.is_null());
        assert_eq!(binary.value().unwrap(), &SAMPLE_BYTES);
    }

    #[test]
    fn to_sql_binary_null() {
        let binary = SqlGuid::NULL.to_sql_binary();
        assert!(binary.is_null());
    }

    // T021: from_sql_binary tests

    #[test]
    fn from_sql_binary_16_bytes() {
        let binary = SqlBinary::new(SAMPLE_BYTES.to_vec());
        let guid = SqlGuid::from_sql_binary(&binary).unwrap();
        assert!(!guid.is_null());
        assert_eq!(guid.value().unwrap(), SAMPLE_BYTES);
    }

    #[test]
    fn from_sql_binary_null() {
        let guid = SqlGuid::from_sql_binary(&SqlBinary::NULL).unwrap();
        assert!(guid.is_null());
    }

    #[test]
    fn from_sql_binary_fewer_than_16_bytes() {
        let binary = SqlBinary::new(vec![1, 2, 3]);
        assert!(matches!(
            SqlGuid::from_sql_binary(&binary),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    #[test]
    fn from_sql_binary_more_than_16_bytes() {
        let binary = SqlBinary::new(vec![0u8; 20]);
        assert!(matches!(
            SqlGuid::from_sql_binary(&binary),
            Err(SqlTypeError::ParseError(_))
        ));
    }

    #[test]
    fn roundtrip_guid_binary_guid() {
        let original = SqlGuid::new(SAMPLE_BYTES);
        let binary = original.to_sql_binary();
        let roundtrip = SqlGuid::from_sql_binary(&binary).unwrap();
        assert_eq!(original.value().unwrap(), roundtrip.value().unwrap());
    }

    // ── Phase 7: PartialEq, Eq, Hash, PartialOrd, Ord ──────────────────────

    // T024: PartialEq / Eq tests

    #[test]
    fn eq_matching_guids() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new(SAMPLE_BYTES);
        assert_eq!(a, b);
    }

    #[test]
    fn eq_different_guids() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new([0u8; 16]);
        assert_ne!(a, b);
    }

    #[test]
    fn eq_null_equals_null() {
        // Rust trait semantics: NULL == NULL
        assert_eq!(SqlGuid::NULL, SqlGuid::NULL);
    }

    #[test]
    fn eq_all_zeros_not_null() {
        assert_ne!(SqlGuid::new([0u8; 16]), SqlGuid::NULL);
    }

    // T025: Hash tests

    #[test]
    fn hash_equal_guids_hash_equal() {
        use std::collections::hash_map::DefaultHasher;
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new(SAMPLE_BYTES);
        let hash_a = {
            let mut h = DefaultHasher::new();
            a.hash(&mut h);
            h.finish()
        };
        let hash_b = {
            let mut h = DefaultHasher::new();
            b.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn hash_null_consistent() {
        use std::collections::hash_map::DefaultHasher;
        let hash1 = {
            let mut h = DefaultHasher::new();
            SqlGuid::NULL.hash(&mut h);
            h.finish()
        };
        let hash2 = {
            let mut h = DefaultHasher::new();
            SqlGuid::NULL.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash1, hash2);
    }

    // T026: PartialOrd / Ord tests

    #[test]
    fn ord_uses_sql_byte_ordering() {
        // Byte 10 is highest priority in SQL ordering
        let a = guid_with_byte(10, 1);
        let b = guid_with_byte(10, 2);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn ord_null_less_than_any_non_null() {
        let a = SqlGuid::new([0u8; 16]);
        assert!(SqlGuid::NULL < a);
        assert!(a > SqlGuid::NULL);
    }

    #[test]
    fn ord_sql_byte_group_priority() {
        // Byte 10 (node, highest priority) vs byte 0 (time_low, lowest)
        let mut a_bytes = [0u8; 16];
        a_bytes[0] = 255;
        a_bytes[10] = 1;
        let a = SqlGuid::new(a_bytes);

        let mut b_bytes = [0u8; 16];
        b_bytes[0] = 0;
        b_bytes[10] = 2;
        let b = SqlGuid::new(b_bytes);

        assert!(a < b); // Byte 10 wins over byte 0
    }

    #[test]
    fn ord_equal_guids_equal() {
        let a = SqlGuid::new(SAMPLE_BYTES);
        let b = SqlGuid::new(SAMPLE_BYTES);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn ord_null_equals_null() {
        assert_eq!(SqlGuid::NULL.cmp(&SqlGuid::NULL), Ordering::Equal);
    }
}
