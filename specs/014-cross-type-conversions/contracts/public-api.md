# Public API Contract: Cross-Type Conversions

**Feature**: 014-cross-type-conversions
**Date**: 2026-03-02

## New `From` Trait Implementations

### Widening Integer Conversions

```rust
// In sql_int32.rs
impl From<SqlByte> for SqlInt32 {
    fn from(value: SqlByte) -> Self;         // NULL → NULL, lossless
}
impl From<SqlInt16> for SqlInt32 {
    fn from(value: SqlInt16) -> Self;        // NULL → NULL, lossless
}

// In sql_int64.rs
impl From<SqlByte> for SqlInt64 {
    fn from(value: SqlByte) -> Self;         // NULL → NULL, lossless
}
impl From<SqlInt16> for SqlInt64 {
    fn from(value: SqlInt16) -> Self;        // NULL → NULL, lossless
}
impl From<SqlInt32> for SqlInt64 {
    fn from(value: SqlInt32) -> Self;        // NULL → NULL, lossless
}
```

### Float/Money → SqlDecimal

```rust
// In sql_decimal.rs
impl From<SqlSingle> for SqlDecimal {
    fn from(value: SqlSingle) -> Self;       // NULL → NULL, panics on NaN/Infinity
}
impl From<SqlDouble> for SqlDecimal {
    fn from(value: SqlDouble) -> Self;       // NULL → NULL, panics on NaN/Infinity
}
impl From<SqlMoney> for SqlDecimal {
    fn from(value: SqlMoney) -> Self;        // NULL → NULL, preserves 4-decimal scale
}
```

> **Note**: `From<SqlSingle>` and `From<SqlDouble>` for `SqlDecimal` will panic on NaN/Infinity inputs because `From` must be infallible. Callers should validate inputs are finite before conversion. This matches the C# behavior where the implicit operator throws `OverflowException`.

## New Methods on Existing Types

### SqlInt32

```rust
impl SqlInt32 {
    /// Converts to SqlBoolean. Zero → FALSE, non-zero → TRUE, NULL → NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean;

    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlInt64

```rust
impl SqlInt64 {
    /// Converts to SqlBoolean. Zero → FALSE, non-zero → TRUE, NULL → NULL.
    pub fn to_sql_boolean(&self) -> SqlBoolean;

    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlBoolean

```rust
impl SqlBoolean {
    /// Converts to SqlString. TRUE → "True", FALSE → "False", NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlByte

```rust
impl SqlByte {
    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlInt16

```rust
impl SqlInt16 {
    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlSingle

```rust
impl SqlSingle {
    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlDouble

```rust
impl SqlDouble {
    /// Widens SqlSingle to SqlDouble. NULL → NULL.
    pub fn from_sql_single(value: SqlSingle) -> SqlDouble;

    /// Narrows to SqlSingle. Returns Err(Overflow) if out of f32 range. NULL → NULL.
    pub fn to_sql_single(&self) -> Result<SqlSingle, SqlTypeError>;

    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlDecimal

```rust
impl SqlDecimal {
    /// Converts to SqlSingle. May lose precision. NULL → NULL.
    pub fn to_sql_single(&self) -> SqlSingle;

    /// Converts to SqlDouble. May lose precision. NULL → NULL.
    pub fn to_sql_double(&self) -> SqlDouble;

    /// Converts to SqlMoney. Returns Err(Overflow) if out of money range. NULL → NULL.
    pub fn to_sql_money(&self) -> Result<SqlMoney, SqlTypeError>;

    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlMoney

```rust
impl SqlMoney {
    /// Constructs from SqlSingle. Returns Err(Overflow) if out of range. NULL → NULL.
    pub fn from_sql_single(value: SqlSingle) -> Result<SqlMoney, SqlTypeError>;

    /// Constructs from SqlDouble. Returns Err(Overflow) if out of range. NULL → NULL.
    pub fn from_sql_double(value: SqlDouble) -> Result<SqlMoney, SqlTypeError>;

    /// Converts to SqlSingle. May lose precision. NULL → NULL.
    pub fn to_sql_single(&self) -> SqlSingle;

    /// Converts to SqlDouble. May lose precision. NULL → NULL.
    pub fn to_sql_double(&self) -> SqlDouble;

    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlDateTime

```rust
impl SqlDateTime {
    /// Converts to SqlString via Display formatting. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;

    /// Parses from SqlString. Returns Err(ParseError) on invalid input. NULL → Ok(NULL).
    pub fn from_sql_string(value: &SqlString) -> Result<SqlDateTime, SqlTypeError>;
}
```

### SqlGuid

```rust
impl SqlGuid {
    /// Converts to SqlString as hyphenated GUID. NULL → NULL.
    pub fn to_sql_string(&self) -> SqlString;
}
```

### SqlString (parsing hub)

```rust
impl SqlString {
    /// Parses to SqlBoolean. Returns Err(ParseError) on invalid input. NULL → Ok(NULL).
    pub fn to_sql_boolean(&self) -> Result<SqlBoolean, SqlTypeError>;

    /// Parses to SqlByte. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>;

    /// Parses to SqlInt16. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>;

    /// Parses to SqlInt32. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_int32(&self) -> Result<SqlInt32, SqlTypeError>;

    /// Parses to SqlInt64. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_int64(&self) -> Result<SqlInt64, SqlTypeError>;

    /// Parses to SqlSingle. Returns Err on invalid/NaN/Infinity. NULL → Ok(NULL).
    pub fn to_sql_single(&self) -> Result<SqlSingle, SqlTypeError>;

    /// Parses to SqlDouble. Returns Err on invalid/NaN/Infinity. NULL → Ok(NULL).
    pub fn to_sql_double(&self) -> Result<SqlDouble, SqlTypeError>;

    /// Parses to SqlDecimal. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_decimal(&self) -> Result<SqlDecimal, SqlTypeError>;

    /// Parses to SqlMoney. Returns Err on invalid/overflow. NULL → Ok(NULL).
    pub fn to_sql_money(&self) -> Result<SqlMoney, SqlTypeError>;

    /// Parses to SqlDateTime. Returns Err on invalid/out-of-range. NULL → Ok(NULL).
    pub fn to_sql_date_time(&self) -> Result<SqlDateTime, SqlTypeError>;

    /// Parses to SqlGuid. Returns Err on invalid format. NULL → Ok(NULL).
    pub fn to_sql_guid(&self) -> Result<SqlGuid, SqlTypeError>;
}
```

## Error Handling Contract

All conversion errors use existing `SqlTypeError` variants:

| Scenario | Error Variant |
|----------|---------------|
| Narrowing overflow (e.g., `SqlDouble(1e300)` → `SqlSingle`) | `SqlTypeError::Overflow` |
| NaN/Infinity → SqlDecimal | `SqlTypeError::OutOfRange(message)` |
| SqlDecimal → SqlMoney range exceeded | `SqlTypeError::Overflow` |
| SqlDouble → SqlMoney range exceeded | `SqlTypeError::Overflow` |
| SqlString parse failure | `SqlTypeError::ParseError(message)` |
| Accessing value of NULL (via `.value()`) | `SqlTypeError::NullValue` |

## NULL Propagation Contract

Every conversion method follows this rule:
- **Infallible methods** (`to_sql_string()`, `to_sql_boolean()`, `to_sql_single()`, etc.): NULL input → target type's NULL constant
- **Fallible methods** (`SqlString::to_sql_int32()`, etc.): NULL input → `Ok(Target::NULL)`
- **`From` impls**: NULL input → target type's NULL constant
