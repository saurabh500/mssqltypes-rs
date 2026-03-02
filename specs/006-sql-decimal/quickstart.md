# Quickstart: SqlDecimal

## Create values

```rust
use mssqltypes::{SqlDecimal, SqlBoolean, SqlByte, SqlInt16, SqlInt32, SqlInt64};

// From components: precision=10, scale=2, positive, mantissa=12345
let a = SqlDecimal::new(10, 2, true, 12345, 0, 0, 0).unwrap();
// Represents 123.45

let b = SqlDecimal::new(10, 2, false, 12345, 0, 0, 0).unwrap();
// Represents -123.45

let null = SqlDecimal::NULL;
assert!(null.is_null());
```

## Constants

```rust
assert_eq!(SqlDecimal::MAX_PRECISION, 38);
assert_eq!(SqlDecimal::MAX_SCALE, 38);
assert!(SqlDecimal::NULL.is_null());

let max = SqlDecimal::max_value();
assert!(max.is_positive().unwrap());
assert_eq!(max.precision().unwrap(), 38);

let min = SqlDecimal::min_value();
assert!(!min.is_positive().unwrap());
```

## Inspect values

```rust
let v = SqlDecimal::new(10, 2, true, 12345, 0, 0, 0).unwrap();
assert_eq!(v.precision().unwrap(), 10);
assert_eq!(v.scale().unwrap(), 2);
assert!(v.is_positive().unwrap());
assert_eq!(v.data().unwrap(), [12345, 0, 0, 0]);
```

## Arithmetic (returns Result)

```rust
let a = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap(); // 123.45
let b = SqlDecimal::new(5, 2, true, 67890, 0, 0, 0).unwrap(); // 678.90

let sum = (&a + &b)?;  // 802.35
let diff = (&a - &b)?; // -555.45

// NULL propagation
let null_sum = (&a + &SqlDecimal::NULL)?;
assert!(null_sum.is_null());

// Divide by zero
let zero = SqlDecimal::from(0i32);
let div_zero = &a / &zero;
assert!(div_zero.is_err()); // SqlTypeError::DivideByZero
```

## Scale adjustment

```rust
let v = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap(); // 123.456

// Round to 2 decimal places
let rounded = v.adjust_scale(2, true).unwrap(); // 123.46
assert_eq!(rounded.scale().unwrap(), 2);

// Truncate to 2 decimal places
let truncated = v.adjust_scale(2, false).unwrap(); // 123.45
assert_eq!(truncated.scale().unwrap(), 2);

// Increase scale (zero-pad)
let padded = v.adjust_scale(5, true).unwrap(); // 123.45600
assert_eq!(padded.scale().unwrap(), 5);
```

## SQL Comparisons (return SqlBoolean)

```rust
let a = SqlDecimal::new(5, 2, true, 10000, 0, 0, 0).unwrap(); // 100.00
let b = SqlDecimal::new(5, 2, true, 20000, 0, 0, 0).unwrap(); // 200.00

let cmp = a.sql_less_than(&b);
assert_eq!(cmp, SqlBoolean::TRUE);

// Different scales, same value → equal
let c = SqlDecimal::new(7, 4, true, 1000000, 0, 0, 0).unwrap(); // 100.0000
assert_eq!(a.sql_equals(&c), SqlBoolean::TRUE);

// NULL comparisons return NULL
let null_cmp = a.sql_equals(&SqlDecimal::NULL);
assert!(null_cmp.is_null());
```

## Mathematical functions

```rust
let v = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap(); // -123.45

let abs_v = v.abs(); // 123.45
assert!(abs_v.is_positive().unwrap());

let floor_v = v.floor().unwrap(); // -124
let ceil_v = v.ceiling().unwrap(); // -123

let pos = SqlDecimal::new(6, 3, true, 123456, 0, 0, 0).unwrap(); // 123.456
let rounded = pos.round(2).unwrap(); // 123.46
let truncated = pos.truncate(2).unwrap(); // 123.45
```

## Conversions

```rust
// From integers
let a: SqlDecimal = 42i32.into();
let b: SqlDecimal = 9_000_000_000i64.into();

// From SqlBoolean
let c: SqlDecimal = SqlBoolean::TRUE.into(); // 1
let d: SqlDecimal = SqlBoolean::FALSE.into(); // 0

// From SqlInt32
let e: SqlDecimal = SqlInt32::new(42).into(); // precision=10, scale=0

// To f64
let f = a.to_f64().unwrap(); // 42.0

// To SqlInt32 (truncates fractional part, range-checked)
let g = SqlDecimal::new(5, 2, true, 4299, 0, 0, 0).unwrap(); // 42.99
let h = g.to_sql_int32().unwrap(); // SqlInt32(42)

// To SqlBoolean
let zero_dec = SqlDecimal::from(0i32);
assert_eq!(zero_dec.to_sql_boolean(), SqlBoolean::FALSE);
assert_eq!(a.to_sql_boolean(), SqlBoolean::TRUE);
```

## Display & Parse

```rust
let v = SqlDecimal::new(5, 2, true, 12345, 0, 0, 0).unwrap();
assert_eq!(v.to_string(), "123.45");

let neg = SqlDecimal::new(5, 2, false, 12345, 0, 0, 0).unwrap();
assert_eq!(neg.to_string(), "-123.45");

assert_eq!(SqlDecimal::NULL.to_string(), "Null");

// Parse
let parsed: SqlDecimal = "123.45".parse().unwrap();
assert_eq!(parsed.to_string(), "123.45");

let neg_parsed: SqlDecimal = "-0.001".parse().unwrap();
assert_eq!(neg_parsed.to_string(), "-0.001");

let int_parsed: SqlDecimal = "42".parse().unwrap();
assert_eq!(int_parsed.scale().unwrap(), 0);
```
