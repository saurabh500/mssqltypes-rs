# Public API Contract: SqlMoney

## Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlMoney {
    value: Option<i64>,  // None = SQL NULL, Some(v) = monetary value × 10,000
}
```

## Constants

```rust
impl SqlMoney {
    pub const NULL: SqlMoney;       // SQL NULL
    pub const ZERO: SqlMoney;       // 0.0000 (internal: 0)
    pub const MIN_VALUE: SqlMoney;  // −922,337,203,685,477.5808 (internal: i64::MIN)
    pub const MAX_VALUE: SqlMoney;  // 922,337,203,685,477.5807 (internal: i64::MAX)

    // Internal scale factor (not public, but documented for clarity)
    const SCALE: i64 = 10_000;
}
```

## Constructors & Accessors

```rust
impl SqlMoney {
    /// Create from i32 (always succeeds — i32 × 10,000 fits in i64)
    pub fn from_i32(v: i32) -> Self;

    /// Create from i64 (range-checked: value × 10,000 must fit in i64)
    pub fn from_i64(v: i64) -> Result<Self, SqlTypeError>;

    /// Create from f64 (reject NaN/Infinity, round to 4dp, range-check)
    pub fn from_f64(v: f64) -> Result<Self, SqlTypeError>;

    /// Create from raw scaled value (no scaling applied, no validation)
    pub fn from_scaled(v: i64) -> Self;

    /// Check if this value is SQL NULL
    pub fn is_null(&self) -> bool;

    /// Get the raw internal scaled value (ticks). Returns Err(NullValue) if NULL.
    pub fn scaled_value(&self) -> Result<i64, SqlTypeError>;

    /// Convert to i64 (round-half-away-from-zero). Returns Err(NullValue) if NULL.
    pub fn to_i64(&self) -> Result<i64, SqlTypeError>;

    /// Convert to i32 (round then range-check). Returns Err(NullValue) if NULL, Err(Overflow) if out of range.
    pub fn to_i32(&self) -> Result<i32, SqlTypeError>;

    /// Convert to f64. Returns Err(NullValue) if NULL.
    pub fn to_f64(&self) -> Result<f64, SqlTypeError>;
}
```

## Checked Arithmetic

All return `Result<SqlMoney, SqlTypeError>`. NULL propagation: if either operand is NULL, returns `Ok(SqlMoney::NULL)`.

```rust
impl SqlMoney {
    pub fn checked_add(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlMoney) -> Result<SqlMoney, SqlTypeError>;
    pub fn checked_neg(self) -> Result<SqlMoney, SqlTypeError>;
}
```

### Error Conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | checked i64 addition overflows | `Overflow` |
| `checked_sub` | checked i64 subtraction overflows | `Overflow` |
| `checked_mul` | i128 intermediate / 10,000 doesn't fit in i64 | `Overflow` |
| `checked_div` | rhs internal value == 0 | `DivideByZero` |
| `checked_div` | i128 intermediate doesn't fit in i64 | `Overflow` |
| `checked_neg` | value == i64::MIN (MIN_VALUE) | `Overflow` |

## Operator Traits (delegate to checked_* methods)

```rust
impl Add for SqlMoney { type Output = Result<SqlMoney, SqlTypeError>; }
impl Sub for SqlMoney { type Output = Result<SqlMoney, SqlTypeError>; }
impl Mul for SqlMoney { type Output = Result<SqlMoney, SqlTypeError>; }
impl Div for SqlMoney { type Output = Result<SqlMoney, SqlTypeError>; }
impl Neg for SqlMoney { type Output = Result<SqlMoney, SqlTypeError>; }

// Also implemented for &SqlMoney (all combinations of owned/borrowed)
impl Add<&SqlMoney> for SqlMoney { ... }
impl Add<SqlMoney> for &SqlMoney { ... }
impl Add<&SqlMoney> for &SqlMoney { ... }
// (same for Sub, Mul, Div)
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlMoney {
    pub fn sql_equals(&self, other: &SqlMoney) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlMoney) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlMoney) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlMoney) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlMoney) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlMoney) -> SqlBoolean;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlMoney;  // NULL == NULL → true (Rust semantics)
impl Eq for SqlMoney;
impl Hash for SqlMoney;        // NULL → hash(0i64), Some(v) → hash(v)
impl PartialOrd for SqlMoney;  // NULL < any value
impl Ord for SqlMoney;         // total ordering: NULL < MIN..MAX
impl Display for SqlMoney;     // NULL → "Null", Some(v) → "#0.00##" format
impl FromStr for SqlMoney;     // "Null" → NULL, valid decimal → scaled, else ParseError
```

## Conversions

### Widening INTO SqlMoney (infallible)

```rust
impl From<SqlBoolean> for SqlMoney;  // NULL→NULL, FALSE→0.0000, TRUE→1.0000
impl From<SqlByte> for SqlMoney;     // NULL→NULL, v→(v as i64) × 10,000
impl From<SqlInt16> for SqlMoney;    // NULL→NULL, v→(v as i64) × 10,000
impl From<SqlInt32> for SqlMoney;    // NULL→NULL, v→(v as i64) × 10,000
```

### Fallible INTO SqlMoney

```rust
impl SqlMoney {
    /// From SqlInt64 — range-checked (value × 10,000 must fit in i64)
    pub fn from_sql_int64(v: SqlInt64) -> Result<Self, SqlTypeError>;
}
```

### OUT of SqlMoney (fallible)

```rust
impl SqlMoney {
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError>;    // round
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError>;    // round + range check
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>;    // round + range check
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;      // round + range check
    pub fn to_sql_boolean(&self) -> SqlBoolean;                      // zero→FALSE, non-zero→TRUE, NULL→NULL
    pub fn to_sql_decimal(&self) -> SqlDecimal;                      // exact, scale=4, NULL→NULL
}
```

## Deferred (until target types are implemented)

- `From<SqlSingle> for SqlMoney` — SqlSingle not yet implemented
- `From<SqlDouble> for SqlMoney` — SqlDouble not yet implemented
- `From<SqlDecimal> for SqlMoney` — deferred to keep scope manageable
- `From<SqlString> for SqlMoney` — SqlString not yet implemented
- `SqlMoney::to_sql_single()` — SqlSingle not yet implemented
- `SqlMoney::to_sql_double()` — SqlDouble not yet implemented
- `SqlMoney::to_sql_string()` — SqlString not yet implemented
