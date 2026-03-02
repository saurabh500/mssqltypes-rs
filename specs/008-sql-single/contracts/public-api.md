# Public API Contract: SqlSingle

**Feature**: 008-sql-single | **Date**: 2026-03-02

## Construction & Inspection

```rust
// Fallible constructor — rejects NaN/Infinity
pub fn new(v: f32) -> Result<SqlSingle, SqlTypeError>;

// Infallible accessor
pub fn is_null(&self) -> bool;

// Fallible accessor — Err(NullValue) if NULL
pub fn value(&self) -> Result<f32, SqlTypeError>;

// Constants
pub const NULL: SqlSingle;      // SQL NULL
pub const ZERO: SqlSingle;      // 0.0
pub const MIN_VALUE: SqlSingle;  // f32::MIN
pub const MAX_VALUE: SqlSingle;  // f32::MAX
```

## From\<f32\> (Panicking)

```rust
impl From<f32> for SqlSingle {
    /// Panics on NaN or Infinity.
    fn from(v: f32) -> Self;
}
```

## Checked Arithmetic

```rust
pub fn checked_add(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError>;
pub fn checked_sub(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError>;
pub fn checked_mul(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError>;
pub fn checked_div(self, rhs: SqlSingle) -> Result<SqlSingle, SqlTypeError>;
```

**Behavior**:
- NULL operand → `Ok(SqlSingle::NULL)`
- Result not finite → `Err(SqlTypeError::Overflow)`
- Division: divisor == 0.0 → `Err(SqlTypeError::DivideByZero)` (checked before compute)

## Operator Traits

```rust
// All return Result<SqlSingle, SqlTypeError>
impl Add<SqlSingle> for SqlSingle { type Output = Result<SqlSingle, SqlTypeError>; }
impl Add<&SqlSingle> for SqlSingle { type Output = Result<SqlSingle, SqlTypeError>; }
impl Add<SqlSingle> for &SqlSingle { type Output = Result<SqlSingle, SqlTypeError>; }
impl Add<&SqlSingle> for &SqlSingle { type Output = Result<SqlSingle, SqlTypeError>; }
// Same pattern for Sub, Mul, Div

// Negation — infallible
impl Neg for SqlSingle { type Output = SqlSingle; }
impl Neg for &SqlSingle { type Output = SqlSingle; }
```

## SQL Comparisons

```rust
pub fn sql_equals(&self, other: &SqlSingle) -> SqlBoolean;
pub fn sql_not_equals(&self, other: &SqlSingle) -> SqlBoolean;
pub fn sql_less_than(&self, other: &SqlSingle) -> SqlBoolean;
pub fn sql_greater_than(&self, other: &SqlSingle) -> SqlBoolean;
pub fn sql_less_than_or_equal(&self, other: &SqlSingle) -> SqlBoolean;
pub fn sql_greater_than_or_equal(&self, other: &SqlSingle) -> SqlBoolean;
```

**Behavior**: NULL operand → `SqlBoolean::NULL`

## Display & FromStr

```rust
impl Display for SqlSingle {
    // NULL → "Null", value → f32 default Display
}

impl FromStr for SqlSingle {
    type Err = SqlTypeError;
    // "Null" (case-insensitive) → SqlSingle::NULL
    // Valid f32 string → SqlSingle (rejects NaN/Infinity)
    // Invalid → Err(ParseError)
}
```

## Type Conversions

### Widening From (infallible)

```rust
pub fn from_sql_byte(v: SqlByte) -> SqlSingle;
pub fn from_sql_int16(v: SqlInt16) -> SqlSingle;
pub fn from_sql_int32(v: SqlInt32) -> SqlSingle;    // may lose precision
pub fn from_sql_int64(v: SqlInt64) -> SqlSingle;    // may lose precision
pub fn from_sql_boolean(v: SqlBoolean) -> SqlSingle; // TRUE→1.0, FALSE→0.0
pub fn from_sql_money(v: SqlMoney) -> SqlSingle;     // via f64 intermediate
```

### Narrowing From (fallible)

```rust
pub fn from_sql_double(v: SqlDouble) -> Result<SqlSingle, SqlTypeError>;
// f64 → f32 narrowing. Returns Err(Overflow) if result is Infinity.
```

### Widening To

```rust
pub fn to_sql_double(&self) -> SqlDouble;    // f32 → f64 lossless
pub fn to_sql_boolean(&self) -> SqlBoolean;  // 0.0→FALSE, non-zero→TRUE
```

## Standard Traits

```rust
// Equality — NaN excluded, so Eq is safe
impl PartialEq for SqlSingle;  // NULL == NULL (Rust semantics)
impl Eq for SqlSingle;

// Hashing — uses f32::to_bits() with -0.0 normalization
impl Hash for SqlSingle;

// Ordering — NULL < any non-NULL
impl PartialOrd for SqlSingle;
impl Ord for SqlSingle;
```

## Error Conditions

| Method | Error | Condition |
|--------|-------|-----------|
| `new()` | `Overflow` | Value is NaN, Infinity, or NEG_INFINITY |
| `value()` | `NullValue` | Called on NULL |
| `checked_add/sub/mul` | `Overflow` | Result is not finite |
| `checked_div` | `DivideByZero` | Divisor is 0.0 |
| `checked_div` | `Overflow` | Result is not finite (non-zero / tiny) |
| `FromStr` | `ParseError` | Invalid string, NaN, Infinity |
| `from_sql_double` | `Overflow` | f64 value outside f32 finite range |
