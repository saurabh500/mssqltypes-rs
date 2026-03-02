# Public API Contract: SqlGuid

## Struct: SqlGuid

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlGuid {
    value: Option<[u8; 16]>,  // None = SQL NULL
}
```

## Constants

```rust
impl SqlGuid {
    pub const NULL: SqlGuid;  // SQL NULL (value: None)

    /// SQL Server's non-standard byte comparison order.
    const SQL_GUID_ORDER: [usize; 16];  // [10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]
}
```

## Constructors & Accessors

```rust
impl SqlGuid {
    /// Create a new SqlGuid from a 16-byte array.
    pub fn new(bytes: [u8; 16]) -> Self;

    /// Returns true if this is SQL NULL.
    pub fn is_null(&self) -> bool;

    /// Returns the 16-byte array, or Err(NullValue) if NULL.
    pub fn value(&self) -> Result<[u8; 16], SqlTypeError>;

    /// Returns the 16-byte array, or Err(NullValue) if NULL.
    /// Alias for value() matching C# ToByteArray().
    pub fn to_byte_array(&self) -> Result<[u8; 16], SqlTypeError>;
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

All comparisons use SQL Server's non-standard byte order `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`.

```rust
impl SqlGuid {
    pub fn sql_equals(&self, other: &SqlGuid) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlGuid) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlGuid) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlGuid) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlGuid) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlGuid) -> SqlBoolean;
}
```

### SQL Comparison Byte Order

| Priority | Byte Indices | GUID Component |
|----------|-------------|----------------|
| 1 (first) | 10, 11, 12, 13, 14, 15 | Node (MAC address) |
| 2 | 8, 9 | Clock sequence |
| 3 | 6, 7 | Time high & version |
| 4 | 4, 5 | Time mid |
| 5 (last) | 0, 1, 2, 3 | Time low |

## Conversions

```rust
impl SqlGuid {
    /// Convert to SqlBinary (16 bytes). NULL returns SqlBinary::NULL.
    pub fn to_sql_binary(&self) -> SqlBinary;

    /// Convert from SqlBinary. Requires exactly 16 bytes.
    /// NULL SqlBinary returns Ok(SqlGuid::NULL).
    pub fn from_sql_binary(binary: &SqlBinary) -> Result<SqlGuid, SqlTypeError>;
}

impl From<[u8; 16]> for SqlGuid;  // Creates non-null SqlGuid
```

## Rust Standard Traits

```rust
impl Display for SqlGuid;      // NULL → "Null", value → "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" (lowercase)
impl FromStr for SqlGuid;      // Parses hyphenated or bare hex; "Null" → NULL; errors → ParseError
impl PartialEq for SqlGuid;    // Byte equality (natural order); NULL == NULL
impl Eq for SqlGuid;
impl Hash for SqlGuid;         // Hash raw bytes; consistent with Eq
impl PartialOrd for SqlGuid;   // SQL Server byte ordering; NULL < any non-NULL
impl Ord for SqlGuid;          // Total ordering: NULL < all non-NULL
```

## NOT Implemented

- `Add`, `Sub`, or any arithmetic — GUIDs have no arithmetic operations
- `FromStr` with braced `{...}` or parenthesized `(...)` formats (deferred)
- Component constructor `new(i32, i16, i16, u8, u8, u8, u8, u8, u8, u8, u8)` (deferred — can be added later)
