# Data Model: SqlDecimal

## Entity: SqlDecimal

A nullable fixed-point decimal number representing SQL Server `DECIMAL`/`NUMERIC` with up to 38 digits of precision.

### Inner Representation: InnerDecimal

| Field | Type | Description |
|-------|------|-------------|
| `precision` | `u8` | Number of significant digits (1–38) |
| `scale` | `u8` | Number of digits after the decimal point (0–precision) |
| `positive` | `bool` | `true` = positive or zero, `false` = negative |
| `data` | `[u32; 4]` | 128-bit unsigned mantissa in little-endian order (`data[0]` = least significant) |

### Outer Representation: SqlDecimal

| Field | Type | Description |
|-------|------|-------------|
| `inner` | `Option<InnerDecimal>` | `None` = SQL NULL, `Some(v)` = a value |

### Invariants

- `precision` is always in range 1–38
- `scale` is always in range 0–precision
- When the mantissa is zero (`data == [0, 0, 0, 0]`), `positive` MUST be `true` (negative zero normalized)
- The mantissa value MUST fit within the declared precision (i.e., the decimal digit count of the mantissa ≤ precision)
- `inner` is always `None` or `Some(v)` satisfying the above invariants
- Fixed-size, stack-allocated (`Clone` but not `Copy`)
- Total struct size: ~20 bytes (1 + 1 + 1 + 1 padding + 16 data)

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `None` | SQL NULL sentinel |
| `MAX_PRECISION` | `38` | Maximum allowed precision |
| `MAX_SCALE` | `38` | Maximum allowed scale |
| `MAX_VALUE` | `Some(precision=38, scale=0, positive=true, data=[0x098A2240, 0x5A86C47A, 0x4B3B4CA8, 0x4EE2D6D4])` | 10^38 - 1 = 99999999999999999999999999999999999999 |
| `MIN_VALUE` | `Some(precision=38, scale=0, positive=false, data=[0x098A2240, 0x5A86C47A, 0x4B3B4CA8, 0x4EE2D6D4])` | -(10^38 - 1) |

### Precision-to-Length Mapping

| Precision Range | Active u32 Words (`_bLen`) |
|----------------|---------------------------|
| 1–9 | 1 |
| 10–19 | 2 |
| 20–28 | 3 |
| 29–38 | 4 |

### State Transitions

```
Construction:
  new(prec, scale, positive, d1, d2, d3, d4) → Some(v) | Err(Overflow/OutOfRange)
  From<i32>                                   → Some(precision=10, scale=0)
  From<i64>                                   → Some(precision=19, scale=0)
  From<SqlBoolean>                            → Some(1) | Some(0) | None
  FromStr("123.45")                           → Some(precision=5, scale=2)
  NULL const                                  → None

Arithmetic (Result-returning):
  (Some(a), Some(b)) → Ok(Some(result)) | Err(Overflow) | Err(DivideByZero)
  (None, _) | (_, None) → Ok(None)          [NULL propagation]

  Precision/scale propagation:
    Add/Sub: ResPrec = min(38, max(p1-s1, p2-s2) + max(s1,s2) + 1)
    Mul:     ResPrec = min(38, s1+s2 + (p1-s1)+(p2-s2) + 1)
    Div:     ResPrec = min(38, max(s1+p2+1, 6) + (p1-s1)+s2 + 1)

Scale adjustment:
  adjust_scale(new_scale, round=true)  → round-half-up
  adjust_scale(new_scale, round=false) → truncate
  
  Increase scale: multiply mantissa by 10^diff, increase scale
  Decrease scale: divide mantissa by 10^diff, optionally round

Negation:
  -Some(v) → Some(-v)     [flip sign, normalize -0 to +0]
  -None    → None

Comparison (→ SqlBoolean):
  (Some(a), Some(b)) → TRUE | FALSE    [scale-normalized before compare]
  (None, _) | (_, None) → SqlBoolean::NULL

Math functions:
  abs(Some(v))       → Some(|v|)
  floor(Some(v))     → Some(⌊v⌋)
  ceiling(Some(v))   → Some(⌈v⌉)
  round(Some(v), n)  → Some(rounded to n digits)
  truncate(Some(v), n) → Some(truncated to n digits)
  sign(Some(v))      → -1 | 0 | 1
  power(Some(v), e)  → Some(v^e) via f64
  Any func on None   → None
```

### Relationships

| Related Type | Direction | Relationship |
|-------------|-----------|--------------|
| `SqlBoolean` | FROM → SqlDecimal | Widening: TRUE→1, FALSE→0, NULL→NULL |
| `SqlBoolean` | SqlDecimal → | Via comparison methods (returns SqlBoolean) |
| `SqlBoolean` | SqlDecimal → | to_sql_boolean(): 0→FALSE, non-zero→TRUE, NULL→NULL |
| `SqlByte` | FROM → SqlDecimal | Widening: precision=3, scale=0 |
| `SqlByte` | SqlDecimal → | Narrowing: truncate fractional, range check 0–255 |
| `SqlInt16` | FROM → SqlDecimal | Widening: precision=5, scale=0 |
| `SqlInt16` | SqlDecimal → | Narrowing: truncate fractional, range check |
| `SqlInt32` | FROM → SqlDecimal | Widening: precision=10, scale=0 |
| `SqlInt32` | SqlDecimal → | Narrowing: truncate fractional, range check |
| `SqlInt64` | FROM → SqlDecimal | Widening: precision=19, scale=0 |
| `SqlInt64` | SqlDecimal → | Narrowing: truncate fractional, range check |
| `f64` | SqlDecimal → | Lossy conversion to closest double |
| `i32` | FROM → SqlDecimal | Widening: precision=10, scale=0 |
| `i64` | FROM → SqlDecimal | Widening: precision=19, scale=0 |
| `SqlTypeError` | returned by | Arithmetic, construction, conversions, value() on NULL |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| Invalid precision | precision < 1 or > 38 | `SqlTypeError::OutOfRange` |
| Invalid scale | scale > precision | `SqlTypeError::OutOfRange` |
| Mantissa too large | digit count exceeds precision | `SqlTypeError::Overflow` |
| Arithmetic overflow | result exceeds precision 38 | `SqlTypeError::Overflow` |
| Division by zero | divisor mantissa is zero | `SqlTypeError::DivideByZero` |
| Scale adjustment overflow | new precision would exceed 38 | `SqlTypeError::Overflow` |
| Narrowing overflow | value outside target integer range | `SqlTypeError::Overflow` |
| NULL access | `value()` on NULL | `SqlTypeError::NullValue` |
| Parse failure | invalid string format | `SqlTypeError::ParseError` |

### Multi-Precision Arithmetic Internals

These are private helper functions, not part of the public API:

| Helper | Purpose |
|--------|---------|
| `mp_add(a, b) → (result, carry)` | Add two u32 arrays with carry |
| `mp_sub(a, b) → (result, borrow)` | Subtract two u32 arrays with borrow |
| `mp_mul(a, b) → result` | Multiply two u32 arrays (schoolbook) |
| `mp_div(a, b) → (quotient, remainder)` | Divide u32 arrays (Knuth Algorithm D) |
| `mp_mul1(a, scalar) → (result, carry)` | Multiply u32 array by single u32 |
| `mp_div1(a, scalar) → (result, remainder)` | Divide u32 array by single u32 |
| `mp_cmp(a, b) → Ordering` | Compare two u32 arrays |
| `calculate_precision(data) → u8` | Count decimal digits in mantissa |
| `is_zero(data) → bool` | Check if mantissa is all zeros |
