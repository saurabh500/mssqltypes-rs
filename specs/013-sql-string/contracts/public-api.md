# Public API Contract: SqlString

## Enum: SqlCompareOptions

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum SqlCompareOptions {
    /// Case-sensitive ordinal comparison
    None,
    /// Case-insensitive ASCII comparison (default)
    #[default]
    IgnoreCase,
    /// Raw UTF-8 byte comparison
    BinarySort,
    /// Identical to BinarySort (C# legacy distinction)
    BinarySort2,
}
```

## Struct: SqlString

```rust
#[derive(Clone, Debug)]
pub struct SqlString {
    value: Option<String>,                // None = SQL NULL
    compare_options: SqlCompareOptions,   // default = IgnoreCase
}
```

## Constants

```rust
impl SqlString {
    pub const NULL: SqlString;  // SQL NULL with default IgnoreCase options
}
```

## Constructors & Accessors

```rust
impl SqlString {
    /// Create with default IgnoreCase compare options
    pub fn new(s: &str) -> Self;

    /// Create with explicit compare options
    pub fn with_options(s: &str, options: SqlCompareOptions) -> Self;

    /// Returns true if this is SQL NULL
    pub fn is_null(&self) -> bool;

    /// Returns the string value, or Err(NullValue) if NULL
    pub fn value(&self) -> Result<&str, SqlTypeError>;

    /// Returns byte length, or Err(NullValue) if NULL
    pub fn len(&self) -> Result<usize, SqlTypeError>;

    /// Returns the compare options for this instance
    pub fn compare_options(&self) -> SqlCompareOptions;
}
```

## Concatenation (infallible, NULL propagation)

```rust
impl Add for SqlString {
    type Output = SqlString;
    // NULL propagation: if either is NULL, returns NULL
    // Result inherits left operand's compare_options
}

impl Add<&SqlString> for SqlString {
    type Output = SqlString;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

All comparisons use the **left operand's** `compare_options`.

```rust
impl SqlString {
    pub fn sql_equals(&self, other: &SqlString) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlString) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlString) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlString) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlString) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlString) -> SqlBoolean;
}
```

### Comparison Behavior by Option

| SqlCompareOptions | Method | Trailing Spaces |
|-------------------|--------|-----------------|
| `None` | `str::cmp` (ordinal, case-sensitive) | Trimmed before comparison |
| `IgnoreCase` | `to_ascii_lowercase()` then `str::cmp` | Trimmed before comparison |
| `BinarySort` | `[u8]::cmp` (raw bytes) | Trimmed before comparison |
| `BinarySort2` | Same as `BinarySort` | Trimmed before comparison |

## Rust Standard Traits

```rust
impl PartialEq for SqlString;  // Case-insensitive ASCII, trailing-space-trimmed; NULL == NULL
impl Eq for SqlString;
impl Hash for SqlString;        // Hash of lowercased + trimmed value; NULL → hash of ""
impl PartialOrd for SqlString;  // Case-insensitive; NULL < any non-NULL
impl Ord for SqlString;         // Total ordering: NULL < all values, then case-insensitive
impl Display for SqlString;     // NULL → "Null", Some(s) → s
impl FromStr for SqlString;     // "Null" (case-insensitive) → NULL, else new(input)
```

## From Conversions

```rust
impl From<&str> for SqlString;     // Creates with default IgnoreCase options
impl From<String> for SqlString;   // Creates with default IgnoreCase options, avoids clone
```

## Deferred (until target types are implemented or follow-up)

- `SqlString::to_sql_boolean() -> Result<SqlBoolean, SqlTypeError>` (parse "True"/"False")
- `SqlString::to_sql_byte() -> Result<SqlByte, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_int16() -> Result<SqlInt16, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_int32() -> Result<SqlInt32, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_int64() -> Result<SqlInt64, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_single() -> Result<SqlSingle, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_double() -> Result<SqlDouble, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_decimal() -> Result<SqlDecimal, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_money() -> Result<SqlMoney, SqlTypeError>` (parse numeric)
- `SqlString::to_sql_date_time() -> Result<SqlDateTime, SqlTypeError>` (parse date)
- `SqlString::to_sql_guid() -> Result<SqlGuid, SqlTypeError>` (parse GUID)
- `From<SqlBoolean> for SqlString` (TRUE→"True", FALSE→"False")
- `From<SqlByte> for SqlString` (via Display)
- `From<SqlInt16> for SqlString` (via Display)
- `From<SqlInt32> for SqlString` (via Display)
- `From<SqlInt64> for SqlString` (via Display)
- `From<SqlSingle> for SqlString` (via Display)
- `From<SqlDouble> for SqlString` (via Display)
- `From<SqlDecimal> for SqlString` (via Display)
- `From<SqlMoney> for SqlString` (via Display)
- `From<SqlDateTime> for SqlString` (via Display)
- `From<SqlGuid> for SqlString` (via Display)
