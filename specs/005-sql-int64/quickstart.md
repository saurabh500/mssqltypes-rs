# Quickstart: SqlInt64

## Create values

```rust
use mssqltypes::{SqlInt64, SqlBoolean, SqlByte, SqlInt16, SqlInt32};

let a = SqlInt64::new(9_000_000_000);
let b = SqlInt64::new(-9_000_000_000);
let null = SqlInt64::NULL;

assert_eq!(a.value().unwrap(), 9_000_000_000);
assert_eq!(b.value().unwrap(), -9_000_000_000);
assert!(null.is_null());
```

## Constants

```rust
assert_eq!(SqlInt64::ZERO.value().unwrap(), 0);
assert_eq!(SqlInt64::MIN_VALUE.value().unwrap(), -9_223_372_036_854_775_808);
assert_eq!(SqlInt64::MAX_VALUE.value().unwrap(), 9_223_372_036_854_775_807);
assert!(SqlInt64::NULL.is_null());
```

## Arithmetic (returns Result)

```rust
let sum = (SqlInt64::new(100) + SqlInt64::new(200))?;
assert_eq!(sum.value().unwrap(), 300);

// Overflow detected
let overflow = SqlInt64::new(i64::MAX) + SqlInt64::new(1);
assert!(overflow.is_err()); // SqlTypeError::Overflow

// NULL propagation
let null_sum = (SqlInt64::new(42) + SqlInt64::NULL)?;
assert!(null_sum.is_null());

// Divide by zero
let div_zero = SqlInt64::new(10) / SqlInt64::new(0);
assert!(div_zero.is_err()); // SqlTypeError::DivideByZero

// MIN_VALUE / -1 overflows
let min_div = SqlInt64::new(i64::MIN) / SqlInt64::new(-1);
assert!(min_div.is_err()); // SqlTypeError::Overflow

// Large multiplication overflow
let big_mul = SqlInt64::new(5_000_000_000) * SqlInt64::new(5_000_000_000);
assert!(big_mul.is_err()); // SqlTypeError::Overflow
```

## Bitwise (infallible, NULL propagation)

```rust
let and = SqlInt64::new(0xFF00) & SqlInt64::new(0x0FF0);
assert_eq!(and.value().unwrap(), 0x0F00);

let not = !SqlInt64::new(0);
assert_eq!(not.value().unwrap(), -1);

let null_or = SqlInt64::new(42) | SqlInt64::NULL;
assert!(null_or.is_null());
```

## SQL Comparisons (return SqlBoolean)

```rust
let cmp = SqlInt64::new(100).sql_less_than(&SqlInt64::new(200));
assert_eq!(cmp, SqlBoolean::TRUE);

// NULL comparisons return NULL
let null_cmp = SqlInt64::new(100).sql_equals(&SqlInt64::NULL);
assert!(null_cmp.is_null());
```

## Conversions

```rust
// From i64
let a: SqlInt64 = 9_000_000_000i64.into();

// From SqlBoolean
let c: SqlInt64 = SqlBoolean::TRUE.into();
assert_eq!(c.value().unwrap(), 1);

// To SqlInt32 (narrowing, may fail)
let d = SqlInt64::new(100).to_sql_int32().unwrap();
assert_eq!(d.value().unwrap(), 100);

let e = SqlInt64::new(3_000_000_000).to_sql_int32();
assert!(e.is_err()); // Overflow: out of i32 range

// To SqlInt16 (narrowing, may fail)
let f = SqlInt64::new(100).to_sql_int16().unwrap();
assert_eq!(f.value().unwrap(), 100);

let g = SqlInt64::new(100_000).to_sql_int16();
assert!(g.is_err()); // Overflow: out of i16 range

// To SqlByte (narrowing, may fail)
let h = SqlInt64::new(200).to_sql_byte().unwrap();
assert_eq!(h.value().unwrap(), 200);

let i = SqlInt64::new(-1).to_sql_byte();
assert!(i.is_err()); // Overflow: negative value
```

## Display & Parse

```rust
assert_eq!(SqlInt64::new(9_000_000_000).to_string(), "9000000000");
assert_eq!(SqlInt64::NULL.to_string(), "Null");

let parsed: SqlInt64 = "9000000000".parse().unwrap();
assert_eq!(parsed.value().unwrap(), 9_000_000_000);

let null_parsed: SqlInt64 = "Null".parse().unwrap();
assert!(null_parsed.is_null());
```
