# Data Model: SqlString

## Entity: SqlCompareOptions

An enum controlling how `SqlString` values are compared.

| Variant | Description |
|---------|-------------|
| `None` | Case-sensitive ordinal comparison (`str::cmp`) |
| `IgnoreCase` | Case-insensitive ASCII comparison (`to_ascii_lowercase` then `str::cmp`) |
| `BinarySort` | Raw UTF-8 byte comparison (`[u8]::cmp`) |
| `BinarySort2` | Identical to `BinarySort` (C# legacy distinction not needed) |

### Traits

- `Copy`, `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`
- `Default` → `IgnoreCase`

### Invariants

- Fixed-size, stack-allocated (simple enum, no data)
- 4 variants, mutually exclusive (not a bitflag)

---

## Entity: SqlString

A nullable string value representing SQL Server `NVARCHAR` / `VARCHAR` with configurable comparison options.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<String>` | `None` = SQL NULL, `Some(s)` = string value (any valid UTF-8) |
| `compare_options` | `SqlCompareOptions` | Controls comparison behavior; default = `IgnoreCase` |

### Invariants

- `value` is always `None` or `Some(s)` where `s` is any valid UTF-8 `String` (including empty)
- Empty string (`Some(String::new())`) is NOT NULL
- Heap-allocated (`Clone` only, NOT `Copy`)
- `compare_options` is always one of the 4 `SqlCompareOptions` variants
- NULL instances have `compare_options = IgnoreCase` (default)

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `value: None, compare_options: IgnoreCase` | SQL NULL sentinel |

### State Transitions

```
Construction:
  new(&str)                          → Some(s), options = IgnoreCase
  with_options(&str, options)        → Some(s), options = given
  From<&str>                         → Some(s), options = IgnoreCase
  From<String>                       → Some(s), options = IgnoreCase
  NULL const                         → None, options = IgnoreCase
  FromStr("Null")                    → None

Concatenation (Add):
  (Some(a), Some(b)) → Some(a + b), options = left operand's options
  (None, _) | (_, None) → None      [NULL propagation]

SQL Comparison (→ SqlBoolean):
  (Some(a), Some(b)) → TRUE | FALSE  [using left operand's compare_options]
  (None, _) | (_, None) → SqlBoolean::NULL  [NULL propagation]

Accessors:
  value() on Some(s)  → Ok(&str)
  value() on None     → Err(NullValue)
  len() on Some(s)    → Ok(s.len())   [byte length]
  len() on None       → Err(NullValue)
```

### Comparison Logic Detail

```
Given left.compare_options:
  None       → trim_trailing_spaces(left) cmp trim_trailing_spaces(right) [ordinal]
  IgnoreCase → lowercase(trim_trailing_spaces(left)) cmp lowercase(trim_trailing_spaces(right))
  BinarySort → bytes(trim_trailing_spaces(left)) cmp bytes(trim_trailing_spaces(right))
  BinarySort2 → same as BinarySort
```

### Relationships

| Related Type | Direction | Relationship |
|-------------|-----------|--------------|
| `SqlCompareOptions` | embedded in | SqlString contains SqlCompareOptions field |
| `SqlBoolean` | returned by | SQL comparison methods return SqlBoolean |
| `SqlTypeError` | returned by | `value()`, `len()` on NULL return NullValue error |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| NULL access | `value()` on NULL | `SqlTypeError::NullValue` |
| NULL length | `len()` on NULL | `SqlTypeError::NullValue` |
| Parse "Null" | `FromStr` with "Null" (case-insensitive) | Returns `SqlString::NULL` (not an error) |

### Eq/Hash Normalization

For `PartialEq`, `Eq`, and `Hash` (Rust standard traits):
- Both NULLs are equal
- NULL ≠ any non-NULL
- Non-NULL values: compare by `to_ascii_lowercase()` of `trim_end_matches(' ')`
- Hash: hash the normalized form (lowercased + trailing-space-trimmed); NULL hashes as empty string

This ensures `Eq` and `Hash` are consistent regardless of `compare_options`.
