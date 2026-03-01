# Public API Contract: SqlInt32

## Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlInt32 {
    value: Option<i32>,  // None = SQL NULL
}
```

## Constants

```rust
impl SqlInt32 {
    pub const NULL: SqlInt32;       // SQL NULL
    pub const ZERO: SqlInt32;       // 0
    pub const MIN_VALUE: SqlInt32;  // -2_147_483_648 (i32::MIN)
    pub const MAX_VALUE: SqlInt32;  // 2_147_483_647 (i32::MAX)
}
```

## Constructors & Accessors

```rust
impl SqlInt32 {
    pub fn new(v: i32) -> Self;
    pub fn is_null(&self) -> bool;
    pub fn value(&self) -> Result<i32, SqlTypeError>;
}
```

## Checked Arithmetic

All return `Result<SqlInt32, SqlTypeError>`. NULL propagation: if either operand is NULL, returns `Ok(SqlInt32::NULL)`.

```rust
impl SqlInt32 {
    pub fn checked_add(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError>;
    pub fn checked_rem(self, rhs: SqlInt32) -> Result<SqlInt32, SqlTypeError>;
    pub fn checked_neg(self) -> Result<SqlInt32, SqlTypeError>;
}
```

### Error Conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | result < i32::MIN or > i32::MAX | `Overflow` |
| `checked_sub` | result < i32::MIN or > i32::MAX | `Overflow` |
| `checked_mul` | result < i32::MIN or > i32::MAX | `Overflow` |
| `checked_div` | rhs == 0 | `DivideByZero` |
| `checked_div` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_rem` | rhs == 0 | `DivideByZero` |
| `checked_rem` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_neg` | value == MIN_VALUE (-2,147,483,648) | `Overflow` |

## Operator Traits (delegate to checked_* methods)

```rust
impl Add for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
impl Sub for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
impl Mul for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
impl Div for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
impl Rem for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
impl Neg for SqlInt32 { type Output = Result<SqlInt32, SqlTypeError>; }
```

## Bitwise Operators (infallible, NULL propagation)

```rust
impl BitAnd for SqlInt32 { type Output = SqlInt32; }
impl BitOr  for SqlInt32 { type Output = SqlInt32; }
impl BitXor for SqlInt32 { type Output = SqlInt32; }
impl Not    for SqlInt32 { type Output = SqlInt32; }  // ones' complement
```

Additional method:

```rust
impl SqlInt32 {
    pub fn ones_complement(self) -> SqlInt32;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlInt32 {
    pub fn sql_equals(&self, other: &SqlInt32) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlInt32) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlInt32) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlInt32) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlInt32) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt32) -> SqlBoolean;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlInt32;  // NULL == NULL → true (Rust semantics)
impl Eq for SqlInt32;
impl Hash for SqlInt32;        // NULL → hash(0), Some(v) → hash(v)
impl PartialOrd for SqlInt32;  // NULL < any value
impl Ord for SqlInt32;         // total ordering: NULL < MIN..MAX
impl Display for SqlInt32;     // NULL → "Null", Some(v) → v.to_string()
impl FromStr for SqlInt32;     // "Null" → NULL, valid i32 → Some(v), else ParseError
```

## Conversions

```rust
// Widening into SqlInt32 (infallible)
impl From<i32> for SqlInt32;
impl From<SqlBoolean> for SqlInt32;  // NULL→NULL, FALSE→0, TRUE→1

// Narrowing out of SqlInt32 (fallible)
impl SqlInt32 {
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>;  // overflow if < -32768 or > 32767
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;    // overflow if < 0 or > 255
}
```

## Deferred (until target types are implemented or follow-up)

- `From<SqlByte> for SqlInt32` (widening — deferred per clarification)
- `From<SqlInt16> for SqlInt32` (widening — deferred per clarification)
- `SqlInt32::to_sql_boolean()` (narrowing — deferred per clarification)
- `From<SqlInt32> for SqlInt64` (widening — type not yet implemented)
- `From<SqlInt32> for SqlSingle` (widening — type not yet implemented)
- `From<SqlInt32> for SqlDouble` (widening — type not yet implemented)
- `From<SqlInt32> for SqlDecimal` (widening — type not yet implemented)
- `From<SqlInt32> for SqlMoney` (widening — type not yet implemented)
