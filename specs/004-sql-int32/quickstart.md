# Quickstart: SqlInt32

## Create values

```rust
use mssqltypes::{SqlInt32, SqlBoolean, SqlByte, SqlInt16};

let a = SqlInt32::new(100_000);
let b = SqlInt32::new(-200_000);
let null = SqlInt32::NULL;

assert_eq!(a.value().unwrap(), 100_000);
assert_eq!(b.value().unwrap(), -200_000);
assert!(null.is_null());
```

## Constants

```rust
assert_eq!(SqlInt32::ZERO.value().unwrap(), 0);
assert_eq!(SqlInt32::MIN_VALUE.value().unwrap(), -2_147_483_648);
assert_eq!(SqlInt32::MAX_VALUE.value().unwrap(), 2_147_483_647);
assert!(SqlInt32::NULL.is_null());
```

## Arithmetic (returns Result)

```rust
let sum = (SqlInt32::new(100) + SqlInt32::new(200))?;
assert_eq!(sum.value().unwrap(), 300);

// Overflow detected
let overflow = SqlInt32::new(i32::MAX) + SqlInt32::new(1);
assert!(overflow.is_err()); // SqlTypeError::Overflow

// NULL propagation
let null_sum = (SqlInt32::new(42) + SqlInt32::NULL)?;
assert!(null_sum.is_null());

// Divide by zero
let div_zero = SqlInt32::new(10) / SqlInt32::new(0);
assert!(div_zero.is_err()); // SqlTypeError::DivideByZero

// MIN_VALUE / -1 overflows
let min_div = SqlInt32::new(i32::MIN) / SqlInt32::new(-1);
assert!(min_div.is_err()); // SqlTypeError::Overflow
```

## Bitwise (infallible, NULL propagation)

```rust
let and = SqlInt32::new(0xFF00) & SqlInt32::new(0x0FF0);
assert_eq!(and.value().unwrap(), 0x0F00);

let not = !SqlInt32::new(0);
assert_eq!(not.value().unwrap(), -1);

let null_or = SqlInt32::new(42) | SqlInt32::NULL;
assert!(null_or.is_null());
```

## SQL Comparisons (return SqlBoolean)

```rust
let cmp = SqlInt32::new(10).sql_less_than(&SqlInt32::new(20));
assert_eq!(cmp, SqlBoolean::TRUE);

// NULL comparisons return NULL
let null_cmp = SqlInt32::new(10).sql_equals(&SqlInt32::NULL);
assert!(null_cmp.is_null());
```

## Conversions

```rust
// From i32
let a: SqlInt32 = 42i32.into();

// From SqlBoolean
let c: SqlInt32 = SqlBoolean::TRUE.into();
assert_eq!(c.value().unwrap(), 1);

// To SqlInt16 (narrowing, may fail)
let d = SqlInt32::new(100).to_sql_int16().unwrap();
assert_eq!(d.value().unwrap(), 100);

let e = SqlInt32::new(100_000).to_sql_int16();
assert!(e.is_err()); // Overflow: out of i16 range

// To SqlByte (narrowing, may fail)
let f = SqlInt32::new(200).to_sql_byte().unwrap();
assert_eq!(f.value().unwrap(), 200);

let g = SqlInt32::new(-1).to_sql_byte();
assert!(g.is_err()); // Overflow: negative value
```

## Display & Parse

```rust
assert_eq!(SqlInt32::new(-123456).to_string(), "-123456");
assert_eq!(SqlInt32::NULL.to_string(), "Null");

let parsed: SqlInt32 = "-123456".parse().unwrap();
assert_eq!(parsed.value().unwrap(), -123456);

let null_parsed: SqlInt32 = "Null".parse().unwrap();
assert!(null_parsed.is_null());
```
