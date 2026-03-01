# Public API Contract: SqlInt16

## Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlInt16 {
    value: Option<i16>,  // None = SQL NULL
}
```

## Constants

```rust
impl SqlInt16 {
    pub const NULL: SqlInt16;       // SQL NULL
    pub const ZERO: SqlInt16;       // 0
    pub const MIN_VALUE: SqlInt16;  // -32768 (i16::MIN)
    pub const MAX_VALUE: SqlInt16;  // 32767 (i16::MAX)
}
```

## Constructors & Accessors

```rust
impl SqlInt16 {
    pub fn new(v: i16) -> Self;
    pub fn is_null(&self) -> bool;
    pub fn value(&self) -> Result<i16, SqlTypeError>;
}
```

## Checked Arithmetic

All return `Result<SqlInt16, SqlTypeError>`. NULL propagation: if either operand is NULL, returns `Ok(SqlInt16::NULL)`.

```rust
impl SqlInt16 {
    pub fn checked_add(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError>;
    pub fn checked_rem(self, rhs: SqlInt16) -> Result<SqlInt16, SqlTypeError>;
    pub fn checked_neg(self) -> Result<SqlInt16, SqlTypeError>;
}
```

### Error conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | result < -32768 or > 32767 | `Overflow` |
| `checked_sub` | result < -32768 or > 32767 | `Overflow` |
| `checked_mul` | result < -32768 or > 32767 | `Overflow` |
| `checked_div` | rhs == 0 | `DivideByZero` |
| `checked_div` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_rem` | rhs == 0 | `DivideByZero` |
| `checked_rem` | lhs == MIN_VALUE && rhs == -1 | `Overflow` |
| `checked_neg` | value == MIN_VALUE (-32768) | `Overflow` |

## Operator Traits (delegate to checked_* methods)

```rust
impl Add for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
impl Sub for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
impl Mul for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
impl Div for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
impl Rem for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
impl Neg for SqlInt16 { type Output = Result<SqlInt16, SqlTypeError>; }
```

## Bitwise Operators (infallible, NULL propagation)

```rust
impl BitAnd for SqlInt16 { type Output = SqlInt16; }
impl BitOr  for SqlInt16 { type Output = SqlInt16; }
impl BitXor for SqlInt16 { type Output = SqlInt16; }
impl Not    for SqlInt16 { type Output = SqlInt16; }  // ones' complement
```

Additional method:

```rust
impl SqlInt16 {
    pub fn ones_complement(self) -> SqlInt16;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlInt16 {
    pub fn sql_equals(&self, other: &SqlInt16) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlInt16) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlInt16) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlInt16) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlInt16) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlInt16) -> SqlBoolean;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlInt16;  // NULL == NULL → true (Rust semantics)
impl Eq for SqlInt16;
impl Hash for SqlInt16;        // NULL → hash(0), Some(v) → hash(v)
impl PartialOrd for SqlInt16;  // NULL < any value
impl Ord for SqlInt16;         // total ordering: NULL < MIN..MAX
impl Display for SqlInt16;     // NULL → "Null", Some(v) → v.to_string()
impl FromStr for SqlInt16;     // "Null" → NULL, valid i16 → Some(v), else ParseError
```

## Conversions

```rust
// Widening into SqlInt16 (infallible)
impl From<i16> for SqlInt16;
impl From<SqlBoolean> for SqlInt16;  // NULL→NULL, FALSE→0, TRUE→1
impl From<SqlByte> for SqlInt16;     // NULL→NULL, u8 fits in i16

// Narrowing out of SqlInt16 (fallible)
impl SqlInt16 {
    pub fn to_sql_boolean(&self) -> SqlBoolean;                    // NULL→NULL, 0→FALSE, nonzero→TRUE
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;    // overflow if <0 or >255
}
```

## Deferred (until target types are implemented)

- `From<SqlInt16> for SqlInt32` (widening)
- `From<SqlInt16> for SqlInt64` (widening)
- `From<SqlInt16> for SqlSingle` (widening)
- `From<SqlInt16> for SqlDouble` (widening)
- `From<SqlInt16> for SqlDecimal` (widening)
- `From<SqlInt16> for SqlMoney` (widening)
