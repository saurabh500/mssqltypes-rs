# Quickstart: SqlInt16

## Create values

```rust
use mssqltypes::{SqlInt16, SqlBoolean, SqlByte};

let a = SqlInt16::new(100);
let b = SqlInt16::new(-200);
let null = SqlInt16::NULL;

assert_eq!(a.value().unwrap(), 100);
assert_eq!(b.value().unwrap(), -200);
assert!(null.is_null());
```

## Constants

```rust
assert_eq!(SqlInt16::ZERO.value().unwrap(), 0);
assert_eq!(SqlInt16::MIN_VALUE.value().unwrap(), -32768);
assert_eq!(SqlInt16::MAX_VALUE.value().unwrap(), 32767);
assert!(SqlInt16::NULL.is_null());
```

## Arithmetic (returns Result)

```rust
let sum = (SqlInt16::new(100) + SqlInt16::new(200))?;
assert_eq!(sum.value().unwrap(), 300);

// Overflow detected
let overflow = SqlInt16::new(i16::MAX) + SqlInt16::new(1);
assert!(overflow.is_err()); // SqlTypeError::Overflow

// NULL propagation
let null_sum = (SqlInt16::new(42) + SqlInt16::NULL)?;
assert!(null_sum.is_null());

// Divide by zero
let div_zero = SqlInt16::new(10) / SqlInt16::new(0);
assert!(div_zero.is_err()); // SqlTypeError::DivideByZero

// MIN_VALUE / -1 overflows
let min_div = SqlInt16::new(i16::MIN) / SqlInt16::new(-1);
assert!(min_div.is_err()); // SqlTypeError::Overflow
```

## Bitwise (infallible, NULL propagation)

```rust
let and = SqlInt16::new(0xFF) & SqlInt16::new(0x0F);
assert_eq!(and.value().unwrap(), 0x0F);

let not = !SqlInt16::new(0);
assert_eq!(not.value().unwrap(), -1);

let null_or = SqlInt16::new(42) | SqlInt16::NULL;
assert!(null_or.is_null());
```

## SQL Comparisons (return SqlBoolean)

```rust
let cmp = SqlInt16::new(10).sql_less_than(&SqlInt16::new(20));
assert_eq!(cmp, SqlBoolean::TRUE);

// NULL comparisons return NULL
let null_cmp = SqlInt16::new(10).sql_equals(&SqlInt16::NULL);
assert!(null_cmp.is_null());
```

## Conversions

```rust
// From i16
let a: SqlInt16 = 42i16.into();

// From SqlByte (widening)
let b: SqlInt16 = SqlByte::new(255).into();
assert_eq!(b.value().unwrap(), 255);

// From SqlBoolean
let c: SqlInt16 = SqlBoolean::TRUE.into();
assert_eq!(c.value().unwrap(), 1);

// To SqlByte (narrowing, may fail)
let d = SqlInt16::new(100).to_sql_byte().unwrap();
assert_eq!(d.value().unwrap(), 100);

let e = SqlInt16::new(-1).to_sql_byte();
assert!(e.is_err()); // Overflow: negative value

// To SqlBoolean
let f = SqlInt16::new(0).to_sql_boolean();
assert_eq!(f, SqlBoolean::FALSE);
```

## Display & Parse

```rust
assert_eq!(SqlInt16::new(-1234).to_string(), "-1234");
assert_eq!(SqlInt16::NULL.to_string(), "Null");

let parsed: SqlInt16 = "-1234".parse().unwrap();
assert_eq!(parsed.value().unwrap(), -1234);

let null_parsed: SqlInt16 = "Null".parse().unwrap();
assert!(null_parsed.is_null());
```
