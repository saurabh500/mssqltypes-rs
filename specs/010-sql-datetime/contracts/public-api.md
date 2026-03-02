# Public API Contract: SqlDateTime

## Struct

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlDateTime {
    value: Option<(i32, i32)>,  // None = SQL NULL, Some((day_ticks, time_ticks))
}
```

## Constants

```rust
impl SqlDateTime {
    pub const NULL: SqlDateTime;       // SQL NULL
    pub const MIN_VALUE: SqlDateTime;  // 1753-01-01 00:00:00.000 (day=-53690, time=0)
    pub const MAX_VALUE: SqlDateTime;  // 9999-12-31 23:59:59.997 (day=2958463, time=25919999)

    pub const TICKS_PER_SECOND: i32;   // 300
    pub const TICKS_PER_MINUTE: i32;   // 18_000
    pub const TICKS_PER_HOUR: i32;     // 1_080_000
    pub const TICKS_PER_DAY: i32;      // 25_920_000
}
```

## Constructors

```rust
impl SqlDateTime {
    /// Create from calendar components. Validates all components and range.
    /// Milliseconds are rounded to nearest 1/300-second tick via (int)(ms * 0.3 + 0.5).
    /// Handles midnight overflow (time rolls over → day increments).
    pub fn new(
        year: i32, month: i32, day: i32,
        hour: i32, minute: i32, second: i32,
        millisecond: f64,
    ) -> Result<SqlDateTime, SqlTypeError>;

    /// Create from raw tick values. Validates day_ticks ∈ [-53690, 2958463]
    /// and time_ticks ∈ [0, 25919999].
    pub fn from_ticks(day_ticks: i32, time_ticks: i32) -> Result<SqlDateTime, SqlTypeError>;
}
```

## Accessors

```rust
impl SqlDateTime {
    pub fn is_null(&self) -> bool;
    pub fn value(&self) -> Result<(i32, i32), SqlTypeError>;  // returns (day_ticks, time_ticks)
    pub fn day_ticks(&self) -> Result<i32, SqlTypeError>;     // NullValue if NULL
    pub fn time_ticks(&self) -> Result<i32, SqlTypeError>;    // NullValue if NULL
}
```

## Calendar Component Extraction

All return `Err(SqlTypeError::NullValue)` if the value is NULL.

```rust
impl SqlDateTime {
    pub fn year(&self) -> Result<i32, SqlTypeError>;
    pub fn month(&self) -> Result<i32, SqlTypeError>;
    pub fn day(&self) -> Result<i32, SqlTypeError>;
    pub fn hour(&self) -> Result<i32, SqlTypeError>;
    pub fn minute(&self) -> Result<i32, SqlTypeError>;
    pub fn second(&self) -> Result<i32, SqlTypeError>;
}
```

## Duration Arithmetic

All return `Result<SqlDateTime, SqlTypeError>`. NULL propagation: if self is NULL, returns `Ok(SqlDateTime::NULL)`. Out-of-range result returns `Err(OutOfRange)`.

```rust
impl SqlDateTime {
    /// Add a duration expressed as day and time tick offsets.
    /// time_delta is normalized: overflow/underflow carries into day_delta.
    pub fn checked_add(self, day_delta: i32, time_delta: i32) -> Result<SqlDateTime, SqlTypeError>;

    /// Subtract a duration expressed as day and time tick offsets.
    pub fn checked_sub(self, day_delta: i32, time_delta: i32) -> Result<SqlDateTime, SqlTypeError>;

    /// Add days only (convenience for checked_add(days, 0)).
    pub fn checked_add_days(self, days: i32) -> Result<SqlDateTime, SqlTypeError>;

    /// Add time ticks only (convenience for checked_add(0, ticks)).
    pub fn checked_add_ticks(self, ticks: i32) -> Result<SqlDateTime, SqlTypeError>;
}
```

### Error Conditions

| Operation | Condition | Error |
|-----------|-----------|-------|
| `checked_add` | result day_ticks < -53690 or > 2958463 | `OutOfRange` |
| `checked_sub` | result day_ticks < -53690 or > 2958463 | `OutOfRange` |
| `checked_add_days` | result day_ticks < -53690 or > 2958463 | `OutOfRange` |
| `checked_add_ticks` | result day_ticks < -53690 or > 2958463 | `OutOfRange` |

## SQL Comparisons (return SqlBoolean, NULL propagation)

```rust
impl SqlDateTime {
    pub fn sql_equals(&self, other: &SqlDateTime) -> SqlBoolean;
    pub fn sql_not_equals(&self, other: &SqlDateTime) -> SqlBoolean;
    pub fn sql_less_than(&self, other: &SqlDateTime) -> SqlBoolean;
    pub fn sql_greater_than(&self, other: &SqlDateTime) -> SqlBoolean;
    pub fn sql_less_than_or_equal(&self, other: &SqlDateTime) -> SqlBoolean;
    pub fn sql_greater_than_or_equal(&self, other: &SqlDateTime) -> SqlBoolean;
}
```

## Rust Standard Traits

```rust
impl PartialEq for SqlDateTime;  // NULL == NULL → true (Rust semantics)
impl Eq for SqlDateTime;
impl Hash for SqlDateTime;        // NULL → hash((0i32, 0i32)), Some → hash((d, t))
impl PartialOrd for SqlDateTime;  // NULL < any non-NULL value
impl Ord for SqlDateTime;         // total ordering: NULL < MIN_VALUE..MAX_VALUE
impl Display for SqlDateTime;     // NULL → "Null", Some → "YYYY-MM-DD HH:MM:SS.fff"
impl FromStr for SqlDateTime;     // "Null" → NULL; ISO 8601 variants; else ParseError/OutOfRange
```

## Conversions

```rust
// From raw Rust types (infallible — panics excluded, uses Result)
// None — SqlDateTime has no From<T> since construction is fallible
```

## Deferred (until target types are implemented or follow-up)

- `From<SqlString> for SqlDateTime` — deferred until SqlString type is implemented
- `SqlDateTime::to_sql_string()` — deferred until SqlString type is implemented
- `From<chrono::NaiveDateTime> for SqlDateTime` — deferred to optional `chrono` feature flag
- `Into<chrono::NaiveDateTime> for SqlDateTime` — deferred to optional `chrono` feature flag
- Named static methods (`Add`, `Subtract`, etc.) — not implemented per Rust idiom
