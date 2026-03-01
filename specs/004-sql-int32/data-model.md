# Data Model: SqlInt32

## Entity: SqlInt32

A nullable 32-bit signed integer representing SQL Server `INT`.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<i32>` | `None` = SQL NULL, `Some(v)` = value in range −2,147,483,648 to 2,147,483,647 |

### Invariants

- `value` is always `None` or `Some(v)` where `v` is any valid `i32` (no invalid states possible)
- Fixed-size, stack-allocated (`Copy + Clone`)

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `None` | SQL NULL sentinel |
| `ZERO` | `Some(0)` | Zero |
| `MIN_VALUE` | `Some(-2_147_483_648)` | `i32::MIN` |
| `MAX_VALUE` | `Some(2_147_483_647)` | `i32::MAX` |

### State Transitions

```
Construction:
  new(i32)    → Some(v)
  From<i32>   → Some(v)
  NULL const  → None

Arithmetic (Result-returning):
  (Some(a), Some(b)) → Ok(Some(result)) | Err(Overflow) | Err(DivideByZero)
  (None, _) | (_, None) → Ok(None)       [NULL propagation]

Bitwise (infallible):
  (Some(a), Some(b)) → Some(result)
  (None, _) | (_, None) → None            [NULL propagation]

Comparison (→ SqlBoolean):
  (Some(a), Some(b)) → TRUE | FALSE
  (None, _) | (_, None) → SqlBoolean::NULL [NULL propagation]
```

### Relationships

| Related Type | Direction | Relationship |
|-------------|-----------|--------------|
| `SqlBoolean` | FROM → SqlInt32 | Widening: TRUE→1, FALSE→0, NULL→NULL |
| `SqlBoolean` | SqlInt32 → | Via comparison methods (returns SqlBoolean) |
| `SqlInt16` | SqlInt32 → | Narrowing: overflow if value < -32768 or > 32767 |
| `SqlByte` | SqlInt32 → | Narrowing: overflow if value < 0 or > 255 |
| `SqlTypeError` | returned by | Arithmetic, narrowing conversions, value() on NULL |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| Overflow on add/sub/mul | Result outside i32 range | `SqlTypeError::Overflow` |
| Overflow on div | `MIN_VALUE / -1` | `SqlTypeError::Overflow` |
| Overflow on rem | `MIN_VALUE % -1` | `SqlTypeError::Overflow` |
| Overflow on neg | `value == MIN_VALUE` | `SqlTypeError::Overflow` |
| Divide by zero | Divisor is 0 (div or rem) | `SqlTypeError::DivideByZero` |
| Narrowing overflow | Value outside target range | `SqlTypeError::Overflow` |
| NULL access | `value()` on NULL | `SqlTypeError::NullValue` |
| Parse failure | Invalid string | `SqlTypeError::ParseError` |
