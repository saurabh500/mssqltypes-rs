# Quickstart: SqlMoney

## Create values

```rust
use mssqltypes::{SqlMoney, SqlBoolean, SqlByte, SqlInt16, SqlInt32, SqlInt64, SqlDecimal};

// From integers
let a = SqlMoney::from_i32(100);              // 100.0000
let b = SqlMoney::from_i64(500).unwrap();     // 500.0000
let null = SqlMoney::NULL;

assert_eq!(a.to_i64().unwrap(), 100);
assert_eq!(b.to_i64().unwrap(), 500);
assert!(null.is_null());

// From f64 (rounded to 4 decimal places)
let c = SqlMoney::from_f64(123.4567).unwrap();  // 123.4567
let d = SqlMoney::from_f64(99.999).unwrap();     // 99.9990

// From raw scaled value (TDS interop)
let raw = SqlMoney::from_scaled(1_234_567);  // 123.4567 (raw ticks / 10,000)
assert_eq!(raw.scaled_value().unwrap(), 1_234_567);
```

## Constants

```rust
assert_eq!(SqlMoney::ZERO.scaled_value().unwrap(), 0);
assert!(SqlMoney::NULL.is_null());
assert_eq!(SqlMoney::MIN_VALUE.scaled_value().unwrap(), i64::MIN);
assert_eq!(SqlMoney::MAX_VALUE.scaled_value().unwrap(), i64::MAX);
```

## Arithmetic (returns Result)

```rust
// Addition and subtraction — exact i64 arithmetic
let sum = (SqlMoney::from_i32(100) + SqlMoney::from_i32(200))?;
assert_eq!(sum.to_i64().unwrap(), 300);

// Multiplication — uses i128 intermediate
let product = (SqlMoney::from_i32(100) * SqlMoney::from_f64(2.5).unwrap())?;
assert_eq!(product.to_f64().unwrap(), 250.0);

// Division
let quot = (SqlMoney::from_i32(100) / SqlMoney::from_i32(3))?;
// 100.0000 / 3.0000 = 33.3333 (rounded to 4dp)

// Overflow detected
let overflow = SqlMoney::MAX_VALUE + SqlMoney::from_scaled(1);
assert!(overflow.is_err()); // SqlTypeError::Overflow

// Divide by zero
let div_zero = SqlMoney::from_i32(10) / SqlMoney::ZERO;
assert!(div_zero.is_err()); // SqlTypeError::DivideByZero

// Negation
let neg = (-SqlMoney::from_i32(42))?;
assert_eq!(neg.to_i64().unwrap(), -42);

// Negation of MIN_VALUE overflows
let neg_min = -SqlMoney::MIN_VALUE;
assert!(neg_min.is_err()); // SqlTypeError::Overflow

// NULL propagation
let null_sum = (SqlMoney::from_i32(42) + SqlMoney::NULL)?;
assert!(null_sum.is_null());
```

## Comparisons (return SqlBoolean)

```rust
let a = SqlMoney::from_i32(100);
let b = SqlMoney::from_i32(200);

assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
assert_eq!(a.sql_equals(&a), SqlBoolean::TRUE);
assert_eq!(a.sql_greater_than(&b), SqlBoolean::FALSE);

// NULL propagation
assert!(a.sql_equals(&SqlMoney::NULL).is_null());
```

## Display and Parsing

```rust
// Display: minimum 2 decimal places, maximum 4
assert_eq!(format!("{}", SqlMoney::from_i32(100)), "100.00");
assert_eq!(format!("{}", SqlMoney::from_f64(123.4567).unwrap()), "123.4567");
assert_eq!(format!("{}", SqlMoney::from_f64(123.45).unwrap()), "123.45");
assert_eq!(format!("{}", SqlMoney::NULL), "Null");

// Parsing
let parsed: SqlMoney = "123.4567".parse().unwrap();
let null_parsed: SqlMoney = "Null".parse().unwrap();
assert!(null_parsed.is_null());
```

## Conversions

```rust
// From other SqlTypes (widening)
let from_bool = SqlMoney::from(SqlBoolean::TRUE);   // 1.0000
let from_byte = SqlMoney::from(SqlByte::new(255));   // 255.0000
let from_i16 = SqlMoney::from(SqlInt16::new(1000));  // 1000.0000
let from_i32 = SqlMoney::from(SqlInt32::new(42));    // 42.0000

// From SqlInt64 (fallible — range checked)
let from_i64 = SqlMoney::from_sql_int64(SqlInt64::new(100)).unwrap(); // 100.0000

// To other SqlTypes (narrowing — may fail)
let money = SqlMoney::from_i32(42);
let as_i64 = money.to_sql_int64().unwrap();  // SqlInt64(42) — rounded
let as_i32 = money.to_sql_int32().unwrap();  // SqlInt32(42)
let as_bool = money.to_sql_boolean();        // SqlBoolean::TRUE (non-zero)
let as_dec = money.to_sql_decimal();         // SqlDecimal with scale=4

// Rounding on conversion to integer
let money = SqlMoney::from_f64(42.9999).unwrap();
let rounded = money.to_sql_int64().unwrap(); // SqlInt64(43) — round-half-away-from-zero
```

## Rust Trait Usage

```rust
use std::collections::HashMap;

// PartialEq — NULL == NULL per Rust semantics
assert_eq!(SqlMoney::NULL, SqlMoney::NULL);
assert_eq!(SqlMoney::from_i32(100), SqlMoney::from_i32(100));

// Ord — NULL < any value
let mut values = vec![SqlMoney::from_i32(3), SqlMoney::NULL, SqlMoney::from_i32(1)];
values.sort();
assert!(values[0].is_null());  // NULL sorts first

// Hash — usable in HashMap
let mut map = HashMap::new();
map.insert(SqlMoney::from_i32(100), "hundred");
```
