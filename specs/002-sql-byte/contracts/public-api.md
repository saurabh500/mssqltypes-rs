# Public API Contract: SqlByte

**Feature**: 002-sql-byte | **Type**: Rust library public API

## Module: `mssqltypes::sql_byte`

### Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlByte { /* private */ }
```

### Constants

```rust
impl SqlByte {
    pub const NULL: SqlByte;
    pub const ZERO: SqlByte;
    pub const MIN_VALUE: SqlByte;
    pub const MAX_VALUE: SqlByte;
}
```

### Constructors & Inspectors

```rust
impl SqlByte {
    pub fn new(value: u8) -> SqlByte;
    pub fn is_null(&self) -> bool;
    pub fn value(&self) -> Result<u8, SqlTypeError>;
}
```

### Checked Arithmetic

```rust
impl SqlByte {
    pub fn checked_add(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError>;
    pub fn checked_sub(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError>;
    pub fn checked_mul(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError>;
    pub fn checked_div(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError>;
    pub fn checked_rem(self, rhs: SqlByte) -> Result<SqlByte, SqlTypeError>;
    pub fn ones_complement(self) -> SqlByte;
}
```

### SQL Comparisons

```rust
impl SqlByte {
    pub fn sql_equals(&self, other: &SqlByte) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlByte) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlByte) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlByte) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlByte) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlByte) -> SqlBoolean;
}
```

### Type Conversions

```rust
impl SqlByte {
    pub fn to_sql_boolean(&self) -> SqlBoolean;
}
```

### Operator Traits

```rust
impl Add for SqlByte { type Output = Result<SqlByte, SqlTypeError>; }
impl Sub for SqlByte { type Output = Result<SqlByte, SqlTypeError>; }
impl Mul for SqlByte { type Output = Result<SqlByte, SqlTypeError>; }
impl Div for SqlByte { type Output = Result<SqlByte, SqlTypeError>; }
impl Rem for SqlByte { type Output = Result<SqlByte, SqlTypeError>; }
impl BitAnd for SqlByte { type Output = SqlByte; }
impl BitOr  for SqlByte { type Output = SqlByte; }
impl BitXor for SqlByte { type Output = SqlByte; }
impl Not    for SqlByte { type Output = SqlByte; }
```

### Standard Traits

```rust
impl PartialEq for SqlByte {}  // NULL == NULL
impl Eq for SqlByte {}
impl Hash for SqlByte {}       // NULL → 0
impl PartialOrd for SqlByte {} // NULL < non-null
impl Ord for SqlByte {}
impl Display for SqlByte {}    // NULL → "Null"
impl FromStr for SqlByte {}    // "Null" → NULL
impl From<u8> for SqlByte {}
impl From<SqlBoolean> for SqlByte {}
```

### Re-export

```rust
// lib.rs additions:
pub mod sql_byte;
pub use sql_byte::SqlByte;
```

## Behavioral Contracts

| Scenario | Result |
|----------|--------|
| `SqlByte(x) + SqlByte::NULL` | `Ok(SqlByte::NULL)` |
| `SqlByte(200) + SqlByte(100)` | `Err(Overflow)` |
| `SqlByte(5) - SqlByte(10)` | `Err(Overflow)` |
| `SqlByte(x) / SqlByte(0)` | `Err(DivideByZero)` |
| `!SqlByte::NULL` | `SqlByte::NULL` |
| `SqlByte(x).sql_equals(&SqlByte::NULL)` | `SqlBoolean::NULL` |
| `"42".parse::<SqlByte>()` | `Ok(SqlByte(42))` |
| `"256".parse::<SqlByte>()` | `Err(ParseError)` |
