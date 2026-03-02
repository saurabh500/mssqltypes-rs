# Public API Contract: SqlDecimal

## Struct

```rust
#[derive(Clone, Debug)]
pub struct SqlDecimal {
    inner: Option<InnerDecimal>,  // None = SQL NULL
}

#[derive(Clone, Copy, Debug)]
struct InnerDecimal {
    precision: u8,      // 1–38
    scale: u8,          // 0–precision
    positive: bool,     // true = positive/zero, false = negative
    data: [u32; 4],     // 128-bit unsigned mantissa, little-endian
}
```

## Constants

```rust
impl SqlDecimal {
    pub const NULL: SqlDecimal;          // SQL NULL
    pub const MAX_PRECISION: u8 = 38;    // Maximum precision
    pub const MAX_SCALE: u8 = 38;        // Maximum scale
}

// These are constructed at runtime (not const) due to complex initialization:
impl SqlDecimal {
    pub fn max_value() -> SqlDecimal;    // 10^38 - 1, precision=38, scale=0
    pub fn min_value() -> SqlDecimal;    // -(10^38 - 1), precision=38, scale=0
}
```

## Constructors & Accessors

```rust
impl SqlDecimal {
    /// Create a new SqlDecimal from components.
    /// Returns Err if precision/scale invalid or mantissa exceeds precision.
    pub fn new(
        precision: u8,
        scale: u8,
        positive: bool,
        data1: u32,
        data2: u32,
        data3: u32,
        data4: u32,
    ) -> Result<SqlDecimal, SqlTypeError>;

    pub fn is_null(&self) -> bool;
    pub fn precision(&self) -> Result<u8, SqlTypeError>;      // Err(NullValue) if NULL
    pub fn scale(&self) -> Result<u8, SqlTypeError>;          // Err(NullValue) if NULL
    pub fn is_positive(&self) -> Result<bool, SqlTypeError>;  // Err(NullValue) if NULL
    pub fn data(&self) -> Result<[u32; 4], SqlTypeError>;     // Err(NullValue) if NULL
}
```

## Checked Arithmetic

All return `Result<SqlDecimal, SqlTypeError>`. NULL propagation: if either operand is NULL, returns `Ok(SqlDecimal::NULL)`.

```rust
impl SqlDecimal {
    pub fn checked_add(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError>;
    pub fn checked_sub(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError>;
    pub fn checked_mul(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError>;
    pub fn checked_div(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError>;
    pub fn checked_rem(&self, rhs: &SqlDecimal) -> Result<SqlDecimal, SqlTypeError>;
    pub fn checked_neg(&self) -> Result<SqlDecimal, SqlTypeError>;
}
```

### Error Conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | result exceeds 38 digits | `Overflow` |
| `checked_sub` | result exceeds 38 digits | `Overflow` |
| `checked_mul` | result exceeds 38 digits | `Overflow` |
| `checked_div` | rhs == 0 | `DivideByZero` |
| `checked_div` | result exceeds 38 digits | `Overflow` |
| `checked_rem` | rhs == 0 | `DivideByZero` |
| `checked_rem` | result exceeds 38 digits | `Overflow` |

### Precision/Scale Rules

| Operation | Result Precision | Result Scale |
|-----------|-----------------|--------------|
| Add/Sub | `min(38, max(p1-s1, p2-s2) + max(s1,s2) + 1)` | `max(s1,s2)` (reduced if precision capped) |
| Multiply | `min(38, (p1-s1)+(p2-s2)+1 + s1+s2)` | `min(resPrec - resInt, s1+s2)`, floor at `min(s1+s2, 6)` |
| Divide | `min(38, max(s1+p2+1, 6) + (p1-s1)+s2 + 1)` | See research.md R4 for full formula |

## Operator Traits (delegate to checked_* methods)

```rust
impl Add for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }  // by value
impl Add for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }  // by reference
impl Sub for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Sub for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Mul for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Mul for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Div for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Div for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Rem for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Rem for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Neg for SqlDecimal  { type Output = Result<SqlDecimal, SqlTypeError>; }
impl Neg for &SqlDecimal { type Output = Result<SqlDecimal, SqlTypeError>; }
```

## Scale Adjustment

```rust
impl SqlDecimal {
    /// Adjust the scale of this SqlDecimal.
    /// If round is true, uses round-half-up. If false, truncates.
    /// Returns Err(Overflow) if resulting precision would exceed 38.
    /// NULL input returns Ok(SqlDecimal::NULL).
    pub fn adjust_scale(&self, new_scale: u8, round: bool) -> Result<SqlDecimal, SqlTypeError>;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlDecimal {
    pub fn sql_equals(&self, other: &SqlDecimal) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlDecimal) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlDecimal) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlDecimal) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlDecimal) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlDecimal) -> SqlBoolean;
}
```

## Mathematical Functions

```rust
impl SqlDecimal {
    /// Absolute value. NULL → NULL.
    pub fn abs(&self) -> SqlDecimal;

    /// Floor (round toward negative infinity). NULL → NULL.
    pub fn floor(&self) -> Result<SqlDecimal, SqlTypeError>;

    /// Ceiling (round toward positive infinity). NULL → NULL.
    pub fn ceiling(&self) -> Result<SqlDecimal, SqlTypeError>;

    /// Round to the given number of decimal places. NULL → NULL.
    pub fn round(&self, position: i32) -> Result<SqlDecimal, SqlTypeError>;

    /// Truncate to the given number of decimal places. NULL → NULL.
    pub fn truncate(&self, position: i32) -> Result<SqlDecimal, SqlTypeError>;

    /// Returns -1, 0, or 1 indicating the sign. NULL → SqlInt32::NULL.
    pub fn sign(&self) -> SqlInt32;

    /// Raise to an integer power. NULL → NULL.
    pub fn power(&self, exponent: i32) -> Result<SqlDecimal, SqlTypeError>;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlDecimal;   // NULL == NULL → true (Rust semantics)
impl Eq for SqlDecimal;
impl Hash for SqlDecimal;         // NULL → consistent hash; equal values with different scale → same hash
impl PartialOrd for SqlDecimal;   // NULL < any value
impl Ord for SqlDecimal;          // total ordering: NULL < MIN..MAX
impl Display for SqlDecimal;      // NULL → "Null", Some(v) → decimal string preserving scale
impl FromStr for SqlDecimal;      // parse decimal string, Err(ParseError) for invalid
impl Clone for SqlDecimal;        // deep copy (fixed-size, no heap)
impl Debug for SqlDecimal;        // derive
```

## Conversions

```rust
// Widening into SqlDecimal (infallible)
impl From<i32> for SqlDecimal;         // precision=10, scale=0
impl From<i64> for SqlDecimal;         // precision=19, scale=0
impl From<SqlBoolean> for SqlDecimal;  // NULL→NULL, FALSE→0, TRUE→1
impl From<SqlByte> for SqlDecimal;     // precision=3, scale=0
impl From<SqlInt16> for SqlDecimal;    // precision=5, scale=0
impl From<SqlInt32> for SqlDecimal;    // precision=10, scale=0
impl From<SqlInt64> for SqlDecimal;    // precision=19, scale=0

// Narrowing/lossy out of SqlDecimal (fallible)
impl SqlDecimal {
    pub fn to_f64(&self) -> Result<f64, SqlTypeError>;           // lossy, NullValue if NULL
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError>;  // truncate + range check
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError>;  // truncate + range check
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>;  // truncate + range check
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;    // truncate + range check
    pub fn to_sql_boolean(&self) -> SqlBoolean;                     // 0→FALSE, non-0→TRUE, NULL→NULL
}
```

## Deferred (until target types are implemented or follow-up)

- `From<SqlSingle> for SqlDecimal` — type not yet implemented
- `From<SqlDouble> for SqlDecimal` — type not yet implemented
- `From<SqlMoney> for SqlDecimal` — type not yet implemented
- `From<SqlString> for SqlDecimal` — type not yet implemented
- `SqlDecimal::to_sql_single()` — type not yet implemented
- `SqlDecimal::to_sql_double()` — type not yet implemented
- `SqlDecimal::to_sql_money()` — type not yet implemented
- `SqlDecimal::to_sql_string()` — type not yet implemented
