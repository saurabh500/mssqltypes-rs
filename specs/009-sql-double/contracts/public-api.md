# Public API Contract: SqlDouble

**Feature**: 009-sql-double
**Date**: 2026-03-02

## Type Definition

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlDouble {
    value: Option<f64>,  // None = SQL NULL, Some(v) = finite f64
}
```

## Constants

```rust
impl SqlDouble {
    pub const NULL: SqlDouble;       // SQL NULL sentinel
    pub const ZERO: SqlDouble;       // 0.0
    pub const MIN_VALUE: SqlDouble;  // f64::MIN
    pub const MAX_VALUE: SqlDouble;  // f64::MAX
}
```

## Constructors

```rust
impl SqlDouble {
    /// Creates a new SqlDouble from a finite f64 value.
    /// Returns Err(SqlTypeError::Overflow) if value is NaN or Infinity.
    pub fn new(value: f64) -> Result<SqlDouble, SqlTypeError>;
}

/// Panics if value is NaN or Infinity.
impl From<f64> for SqlDouble;
```

## Accessors

```rust
impl SqlDouble {
    /// Returns true if this value is SQL NULL.
    pub fn is_null(&self) -> bool;

    /// Returns the inner f64 value, or Err(NullValue) if NULL.
    pub fn value(&self) -> Result<f64, SqlTypeError>;
}
```

## Arithmetic (Operator Traits)

All arithmetic returns `Result<SqlDouble, SqlTypeError>`. NULL propagates.

```rust
impl Add for SqlDouble { type Output = Result<SqlDouble, SqlTypeError>; }
impl Sub for SqlDouble { type Output = Result<SqlDouble, SqlTypeError>; }
impl Mul for SqlDouble { type Output = Result<SqlDouble, SqlTypeError>; }
impl Div for SqlDouble { type Output = Result<SqlDouble, SqlTypeError>; }
impl Neg for SqlDouble { type Output = SqlDouble; }  // infallible

// Also implemented for &SqlDouble references
```

### Error conditions:
- `Add`/`Sub`/`Mul`: If result is not finite → `Err(SqlTypeError::Overflow)`
- `Div`: If divisor is ±0.0 → `Err(SqlTypeError::DivideByZero)`. If result is not finite → `Err(SqlTypeError::Overflow)`

## Checked Arithmetic Methods

```rust
impl SqlDouble {
    pub fn checked_add(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlDouble) -> Result<SqlDouble, SqlTypeError>;
}
```

## SQL Comparisons

All return `SqlBoolean`. NULL operand → `SqlBoolean::NULL`.

```rust
impl SqlDouble {
    pub fn sql_equals(&self, other: &SqlDouble) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlDouble) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlDouble) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlDouble) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlDouble) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlDouble) -> SqlBoolean;
}
```

## Display & Parsing

```rust
impl fmt::Display for SqlDouble;  // NULL → "Null", value → f64 Display
impl FromStr for SqlDouble;       // "Null" → NULL, valid number → value, NaN/Infinity → error
```

## Standard Traits

```rust
impl PartialEq for SqlDouble;  // NULL == NULL (Rust structural equality)
impl Eq for SqlDouble;          // Safe because NaN excluded
impl Hash for SqlDouble;        // to_bits() with -0.0 normalization; NULL hashes as 0
impl PartialOrd for SqlDouble;  // NULL < any non-NULL
impl Ord for SqlDouble;         // NULL < any non-NULL, then f64 ordering
```

## Type Conversions

### Widening (From other types → SqlDouble)

```rust
impl SqlDouble {
    pub fn from_sql_byte(v: SqlByte) -> SqlDouble;
    pub fn from_sql_int16(v: SqlInt16) -> SqlDouble;
    pub fn from_sql_int32(v: SqlInt32) -> SqlDouble;
    pub fn from_sql_int64(v: SqlInt64) -> SqlDouble;
    pub fn from_sql_money(v: SqlMoney) -> SqlDouble;
    pub fn from_sql_boolean(v: SqlBoolean) -> SqlDouble;
    // from_sql_single: DEFERRED until SqlSingle is implemented
}
```

### Narrowing (SqlDouble → other types)

```rust
impl SqlDouble {
    pub fn to_sql_boolean(&self) -> SqlBoolean;
    // to_sql_single: DEFERRED until SqlSingle is implemented
}
```

## Error Handling

All errors use `SqlTypeError` enum variants:
- `SqlTypeError::NullValue` — accessing value of NULL
- `SqlTypeError::Overflow` — NaN/Infinity result or non-finite construction
- `SqlTypeError::DivideByZero` — division by ±0.0
- `SqlTypeError::ParseError(String)` — invalid string parsing
