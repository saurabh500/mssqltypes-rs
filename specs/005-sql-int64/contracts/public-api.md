# Public API Contract: SqlInt64

## Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlInt64 {
    value: Option<i64>,  // None = SQL NULL
}
```

## Constants

```rust
impl SqlInt64 {
    pub const NULL: SqlInt64;       // SQL NULL
    pub const ZERO: SqlInt64;       // 0
    pub const MIN_VALUE: SqlInt64;  // -9_223_372_036_854_775_808 (i64::MIN)
    pub const MAX_VALUE: SqlInt64;  // 9_223_372_036_854_775_807 (i64::MAX)
}
```

## Constructors & Accessors

```rust
impl SqlInt64 {
    pub fn new(v: i64) -> Self;
    pub fn is_null(&self) -> bool;
    pub fn value(&self) -> Result<i64, SqlTypeError>;
}
```

## Checked Arithmetic

All return `Result<SqlInt64, SqlTypeError>`. NULL propagation: if either operand is NULL, returns `Ok(SqlInt64::NULL)`.

```rust
impl SqlInt64 {
    pub fn checked_add(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError>;
    pub fn checked_rem(self, rhs: SqlInt64) -> Result<SqlInt64, SqlTypeError>;
    pub fn checked_neg(self) -> Result<SqlInt64, SqlTypeError>;
}
```

### Error Conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | result < i64::MIN or > i64::MAX | `Overflow` |
| `checked_sub` | result < i64::MIN or > i64::MAX | `Overflow` |
| `checked_mul` | result < i64::MIN or > i64::MAX | `Overflow` |
| `checked_div` | rhs == 0 | `DivideByZero` |
| `checked_div` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_rem` | rhs == 0 | `DivideByZero` |
| `checked_rem` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_neg` | value == MIN_VALUE (-9,223,372,036,854,775,808) | `Overflow` |

## Operator Traits (delegate to checked_* methods)

```rust
impl Add for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
impl Sub for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
impl Mul for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
impl Div for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
impl Rem for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
impl Neg for SqlInt64 { type Output = Result<SqlInt64, SqlTypeError>; }
```

## Bitwise Operators (infallible, NULL propagation)

```rust
impl BitAnd for SqlInt64 { type Output = SqlInt64; }
impl BitOr  for SqlInt64 { type Output = SqlInt64; }
impl BitXor for SqlInt64 { type Output = SqlInt64; }
impl Not    for SqlInt64 { type Output = SqlInt64; }  // ones' complement
```

Additional method:

```rust
impl SqlInt64 {
    pub fn ones_complement(self) -> SqlInt64;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlInt64 {
    pub fn sql_equals(&self, other: &SqlInt64) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlInt64) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlInt64) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlInt64) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlInt64) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt64) -> SqlBoolean;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlInt64;  // NULL == NULL → true (Rust semantics)
impl Eq for SqlInt64;
impl Hash for SqlInt64;        // NULL → hash(0), Some(v) → hash(v)
impl PartialOrd for SqlInt64;  // NULL < any value
impl Ord for SqlInt64;         // total ordering: NULL < MIN..MAX
impl Display for SqlInt64;     // NULL → "Null", Some(v) → v.to_string()
impl FromStr for SqlInt64;     // "Null" → NULL, valid i64 → Some(v), else ParseError
```

## Conversions

```rust
// Widening into SqlInt64 (infallible)
impl From<i64> for SqlInt64;
impl From<SqlBoolean> for SqlInt64;  // NULL→NULL, FALSE→0, TRUE→1

// Narrowing out of SqlInt64 (fallible)
impl SqlInt64 {
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError>;  // overflow if < i32::MIN or > i32::MAX
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>;  // overflow if < i16::MIN or > i16::MAX
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;    // overflow if < 0 or > 255
}
```

## Deferred (until target types are implemented or follow-up)

- `From<SqlByte> for SqlInt64` (widening — deferred per pattern)
- `From<SqlInt16> for SqlInt64` (widening — deferred per pattern)
- `From<SqlInt32> for SqlInt64` (widening — deferred per pattern)
- `SqlInt64::to_sql_boolean()` (narrowing — deferred per pattern)
- `From<SqlSingle> for SqlInt64` (narrowing — type not yet implemented)
- `From<SqlDouble> for SqlInt64` (narrowing — type not yet implemented)
- `From<SqlMoney> for SqlInt64` (narrowing — type not yet implemented)
- `From<SqlDecimal> for SqlInt64` (narrowing — type not yet implemented)
- `SqlInt64::to_sql_single()` (widening — type not yet implemented)
- `SqlInt64::to_sql_double()` (widening — type not yet implemented)
- `SqlInt64::to_sql_decimal()` (widening — type not yet implemented)
- `SqlInt64::to_sql_money()` (widening — type not yet implemented)
