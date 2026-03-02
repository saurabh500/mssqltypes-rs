// ── T001: SqlDecimal module ────────────────────────────────────────────────────

//! `SqlDecimal` — a fixed-point decimal number with SQL NULL support, equivalent to
//! C# `System.Data.SqlTypes.SqlDecimal` / SQL Server `DECIMAL`/`NUMERIC`.
//!
//! Uses `Option<InnerDecimal>` internally: `None` = SQL NULL, `Some(v)` = a value.
//! Up to 38 digits of precision with configurable scale. Internal representation
//! uses four `u32` components forming a 128-bit unsigned mantissa in little-endian
//! order, plus sign, precision, and scale metadata.
//!
//! All arithmetic returns `Result<SqlDecimal, SqlTypeError>` with precision/scale
//! propagation per SQL Server rules and overflow detection.
//! Comparisons return `SqlBoolean` for three-valued NULL logic.

use crate::error::SqlTypeError;
use crate::sql_boolean::SqlBoolean;
use crate::sql_byte::SqlByte;
use crate::sql_double::SqlDouble;
use crate::sql_int16::SqlInt16;
use crate::sql_int32::SqlInt32;
use crate::sql_int64::SqlInt64;
use crate::sql_money::SqlMoney;
use crate::sql_single::SqlSingle;
use crate::sql_string::SqlString;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::str::FromStr;

// ── T003: Struct definitions ──────────────────────────────────────────────────

/// Internal representation of a non-NULL decimal value.
/// Stores precision, scale, sign, and a 128-bit unsigned mantissa as four `u32`
/// components in little-endian order (`data[0]` = least significant).
#[derive(Clone, Copy, Debug)]
struct InnerDecimal {
    precision: u8,
    scale: u8,
    positive: bool,
    data: [u32; 4],
}

/// A fixed-point decimal number (up to 38 digits of precision) with SQL NULL support,
/// equivalent to C# `System.Data.SqlTypes.SqlDecimal` / SQL Server `DECIMAL`/`NUMERIC`.
///
/// Uses `Option<InnerDecimal>` internally: `None` = SQL NULL, `Some(v)` = a value.
/// All arithmetic returns `Result<SqlDecimal, SqlTypeError>` with precision/scale
/// propagation per SQL Server rules and overflow detection.
/// Comparisons return `SqlBoolean` for three-valued NULL logic.
#[derive(Clone, Debug)]
pub struct SqlDecimal {
    inner: Option<InnerDecimal>,
}

// ── T004: Constants ───────────────────────────────────────────────────────────

impl SqlDecimal {
    /// SQL NULL.
    pub const NULL: SqlDecimal = SqlDecimal { inner: None };

    /// Maximum allowed precision (38 digits).
    pub const MAX_PRECISION: u8 = 38;

    /// Maximum allowed scale (38).
    pub const MAX_SCALE: u8 = 38;

    /// Returns the maximum value: 10^38 - 1 (precision=38, scale=0, positive).
    /// 99999999999999999999999999999999999999 = 0x4B3B4CA8_5A86C47A_098A223F_FFFFFFFF
    pub fn max_value() -> SqlDecimal {
        SqlDecimal {
            inner: Some(InnerDecimal {
                precision: 38,
                scale: 0,
                positive: true,
                data: [0xFFFF_FFFF, 0x098A_223F, 0x5A86_C47A, 0x4B3B_4CA8],
            }),
        }
    }

    /// Returns the minimum value: -(10^38 - 1) (precision=38, scale=0, negative).
    pub fn min_value() -> SqlDecimal {
        SqlDecimal {
            inner: Some(InnerDecimal {
                precision: 38,
                scale: 0,
                positive: false,
                data: [0xFFFF_FFFF, 0x098A_223F, 0x5A86_C47A, 0x4B3B_4CA8],
            }),
        }
    }
}

// ── T005: Private helpers ─────────────────────────────────────────────────────

/// Returns `true` if the mantissa is zero (all four words are zero).
fn is_zero(data: &[u32; 4]) -> bool {
    data[0] == 0 && data[1] == 0 && data[2] == 0 && data[3] == 0
}

/// Normalizes negative zero: if the mantissa is zero, sets positive to `true`.
fn normalize_zero(inner: &mut InnerDecimal) {
    if is_zero(&inner.data) {
        inner.positive = true;
    }
}

// ── T006: Multi-precision comparison ──────────────────────────────────────────

/// Compares two 4×u32 mantissas in little-endian order.
/// Returns `Ordering` based on unsigned magnitude comparison.
fn mp_cmp(a: &[u32; 4], b: &[u32; 4]) -> Ordering {
    // Compare from most-significant word down
    for i in (0..4).rev() {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

// ── T007: Multi-precision single-word ops ─────────────────────────────────────

/// Multiplies a 4×u32 mantissa by a scalar `u32`.
/// Returns (result, carry). Carry is non-zero if the result overflows 4 words.
fn mp_mul1(a: &[u32; 4], b: u32) -> ([u32; 4], u32) {
    let mut result = [0u32; 4];
    let mut carry: u64 = 0;
    let b = b as u64;
    for i in 0..4 {
        let prod = (a[i] as u64) * b + carry;
        result[i] = prod as u32;
        carry = prod >> 32;
    }
    (result, carry as u32)
}

/// Divides a 4×u32 mantissa by a scalar `u32`.
/// Returns (quotient, remainder).
/// Panics if `b` is zero (caller must check).
fn mp_div1(a: &[u32; 4], b: u32) -> ([u32; 4], u32) {
    let mut quotient = [0u32; 4];
    let mut remainder: u64 = 0;
    let b = b as u64;
    // Divide from most-significant word down
    for i in (0..4).rev() {
        let cur = (remainder << 32) | (a[i] as u64);
        quotient[i] = (cur / b) as u32;
        remainder = cur % b;
    }
    (quotient, remainder as u32)
}

// ── T008: Multi-precision array ops ───────────────────────────────────────────

/// Adds two 4×u32 mantissas with carry propagation.
/// Returns (result, carry). Carry is 0 or 1.
fn mp_add(a: &[u32; 4], b: &[u32; 4]) -> ([u32; 4], u32) {
    let mut result = [0u32; 4];
    let mut carry: u64 = 0;
    for i in 0..4 {
        let sum = (a[i] as u64) + (b[i] as u64) + carry;
        result[i] = sum as u32;
        carry = sum >> 32;
    }
    (result, carry as u32)
}

/// Subtracts b from a (a - b) with borrow propagation.
/// Returns (result, borrow). Borrow is 0 or 1 (1 means a < b, result is 2's complement).
fn mp_sub(a: &[u32; 4], b: &[u32; 4]) -> ([u32; 4], u32) {
    let mut result = [0u32; 4];
    let mut borrow: i64 = 0;
    for i in 0..4 {
        let diff = (a[i] as i64) - (b[i] as i64) - borrow;
        if diff < 0 {
            result[i] = (diff + (1i64 << 32)) as u32;
            borrow = 1;
        } else {
            result[i] = diff as u32;
            borrow = 0;
        }
    }
    (result, borrow as u32)
}

// ── T009: Multi-precision multiply ────────────────────────────────────────────

/// Multiplies two 4×u32 mantissas using schoolbook O(n²) algorithm.
/// Returns the full 8-word product as `[u32; 8]` in little-endian order.
///
/// The caller can check whether the result fits in 4 words by verifying
/// `result[4..8]` are all zero.
fn mp_mul(a: &[u32; 4], b: &[u32; 4]) -> [u32; 8] {
    let mut result = [0u128; 9]; // u128 to avoid overflow during accumulation

    for i in 0..4 {
        if a[i] == 0 {
            continue;
        }
        let ai = a[i] as u128;
        for j in 0..4 {
            let prod = ai * (b[j] as u128);
            result[i + j] += prod;
        }
    }

    // Propagate carries through the intermediate buffer
    for i in 0..8 {
        let carry = result[i] >> 32;
        result[i] &= 0xFFFF_FFFF;
        result[i + 1] += carry;
    }

    let mut out = [0u32; 8];
    for i in 0..8 {
        out[i] = result[i] as u32;
    }

    out
}

// ── T010: Multi-precision divide (Knuth Algorithm D) ──────────────────────────

/// Returns the number of significant (active) u32 words in a mantissa.
fn active_words(data: &[u32; 4]) -> usize {
    for i in (0..4).rev() {
        if data[i] != 0 {
            return i + 1;
        }
    }
    0 // all zeros — but callers should check is_zero first
}

/// Number of leading zeros in a u32 (for normalization in Algorithm D).
fn nlz32(x: u32) -> u32 {
    if x == 0 { 32 } else { x.leading_zeros() }
}

/// Divides a by b using Knuth's Algorithm D for multi-digit division.
/// Returns `None` if b is zero.
/// Returns `Some((quotient, remainder))` otherwise.
fn mp_div(a: &[u32; 4], b: &[u32; 4]) -> Option<([u32; 4], [u32; 4])> {
    if is_zero(b) {
        return None;
    }

    let n = active_words(b); // number of divisor words
    let m_plus_n = active_words(a); // total dividend words

    // If dividend == 0, result is 0
    if m_plus_n == 0 {
        return Some(([0; 4], [0; 4]));
    }

    // If dividend < divisor, quotient = 0, remainder = dividend
    if m_plus_n < n || (m_plus_n == n && mp_cmp(a, b) == Ordering::Less) {
        return Some(([0; 4], *a));
    }

    // Fast path: single-word divisor
    if n == 1 {
        let (q, r) = mp_div1(a, b[0]);
        let mut rem = [0u32; 4];
        rem[0] = r;
        return Some((q, rem));
    }

    // Full Algorithm D for n >= 2
    // We work with (m_plus_n + 1) word dividend and n word divisor
    // Normalize: shift so that the MSB of divisor's top word is set
    let shift = nlz32(b[n - 1]);

    // Shift divisor left by `shift` bits
    let mut v = [0u32; 4];
    if shift > 0 {
        for i in (1..n).rev() {
            v[i] = (b[i] << shift) | (b[i - 1] >> (32 - shift));
        }
        v[0] = b[0] << shift;
    } else {
        v[..n].copy_from_slice(&b[..n]);
    }

    // Shift dividend left by `shift` bits into a (m_plus_n + 1) word array
    // Use a 5-element array (max 4 words + 1 overflow)
    let mut u = [0u32; 5];
    if shift > 0 {
        u[m_plus_n] = a[m_plus_n - 1] >> (32 - shift);
        for i in (1..m_plus_n).rev() {
            u[i] = (a[i] << shift) | (a[i - 1] >> (32 - shift));
        }
        u[0] = a[0] << shift;
    } else {
        u[..m_plus_n].copy_from_slice(&a[..m_plus_n]);
    }

    let m = m_plus_n - n; // number of quotient words

    let mut q = [0u32; 4];

    for j in (0..=m).rev() {
        // Trial quotient
        let u_hi = ((u[j + n] as u64) << 32) | (u[j + n - 1] as u64);
        let mut qhat = u_hi / (v[n - 1] as u64);
        let mut rhat = u_hi % (v[n - 1] as u64);

        // Refine qhat
        loop {
            if qhat >= (1u64 << 32)
                || (n >= 2 && qhat * (v[n - 2] as u64) > ((rhat << 32) | (u[j + n - 2] as u64)))
            {
                qhat -= 1;
                rhat += v[n - 1] as u64;
                if rhat < (1u64 << 32) {
                    continue;
                }
            }
            break;
        }

        // Multiply and subtract: u[j..j+n] -= qhat * v[0..n]
        let mut borrow: i64 = 0;
        for i in 0..n {
            let prod = qhat * (v[i] as u64);
            let diff = (u[j + i] as i64) - (prod as u32 as i64) - borrow;
            u[j + i] = diff as u32;
            borrow = ((prod >> 32) as i64) - (diff >> 32);
        }
        let diff = (u[j + n] as i64) - borrow;
        u[j + n] = diff as u32;

        q[j] = qhat as u32;

        // If we subtracted too much, add back
        if diff < 0 {
            q[j] -= 1;
            let mut carry: u64 = 0;
            for i in 0..n {
                let sum = (u[j + i] as u64) + (v[i] as u64) + carry;
                u[j + i] = sum as u32;
                carry = sum >> 32;
            }
            u[j + n] = u[j + n].wrapping_add(carry as u32);
        }
    }

    // Un-normalize remainder: shift right by `shift` bits
    let mut rem = [0u32; 4];
    if shift > 0 {
        for i in 0..n - 1 {
            rem[i] = (u[i] >> shift) | (u[i + 1] << (32 - shift));
        }
        rem[n - 1] = u[n - 1] >> shift;
    } else {
        rem[..n].copy_from_slice(&u[..n]);
    }

    Some((q, rem))
}

// ── T011: calculate_precision ─────────────────────────────────────────────────

/// Counts the number of decimal digits in the mantissa.
/// Returns 0 if the mantissa is zero (callers handle this case).
fn calculate_precision(data: &[u32; 4]) -> u8 {
    if is_zero(data) {
        return 1; // zero is represented as 1 digit
    }
    let mut temp = *data;
    let mut digits: u8 = 0;
    while !is_zero(&temp) {
        let (q, _) = mp_div1(&temp, 10);
        temp = q;
        digits += 1;
    }
    digits
}

// ── T012: Constructor and factory methods ─────────────────────────────────────

impl SqlDecimal {
    /// Creates a new `SqlDecimal` from components.
    ///
    /// # Arguments
    /// * `precision` — Number of significant digits (1–38)
    /// * `scale` — Number of digits after the decimal point (0–precision)
    /// * `positive` — `true` for positive/zero, `false` for negative
    /// * `data1`..`data4` — 128-bit unsigned mantissa in little-endian order
    ///
    /// # Errors
    /// * `OutOfRange` if precision < 1 or > 38, or scale > precision
    /// * `Overflow` if the mantissa's digit count exceeds the declared precision
    pub fn new(
        precision: u8,
        scale: u8,
        positive: bool,
        data1: u32,
        data2: u32,
        data3: u32,
        data4: u32,
    ) -> Result<SqlDecimal, SqlTypeError> {
        if !(1..=Self::MAX_PRECISION).contains(&precision) {
            return Err(SqlTypeError::OutOfRange(format!(
                "Precision must be between 1 and {}, got {precision}",
                Self::MAX_PRECISION,
            )));
        }
        if scale > precision {
            return Err(SqlTypeError::OutOfRange(format!(
                "Scale ({scale}) must be <= precision ({precision})"
            )));
        }

        let data = [data1, data2, data3, data4];

        // Verify mantissa fits within declared precision
        let actual_digits = calculate_precision(&data);
        if actual_digits > precision {
            return Err(SqlTypeError::Overflow);
        }

        let mut inner = InnerDecimal {
            precision,
            scale,
            positive,
            data,
        };
        normalize_zero(&mut inner);

        Ok(SqlDecimal { inner: Some(inner) })
    }
}

// ── T013: Accessors ───────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Returns `true` if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Returns the precision, or `Err(NullValue)` if NULL.
    pub fn precision(&self) -> Result<u8, SqlTypeError> {
        self.inner
            .map(|v| v.precision)
            .ok_or(SqlTypeError::NullValue)
    }

    /// Returns the scale, or `Err(NullValue)` if NULL.
    pub fn scale(&self) -> Result<u8, SqlTypeError> {
        self.inner.map(|v| v.scale).ok_or(SqlTypeError::NullValue)
    }

    /// Returns `true` if the value is positive or zero, `false` if negative.
    /// Returns `Err(NullValue)` if NULL.
    pub fn is_positive(&self) -> Result<bool, SqlTypeError> {
        self.inner
            .map(|v| v.positive)
            .ok_or(SqlTypeError::NullValue)
    }

    /// Returns the four `u32` components of the mantissa.
    /// Returns `Err(NullValue)` if NULL.
    pub fn data(&self) -> Result<[u32; 4], SqlTypeError> {
        self.inner.map(|v| v.data).ok_or(SqlTypeError::NullValue)
    }

    /// Returns the inner decimal value, or `Err(NullValue)` if NULL.
    #[cfg(test)]
    fn value(&self) -> Result<InnerDecimal, SqlTypeError> {
        self.inner.ok_or(SqlTypeError::NullValue)
    }
}

// ── T016: adjust_scale ────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Adjusts the scale of this `SqlDecimal`.
    ///
    /// * If `new_scale > current_scale`: multiplies mantissa by powers of 10 (zero-pads).
    /// * If `new_scale < current_scale`: divides mantissa by powers of 10.
    ///   - If `round` is `true`, uses round-half-up (away from zero in magnitude).
    ///   - If `round` is `false`, truncates.
    ///
    /// Returns `Err(Overflow)` if the resulting value would exceed precision 38.
    /// NULL input returns `Ok(SqlDecimal::NULL)`.
    pub fn adjust_scale(&self, new_scale: u8, round: bool) -> Result<SqlDecimal, SqlTypeError> {
        let inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };

        if new_scale == inner.scale {
            return Ok(self.clone());
        }

        if new_scale > Self::MAX_SCALE {
            return Err(SqlTypeError::Overflow);
        }

        if new_scale > inner.scale {
            // Increase scale: multiply mantissa by 10^diff
            let diff = new_scale - inner.scale;
            let mut data = inner.data;
            for _ in 0..diff {
                let (new_data, carry) = mp_mul1(&data, 10);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = new_data;
            }

            // Check resulting precision
            let new_digits = calculate_precision(&data);
            if new_digits > Self::MAX_PRECISION {
                return Err(SqlTypeError::Overflow);
            }

            // New precision: at least enough for the digits, but at least int_part + new_scale
            let int_digits = inner.precision.saturating_sub(inner.scale);
            let new_precision = new_digits
                .max(int_digits + new_scale)
                .min(Self::MAX_PRECISION);

            let mut result = InnerDecimal {
                precision: new_precision,
                scale: new_scale,
                positive: inner.positive,
                data,
            };
            normalize_zero(&mut result);
            Ok(SqlDecimal {
                inner: Some(result),
            })
        } else {
            // Decrease scale: divide mantissa by 10^diff
            let diff = inner.scale - new_scale;
            let mut data = inner.data;
            let mut last_remainder: u32 = 0;
            let mut last_divisor: u32 = 10;

            for i in 0..diff {
                let (new_data, rem) = mp_div1(&data, 10);
                data = new_data;
                if i == diff - 1 {
                    last_remainder = rem;
                    last_divisor = 10;
                }
            }

            // Round-half-up: if remainder >= divisor/2, increment mantissa
            if round && last_remainder >= last_divisor / 2 {
                // Add 1 to mantissa
                let one = [1u32, 0, 0, 0];
                let (new_data, carry) = mp_add(&data, &one);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = new_data;
            }

            let new_digits = calculate_precision(&data);
            let int_digits = inner.precision.saturating_sub(inner.scale);
            let new_precision = new_digits
                .max(int_digits + new_scale)
                .min(Self::MAX_PRECISION);

            let mut result = InnerDecimal {
                precision: new_precision,
                scale: new_scale,
                positive: inner.positive,
                data,
            };
            normalize_zero(&mut result);
            Ok(SqlDecimal {
                inner: Some(result),
            })
        }
    }
}

// ── T027: Precision/scale computation helpers ─────────────────────────────────

/// Computes result precision and scale for addition/subtraction per SQL Server rules.
/// Returns `(precision, scale)`.
fn add_sub_result_prec_scale(p1: u8, s1: u8, p2: u8, s2: u8) -> (u8, u8) {
    let int1 = (p1 as i16) - (s1 as i16);
    let int2 = (p2 as i16) - (s2 as i16);
    let res_integer = int1.max(int2);
    let res_scale = (s1 as i16).max(s2 as i16);
    let mut res_prec = res_integer + res_scale + 1; // +1 for carry
    res_prec = res_prec.min(38);

    let mut final_scale = res_scale;
    if res_prec - res_integer < final_scale {
        final_scale = res_prec - res_integer;
    }

    (res_prec as u8, final_scale.max(0) as u8)
}

/// Computes result precision and scale for multiplication per SQL Server rules.
/// Returns `(precision, scale)`.
fn mul_result_prec_scale(p1: u8, s1: u8, p2: u8, s2: u8) -> (u8, u8) {
    let actual_scale = (s1 as i16) + (s2 as i16);
    let res_integer = ((p1 as i16) - (s1 as i16)) + ((p2 as i16) - (s2 as i16)) + 1;
    let mut res_prec = actual_scale + res_integer;

    if res_prec > 38 {
        res_prec = 38;
    }
    let capped_scale = actual_scale.min(38);

    let mut res_scale = (res_prec - res_integer).min(capped_scale);
    let min_scale = capped_scale.min(6);
    res_scale = res_scale.max(min_scale);

    // Ensure scale doesn't exceed precision
    res_scale = res_scale.min(res_prec);

    (res_prec as u8, res_scale.max(0) as u8)
}

/// Computes result precision and scale for division per SQL Server rules.
/// Returns `(precision, scale, scale_adjust)` where `scale_adjust` is how much
/// extra scale the dividend needs before division.
fn div_result_prec_scale(p1: u8, s1: u8, p2: u8, s2: u8) -> (u8, u8, i16) {
    let mut res_scale = ((s1 as i16) + (p2 as i16) + 1).max(6); // min division scale 6
    let mut res_integer = ((p1 as i16) - (s1 as i16)) + (s2 as i16);
    let min_scale = res_scale.min(6);

    res_integer = res_integer.min(38);
    let mut res_prec = res_integer + res_scale;
    if res_prec > 38 {
        res_prec = 38;
    }

    res_scale = (res_prec - res_integer).min(res_scale);
    res_scale = res_scale.max(min_scale);
    res_scale = res_scale.min(res_prec);

    let scale_adjust = res_scale - (s1 as i16) + (s2 as i16);

    (res_prec as u8, res_scale.max(0) as u8, scale_adjust)
}

// ── T028: checked_add ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Adds two SqlDecimal values with SQL Server precision/scale propagation.
    ///
    /// NULL propagation: if either operand is NULL, returns `Ok(SqlDecimal::NULL)`.
    /// Returns `Err(Overflow)` if the result exceeds 38 digits.
    pub fn checked_add(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError> {
        let lhs_inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        let rhs_inner = match rhs.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };

        let (res_prec, res_scale) = add_sub_result_prec_scale(
            lhs_inner.precision,
            lhs_inner.scale,
            rhs_inner.precision,
            rhs_inner.scale,
        );

        // Normalize to same scale (the maximum of both)
        let target_scale = lhs_inner.scale.max(rhs_inner.scale);
        let lhs_norm = self.adjust_scale(target_scale, false)?;
        let rhs_norm = rhs.adjust_scale(target_scale, false)?;
        let l = lhs_norm.inner.unwrap();
        let r = rhs_norm.inner.unwrap();

        let (data, positive) = if l.positive == r.positive {
            // Same sign: add magnitudes
            let (result, carry) = mp_add(&l.data, &r.data);
            if carry != 0 {
                return Err(SqlTypeError::Overflow);
            }
            (result, l.positive)
        } else {
            // Different signs: subtract smaller from larger
            match mp_cmp(&l.data, &r.data) {
                Ordering::Equal => ([0u32; 4], true), // zero is positive
                Ordering::Greater => {
                    let (result, _) = mp_sub(&l.data, &r.data);
                    (result, l.positive)
                }
                Ordering::Less => {
                    let (result, _) = mp_sub(&r.data, &l.data);
                    (result, r.positive)
                }
            }
        };

        // Check if result fits in target precision
        let actual_digits = calculate_precision(&data);
        let final_prec = actual_digits.max(res_prec).min(SqlDecimal::MAX_PRECISION);

        if actual_digits > SqlDecimal::MAX_PRECISION {
            return Err(SqlTypeError::Overflow);
        }

        // May need to adjust scale down if precision is capped
        let final_scale = res_scale.min(final_prec);

        let mut inner = InnerDecimal {
            precision: final_prec,
            scale: final_scale,
            positive,
            data,
        };
        normalize_zero(&mut inner);

        let result = SqlDecimal { inner: Some(inner) };

        // If the scale ended up larger than target, truncate
        if final_scale < target_scale {
            result.adjust_scale(final_scale, true)
        } else {
            Ok(result)
        }
    }
}

// ── T029: checked_sub ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Subtracts rhs from self. Delegates to `checked_add` with negated rhs.
    pub fn checked_sub(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError> {
        match rhs.inner {
            None => Ok(SqlDecimal::NULL),
            Some(mut v) => {
                if self.inner.is_none() {
                    return Ok(SqlDecimal::NULL);
                }
                v.positive = !v.positive;
                normalize_zero(&mut v);
                let neg_rhs = SqlDecimal { inner: Some(v) };
                self.checked_add(&neg_rhs)
            }
        }
    }
}

// ── T030: checked_mul ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Multiplies two SqlDecimal values with SQL Server precision/scale propagation.
    pub fn checked_mul(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError> {
        let lhs_inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        let rhs_inner = match rhs.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };

        let (res_prec, res_scale) = mul_result_prec_scale(
            lhs_inner.precision,
            lhs_inner.scale,
            rhs_inner.precision,
            rhs_inner.scale,
        );

        // Multiply mantissas (produces up to 8 words)
        let product = mp_mul(&lhs_inner.data, &rhs_inner.data);

        // The actual scale of the raw product is s1 + s2
        let actual_scale = lhs_inner.scale + rhs_inner.scale;
        let positive = lhs_inner.positive == rhs_inner.positive;

        // Reduce from 8-word product to 4-word result by dividing by 10^(actual_scale - res_scale)
        let scale_diff = actual_scale as i16 - res_scale as i16;

        // First, check if the product fits in 4 words already
        let mut data = [product[0], product[1], product[2], product[3]];
        let has_high_words =
            product[4] != 0 || product[5] != 0 || product[6] != 0 || product[7] != 0;

        if has_high_words {
            // Need to reduce: divide the full product by appropriate power of 10
            // Work with the full 8-word product
            let mut full = product;
            let reduce_count = if scale_diff > 0 { scale_diff as u32 } else { 0 };

            // We must divide away enough to fit in 4 words
            // First try the scale adjustment
            let mut remaining_scale_adj = reduce_count;
            let mut last_rem: u32 = 0;

            // Keep dividing by 10 until we fit in 4 words or run out of scale
            loop {
                let has_high = full[4] != 0 || full[5] != 0 || full[6] != 0 || full[7] != 0;
                if !has_high {
                    break;
                }
                if remaining_scale_adj == 0 {
                    // Can't reduce further — overflow
                    return Err(SqlTypeError::Overflow);
                }

                // Divide all 8 words by 10
                let mut remainder: u64 = 0;
                for i in (0..8).rev() {
                    let cur = (remainder << 32) | (full[i] as u64);
                    full[i] = (cur / 10) as u32;
                    remainder = cur % 10;
                }
                last_rem = remainder as u32;
                remaining_scale_adj -= 1;
            }

            data = [full[0], full[1], full[2], full[3]];

            // Continue dividing for remaining scale adjustment
            while remaining_scale_adj > 0 {
                let (new_data, rem) = mp_div1(&data, 10);
                last_rem = rem;
                data = new_data;
                remaining_scale_adj -= 1;
            }

            // Round-half-up
            if last_rem >= 5 {
                let one = [1u32, 0, 0, 0];
                let (new_data, carry) = mp_add(&data, &one);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = new_data;
            }
        } else if scale_diff > 0 {
            // Product fits in 4 words but need to reduce scale
            let mut last_rem: u32 = 0;
            for _ in 0..scale_diff {
                let (new_data, rem) = mp_div1(&data, 10);
                last_rem = rem;
                data = new_data;
            }
            // Round-half-up
            if last_rem >= 5 {
                let one = [1u32, 0, 0, 0];
                let (new_data, carry) = mp_add(&data, &one);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = new_data;
            }
        }

        // Check final result fits in declared precision
        let actual_digits = calculate_precision(&data);
        if actual_digits > SqlDecimal::MAX_PRECISION {
            return Err(SqlTypeError::Overflow);
        }

        let final_prec = actual_digits.max(res_prec).min(SqlDecimal::MAX_PRECISION);

        let mut inner = InnerDecimal {
            precision: final_prec,
            scale: res_scale,
            positive,
            data,
        };
        normalize_zero(&mut inner);

        Ok(SqlDecimal { inner: Some(inner) })
    }
}

// ── T031: checked_div ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Divides self by rhs with SQL Server precision/scale propagation.
    ///
    /// Returns `Err(DivideByZero)` if rhs is zero.
    /// Returns `Err(Overflow)` if the result exceeds 38 digits.
    pub fn checked_div(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError> {
        let lhs_inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        let rhs_inner = match rhs.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };

        if is_zero(&rhs_inner.data) {
            return Err(SqlTypeError::DivideByZero);
        }

        let (res_prec, res_scale, scale_adjust) = div_result_prec_scale(
            lhs_inner.precision,
            lhs_inner.scale,
            rhs_inner.precision,
            rhs_inner.scale,
        );

        // Scale up dividend to achieve desired result scale
        // We need to multiply dividend by 10^scale_adjust before integer division
        let mut dividend = lhs_inner.data;
        let mut dividend_scale = lhs_inner.scale;

        if scale_adjust > 0 {
            for _ in 0..scale_adjust {
                let (new_data, carry) = mp_mul1(&dividend, 10);
                if carry != 0 {
                    // If scaling up overflows, try doing the division in stages
                    // For now, overflow
                    return Err(SqlTypeError::Overflow);
                }
                dividend = new_data;
                dividend_scale += 1;
            }
        } else if scale_adjust < 0 {
            // Reduce dividend scale
            for _ in 0..(-scale_adjust) {
                let (new_data, _) = mp_div1(&dividend, 10);
                dividend = new_data;
                dividend_scale = dividend_scale.saturating_sub(1);
            }
        }

        // Integer division: quotient = dividend / divisor
        let (quotient, _remainder) = match mp_div(&dividend, &rhs_inner.data) {
            Some(qr) => qr,
            None => return Err(SqlTypeError::DivideByZero),
        };

        // The quotient's effective scale is: dividend_scale - rhs_scale
        // But we want res_scale, so adjust
        let quotient_scale = dividend_scale as i16 - rhs_inner.scale as i16;
        let positive = lhs_inner.positive == rhs_inner.positive;

        let actual_digits = calculate_precision(&quotient);
        if actual_digits > SqlDecimal::MAX_PRECISION {
            return Err(SqlTypeError::Overflow);
        }

        let final_prec = actual_digits.max(res_prec).min(SqlDecimal::MAX_PRECISION);
        let final_scale = if quotient_scale >= 0 {
            (quotient_scale as u8).min(res_scale)
        } else {
            0
        };

        let mut inner = InnerDecimal {
            precision: final_prec,
            scale: final_scale,
            positive,
            data: quotient,
        };
        normalize_zero(&mut inner);

        let result = SqlDecimal { inner: Some(inner) };

        // Adjust to target scale if needed
        if final_scale != res_scale {
            result.adjust_scale(res_scale, true)
        } else {
            Ok(result)
        }
    }
}

// ── T032: checked_rem ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Computes remainder (self % rhs) = self - truncate(self / rhs) * rhs.
    ///
    /// Returns `Err(DivideByZero)` if rhs is zero.
    pub fn checked_rem(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError> {
        let lhs_inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        let rhs_inner = match rhs.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };

        if is_zero(&rhs_inner.data) {
            return Err(SqlTypeError::DivideByZero);
        }

        // remainder = a - truncate(a/b) * b
        // First normalize to same scale
        let target_scale = lhs_inner.scale.max(rhs_inner.scale);
        let lhs_norm = self.adjust_scale(target_scale, false)?;
        let rhs_norm = rhs.adjust_scale(target_scale, false)?;
        let l = lhs_norm.inner.unwrap();
        let r = rhs_norm.inner.unwrap();

        // Integer-divide the mantissas
        let (quotient, _) = match mp_div(&l.data, &r.data) {
            Some(qr) => qr,
            None => return Err(SqlTypeError::DivideByZero),
        };

        // Multiply quotient back by divisor
        let product = mp_mul(&quotient, &r.data);
        // Check product fits in 4 words
        if product[4] != 0 || product[5] != 0 || product[6] != 0 || product[7] != 0 {
            return Err(SqlTypeError::Overflow);
        }
        let prod4 = [product[0], product[1], product[2], product[3]];

        // Subtract: remainder = lhs_norm - product
        let (rem_data, borrow) = mp_sub(&l.data, &prod4);
        if borrow != 0 {
            // This shouldn't happen for valid division, but guard
            return Err(SqlTypeError::Overflow);
        }

        let actual_digits = calculate_precision(&rem_data);
        let res_prec = actual_digits.clamp(1, SqlDecimal::MAX_PRECISION);

        let mut inner = InnerDecimal {
            precision: res_prec,
            scale: target_scale,
            positive: l.positive,
            data: rem_data,
        };
        normalize_zero(&mut inner);

        Ok(SqlDecimal { inner: Some(inner) })
    }
}

// ── T033: checked_neg ─────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Negates this value. NULL returns NULL.
    pub fn checked_neg(&self) -> Result<SqlDecimal, SqlTypeError> {
        match self.inner {
            None => Ok(SqlDecimal::NULL),
            Some(v) => {
                let mut inner = v;
                inner.positive = !inner.positive;
                normalize_zero(&mut inner);
                Ok(SqlDecimal { inner: Some(inner) })
            }
        }
    }
}

// ── T034: Operator traits ─────────────────────────────────────────────────────

impl Add for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn add(self, rhs: SqlDecimal) -> Self::Output {
        self.checked_add(&rhs)
    }
}

impl Add for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn add(self, rhs: &SqlDecimal) -> Self::Output {
        self.checked_add(rhs)
    }
}

impl Sub for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn sub(self, rhs: SqlDecimal) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl Sub for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn sub(self, rhs: &SqlDecimal) -> Self::Output {
        self.checked_sub(rhs)
    }
}

impl Mul for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn mul(self, rhs: SqlDecimal) -> Self::Output {
        self.checked_mul(&rhs)
    }
}

impl Mul for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn mul(self, rhs: &SqlDecimal) -> Self::Output {
        self.checked_mul(rhs)
    }
}

impl Div for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn div(self, rhs: SqlDecimal) -> Self::Output {
        self.checked_div(&rhs)
    }
}

impl Div for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn div(self, rhs: &SqlDecimal) -> Self::Output {
        self.checked_div(rhs)
    }
}

impl Rem for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn rem(self, rhs: SqlDecimal) -> Self::Output {
        self.checked_rem(&rhs)
    }
}

impl Rem for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn rem(self, rhs: &SqlDecimal) -> Self::Output {
        self.checked_rem(rhs)
    }
}

impl Neg for SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

impl Neg for &SqlDecimal {
    type Output = Result<SqlDecimal, SqlTypeError>;
    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

// ── T039: Display ─────────────────────────────────────────────────────────────

impl fmt::Display for SqlDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = match self.inner {
            None => return write!(f, "Null"),
            Some(v) => v,
        };

        if is_zero(&inner.data) {
            if !inner.positive {
                write!(f, "-")?;
            }
            if inner.scale == 0 {
                return write!(f, "0");
            }
            write!(f, "0.")?;
            for _ in 0..inner.scale {
                write!(f, "0")?;
            }
            return Ok(());
        }

        // Extract digits by repeated division by 10
        let mut digits = Vec::new();
        let mut temp = inner.data;
        while !is_zero(&temp) {
            let (q, r) = mp_div1(&temp, 10);
            digits.push(r as u8);
            temp = q;
        }
        // digits are in reverse order (least significant first)
        digits.reverse();

        // Pad with leading zeros if needed (e.g., scale > digit count)
        while digits.len() <= inner.scale as usize {
            digits.insert(0, 0);
        }

        if !inner.positive {
            write!(f, "-")?;
        }

        let int_len = digits.len() - inner.scale as usize;
        // Write integer part
        for d in &digits[..int_len] {
            write!(f, "{d}")?;
        }

        // Write fractional part
        if inner.scale > 0 {
            write!(f, ".")?;
            for d in &digits[int_len..] {
                write!(f, "{d}")?;
            }
        }

        Ok(())
    }
}

// ── T040: FromStr ─────────────────────────────────────────────────────────────

impl FromStr for SqlDecimal {
    type Err = SqlTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("null") {
            return Ok(SqlDecimal::NULL);
        }

        if s.is_empty() {
            return Err(SqlTypeError::ParseError("empty string".to_string()));
        }

        let (positive, s) = if let Some(rest) = s.strip_prefix('-') {
            (false, rest)
        } else if let Some(rest) = s.strip_prefix('+') {
            (true, rest)
        } else {
            (true, s)
        };

        if s.is_empty() {
            return Err(SqlTypeError::ParseError("no digits after sign".to_string()));
        }

        // Split on decimal point
        let (int_part, frac_part) = if let Some(dot_pos) = s.find('.') {
            (&s[..dot_pos], &s[dot_pos + 1..])
        } else {
            (s, "")
        };

        // Validate: at least one digit somewhere
        if int_part.is_empty() && frac_part.is_empty() {
            return Err(SqlTypeError::ParseError("no digits found".to_string()));
        }

        // Strip leading zeros from integer part (but keep at least one)
        let int_digits = if int_part.is_empty() {
            "0"
        } else {
            let stripped = int_part.trim_start_matches('0');
            if stripped.is_empty() { "0" } else { stripped }
        };

        // Count significant digits
        let int_sig = if int_digits == "0" {
            0
        } else {
            int_digits.len()
        };
        let scale = frac_part.len();
        let total_digits = int_sig + scale;

        if total_digits > 38 {
            return Err(SqlTypeError::Overflow);
        }
        if total_digits == 0 {
            // "0" or "0.0" etc.
            let prec = if scale > 0 { scale as u8 } else { 1 };
            return SqlDecimal::new(prec.max(1), scale as u8, true, 0, 0, 0, 0);
        }

        // Build mantissa: process all digit characters
        let mut data = [0u32; 4];
        for ch in int_digits.chars() {
            if !ch.is_ascii_digit() {
                return Err(SqlTypeError::ParseError(format!("invalid character: {ch}")));
            }
            let digit = ch as u32 - '0' as u32;
            let (new_data, carry) = mp_mul1(&data, 10);
            if carry != 0 {
                return Err(SqlTypeError::Overflow);
            }
            data = new_data;
            let digit_arr = [digit, 0, 0, 0];
            let (new_data, carry) = mp_add(&data, &digit_arr);
            if carry != 0 {
                return Err(SqlTypeError::Overflow);
            }
            data = new_data;
        }

        for ch in frac_part.chars() {
            if !ch.is_ascii_digit() {
                return Err(SqlTypeError::ParseError(format!("invalid character: {ch}")));
            }
            let digit = ch as u32 - '0' as u32;
            let (new_data, carry) = mp_mul1(&data, 10);
            if carry != 0 {
                return Err(SqlTypeError::Overflow);
            }
            data = new_data;
            let digit_arr = [digit, 0, 0, 0];
            let (new_data, carry) = mp_add(&data, &digit_arr);
            if carry != 0 {
                return Err(SqlTypeError::Overflow);
            }
            data = new_data;
        }

        let precision = total_digits.max(scale) as u8;
        let precision = precision.max(1);
        SqlDecimal::new(
            precision,
            scale as u8,
            positive,
            data[0],
            data[1],
            data[2],
            data[3],
        )
    }
}

// ── T036: SQL Comparison methods ──────────────────────────────────────────────

/// Private helper: compares two non-NULL InnerDecimals after normalizing to the same scale.
/// Returns `Ordering` (Less, Equal, Greater).
fn compare_inner(a: &InnerDecimal, b: &InnerDecimal) -> Ordering {
    // Handle sign differences first
    let a_zero = is_zero(&a.data);
    let b_zero = is_zero(&b.data);

    if a_zero && b_zero {
        return Ordering::Equal;
    }

    if a.positive && !b.positive {
        return Ordering::Greater;
    }
    if !a.positive && b.positive {
        return Ordering::Less;
    }

    // Same sign — compare magnitudes
    // Normalize to same scale
    let target_scale = a.scale.max(b.scale);
    let mut a_data = a.data;
    let mut b_data = b.data;

    if a.scale < target_scale {
        let diff = target_scale - a.scale;
        for _ in 0..diff {
            let (new_data, carry) = mp_mul1(&a_data, 10);
            if carry != 0 {
                // a has more integer digits — if positive, a > b
                return if a.positive {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
            a_data = new_data;
        }
    }
    if b.scale < target_scale {
        let diff = target_scale - b.scale;
        for _ in 0..diff {
            let (new_data, carry) = mp_mul1(&b_data, 10);
            if carry != 0 {
                return if a.positive {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            b_data = new_data;
        }
    }

    let mag_cmp = mp_cmp(&a_data, &b_data);
    if a.positive {
        mag_cmp
    } else {
        mag_cmp.reverse() // negative: larger magnitude is smaller value
    }
}

impl SqlDecimal {
    /// SQL equals: returns `SqlBoolean::TRUE` if values are mathematically equal,
    /// `SqlBoolean::NULL` if either is NULL.
    pub fn sql_equals(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) == Ordering::Equal {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }

    /// SQL not-equals.
    pub fn sql_not_equals(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) != Ordering::Equal {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }

    /// SQL less-than.
    pub fn sql_less_than(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) == Ordering::Less {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }

    /// SQL greater-than.
    pub fn sql_greater_than(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) == Ordering::Greater {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }

    /// SQL less-than-or-equal.
    pub fn sql_less_than_or_equal(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) != Ordering::Greater {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }

    /// SQL greater-than-or-equal.
    pub fn sql_greater_than_or_equal(&self, other: &SqlDecimal) -> SqlBoolean {
        match (self.inner, other.inner) {
            (None, _) | (_, None) => SqlBoolean::NULL,
            (Some(a), Some(b)) => {
                if compare_inner(&a, &b) != Ordering::Less {
                    SqlBoolean::TRUE
                } else {
                    SqlBoolean::FALSE
                }
            }
        }
    }
}

// ── T044: From<i32>, From<i64> ────────────────────────────────────────────────

impl From<i32> for SqlDecimal {
    fn from(v: i32) -> Self {
        let positive = v >= 0;
        let abs = (v as i64).unsigned_abs() as u32;
        SqlDecimal {
            inner: Some(InnerDecimal {
                precision: 10,
                scale: 0,
                positive,
                data: [abs, 0, 0, 0],
            }),
        }
    }
}

impl From<i64> for SqlDecimal {
    fn from(v: i64) -> Self {
        let positive = v >= 0;
        let abs = v.unsigned_abs();
        SqlDecimal {
            inner: Some(InnerDecimal {
                precision: 19,
                scale: 0,
                positive,
                data: [abs as u32, (abs >> 32) as u32, 0, 0],
            }),
        }
    }
}

// ── T045: From<SqlBoolean>, From<SqlByte>, From<SqlInt16/32/64> ───────────────

impl From<SqlBoolean> for SqlDecimal {
    fn from(v: SqlBoolean) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            let b = v.value().unwrap();
            SqlDecimal::from(if b { 1i32 } else { 0i32 })
        }
    }
}

impl From<SqlByte> for SqlDecimal {
    fn from(v: SqlByte) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            let val = v.value().unwrap();
            SqlDecimal {
                inner: Some(InnerDecimal {
                    precision: 3,
                    scale: 0,
                    positive: true,
                    data: [val as u32, 0, 0, 0],
                }),
            }
        }
    }
}

impl From<SqlInt16> for SqlDecimal {
    fn from(v: SqlInt16) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            let val = v.value().unwrap();
            let positive = val >= 0;
            let abs = (val as i32).unsigned_abs();
            SqlDecimal {
                inner: Some(InnerDecimal {
                    precision: 5,
                    scale: 0,
                    positive,
                    data: [abs, 0, 0, 0],
                }),
            }
        }
    }
}

impl From<SqlInt32> for SqlDecimal {
    fn from(v: SqlInt32) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            SqlDecimal::from(v.value().unwrap())
        }
    }
}

impl From<SqlInt64> for SqlDecimal {
    fn from(v: SqlInt64) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            SqlDecimal::from(v.value().unwrap())
        }
    }
}

impl From<SqlSingle> for SqlDecimal {
    /// Converts `SqlSingle` to `SqlDecimal`. NULL → NULL.
    /// # Panics
    /// Panics if the value is NaN or Infinity (matches C# OverflowException).
    fn from(v: SqlSingle) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            let f = v.value().unwrap();
            // SqlSingle already rejects NaN/Infinity on construction,
            // but guard defensively
            assert!(f.is_finite(), "Cannot convert NaN/Infinity to SqlDecimal");
            let s = format!("{f}");
            s.parse::<SqlDecimal>()
                .expect("finite f32 must parse as SqlDecimal")
        }
    }
}

impl From<SqlDouble> for SqlDecimal {
    /// Converts `SqlDouble` to `SqlDecimal`. NULL → NULL.
    /// # Panics
    /// Panics if the value is NaN or Infinity (matches C# OverflowException).
    fn from(v: SqlDouble) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            let f = v.value().unwrap();
            assert!(f.is_finite(), "Cannot convert NaN/Infinity to SqlDecimal");
            let s = format!("{f}");
            s.parse::<SqlDecimal>()
                .expect("finite f64 must parse as SqlDecimal")
        }
    }
}

impl From<SqlMoney> for SqlDecimal {
    /// Converts `SqlMoney` to `SqlDecimal`. NULL → NULL. Preserves 4-decimal scale.
    fn from(v: SqlMoney) -> Self {
        if v.is_null() {
            SqlDecimal::NULL
        } else {
            v.to_sql_decimal()
        }
    }
}

// ── T046: to_f64 ──────────────────────────────────────────────────────────────

impl SqlDecimal {
    /// Convert to `f64`. Lossy for values that cannot be exactly represented.
    /// Returns `Err(NullValue)` if NULL.
    pub fn to_f64(&self) -> Result<f64, SqlTypeError> {
        let inner = self.inner.ok_or(SqlTypeError::NullValue)?;
        let mantissa = inner.data[0] as f64
            + inner.data[1] as f64 * (u32::MAX as f64 + 1.0)
            + inner.data[2] as f64 * (u32::MAX as f64 + 1.0) * (u32::MAX as f64 + 1.0)
            + inner.data[3] as f64
                * (u32::MAX as f64 + 1.0)
                * (u32::MAX as f64 + 1.0)
                * (u32::MAX as f64 + 1.0);
        let divisor = 10f64.powi(inner.scale as i32);
        let result = mantissa / divisor;
        if inner.positive {
            Ok(result)
        } else {
            Ok(-result)
        }
    }

    // ── T047: to_sql_int32, to_sql_int64, to_sql_int16, to_sql_byte, to_sql_boolean ─

    /// Convert to `SqlInt32`. Truncates fractional part, checks range.
    /// NULL → `Ok(SqlInt32::NULL)`.
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlInt32::NULL);
        }
        let truncated = self.adjust_scale(0, false)?;
        let inner = truncated.inner.unwrap();
        // Check if value fits in i32
        if inner.data[1] != 0 || inner.data[2] != 0 || inner.data[3] != 0 {
            return Err(SqlTypeError::Overflow);
        }
        let val = inner.data[0];
        if inner.positive {
            if val > i32::MAX as u32 {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt32::new(val as i32))
        } else {
            // i32::MIN abs is 2147483648 which is > i32::MAX
            if val > (i32::MIN as i64).unsigned_abs() as u32 {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt32::new(-(val as i64) as i32))
        }
    }

    /// Convert to `SqlInt64`. Truncates fractional part, checks range.
    /// NULL → `Ok(SqlInt64::NULL)`.
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlInt64::NULL);
        }
        let truncated = self.adjust_scale(0, false)?;
        let inner = truncated.inner.unwrap();
        if inner.data[2] != 0 || inner.data[3] != 0 {
            return Err(SqlTypeError::Overflow);
        }
        let val = (inner.data[1] as u64) << 32 | inner.data[0] as u64;
        if inner.positive {
            if val > i64::MAX as u64 {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt64::new(val as i64))
        } else {
            // i64::MIN abs is 9223372036854775808
            if val > (i64::MIN as i128).unsigned_abs() as u64 {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt64::new(-(val as i128) as i64))
        }
    }

    /// Convert to `SqlInt16`. Truncates fractional part, checks range.
    /// NULL → `Ok(SqlInt16::NULL)`.
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlInt16::NULL);
        }
        let truncated = self.adjust_scale(0, false)?;
        let inner = truncated.inner.unwrap();
        if inner.data[1] != 0 || inner.data[2] != 0 || inner.data[3] != 0 {
            return Err(SqlTypeError::Overflow);
        }
        let val = inner.data[0];
        if inner.positive {
            if val > i16::MAX as u32 {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt16::new(val as i16))
        } else {
            if val > (i16::MIN as i32).unsigned_abs() {
                return Err(SqlTypeError::Overflow);
            }
            Ok(SqlInt16::new(-(val as i32) as i16))
        }
    }

    /// Convert to `SqlByte`. Truncates fractional part, checks range.
    /// NULL → `Ok(SqlByte::NULL)`.
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlByte::NULL);
        }
        let truncated = self.adjust_scale(0, false)?;
        let inner = truncated.inner.unwrap();
        if !inner.positive && !is_zero(&inner.data) {
            return Err(SqlTypeError::Overflow);
        }
        if inner.data[1] != 0 || inner.data[2] != 0 || inner.data[3] != 0 {
            return Err(SqlTypeError::Overflow);
        }
        if inner.data[0] > u8::MAX as u32 {
            return Err(SqlTypeError::Overflow);
        }
        Ok(SqlByte::new(inner.data[0] as u8))
    }

    /// Convert to `SqlBoolean`. 0→FALSE, non-0→TRUE, NULL→NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean {
        match self.inner {
            None => SqlBoolean::NULL,
            Some(inner) => {
                if is_zero(&inner.data) {
                    SqlBoolean::new(false)
                } else {
                    SqlBoolean::new(true)
                }
            }
        }
    }
}

impl SqlDecimal {
    /// Converts to `SqlString` via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString {
        if self.is_null() {
            SqlString::NULL
        } else {
            SqlString::new(&format!("{self}"))
        }
    }
}

impl SqlDecimal {
    /// Converts to `SqlSingle`. NULL → NULL. May lose precision.
    pub fn to_sql_single(&self) -> SqlSingle {
        if self.is_null() {
            SqlSingle::NULL
        } else {
            let f = self.to_f64().unwrap() as f32;
            SqlSingle::new(f).unwrap_or(SqlSingle::NULL)
        }
    }

    /// Converts to `SqlDouble`. NULL → NULL. May lose precision.
    pub fn to_sql_double(&self) -> SqlDouble {
        if self.is_null() {
            SqlDouble::NULL
        } else {
            let f = self.to_f64().unwrap();
            SqlDouble::new(f).unwrap_or(SqlDouble::NULL)
        }
    }

    /// Converts to `SqlMoney`. NULL → NULL.
    /// Returns `Err(Overflow)` if the value is outside the money range.
    pub fn to_sql_money(&self) -> Result<SqlMoney, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlMoney::NULL);
        }
        // Convert decimal to f64, then to i64×10000
        let f = self.to_f64()?;
        let scaled = f * 10_000.0;
        if scaled > i64::MAX as f64 || scaled < i64::MIN as f64 || !scaled.is_finite() {
            return Err(SqlTypeError::Overflow);
        }
        let raw = scaled.round() as i64;
        Ok(SqlMoney::from_scaled(raw))
    }
}

// ── T050-T051: US7 Math Functions ─────────────────────────────────────────────

impl SqlDecimal {
    /// Absolute value. NULL → NULL.
    pub fn abs(&self) -> SqlDecimal {
        match self.inner {
            None => SqlDecimal::NULL,
            Some(mut inner) => {
                inner.positive = true;
                SqlDecimal { inner: Some(inner) }
            }
        }
    }

    /// Floor (round toward negative infinity). NULL → NULL.
    pub fn floor(&self) -> Result<SqlDecimal, SqlTypeError> {
        let inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        if inner.scale == 0 {
            return Ok(self.clone());
        }
        let truncated = self.adjust_scale(0, false)?;
        let trunc_inner = truncated.inner.unwrap();
        // If negative and there was a fractional part, floor = truncated - 1
        if !inner.positive && !is_zero(&inner.data) {
            // Check if there was actual fractional content
            // Compare truncated * 10^scale back to original
            let mut scaled_back = trunc_inner.data;
            for _ in 0..inner.scale {
                let (result, carry) = mp_mul1(&scaled_back, 10);
                if carry != 0 {
                    // Very unlikely but safe: original had fractional
                    break;
                }
                scaled_back = result;
            }
            if mp_cmp(&scaled_back, &inner.data) != Ordering::Equal {
                // Had fractional part, add 1 to magnitude (floor goes more negative)
                let (result, carry) = mp_add(&trunc_inner.data, &[1, 0, 0, 0]);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                let prec = calculate_precision(&result);
                if prec > 38 {
                    return Err(SqlTypeError::Overflow);
                }
                let new_inner = InnerDecimal {
                    precision: if prec == 0 { 1 } else { prec },
                    scale: 0,
                    positive: false,
                    data: result,
                };
                return Ok(SqlDecimal {
                    inner: Some(new_inner),
                });
            }
        }
        Ok(truncated)
    }

    /// Ceiling (round toward positive infinity). NULL → NULL.
    pub fn ceiling(&self) -> Result<SqlDecimal, SqlTypeError> {
        let inner = match self.inner {
            None => return Ok(SqlDecimal::NULL),
            Some(v) => v,
        };
        if inner.scale == 0 {
            return Ok(self.clone());
        }
        let truncated = self.adjust_scale(0, false)?;
        let trunc_inner = truncated.inner.unwrap();
        // If positive and there was a fractional part, ceiling = truncated + 1
        if inner.positive && !is_zero(&inner.data) {
            let mut scaled_back = trunc_inner.data;
            for _ in 0..inner.scale {
                let (result, carry) = mp_mul1(&scaled_back, 10);
                if carry != 0 {
                    break;
                }
                scaled_back = result;
            }
            if mp_cmp(&scaled_back, &inner.data) != Ordering::Equal {
                // Had fractional part, add 1
                let (result, carry) = mp_add(&trunc_inner.data, &[1, 0, 0, 0]);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                let prec = calculate_precision(&result);
                if prec > 38 {
                    return Err(SqlTypeError::Overflow);
                }
                let new_inner = InnerDecimal {
                    precision: if prec == 0 { 1 } else { prec },
                    scale: 0,
                    positive: true,
                    data: result,
                };
                return Ok(SqlDecimal {
                    inner: Some(new_inner),
                });
            }
        }
        Ok(truncated)
    }

    /// Round to the given number of decimal places. NULL → NULL.
    pub fn round(&self, position: i32) -> Result<SqlDecimal, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlDecimal::NULL);
        }
        let inner = self.inner.unwrap();
        if position < 0 {
            // Round to the left of decimal point
            let shift = (-position) as u8;
            // First truncate to scale 0
            let truncated = self.adjust_scale(0, false)?;
            let mut data = truncated.inner.unwrap().data;
            // Divide by 10^shift
            let mut remainder = 0u32;
            for _ in 0..shift {
                let (q, r) = mp_div1(&data, 10);
                remainder = r;
                data = q;
            }
            // Round half up
            if remainder >= 5 {
                let (res, carry) = mp_add(&data, &[1, 0, 0, 0]);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = res;
            }
            // Multiply back
            for _ in 0..shift {
                let (res, carry) = mp_mul1(&data, 10);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = res;
            }
            let prec = calculate_precision(&data).max(1);
            if prec > 38 {
                return Err(SqlTypeError::Overflow);
            }
            let mut new_inner = InnerDecimal {
                precision: prec,
                scale: 0,
                positive: inner.positive,
                data,
            };
            normalize_zero(&mut new_inner);
            Ok(SqlDecimal {
                inner: Some(new_inner),
            })
        } else {
            let target_scale = position as u8;
            if target_scale >= inner.scale {
                return Ok(self.clone());
            }
            self.adjust_scale(target_scale, true)
        }
    }

    /// Truncate to the given number of decimal places. NULL → NULL.
    pub fn truncate(&self, position: i32) -> Result<SqlDecimal, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlDecimal::NULL);
        }
        let inner = self.inner.unwrap();
        if position < 0 {
            let shift = (-position) as u8;
            let truncated = self.adjust_scale(0, false)?;
            let mut data = truncated.inner.unwrap().data;
            for _ in 0..shift {
                let (q, _r) = mp_div1(&data, 10);
                data = q;
            }
            for _ in 0..shift {
                let (res, carry) = mp_mul1(&data, 10);
                if carry != 0 {
                    return Err(SqlTypeError::Overflow);
                }
                data = res;
            }
            let prec = calculate_precision(&data).max(1);
            let mut new_inner = InnerDecimal {
                precision: prec,
                scale: 0,
                positive: inner.positive,
                data,
            };
            normalize_zero(&mut new_inner);
            Ok(SqlDecimal {
                inner: Some(new_inner),
            })
        } else {
            let target_scale = position as u8;
            if target_scale >= inner.scale {
                return Ok(self.clone());
            }
            self.adjust_scale(target_scale, false)
        }
    }

    /// Returns -1, 0, or 1 indicating the sign. NULL → SqlInt32::NULL.
    pub fn sign(&self) -> SqlInt32 {
        match self.inner {
            None => SqlInt32::NULL,
            Some(inner) => {
                if is_zero(&inner.data) {
                    SqlInt32::new(0)
                } else if inner.positive {
                    SqlInt32::new(1)
                } else {
                    SqlInt32::new(-1)
                }
            }
        }
    }

    /// Raise to an integer power. NULL → NULL.
    pub fn power(&self, exponent: i32) -> Result<SqlDecimal, SqlTypeError> {
        if self.is_null() {
            return Ok(SqlDecimal::NULL);
        }
        let base = self.to_f64()?;
        let result = base.powi(exponent);
        if result.is_nan() || result.is_infinite() {
            return Err(SqlTypeError::Overflow);
        }
        // Convert f64 back to SqlDecimal
        // Use string round-trip for accuracy
        let s = if result == 0.0 {
            "0".to_string()
        } else {
            // Format with enough decimal places
            let inner = self.inner.unwrap();
            let result_scale = (inner.scale as i32 * exponent).clamp(0, 38) as usize;
            if result_scale == 0 {
                format!("{:.0}", result)
            } else {
                format!("{:.prec$}", result, prec = result_scale.min(6))
            }
        };
        s.parse::<SqlDecimal>().map_err(|_| SqlTypeError::Overflow)
    }
}

// ── T054: PartialEq, Eq, Hash ─────────────────────────────────────────────────

impl PartialEq for SqlDecimal {
    fn eq(&self, other: &Self) -> bool {
        match (self.inner, other.inner) {
            (None, None) => true, // NULL == NULL per Rust semantics
            (None, Some(_)) | (Some(_), None) => false,
            (Some(a), Some(b)) => compare_inner(&a, &b) == Ordering::Equal,
        }
    }
}

impl Eq for SqlDecimal {}

impl Hash for SqlDecimal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.inner {
            None => {
                // Use a discriminant for NULL
                0u8.hash(state);
            }
            Some(inner) => {
                1u8.hash(state);
                // Normalize to canonical form: strip trailing fractional zeros
                let mut data = inner.data;
                let mut scale = inner.scale;
                while scale > 0 {
                    let (q, r) = mp_div1(&data, 10);
                    if r != 0 {
                        break;
                    }
                    data = q;
                    scale -= 1;
                }
                // Hash sign (but positive zero == negative zero)
                if is_zero(&data) {
                    true.hash(state);
                } else {
                    inner.positive.hash(state);
                }
                scale.hash(state);
                data.hash(state);
            }
        }
    }
}

// ── T055: PartialOrd, Ord ─────────────────────────────────────────────────────

impl PartialOrd for SqlDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SqlDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.inner, other.inner) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less, // NULL < any value
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => compare_inner(&a, &b),
        }
    }
}

// ── T014-T015: Private multi-precision helper tests ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── mp_cmp tests ──────────────────────────────────────────────────────

    #[test]
    fn mp_cmp_equal() {
        let a = [1, 2, 3, 4];
        assert_eq!(mp_cmp(&a, &a), Ordering::Equal);
    }

    #[test]
    fn mp_cmp_less_high_word() {
        let a = [1, 2, 3, 4];
        let b = [1, 2, 3, 5];
        assert_eq!(mp_cmp(&a, &b), Ordering::Less);
    }

    #[test]
    fn mp_cmp_greater_low_word() {
        let a = [5, 0, 0, 0];
        let b = [3, 0, 0, 0];
        assert_eq!(mp_cmp(&a, &b), Ordering::Greater);
    }

    #[test]
    fn mp_cmp_zeros() {
        assert_eq!(mp_cmp(&[0; 4], &[0; 4]), Ordering::Equal);
    }

    // ── mp_add tests ──────────────────────────────────────────────────────

    #[test]
    fn mp_add_no_carry() {
        let a = [1, 0, 0, 0];
        let b = [2, 0, 0, 0];
        let (result, carry) = mp_add(&a, &b);
        assert_eq!(result, [3, 0, 0, 0]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mp_add_with_carry() {
        let a = [u32::MAX, 0, 0, 0];
        let b = [1, 0, 0, 0];
        let (result, carry) = mp_add(&a, &b);
        assert_eq!(result, [0, 1, 0, 0]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mp_add_full_carry_out() {
        let a = [u32::MAX; 4];
        let b = [1, 0, 0, 0];
        let (result, carry) = mp_add(&a, &b);
        assert_eq!(result, [0, 0, 0, 0]);
        assert_eq!(carry, 1);
    }

    // ── mp_sub tests ──────────────────────────────────────────────────────

    #[test]
    fn mp_sub_no_borrow() {
        let a = [5, 0, 0, 0];
        let b = [3, 0, 0, 0];
        let (result, borrow) = mp_sub(&a, &b);
        assert_eq!(result, [2, 0, 0, 0]);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn mp_sub_with_borrow() {
        let a = [0, 1, 0, 0];
        let b = [1, 0, 0, 0];
        let (result, borrow) = mp_sub(&a, &b);
        assert_eq!(result, [u32::MAX, 0, 0, 0]);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn mp_sub_underflow() {
        let a = [0, 0, 0, 0];
        let b = [1, 0, 0, 0];
        let (_, borrow) = mp_sub(&a, &b);
        assert_eq!(borrow, 1);
    }

    // ── mp_mul1 tests ─────────────────────────────────────────────────────

    #[test]
    fn mp_mul1_simple() {
        let a = [5, 0, 0, 0];
        let (result, carry) = mp_mul1(&a, 3);
        assert_eq!(result, [15, 0, 0, 0]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mp_mul1_with_carry() {
        let a = [u32::MAX, 0, 0, 0];
        let (result, carry) = mp_mul1(&a, 2);
        assert_eq!(result, [u32::MAX - 1, 1, 0, 0]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn mp_mul1_overflow() {
        let a = [u32::MAX; 4];
        let (_, carry) = mp_mul1(&a, 2);
        assert!(carry > 0);
    }

    // ── mp_div1 tests ─────────────────────────────────────────────────────

    #[test]
    fn mp_div1_exact() {
        let a = [15, 0, 0, 0];
        let (result, rem) = mp_div1(&a, 5);
        assert_eq!(result, [3, 0, 0, 0]);
        assert_eq!(rem, 0);
    }

    #[test]
    fn mp_div1_with_remainder() {
        let a = [16, 0, 0, 0];
        let (result, rem) = mp_div1(&a, 5);
        assert_eq!(result, [3, 0, 0, 0]);
        assert_eq!(rem, 1);
    }

    #[test]
    fn mp_div1_multi_word() {
        // 0x1_0000_0000 = 4_294_967_296 / 2 = 2_147_483_648
        let a = [0, 1, 0, 0];
        let (result, rem) = mp_div1(&a, 2);
        assert_eq!(result, [0x8000_0000, 0, 0, 0]);
        assert_eq!(rem, 0);
    }

    // ── mp_mul tests ──────────────────────────────────────────────────────

    #[test]
    fn mp_mul_simple() {
        let a = [6, 0, 0, 0];
        let b = [7, 0, 0, 0];
        let result = mp_mul(&a, &b);
        assert_eq!(result[0], 42);
        assert_eq!(result[1..], [0; 7]);
    }

    #[test]
    fn mp_mul_large() {
        // 1_000_000_000 * 1_000_000_000 = 1_000_000_000_000_000_000
        // = 0x0DE0_B6B3_A764_0000
        let a = [1_000_000_000, 0, 0, 0];
        let b = [1_000_000_000, 0, 0, 0];
        let result = mp_mul(&a, &b);
        assert_eq!(result[0], 0xA764_0000);
        assert_eq!(result[1], 0x0DE0_B6B3);
        assert_eq!(result[2..], [0; 6]);
    }

    #[test]
    fn mp_mul_overflow_4_words() {
        // (2^128 - 1) * 2 = 2^129 - 2: overflows 4 words into word[4]
        let a = [u32::MAX, u32::MAX, u32::MAX, u32::MAX];
        let b = [2, 0, 0, 0];
        let result = mp_mul(&a, &b);
        assert_eq!(result[4], 1); // 5th word non-zero
    }

    #[test]
    fn mp_mul_large_product() {
        // (2^128 - 1)^2 should fill many words
        let a = [u32::MAX; 4];
        let result = mp_mul(&a, &a);
        // result[7] should be non-zero for such a large product
        assert_ne!(result[7], 0);
    }

    // ── mp_div tests ──────────────────────────────────────────────────────

    #[test]
    fn mp_div_divide_by_zero() {
        let a = [100, 0, 0, 0];
        let b = [0; 4];
        assert!(mp_div(&a, &b).is_none());
    }

    #[test]
    fn mp_div_exact() {
        let a = [42, 0, 0, 0];
        let b = [6, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [7, 0, 0, 0]);
        assert_eq!(r, [0; 4]);
    }

    #[test]
    fn mp_div_with_remainder() {
        let a = [43, 0, 0, 0];
        let b = [6, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [7, 0, 0, 0]);
        assert_eq!(r, [1, 0, 0, 0]);
    }

    #[test]
    fn mp_div_single_word_fast_path() {
        let a = [100, 0, 0, 0];
        let b = [10, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [10, 0, 0, 0]);
        assert_eq!(r, [0; 4]);
    }

    #[test]
    fn mp_div_dividend_less_than_divisor() {
        let a = [5, 0, 0, 0];
        let b = [10, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [0; 4]);
        assert_eq!(r, [5, 0, 0, 0]);
    }

    #[test]
    fn mp_div_multi_word() {
        // 1_000_000_000_000_000_000 / 1_000_000_000 = 1_000_000_000
        let a = [0xA764_0000, 0x0DE0_B6B3, 0, 0];
        let b = [1_000_000_000, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [1_000_000_000, 0, 0, 0]);
        assert_eq!(r, [0; 4]);
    }

    #[test]
    fn mp_div_zero_dividend() {
        let a = [0; 4];
        let b = [5, 0, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [0; 4]);
        assert_eq!(r, [0; 4]);
    }

    // ── calculate_precision tests ─────────────────────────────────────────

    #[test]
    fn calculate_precision_zero() {
        assert_eq!(calculate_precision(&[0; 4]), 1);
    }

    #[test]
    fn calculate_precision_single_digit() {
        assert_eq!(calculate_precision(&[9, 0, 0, 0]), 1);
    }

    #[test]
    fn calculate_precision_two_digits() {
        assert_eq!(calculate_precision(&[99, 0, 0, 0]), 2);
    }

    #[test]
    fn calculate_precision_ten_digits() {
        // 1_000_000_000
        assert_eq!(calculate_precision(&[1_000_000_000, 0, 0, 0]), 10);
    }

    #[test]
    fn calculate_precision_max_value() {
        // 10^38 - 1 = 0x4B3B4CA8_5A86C47A_098A223F_FFFFFFFF
        let data = [0xFFFF_FFFF, 0x098A_223F, 0x5A86_C47A, 0x4B3B_4CA8];
        assert_eq!(calculate_precision(&data), 38);
    }

    // ── is_zero tests ─────────────────────────────────────────────────────

    #[test]
    fn is_zero_true() {
        assert!(is_zero(&[0; 4]));
    }

    #[test]
    fn is_zero_false() {
        assert!(!is_zero(&[1, 0, 0, 0]));
        assert!(!is_zero(&[0, 0, 0, 1]));
    }

    // ── mp_div multi-word divisor test ─────────────────────────────────────

    #[test]
    fn mp_div_two_word_divisor() {
        // Dividend: 2^32 * 10 = [0, 10, 0, 0]
        // Divisor: [0, 2, 0, 0] = 2 * 2^32
        // Quotient should be 5, remainder 0
        let a = [0, 10, 0, 0];
        let b = [0, 2, 0, 0];
        let (q, r) = mp_div(&a, &b).unwrap();
        assert_eq!(q, [5, 0, 0, 0]);
        assert_eq!(r, [0; 4]);
    }

    // ── T017: US1 new() tests ─────────────────────────────────────────────

    #[test]
    fn new_valid_construction() {
        // 12345 with precision=10, scale=2 represents 123.45
        let d = SqlDecimal::new(10, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(!d.is_null());
        assert_eq!(d.precision().unwrap(), 10);
        assert_eq!(d.scale().unwrap(), 2);
        assert!(d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn new_negative_value() {
        let d = SqlDecimal::new(10, 2, false, 12345, 0, 0, 0).unwrap();
        assert!(!d.is_positive().unwrap());
    }

    #[test]
    fn new_precision_zero_returns_out_of_range() {
        let err = SqlDecimal::new(0, 0, true, 1, 0, 0, 0).unwrap_err();
        assert!(matches!(err, SqlTypeError::OutOfRange(_)));
    }

    #[test]
    fn new_precision_39_returns_out_of_range() {
        let err = SqlDecimal::new(39, 0, true, 1, 0, 0, 0).unwrap_err();
        assert!(matches!(err, SqlTypeError::OutOfRange(_)));
    }

    #[test]
    fn new_scale_greater_than_precision_returns_out_of_range() {
        let err = SqlDecimal::new(5, 6, true, 1, 0, 0, 0).unwrap_err();
        assert!(matches!(err, SqlTypeError::OutOfRange(_)));
    }

    #[test]
    fn new_mantissa_exceeds_precision_returns_overflow() {
        // 12345 has 5 digits but precision is 4
        let err = SqlDecimal::new(4, 0, true, 12345, 0, 0, 0).unwrap_err();
        assert!(matches!(err, SqlTypeError::Overflow));
    }

    #[test]
    fn new_large_128bit_value() {
        // All four u32 words populated
        let d = SqlDecimal::new(38, 0, true, 1, 1, 1, 1).unwrap();
        assert_eq!(d.data().unwrap(), [1, 1, 1, 1]);
    }

    #[test]
    fn new_trailing_fractional_zeros_preserved() {
        // 10000 with scale=2 represents 100.00 — scale is preserved
        let d = SqlDecimal::new(10, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(d.scale().unwrap(), 2);
    }

    // ── T018: US1 constants tests ─────────────────────────────────────────

    #[test]
    fn null_is_null() {
        assert!(SqlDecimal::NULL.is_null());
    }

    #[test]
    fn max_precision_is_38() {
        assert_eq!(SqlDecimal::MAX_PRECISION, 38);
    }

    #[test]
    fn max_scale_is_38() {
        assert_eq!(SqlDecimal::MAX_SCALE, 38);
    }

    #[test]
    fn max_value_properties() {
        let max = SqlDecimal::max_value();
        assert_eq!(max.precision().unwrap(), 38);
        assert_eq!(max.scale().unwrap(), 0);
        assert!(max.is_positive().unwrap());
        assert_eq!(
            max.data().unwrap(),
            [0xFFFF_FFFF, 0x098A_223F, 0x5A86_C47A, 0x4B3B_4CA8]
        );
    }

    #[test]
    fn min_value_properties() {
        let min = SqlDecimal::min_value();
        assert_eq!(min.precision().unwrap(), 38);
        assert_eq!(min.scale().unwrap(), 0);
        assert!(!min.is_positive().unwrap());
        assert_eq!(
            min.data().unwrap(),
            [0xFFFF_FFFF, 0x098A_223F, 0x5A86_C47A, 0x4B3B_4CA8]
        );
    }

    // ── T019: US1 accessor tests ──────────────────────────────────────────

    #[test]
    fn accessors_on_valid_value() {
        let d = SqlDecimal::new(10, 3, false, 42000, 0, 0, 0).unwrap();
        assert_eq!(d.precision().unwrap(), 10);
        assert_eq!(d.scale().unwrap(), 3);
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [42000, 0, 0, 0]);
    }

    #[test]
    fn value_on_null_returns_err() {
        let n = SqlDecimal::NULL;
        assert!(matches!(n.precision(), Err(SqlTypeError::NullValue)));
        assert!(matches!(n.scale(), Err(SqlTypeError::NullValue)));
        assert!(matches!(n.is_positive(), Err(SqlTypeError::NullValue)));
        assert!(matches!(n.data(), Err(SqlTypeError::NullValue)));
    }

    // ── T020: US1 negative zero normalization ─────────────────────────────

    #[test]
    fn negative_zero_normalizes_to_positive() {
        let d = SqlDecimal::new(5, 2, false, 0, 0, 0, 0).unwrap();
        assert!(d.is_positive().unwrap());
    }

    // ── T021: US2 checked_add tests ───────────────────────────────────────

    #[test]
    fn checked_add_same_scale() {
        // 123.45 + 678.90 = 802.35
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 67890, 0, 0, 0).unwrap();
        let result = a.checked_add(&b).unwrap();
        assert_eq!(result.data().unwrap(), [80235, 0, 0, 0]);
        assert!(result.is_positive().unwrap());
    }

    #[test]
    fn checked_add_different_scales() {
        // 100.5 + 0.25 = 100.75
        let a = SqlDecimal::new(4, 1, true, 1005, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(3, 2, true, 25, 0, 0, 0).unwrap();
        let result = a.checked_add(&b).unwrap();
        assert_eq!(result.data().unwrap(), [10075, 0, 0, 0]);
    }

    #[test]
    fn checked_add_null_propagation_lhs() {
        let b = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = SqlDecimal::NULL.checked_add(&b).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn checked_add_null_propagation_rhs() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = a.checked_add(&SqlDecimal::NULL).unwrap();
        assert!(result.is_null());
    }

    // ── T022: US2 checked_sub tests ───────────────────────────────────────

    #[test]
    fn checked_sub_positive_result() {
        // 200.00 - 100.00 = 100.00
        let a = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let result = a.checked_sub(&b).unwrap();
        assert_eq!(result.data().unwrap(), [10000, 0, 0, 0]);
        assert!(result.is_positive().unwrap());
    }

    #[test]
    fn checked_sub_negative_result() {
        // 100.00 - 200.00 = -100.00
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        let result = a.checked_sub(&b).unwrap();
        assert_eq!(result.data().unwrap(), [10000, 0, 0, 0]);
        assert!(!result.is_positive().unwrap());
    }

    #[test]
    fn checked_sub_null_propagation() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(a.checked_sub(&SqlDecimal::NULL).unwrap().is_null());
        assert!(SqlDecimal::NULL.checked_sub(&a).unwrap().is_null());
    }

    // ── T023: US2 checked_mul tests ───────────────────────────────────────

    #[test]
    fn checked_mul_simple() {
        // 12.00 * 3.00 = 36.0000 (scale = s1+s2 = 4, reduced per rules)
        let a = SqlDecimal::new(4, 2, true, 1200, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(3, 2, true, 300, 0, 0, 0).unwrap();
        let result = a.checked_mul(&b).unwrap();
        // 1200 * 300 = 360000, actual_scale = 4
        let data = result.data().unwrap();
        // Verify the mathematical value: mantissa / 10^scale should == 36
        let scale = result.scale().unwrap();
        let mut value = data[0];
        for _ in 0..scale {
            value /= 10;
        }
        assert_eq!(value, 36);
    }

    #[test]
    fn checked_mul_by_zero() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let zero = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        let result = a.checked_mul(&zero).unwrap();
        assert!(is_zero(&result.data().unwrap()));
    }

    #[test]
    fn checked_mul_null_propagation() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(a.checked_mul(&SqlDecimal::NULL).unwrap().is_null());
        assert!(SqlDecimal::NULL.checked_mul(&a).unwrap().is_null());
    }

    // ── T024: US2 checked_div tests ───────────────────────────────────────

    #[test]
    fn checked_div_exact() {
        // 10.00 / 2.00 = 5.000000 (min scale 6)
        let a = SqlDecimal::new(4, 2, true, 1000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(3, 2, true, 200, 0, 0, 0).unwrap();
        let result = a.checked_div(&b).unwrap();
        assert!(result.is_positive().unwrap());
        // The result should be 5 with some scale
        let scale = result.scale().unwrap();
        let data = result.data().unwrap();
        // 5 * 10^scale
        let mut expected = 5u32;
        for _ in 0..scale {
            expected *= 10;
        }
        assert_eq!(data[0], expected);
    }

    #[test]
    fn checked_div_by_zero() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let zero = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        let result = a.checked_div(&zero);
        assert!(matches!(result, Err(SqlTypeError::DivideByZero)));
    }

    #[test]
    fn checked_div_null_propagation() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(a.checked_div(&SqlDecimal::NULL).unwrap().is_null());
        assert!(SqlDecimal::NULL.checked_div(&a).unwrap().is_null());
    }

    // ── T025: US2 checked_rem tests ───────────────────────────────────────

    #[test]
    fn checked_rem_simple() {
        // 10.00 % 3.00 = 1.00
        let a = SqlDecimal::new(4, 2, true, 1000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(3, 2, true, 300, 0, 0, 0).unwrap();
        let result = a.checked_rem(&b).unwrap();
        assert_eq!(result.data().unwrap(), [100, 0, 0, 0]);
    }

    #[test]
    fn checked_rem_divide_by_zero() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let zero = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        assert!(matches!(
            a.checked_rem(&zero),
            Err(SqlTypeError::DivideByZero)
        ));
    }

    #[test]
    fn checked_rem_null_propagation() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(a.checked_rem(&SqlDecimal::NULL).unwrap().is_null());
        assert!(SqlDecimal::NULL.checked_rem(&a).unwrap().is_null());
    }

    // ── T026: US2 checked_neg tests ───────────────────────────────────────

    #[test]
    fn checked_neg_positive_to_negative() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = a.checked_neg().unwrap();
        assert!(!result.is_positive().unwrap());
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn checked_neg_negative_to_positive() {
        let a = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let result = a.checked_neg().unwrap();
        assert!(result.is_positive().unwrap());
    }

    #[test]
    fn checked_neg_zero_stays_positive() {
        let a = SqlDecimal::new(5, 2, true, 0, 0, 0, 0).unwrap();
        let result = a.checked_neg().unwrap();
        assert!(result.is_positive().unwrap());
    }

    #[test]
    fn checked_neg_null_returns_null() {
        assert!(SqlDecimal::NULL.checked_neg().unwrap().is_null());
    }

    // ── T034: operator trait tests ────────────────────────────────────────

    #[test]
    fn add_operator_works() {
        let a = SqlDecimal::new(5, 2, true, 100, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 200, 0, 0, 0).unwrap();
        let result = (a + b).unwrap();
        assert_eq!(result.data().unwrap(), [300, 0, 0, 0]);
    }

    #[test]
    fn add_operator_ref_works() {
        let a = SqlDecimal::new(5, 2, true, 100, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 200, 0, 0, 0).unwrap();
        let result = (&a + &b).unwrap();
        assert_eq!(result.data().unwrap(), [300, 0, 0, 0]);
    }

    #[test]
    fn neg_operator_works() {
        let a = SqlDecimal::new(5, 2, true, 100, 0, 0, 0).unwrap();
        let result = (-a).unwrap();
        assert!(!result.is_positive().unwrap());
    }

    // ── T035: US3 SQL comparison tests ────────────────────────────────────

    #[test]
    fn sql_equals_same_values() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_equals(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_equals_different_scales() {
        // 100.00 (scale 2) == 100.0000 (scale 4)
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(7, 4, true, 1000000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_equals(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_equals_unequal_values() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_equals(&b).value().unwrap(), false);
    }

    #[test]
    fn sql_less_than_positive() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_less_than(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_greater_than_positive() {
        let a = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_greater_than(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_less_than_or_equal_at_equality() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_less_than_or_equal(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_greater_than_or_equal_at_equality() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_greater_than_or_equal(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_not_equals_different_values() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap();
        assert_eq!(a.sql_not_equals(&b).value().unwrap(), true);
    }

    #[test]
    fn sql_comparison_negative_vs_positive() {
        let neg = SqlDecimal::new(5, 2, false, 10000, 0, 0, 0).unwrap();
        let pos = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(neg.sql_less_than(&pos).value().unwrap(), true);
        assert_eq!(pos.sql_greater_than(&neg).value().unwrap(), true);
    }

    #[test]
    fn sql_comparison_null_propagation() {
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert!(a.sql_equals(&SqlDecimal::NULL).is_null());
        assert!(SqlDecimal::NULL.sql_equals(&a).is_null());
        assert!(SqlDecimal::NULL.sql_less_than(&SqlDecimal::NULL).is_null());
    }

    // ── T037: US4 Display tests ───────────────────────────────────────────

    #[test]
    fn display_positive_decimal() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert_eq!(format!("{d}"), "123.45");
    }

    #[test]
    fn display_negative_decimal() {
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        assert_eq!(format!("{d}"), "-123.45");
    }

    #[test]
    fn display_integer_with_scale() {
        // 10000 with scale=2 = 100.00
        let d = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        assert_eq!(format!("{d}"), "100.00");
    }

    #[test]
    fn display_zero_with_scale() {
        let d = SqlDecimal::new(3, 2, true, 0, 0, 0, 0).unwrap();
        assert_eq!(format!("{d}"), "0.00");
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", SqlDecimal::NULL), "Null");
    }

    #[test]
    fn display_integer_no_scale() {
        let d = SqlDecimal::new(5, 0, true, 42, 0, 0, 0).unwrap();
        assert_eq!(format!("{d}"), "42");
    }

    // ── T038: US4 FromStr tests ───────────────────────────────────────────

    #[test]
    fn parse_decimal_string() {
        let d: SqlDecimal = "123.45".parse().unwrap();
        assert_eq!(d.data().unwrap(), [12345, 0, 0, 0]);
        assert_eq!(d.scale().unwrap(), 2);
    }

    #[test]
    fn parse_negative_decimal() {
        let d: SqlDecimal = "-0.001".parse().unwrap();
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.scale().unwrap(), 3);
        assert_eq!(d.data().unwrap(), [1, 0, 0, 0]);
    }

    #[test]
    fn parse_integer() {
        let d: SqlDecimal = "42".parse().unwrap();
        assert_eq!(d.scale().unwrap(), 0);
        assert_eq!(d.data().unwrap(), [42, 0, 0, 0]);
    }

    #[test]
    fn parse_invalid_returns_error() {
        let result = "abc".parse::<SqlDecimal>();
        assert!(matches!(result, Err(SqlTypeError::ParseError(_))));
    }

    #[test]
    fn parse_null_returns_null() {
        let d: SqlDecimal = "Null".parse().unwrap();
        assert!(d.is_null());
    }

    #[test]
    fn parse_leading_zeros() {
        let d: SqlDecimal = "007.50".parse().unwrap();
        assert_eq!(d.data().unwrap(), [750, 0, 0, 0]);
        assert_eq!(d.scale().unwrap(), 2);
    }

    #[test]
    fn parse_display_roundtrip() {
        let original = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let s = format!("{original}");
        let parsed: SqlDecimal = s.parse().unwrap();
        assert_eq!(parsed.data().unwrap(), original.data().unwrap());
        assert_eq!(parsed.scale().unwrap(), original.scale().unwrap());
    }

    #[test]
    fn parse_trailing_zeros_different_scale() {
        let d1: SqlDecimal = "1.0".parse().unwrap();
        let d2: SqlDecimal = "1.00".parse().unwrap();
        assert_eq!(d1.scale().unwrap(), 1);
        assert_eq!(d2.scale().unwrap(), 2);
    }

    // ── T041: US5 adjust_scale tests ──────────────────────────────────────

    #[test]
    fn adjust_scale_increase_zero_pad() {
        // 123.45 → 123.4500
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.adjust_scale(4, false).unwrap();
        assert_eq!(result.data().unwrap(), [1234500, 0, 0, 0]);
        assert_eq!(result.scale().unwrap(), 4);
    }

    #[test]
    fn adjust_scale_decrease_round_half_up() {
        // 123.456 → 123.46 (round-half-up: 6 >= 5 → round up)
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.adjust_scale(2, true).unwrap();
        assert_eq!(result.data().unwrap(), [12346, 0, 0, 0]);
    }

    #[test]
    fn adjust_scale_decrease_truncate() {
        // 123.456 → 123.45 (truncate)
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.adjust_scale(2, false).unwrap();
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn adjust_scale_round_at_midpoint() {
        // 123.455 → 123.46 (round-half-up, 5 >= 5)
        let d = SqlDecimal::new(6, 3, true, 123455, 0, 0, 0).unwrap();
        let result = d.adjust_scale(2, true).unwrap();
        assert_eq!(result.data().unwrap(), [12346, 0, 0, 0]);
    }

    #[test]
    fn adjust_scale_round_negative_value() {
        // -123.455 → -123.46 (round-half-up away from zero)
        let d = SqlDecimal::new(6, 3, false, 123455, 0, 0, 0).unwrap();
        let result = d.adjust_scale(2, true).unwrap();
        assert_eq!(result.data().unwrap(), [12346, 0, 0, 0]);
        assert!(!result.is_positive().unwrap());
    }

    #[test]
    fn adjust_scale_to_zero() {
        // 123.45 → 123
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.adjust_scale(0, false).unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
        assert_eq!(result.scale().unwrap(), 0);
    }

    #[test]
    fn adjust_scale_no_op() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.adjust_scale(2, false).unwrap();
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn adjust_scale_null_propagation() {
        let result = SqlDecimal::NULL.adjust_scale(5, true).unwrap();
        assert!(result.is_null());
    }

    // ── T042: US6 widening conversion tests ───────────────────────────────

    #[test]
    fn from_i32_positive() {
        let d = SqlDecimal::from(42);
        assert_eq!(d.precision().unwrap(), 10);
        assert_eq!(d.scale().unwrap(), 0);
        assert!(d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [42, 0, 0, 0]);
    }

    #[test]
    fn from_i32_negative() {
        let d = SqlDecimal::from(-42i32);
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [42, 0, 0, 0]);
    }

    #[test]
    fn from_i32_zero() {
        let d = SqlDecimal::from(0i32);
        assert!(d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [0, 0, 0, 0]);
    }

    #[test]
    fn from_i32_min() {
        let d = SqlDecimal::from(i32::MIN);
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [2147483648, 0, 0, 0]);
    }

    #[test]
    fn from_i64_large_positive() {
        let d = SqlDecimal::from(9_000_000_000i64);
        assert_eq!(d.precision().unwrap(), 19);
        assert_eq!(d.scale().unwrap(), 0);
        assert!(d.is_positive().unwrap());
        // 9_000_000_000 = 0x2_18711A00 → data[0] = 0x18711A00, data[1] = 0x2
        assert_eq!(d.data().unwrap()[0], 0x18711A00);
        assert_eq!(d.data().unwrap()[1], 2);
    }

    #[test]
    fn from_i64_negative() {
        let d = SqlDecimal::from(-9_000_000_000i64);
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.data().unwrap()[0], 0x18711A00);
        assert_eq!(d.data().unwrap()[1], 2);
    }

    #[test]
    fn from_sql_boolean_true() {
        let d = SqlDecimal::from(SqlBoolean::new(true));
        assert_eq!(d.data().unwrap(), [1, 0, 0, 0]);
    }

    #[test]
    fn from_sql_boolean_false() {
        let d = SqlDecimal::from(SqlBoolean::new(false));
        assert_eq!(d.data().unwrap(), [0, 0, 0, 0]);
    }

    #[test]
    fn from_sql_boolean_null() {
        let d = SqlDecimal::from(SqlBoolean::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_byte() {
        let d = SqlDecimal::from(SqlByte::new(200));
        assert_eq!(d.precision().unwrap(), 3);
        assert_eq!(d.scale().unwrap(), 0);
        assert_eq!(d.data().unwrap(), [200, 0, 0, 0]);
    }

    #[test]
    fn from_sql_byte_null() {
        let d = SqlDecimal::from(SqlByte::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_int16() {
        let d = SqlDecimal::from(SqlInt16::new(1000));
        assert_eq!(d.precision().unwrap(), 5);
        assert_eq!(d.scale().unwrap(), 0);
        assert_eq!(d.data().unwrap(), [1000, 0, 0, 0]);
    }

    #[test]
    fn from_sql_int16_negative() {
        let d = SqlDecimal::from(SqlInt16::new(-1000));
        assert!(!d.is_positive().unwrap());
        assert_eq!(d.data().unwrap(), [1000, 0, 0, 0]);
    }

    #[test]
    fn from_sql_int16_null() {
        let d = SqlDecimal::from(SqlInt16::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_int32() {
        let d = SqlDecimal::from(SqlInt32::new(42));
        assert_eq!(d.precision().unwrap(), 10);
        assert_eq!(d.data().unwrap(), [42, 0, 0, 0]);
    }

    #[test]
    fn from_sql_int32_null() {
        let d = SqlDecimal::from(SqlInt32::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_int64() {
        let d = SqlDecimal::from(SqlInt64::new(9_000_000_000));
        assert_eq!(d.precision().unwrap(), 19);
        assert_eq!(d.data().unwrap()[0], 0x18711A00);
        assert_eq!(d.data().unwrap()[1], 2);
    }

    #[test]
    fn from_sql_int64_null() {
        let d = SqlDecimal::from(SqlInt64::NULL);
        assert!(d.is_null());
    }

    // ── T043: US6 narrowing conversion tests ──────────────────────────────

    #[test]
    fn to_f64_decimal() {
        let d = SqlDecimal::new(5, 2, true, 4299, 0, 0, 0).unwrap(); // 42.99
        let f = d.to_f64().unwrap();
        assert!((f - 42.99).abs() < 1e-10);
    }

    #[test]
    fn to_f64_negative() {
        let d = SqlDecimal::new(5, 2, false, 4299, 0, 0, 0).unwrap();
        let f = d.to_f64().unwrap();
        assert!((f + 42.99).abs() < 1e-10);
    }

    #[test]
    fn to_f64_null() {
        assert!(matches!(
            SqlDecimal::NULL.to_f64(),
            Err(SqlTypeError::NullValue)
        ));
    }

    #[test]
    fn to_sql_int32_truncate() {
        let d = SqlDecimal::new(5, 2, true, 4299, 0, 0, 0).unwrap(); // 42.99
        let result = d.to_sql_int32().unwrap();
        assert_eq!(result.value().unwrap(), 42);
    }

    #[test]
    fn to_sql_int32_negative() {
        let d = SqlDecimal::new(5, 2, false, 4299, 0, 0, 0).unwrap(); // -42.99
        let result = d.to_sql_int32().unwrap();
        assert_eq!(result.value().unwrap(), -42);
    }

    #[test]
    fn to_sql_int32_overflow() {
        // Value larger than i32::MAX
        let d = SqlDecimal::new(10, 0, true, 3_000_000_000, 0, 0, 0).unwrap();
        assert!(matches!(d.to_sql_int32(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int32_null() {
        let result = SqlDecimal::NULL.to_sql_int32().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_int64_truncate() {
        let d = SqlDecimal::new(5, 2, true, 4299, 0, 0, 0).unwrap(); // 42.99
        let result = d.to_sql_int64().unwrap();
        assert_eq!(result.value().unwrap(), 42);
    }

    #[test]
    fn to_sql_int64_overflow() {
        // Value using upper u32 words that exceeds i64::MAX
        let d = SqlDecimal::new(38, 0, true, 0, 0, 1, 0).unwrap();
        assert!(matches!(d.to_sql_int64(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int64_null() {
        let result = SqlDecimal::NULL.to_sql_int64().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_int16_truncate() {
        let d = SqlDecimal::new(5, 2, true, 10050, 0, 0, 0).unwrap(); // 100.50
        let result = d.to_sql_int16().unwrap();
        assert_eq!(result.value().unwrap(), 100);
    }

    #[test]
    fn to_sql_int16_overflow() {
        let d = SqlDecimal::new(5, 0, true, 40000, 0, 0, 0).unwrap();
        assert!(matches!(d.to_sql_int16(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_int16_null() {
        let result = SqlDecimal::NULL.to_sql_int16().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_byte_value() {
        let d = SqlDecimal::new(3, 0, true, 200, 0, 0, 0).unwrap();
        let result = d.to_sql_byte().unwrap();
        assert_eq!(result.value().unwrap(), 200);
    }

    #[test]
    fn to_sql_byte_overflow_large() {
        let d = SqlDecimal::new(3, 0, true, 256, 0, 0, 0).unwrap();
        assert!(matches!(d.to_sql_byte(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_overflow_negative() {
        let d = SqlDecimal::new(1, 0, false, 1, 0, 0, 0).unwrap();
        assert!(matches!(d.to_sql_byte(), Err(SqlTypeError::Overflow)));
    }

    #[test]
    fn to_sql_byte_null() {
        let result = SqlDecimal::NULL.to_sql_byte().unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn to_sql_boolean_zero_is_false() {
        let d = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        assert!(!d.to_sql_boolean().value().unwrap());
    }

    #[test]
    fn to_sql_boolean_nonzero_is_true() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert!(d.to_sql_boolean().value().unwrap());
    }

    #[test]
    fn to_sql_boolean_null() {
        assert!(SqlDecimal::NULL.to_sql_boolean().is_null());
    }

    // ── T048: US7 abs/floor/ceiling/sign tests ────────────────────────────

    #[test]
    fn abs_negative_to_positive() {
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let result = d.abs();
        assert!(result.is_positive().unwrap());
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn abs_positive_unchanged() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.abs();
        assert!(result.is_positive().unwrap());
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn abs_zero() {
        let d = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        let result = d.abs();
        assert!(result.is_positive().unwrap());
    }

    #[test]
    fn abs_null() {
        assert!(SqlDecimal::NULL.abs().is_null());
    }

    #[test]
    fn floor_positive_with_fraction() {
        // 123.45 → 123
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.floor().unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
        assert_eq!(result.scale().unwrap(), 0);
    }

    #[test]
    fn floor_negative_with_fraction() {
        // -123.45 → -124
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let result = d.floor().unwrap();
        assert_eq!(result.data().unwrap(), [124, 0, 0, 0]);
        assert!(!result.is_positive().unwrap());
    }

    #[test]
    fn floor_integer_no_change() {
        let d = SqlDecimal::new(3, 0, true, 123, 0, 0, 0).unwrap();
        let result = d.floor().unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
    }

    #[test]
    fn floor_null() {
        assert!(SqlDecimal::NULL.floor().unwrap().is_null());
    }

    #[test]
    fn ceiling_positive_with_fraction() {
        // 123.45 → 124
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.ceiling().unwrap();
        assert_eq!(result.data().unwrap(), [124, 0, 0, 0]);
    }

    #[test]
    fn ceiling_negative_with_fraction() {
        // -123.45 → -123
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let result = d.ceiling().unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
    }

    #[test]
    fn ceiling_integer_no_change() {
        let d = SqlDecimal::new(3, 0, true, 123, 0, 0, 0).unwrap();
        let result = d.ceiling().unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
    }

    #[test]
    fn ceiling_null() {
        assert!(SqlDecimal::NULL.ceiling().unwrap().is_null());
    }

    #[test]
    fn sign_positive() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert_eq!(d.sign().value().unwrap(), 1);
    }

    #[test]
    fn sign_negative() {
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        assert_eq!(d.sign().value().unwrap(), -1);
    }

    #[test]
    fn sign_zero() {
        let d = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        assert_eq!(d.sign().value().unwrap(), 0);
    }

    #[test]
    fn sign_null() {
        assert!(SqlDecimal::NULL.sign().is_null());
    }

    // ── T049: US7 round/truncate/power tests ──────────────────────────────

    #[test]
    fn round_to_2_decimal_places() {
        // 123.456 → 123.46
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.round(2).unwrap();
        assert_eq!(result.data().unwrap(), [12346, 0, 0, 0]);
        assert_eq!(result.scale().unwrap(), 2);
    }

    #[test]
    fn round_to_0_decimal_places() {
        // 123.456 → 123
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.round(0).unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
    }

    #[test]
    fn round_null() {
        assert!(SqlDecimal::NULL.round(2).unwrap().is_null());
    }

    #[test]
    fn truncate_to_2_decimal_places() {
        // 123.456 → 123.45
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.truncate(2).unwrap();
        assert_eq!(result.data().unwrap(), [12345, 0, 0, 0]);
    }

    #[test]
    fn truncate_to_0_decimal_places() {
        // 123.456 → 123
        let d = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap();
        let result = d.truncate(0).unwrap();
        assert_eq!(result.data().unwrap(), [123, 0, 0, 0]);
    }

    #[test]
    fn truncate_null() {
        assert!(SqlDecimal::NULL.truncate(2).unwrap().is_null());
    }

    #[test]
    fn power_integer() {
        // 5^3 = 125
        let d = SqlDecimal::new(1, 0, true, 5, 0, 0, 0).unwrap();
        let result = d.power(3).unwrap();
        assert_eq!(result.to_f64().unwrap(), 125.0);
    }

    #[test]
    fn power_2_to_10() {
        // 2^10 = 1024
        let d = SqlDecimal::new(1, 0, true, 2, 0, 0, 0).unwrap();
        let result = d.power(10).unwrap();
        assert_eq!(result.to_f64().unwrap(), 1024.0);
    }

    #[test]
    fn power_zero_base() {
        // 0^5 = 0
        let d = SqlDecimal::new(1, 0, true, 0, 0, 0, 0).unwrap();
        let result = d.power(5).unwrap();
        assert_eq!(result.to_f64().unwrap(), 0.0);
    }

    #[test]
    fn power_zero_exponent() {
        // x^0 = 1
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let result = d.power(0).unwrap();
        assert_eq!(result.to_f64().unwrap(), 1.0);
    }

    #[test]
    fn power_null() {
        assert!(SqlDecimal::NULL.power(3).unwrap().is_null());
    }

    // ── T052: PartialEq/Eq tests ─────────────────────────────────────────

    #[test]
    fn eq_same_values() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn eq_null_equals_null() {
        assert_eq!(SqlDecimal::NULL, SqlDecimal::NULL);
    }

    #[test]
    fn eq_different_values() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 12346, 0, 0, 0).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn eq_different_scales_same_value() {
        // 100.00 == 100.0000
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(7, 4, true, 1000000, 0, 0, 0).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn eq_null_not_equal_to_value() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert_ne!(SqlDecimal::NULL, a);
    }

    // ── T053: Hash and PartialOrd/Ord tests ───────────────────────────────

    #[test]
    fn hash_equal_values_same_hash() {
        use std::collections::hash_map::DefaultHasher;
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
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
    fn hash_equal_values_different_scale_same_hash() {
        use std::collections::hash_map::DefaultHasher;
        // 100.00 and 100.0000 should have the same hash
        let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(7, 4, true, 1000000, 0, 0, 0).unwrap();
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
            SqlDecimal::NULL.hash(&mut h);
            h.finish()
        };
        let hash2 = {
            let mut h = DefaultHasher::new();
            SqlDecimal::NULL.hash(&mut h);
            h.finish()
        };
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn ord_null_less_than_value() {
        let val = SqlDecimal::new(1, 0, true, 1, 0, 0, 0).unwrap();
        assert!(SqlDecimal::NULL < val);
    }

    #[test]
    fn ord_min_less_than_max() {
        assert!(SqlDecimal::min_value() < SqlDecimal::max_value());
    }

    #[test]
    fn ord_negative_less_than_positive() {
        let neg = SqlDecimal::new(3, 0, false, 100, 0, 0, 0).unwrap();
        let pos = SqlDecimal::new(3, 0, true, 100, 0, 0, 0).unwrap();
        assert!(neg < pos);
    }

    #[test]
    fn ord_equal_values() {
        let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let b = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    // ── to_sql_string() tests ────────────────────────────────────────────────

    #[test]
    fn to_sql_string_integer() {
        let d = SqlDecimal::new(5, 0, true, 12345, 0, 0, 0).unwrap();
        let s = d.to_sql_string();
        assert_eq!(s.value().unwrap(), "12345");
    }

    #[test]
    fn to_sql_string_with_scale() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let s = d.to_sql_string();
        assert_eq!(s.value().unwrap(), "123.45");
    }

    #[test]
    fn to_sql_string_negative() {
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let s = d.to_sql_string();
        assert_eq!(s.value().unwrap(), "-123.45");
    }

    #[test]
    fn to_sql_string_null() {
        let s = SqlDecimal::NULL.to_sql_string();
        assert!(s.is_null());
    }

    // ── From<SqlSingle/SqlDouble/SqlMoney> tests ─────────────────────────────

    #[test]
    fn from_sql_single_normal() {
        let d = SqlDecimal::from(SqlSingle::new(3.14).unwrap());
        assert!(!d.is_null());
        let f = d.to_f64().unwrap();
        assert!((f - 3.14).abs() < 0.01);
    }

    #[test]
    fn from_sql_single_null() {
        let d = SqlDecimal::from(SqlSingle::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_double_normal() {
        let d = SqlDecimal::from(SqlDouble::new(100.5).unwrap());
        assert!(!d.is_null());
        let f = d.to_f64().unwrap();
        assert!((f - 100.5).abs() < 0.001);
    }

    #[test]
    fn from_sql_double_null() {
        let d = SqlDecimal::from(SqlDouble::NULL);
        assert!(d.is_null());
    }

    #[test]
    fn from_sql_money_normal() {
        let m = SqlMoney::from_i32(100);
        let d = SqlDecimal::from(m);
        assert!(!d.is_null());
        let f = d.to_f64().unwrap();
        assert!((f - 100.0).abs() < 0.0001);
    }

    #[test]
    fn from_sql_money_null() {
        let d = SqlDecimal::from(SqlMoney::NULL);
        assert!(d.is_null());
    }

    // ── to_sql_single/double/money tests ─────────────────────────────────────

    #[test]
    fn to_sql_single_normal() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let s = d.to_sql_single();
        assert!(!s.is_null());
        assert!((s.value().unwrap() - 123.45).abs() < 0.01);
    }

    #[test]
    fn to_sql_single_null() {
        assert!(SqlDecimal::NULL.to_sql_single().is_null());
    }

    #[test]
    fn to_sql_double_normal() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let dbl = d.to_sql_double();
        assert!(!dbl.is_null());
        assert!((dbl.value().unwrap() - 123.45).abs() < 0.001);
    }

    #[test]
    fn to_sql_double_null() {
        assert!(SqlDecimal::NULL.to_sql_double().is_null());
    }

    #[test]
    fn to_sql_money_normal() {
        let d = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
        let m = d.to_sql_money().unwrap();
        assert!(!m.is_null());
    }

    #[test]
    fn to_sql_money_null() {
        let m = SqlDecimal::NULL.to_sql_money().unwrap();
        assert!(m.is_null());
    }

    #[test]
    fn to_sql_money_negative() {
        let d = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
        let m = d.to_sql_money().unwrap();
        assert!(!m.is_null());
    }
}
