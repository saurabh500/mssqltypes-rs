# Quickstart: SqlDouble

## Create values

```rust
use mssqltypes::{SqlDouble, SqlBoolean};

// From f64 value
let pi = SqlDouble::new(3.14159265358979)?;
assert_eq!(pi.value()?, 3.14159265358979);

// Negative values
let neg = SqlDouble::new(-2.718281828)?;
assert_eq!(neg.value()?, -2.718281828);

// Zero
let zero = SqlDouble::new(0.0)?;
assert_eq!(zero.value()?, 0.0);

// Constants
assert_eq!(SqlDouble::ZERO.value()?, 0.0);
assert_eq!(SqlDouble::MIN_VALUE.value()?, f64::MIN);
assert_eq!(SqlDouble::MAX_VALUE.value()?, f64::MAX);

// NULL
let null = SqlDouble::NULL;
assert!(null.is_null());
assert!(null.value().is_err());
```

## NaN and Infinity are rejected

```rust
// NaN rejected on construction
let err = SqlDouble::new(f64::NAN);
assert!(err.is_err());

// Infinity rejected on construction
let err = SqlDouble::new(f64::INFINITY);
assert!(err.is_err());

let err = SqlDouble::new(f64::NEG_INFINITY);
assert!(err.is_err());
```

## Arithmetic

```rust
let a = SqlDouble::new(2.5)?;
let b = SqlDouble::new(3.5)?;

// Addition
let sum = (a + b)?;
assert_eq!(sum.value()?, 6.0);

// Subtraction
let diff = (SqlDouble::new(10.0)? - SqlDouble::new(3.0)?)?;
assert_eq!(diff.value()?, 7.0);

// Multiplication
let product = (SqlDouble::new(4.0)? * SqlDouble::new(2.5)?)?;
assert_eq!(product.value()?, 10.0);

// Division
let quotient = (SqlDouble::new(10.0)? / SqlDouble::new(4.0)?)?;
assert_eq!(quotient.value()?, 2.5);

// Negation (infallible)
let neg = -SqlDouble::new(5.0)?;
assert_eq!(neg.value()?, -5.0);

// NULL propagation
let null_result = (SqlDouble::new(1.0)? + SqlDouble::NULL)?;
assert!(null_result.is_null());
```

## Overflow and division by zero

```rust
// Overflow (result is Infinity)
let overflow = SqlDouble::MAX_VALUE + SqlDouble::MAX_VALUE;
assert!(overflow.is_err());

// Division by zero
let div_zero = SqlDouble::new(1.0)? / SqlDouble::new(0.0)?;
assert!(div_zero.is_err());

// 0.0 / 0.0 is also division by zero (would produce NaN)
let nan_div = SqlDouble::new(0.0)? / SqlDouble::new(0.0)?;
assert!(nan_div.is_err());
```

## SQL comparisons (return SqlBoolean)

```rust
let x = SqlDouble::new(1.0)?;
let y = SqlDouble::new(2.0)?;

assert_eq!(x.sql_less_than(&y), SqlBoolean::TRUE);
assert_eq!(y.sql_greater_than(&x), SqlBoolean::TRUE);
assert_eq!(x.sql_equals(&x), SqlBoolean::TRUE);
assert_eq!(x.sql_not_equals(&y), SqlBoolean::TRUE);

// NULL comparisons return NULL
assert!(x.sql_equals(&SqlDouble::NULL).is_null());
```

## Display and parsing

```rust
use std::str::FromStr;

// Display
let val = SqlDouble::new(3.14159265358979)?;
assert_eq!(format!("{}", val), "3.14159265358979");
assert_eq!(format!("{}", SqlDouble::NULL), "Null");

// Parse
let parsed: SqlDouble = "3.14".parse()?;
assert_eq!(parsed.value()?, 3.14);

// Parse NULL
let null: SqlDouble = "Null".parse()?;
assert!(null.is_null());

// Invalid strings
let err = "abc".parse::<SqlDouble>();
assert!(err.is_err());

// NaN/Infinity rejected during parsing
let err = "NaN".parse::<SqlDouble>();
assert!(err.is_err());
```

## Type conversions

```rust
use mssqltypes::{SqlByte, SqlInt16, SqlInt32, SqlInt64, SqlMoney};

// Widening from integer types (lossless for small values)
let from_byte = SqlDouble::from_sql_byte(SqlByte::new(42));
assert_eq!(from_byte.value()?, 42.0);

let from_i32 = SqlDouble::from_sql_int32(SqlInt32::new(100_000));
assert_eq!(from_i32.value()?, 100_000.0);

let from_i64 = SqlDouble::from_sql_int64(SqlInt64::new(1_000_000_000));
assert_eq!(from_i64.value()?, 1_000_000_000.0);

// From SqlBoolean
let from_true = SqlDouble::from_sql_boolean(SqlBoolean::TRUE);
assert_eq!(from_true.value()?, 1.0);

let from_false = SqlDouble::from_sql_boolean(SqlBoolean::FALSE);
assert_eq!(from_false.value()?, 0.0);

// NULL propagation in conversions
let from_null = SqlDouble::from_sql_byte(SqlByte::NULL);
assert!(from_null.is_null());

// To SqlBoolean
let to_bool = SqlDouble::new(42.0)?.to_sql_boolean();
assert_eq!(to_bool, SqlBoolean::TRUE);

let to_bool_zero = SqlDouble::new(0.0)?.to_sql_boolean();
assert_eq!(to_bool_zero, SqlBoolean::FALSE);
```
