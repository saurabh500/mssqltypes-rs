# Data Model: SqlDateTime

## Entity: SqlDateTime

A nullable date/time value representing SQL Server `DATETIME`, stored as days since 1900-01-01 and 1/300-second ticks since midnight.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<(i32, i32)>` | `None` = SQL NULL, `Some((day_ticks, time_ticks))` = valid date/time |

- `day_ticks`: Days since 1900-01-01. Negative for dates before 1900. Range: [-53690, 2958463]
- `time_ticks`: 1/300-second intervals since midnight. Range: [0, 25919999]

### Invariants

- `value` is always `None` or `Some((d, t))` where `d` ∈ [-53690, 2958463] and `t` ∈ [0, 25919999]
- No invalid state is representable after construction (all constructors validate)
- Fixed-size, stack-allocated (`Copy + Clone`)
- `time_ticks` is always non-negative (midnight = 0, last representable tick = 25919999)
- Millisecond rounding follows C# formula: `ticks = (int)(milliseconds * 0.3 + 0.5)`

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `NULL` | `None` | SQL NULL sentinel |
| `MIN_VALUE` | `Some((-53690, 0))` | 1753-01-01 00:00:00.000 |
| `MAX_VALUE` | `Some((2958463, 25919999))` | 9999-12-31 23:59:59.997 |

#### Tick-Rate Constants

| Name | Value | Description |
|------|-------|-------------|
| `TICKS_PER_SECOND` | `300` | 1/300 second resolution |
| `TICKS_PER_MINUTE` | `18_000` | 300 × 60 |
| `TICKS_PER_HOUR` | `1_080_000` | 300 × 3600 |
| `TICKS_PER_DAY` | `25_920_000` | 300 × 86400 |

#### Internal Constants (private)

| Name | Value | Description |
|------|-------|-------------|
| `DAY_BASE` | `693_595` | Absolute day number for 1900-01-01 (from year 1) |
| `MIN_DAY` | `-53_690` | Minimum day_ticks (1753-01-01) |
| `MAX_DAY` | `2_958_463` | Maximum day_ticks (9999-12-31) |
| `MIN_TIME` | `0` | Minimum time_ticks (midnight) |
| `MAX_TIME` | `25_919_999` | Maximum time_ticks (one tick before midnight) |
| `DAYS_TO_MONTH_365` | `[0,31,59,90,120,151,181,212,243,273,304,334,365]` | Cumulative days per month (non-leap year) |
| `DAYS_TO_MONTH_366` | `[0,31,60,91,121,152,182,213,244,274,305,335,366]` | Cumulative days per month (leap year) |

### State Transitions

```
Construction:
  new(year, month, day, hour, min, sec, ms) → Ok(Some((d, t))) | Err(OutOfRange)
  from_ticks(day, time)                     → Ok(Some((d, t))) | Err(OutOfRange)
  NULL const                                → None
  MIN_VALUE const                           → Some((-53690, 0))
  MAX_VALUE const                           → Some((2958463, 25919999))

Duration Arithmetic (Result-returning):
  Some((d, t)) + (day_delta, time_delta) → Ok(Some((d', t'))) | Err(OutOfRange)
  None + (any, any)                      → Ok(None)  [NULL propagation]

Comparison (→ SqlBoolean):
  (Some(a), Some(b)) → TRUE | FALSE       [lexicographic (day, time)]
  (None, _) | (_, None) → SqlBoolean::NULL [NULL propagation]

Component Extraction:
  Some((d, t)) → Ok(year | month | day | hour | minute | second)
  None         → Err(NullValue)

Calendar Computation (internal):
  (year, month, day) → day_ticks     [via Gregorian formula]
  day_ticks → (year, month, day)     [via 400/100/4/1-year cycle decomposition]
  (hour, min, sec, ms) → time_ticks  [via tick arithmetic + rounding]
  time_ticks → (hour, min, sec)      [via integer division by tick constants]
```

### Relationships

| Related Type | Direction | Relationship |
|-------------|-----------|--------------|
| `SqlBoolean` | SqlDateTime → | Via comparison methods (returns SqlBoolean) |
| `SqlTypeError` | returned by | Construction, arithmetic, value() on NULL, accessors on NULL |

### Validation Rules

| Rule | Trigger | Error |
|------|---------|-------|
| Day range | `day_ticks < -53690` or `day_ticks > 2958463` | `SqlTypeError::OutOfRange` |
| Time range | `time_ticks < 0` or `time_ticks > 25919999` | `SqlTypeError::OutOfRange` |
| Year range | `year < 1753` or `year > 9999` | `SqlTypeError::OutOfRange` |
| Month range | `month < 1` or `month > 12` | `SqlTypeError::OutOfRange` |
| Day range (calendar) | `day < 1` or `day > days_in_month(year, month)` | `SqlTypeError::OutOfRange` |
| Hour range | `hour < 0` or `hour > 23` | `SqlTypeError::OutOfRange` |
| Minute range | `minute < 0` or `minute > 59` | `SqlTypeError::OutOfRange` |
| Second range | `second < 0` or `second > 59` | `SqlTypeError::OutOfRange` |
| Millisecond range | `ms < 0.0` or `ms >= 1000.0` | `SqlTypeError::OutOfRange` |
| Midnight overflow | Rounding causes `time_ticks > MAX_TIME` | Time resets to 0, day increments by 1 |
| Day overflow from midnight | Day increment exceeds `MAX_DAY` | `SqlTypeError::OutOfRange` |
| Arithmetic result out of range | Result `day_ticks` outside [MIN_DAY, MAX_DAY] | `SqlTypeError::OutOfRange` |
| NULL access | `value()` or any accessor on NULL | `SqlTypeError::NullValue` |
| Parse failure | Invalid string format | `SqlTypeError::ParseError` |
| Parse out of range | Valid format but date outside valid range | `SqlTypeError::OutOfRange` |
