# Data Model: SqlByte

**Feature**: 002-sql-byte | **Date**: 2026-03-01

## Entity: SqlByte

Rust equivalent of C# `System.Data.SqlTypes.SqlByte` — an unsigned 8-bit integer (0–255) with SQL NULL support.

### Internal Representation

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlByte {
    value: Option<u8>,  // None = SQL NULL
}
```

**Size**: 2 bytes (1-byte discriminant + 1-byte `u8`)

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `SqlByte { value: None }` | SQL NULL |
| `ZERO` | `SqlByte { value: Some(0) }` | Zero |
| `MIN_VALUE` | `SqlByte { value: Some(0) }` | Minimum (0) |
| `MAX_VALUE` | `SqlByte { value: Some(255) }` | Maximum (255) |

### Constructor & Inspectors

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(v: u8) -> Self` | Create non-null SqlByte |
| `is_null` | `fn is_null(&self) -> bool` | Check NULL |
| `value` | `fn value(&self) -> Result<u8, SqlTypeError>` | Extract value or NullValue error |

### Arithmetic (return `Result<SqlByte, SqlTypeError>`)

NULL + anything → `Ok(SqlByte::NULL)`. Overflow → `Err(Overflow)`. Div/mod by 0 → `Err(DivideByZero)`.

| Op | Overflow Detection |
|----|--------------------|
| `+` `-` `*` | `(i32_result & !0xFF) != 0` |
| `/` `%` | Divisor == 0 check only |

Named methods: `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`.

### Bitwise (return `SqlByte`)

NULL-propagating, never fail.

| Op | Behavior |
|----|----------|
| `&` `\|` `^` | Byte-level operation |
| `!` / `ones_complement()` | `!value` truncated to u8 |

### SQL Comparisons (return `SqlBoolean`)

NULL-propagating: either NULL → `SqlBoolean::NULL`.

`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`

### Rust Traits

| Trait | Behavior |
|-------|----------|
| `PartialEq`/`Eq` | NULL == NULL; NULL ≠ non-null |
| `Hash` | NULL → 0; else `u8.hash()` |
| `PartialOrd`/`Ord` | NULL < any non-null |
| `Display` | NULL → `"Null"`, else decimal |
| `FromStr` | `"Null"` → NULL, valid u8 → value, else error |
| `From<u8>` | Wraps in SqlByte |
| `From<SqlBoolean>` | TRUE→1, FALSE→0, NULL→NULL |

### Type Conversion Methods

| Method | Notes |
|--------|-------|
| `to_sql_boolean()` | 0→FALSE, non-zero→TRUE, NULL→NULL |

Widening conversions (`to_sql_int16`, etc.) will be added when target types are implemented.

### Validation Rules

1. `u8` is inherently bounded [0, 255] — no construction validation
2. Arithmetic overflow detected via widened `i32` + bitmask
3. Division/modulus by zero → `Err(DivideByZero)`
4. NULL propagation mandatory for all operations
