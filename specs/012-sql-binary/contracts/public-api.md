# Public API Contract: SqlBinary

## Struct: SqlBinary

```rust
#[derive(Clone, Debug)]
pub struct SqlBinary {
    value: Option<Vec<u8>>,  // None = SQL NULL
}
```

## Constants

```rust
impl SqlBinary {
    pub const NULL: SqlBinary;  // SQL NULL (value: None)
}
```

## Constructors & Accessors

```rust
impl SqlBinary {
    /// Create a new SqlBinary taking ownership of a byte vector
    pub fn new(v: Vec<u8>) -> Self;

    /// Returns true if this is SQL NULL
    pub fn is_null(&self) -> bool;

    /// Returns the byte slice, or Err(NullValue) if NULL
    pub fn value(&self) -> Result<&[u8], SqlTypeError>;

    /// Returns byte count, or Err(NullValue) if NULL
    pub fn len(&self) -> Result<usize, SqlTypeError>;

    /// Returns true if length is 0, or Err(NullValue) if NULL
    pub fn is_empty(&self) -> Result<bool, SqlTypeError>;

    /// Returns byte at index, or Err(OutOfRange) / Err(NullValue)
    pub fn get(&self, index: usize) -> Result<u8, SqlTypeError>;
}
```

## Concatenation (infallible, NULL propagation)

```rust
impl Add for SqlBinary {
    type Output = SqlBinary;
    // NULL propagation: if either is NULL, returns NULL
    // Result contains left bytes followed by right bytes
}
```

## SQL Comparisons (return SqlBoolean, NULL propagation)

All comparisons use trailing-zero-padded byte comparison.

```rust
impl SqlBinary {
    pub fn sql_equals(&self, other: &SqlBinary) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlBinary) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlBinary) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlBinary) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlBinary) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlBinary) -> SqlBoolean;
}
```

### Comparison Behavior (Trailing-Zero Padding)

| Left | Right | Result | Reason |
|------|-------|--------|--------|
| `[1,2]` | `[1,2,0,0]` | Equal | Trailing zeros padded |
| `[1,2]` | `[1,3]` | Less | Byte index 1 differs |
| `[1,2,1]` | `[1,2]` | Greater | Extra non-zero byte |
| `[0]` | `[]` | Equal | Trailing zero padding |
| `[]` | `[]` | Equal | Both empty |
| any | NULL | NULL | NULL propagation |

## Rust Standard Traits

```rust
impl Display for SqlBinary;     // NULL → "Null", Some(v) → lowercase hex (e.g., "0aff")
impl PartialEq for SqlBinary;   // Trailing-zero-padded; NULL == NULL
impl Eq for SqlBinary;
impl Hash for SqlBinary;        // Trim trailing zeros before hashing; NULL → hash of empty
impl PartialOrd for SqlBinary;  // Trailing-zero-padded ordering; NULL < any non-NULL
impl Ord for SqlBinary;         // Total ordering: NULL < all non-NULL values
```

## From Conversions

```rust
impl From<&[u8]> for SqlBinary;    // Creates by cloning the slice
impl From<Vec<u8>> for SqlBinary;  // Creates by taking ownership (no clone)
```

## NOT Implemented

- `FromStr` — binary data is not parsed from strings (no C# equivalent)
- `Copy` — SqlBinary contains `Vec<u8>` (heap-allocated)
- `Index` trait — use `get()` method instead (returns Result, no panics)
