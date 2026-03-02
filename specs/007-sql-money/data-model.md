# Data Model: SqlMoney

## Entity: SqlMoney

A nullable fixed-point currency value representing SQL Server `MONEY` / `SMALLMONEY`.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<i64>` | `None` = SQL NULL, `Some(v)` = monetary value × 10,000 |

### Scale Factor

All values are stored internally as `actual_value × 10,000`. For example:
- `$100.00` → `Some(1_000_000)`
- `$0.0001` → `Some(1)`
- `$-50.1234` → `Some(-501_234)`

### Invariants

- `value` is always `None` or `Some(v)` where `v` is any valid `i64`
- No invalid states are possible — every `Option<i64>` is a valid SqlMoney
- Fixed-size, stack-allocated (`Copy + Clone`)
- The scale factor 10,000 is never stored; it is a compile-time constant

### Constants

| Name | Internal Value | User-Facing Value | Description |
|------|---------------|-------------------|-------------|
| `NULL` | `None` | N/A | SQL NULL sentinel |
| `ZERO` | `Some(0)` | 0.0000 | Zero |
| `MIN_VALUE` | `Some(i64::MIN)` | −922,337,203,685,477.5808 | Minimum representable value |
| `MAX_VALUE` | `Some(i64::MAX)` | 922,337,203,685,477.5807 | Maximum representable value |

### Constructors

| Constructor | Input | Scaling | Validation |
|-------------|-------|---------|------------|
| `from_i32(i32)` | Integer | `value * 10_000` | Always succeeds (i32 × 10,000 fits in i64) |
| `from_i64(i64)` | Integer | `value * 10_000` | Range check: `i64::MIN/10_000 ≤ value ≤ i64::MAX/10_000` |
| `from_f64(f64)` | Float | `round(value * 10_000.0)` | Reject NaN/Infinity, range check |
| `from_scaled(i64)` | Raw ticks | None (direct) | No validation — accepts any i64 |

### State Transitions

```
Construction:
  from_i32(i32)       → Some(v * 10_000)
  from_i64(i64)       → Ok(Some(v * 10_000)) | Err(Overflow)
  from_f64(f64)       → Ok(Some(round(v * 10_000.0))) | Err(OutOfRange/Overflow)
  from_scaled(i64)    → Some(v)  [no scaling]
  NULL const          → None

Checked Arithmetic (all Result-returning):
  (Some(a), Some(b))  → Ok(Some(result)) | Err(Overflow) | Err(DivideByZero)
  (None, _) | (_, None) → Ok(None)       [NULL propagation]

  Add/Sub: checked i64 on raw ticks, exact
  Mul:     i128 intermediate, re-scale by / 10_000, round, range check
  Div:     i128 intermediate, pre-scale by * 10_000, divide, round, range check

Negation:
  Some(v)              → Ok(Some(-v)) | Err(Overflow) if v == i64::MIN
  None                 → Ok(None)

Comparison (→ SqlBoolean):
  (Some(a), Some(b))   → TRUE | FALSE  [compare raw i64 ticks directly]
  (None, _) | (_, None) → SqlBoolean::NULL

Display:
  None                 → "Null"
  Some(v)              → format "#0.00##" (min 2dp, max 4dp)

Parse:
  "Null"               → None
  valid decimal string → Some(parsed * 10_000)
  invalid              → Err(ParseError)
```

### Relationships

| Related Type | Direction | Relationship |
|-------------|-----------|--------------|
| `SqlBoolean` | FROM → SqlMoney | Widening: TRUE→1.0000, FALSE→0.0000, NULL→NULL |
| `SqlBoolean` | SqlMoney → | Via `to_sql_boolean()`: zero→FALSE, non-zero→TRUE, NULL→NULL |
| `SqlBoolean` | SqlMoney → | Via comparison methods (returns SqlBoolean) |
| `SqlByte` | FROM → SqlMoney | Widening: value × 10,000, NULL→NULL |
| `SqlByte` | SqlMoney → | Via `to_sql_byte()`: round + range check (0..255) |
| `SqlInt16` | FROM → SqlMoney | Widening: value × 10,000, NULL→NULL |
| `SqlInt16` | SqlMoney → | Via `to_sql_int16()`: round + range check |
| `SqlInt32` | FROM → SqlMoney | Widening: value × 10,000, NULL→NULL |
| `SqlInt32` | SqlMoney → | Via `to_sql_int32()`: round + range check |
| `SqlInt64` | FROM → SqlMoney | Fallible: value × 10,000, range checked |
| `SqlInt64` | SqlMoney → | Via `to_sql_int64()`: round |
| `SqlDecimal` | SqlMoney → | Via `to_sql_decimal()`: exact, scale=4 |
| `SqlTypeError` | returned by | Arithmetic, accessors on NULL, narrowing conversions |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| Overflow on add/sub | Result outside i64 range | `SqlTypeError::Overflow` |
| Overflow on mul | i128 result doesn't fit in i64 after re-scaling | `SqlTypeError::Overflow` |
| Overflow on neg | `value == i64::MIN` | `SqlTypeError::Overflow` |
| Divide by zero | Divisor ticks == 0 | `SqlTypeError::DivideByZero` |
| Range overflow (from_i64) | `value * 10_000` overflows i64 | `SqlTypeError::Overflow` |
| Invalid float (from_f64) | NaN or Infinity input | `SqlTypeError::OutOfRange` |
| Float range (from_f64) | Scaled value outside i64 range | `SqlTypeError::Overflow` |
| Narrowing overflow | Rounded value outside target type range | `SqlTypeError::Overflow` |
| NULL access | `value()` or `scaled_value()` on NULL | `SqlTypeError::NullValue` |
| Parse failure | Invalid string input | `SqlTypeError::ParseError` |
