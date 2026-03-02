# Quickstart: SqlDateTime

## Create values from calendar components

```rust
use mssqltypes::{SqlDateTime, SqlBoolean};

// Full date/time with milliseconds
let dt = SqlDateTime::new(2025, 7, 17, 12, 30, 45, 333.0)?;
assert_eq!(dt.year()?, 2025);
assert_eq!(dt.month()?, 7);
assert_eq!(dt.day()?, 17);
assert_eq!(dt.hour()?, 12);
assert_eq!(dt.minute()?, 30);
assert_eq!(dt.second()?, 45);

// Date only (time defaults to midnight)
let date = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0)?;
assert_eq!(date.time_ticks()?, 0);

// NULL value
let null = SqlDateTime::NULL;
assert!(null.is_null());
assert!(null.value().is_err());
```

## Create values from raw ticks

```rust
// Epoch date: 1900-01-01 00:00:00
let epoch = SqlDateTime::from_ticks(0, 0)?;
assert_eq!(epoch.year()?, 1900);
assert_eq!(epoch.month()?, 1);
assert_eq!(epoch.day()?, 1);

// Dates before 1900 have negative day_ticks
let old = SqlDateTime::from_ticks(-53690, 0)?;  // 1753-01-01 (MIN_VALUE)
assert_eq!(old.year()?, 1753);
```

## Constants

```rust
assert_eq!(SqlDateTime::MIN_VALUE.day_ticks()?, -53690);  // 1753-01-01
assert_eq!(SqlDateTime::MAX_VALUE.day_ticks()?, 2958463); // 9999-12-31
assert!(SqlDateTime::NULL.is_null());

// Tick-rate constants
assert_eq!(SqlDateTime::TICKS_PER_SECOND, 300);
assert_eq!(SqlDateTime::TICKS_PER_MINUTE, 18_000);
assert_eq!(SqlDateTime::TICKS_PER_HOUR, 1_080_000);
assert_eq!(SqlDateTime::TICKS_PER_DAY, 25_920_000);
```

## Duration arithmetic

```rust
let dt = SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0)?;

// Add 1 day
let next_day = dt.checked_add_days(1)?;
assert_eq!(next_day.day()?, 16);

// Add 2 hours (in ticks)
let later = dt.checked_add_ticks(2 * SqlDateTime::TICKS_PER_HOUR)?;
assert_eq!(later.hour()?, 14);

// Add with day and time components together
let adjusted = dt.checked_add(1, SqlDateTime::TICKS_PER_HOUR)?;
assert_eq!(adjusted.day()?, 16);
assert_eq!(adjusted.hour()?, 13);

// Subtract days
let earlier = dt.checked_sub(1, 0)?;
assert_eq!(earlier.day()?, 14);

// NULL propagation
let null_result = SqlDateTime::NULL.checked_add_days(1)?;
assert!(null_result.is_null());

// Out of range
let overflow = SqlDateTime::MAX_VALUE.checked_add_ticks(1);
assert!(overflow.is_err());
```

## SQL comparisons (return SqlBoolean)

```rust
let jan1 = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 0.0)?;
let jan2 = SqlDateTime::new(2025, 1, 2, 0, 0, 0, 0.0)?;

assert_eq!(jan1.sql_less_than(&jan2), SqlBoolean::TRUE);
assert_eq!(jan2.sql_greater_than(&jan1), SqlBoolean::TRUE);
assert_eq!(jan1.sql_equals(&jan1), SqlBoolean::TRUE);
assert_eq!(jan1.sql_not_equals(&jan2), SqlBoolean::TRUE);

// NULL comparisons return NULL
assert!(jan1.sql_equals(&SqlDateTime::NULL).is_null());
```

## Display and parsing

```rust
use std::str::FromStr;

// Display
let dt = SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0)?;
assert_eq!(format!("{}", dt), "2025-07-17 12:30:00.000");
assert_eq!(format!("{}", SqlDateTime::NULL), "Null");

// Parse
let parsed: SqlDateTime = "2025-07-17 12:30:00".parse()?;
assert_eq!(parsed.year()?, 2025);

// Parse with T separator
let iso: SqlDateTime = "2025-07-17T12:30:00.000".parse()?;

// Parse date only (time defaults to midnight)
let date_only: SqlDateTime = "2025-07-17".parse()?;
assert_eq!(date_only.hour()?, 0);

// Parse NULL
let null: SqlDateTime = "Null".parse()?;
assert!(null.is_null());
```

## Millisecond rounding

```rust
// Milliseconds are rounded to nearest 1/300-second tick
// Formula: ticks = (int)(ms * 0.3 + 0.5)
let dt = SqlDateTime::new(2025, 1, 1, 0, 0, 0, 3.33)?;
assert_eq!(dt.time_ticks()?, 1);  // 1 SQL tick ≈ 3.33ms

// Midnight rollover: 23:59:59.998+ rounds up past midnight → next day
let rollover = SqlDateTime::new(2025, 1, 1, 23, 59, 59, 998.0)?;
assert_eq!(rollover.day()?, 2);
assert_eq!(rollover.time_ticks()?, 0);
```

## Leap year handling

```rust
// 2024 is a leap year (divisible by 4)
let leap = SqlDateTime::new(2024, 2, 29, 0, 0, 0, 0.0)?;
assert_eq!(leap.day()?, 29);

// 2023 is not a leap year
let err = SqlDateTime::new(2023, 2, 29, 0, 0, 0, 0.0);
assert!(err.is_err());

// 1900 is NOT a leap year (divisible by 100 but not 400)
let err = SqlDateTime::new(1900, 2, 29, 0, 0, 0, 0.0);
assert!(err.is_err());

// 2000 IS a leap year (divisible by 400)
let leap = SqlDateTime::new(2000, 2, 29, 0, 0, 0, 0.0)?;
assert_eq!(leap.month()?, 2);
```
