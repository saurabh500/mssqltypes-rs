# Data Model: SqlInt64

## Entity: SqlInt64

A nullable 64-bit signed integer representing SQL Server `BIGINT`.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<i64>` | `None` = SQL NULL, `Some(v)` = value in range −9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 |

### Invariants

- `value` is always `None` or `Some(v)` where `v` is any valid `i64` (no invalid states possible)
- Fixed-size, stack-allocated (`Copy + Clone`)

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `None` | SQL NULL sentinel |
| `ZERO` | `Some(0)` | Zero |
| `MIN_VALUE` | `Some(-9_223_372_036_854_775_808)` | `i64::MIN` |
| `MAX_VALUE` | `Some(9_223_372_036_854_775_807)` | `i64::MAX` |

### State Transitions

```
Construction:
  new(i64)    → Some(v)
  From<i64>   → Some(v)
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
| `SqlBoolean` | FROM → SqlInt64 | Widening: TRUE→1, FALSE→0, NULL→NULL |
| `SqlBoolean` | SqlInt64 → | Via comparison methods (returns SqlBoolean) |
| `SqlInt32` | SqlInt64 → | Narrowing: overflow if value < i32::MIN or > i32::MAX |
| `SqlInt16` | SqlInt64 → | Narrowing: overflow if value < i16::MIN or > i16::MAX |
| `SqlByte` | SqlInt64 → | Narrowing: overflow if value < 0 or > 255 |
| `SqlTypeError` | returned by | Arithmetic, narrowing conversions, value() on NULL |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| Overflow on add/sub/mul | Result outside i64 range | `SqlTypeError::Overflow` |
| Overflow on div | `MIN_VALUE / -1` | `SqlTypeError::Overflow` |
| Overflow on rem | `MIN_VALUE % -1` | `SqlTypeError::Overflow` |
| Overflow on neg | `value == MIN_VALUE` | `SqlTypeError::Overflow` |
| Divide by zero | Divisor is 0 (div or rem) | `SqlTypeError::DivideByZero` |
| Narrowing overflow | Value outside target range | `SqlTypeError::Overflow` |
| NULL access | `value()` on NULL | `SqlTypeError::NullValue` |
| Parse failure | Invalid string | `SqlTypeError::ParseError` |
