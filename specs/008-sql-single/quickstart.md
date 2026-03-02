# Quickstart: SqlSingle

**Feature**: 008-sql-single | **Date**: 2026-03-02

## Creating Values

```rust
use mssqltypes::{SqlSingle, SqlTypeError};

// Fallible constructor
let x = SqlSingle::new(3.14)?;
let y = SqlSingle::new(-2.5)?;

// From<f32> — panics on NaN/Infinity
let z = SqlSingle::from(42.0);

// Constants
let null = SqlSingle::NULL;
let zero = SqlSingle::ZERO;
let min = SqlSingle::MIN_VALUE;
let max = SqlSingle::MAX_VALUE;

// NaN/Infinity rejected
assert!(SqlSingle::new(f32::NAN).is_err());
assert!(SqlSingle::new(f32::INFINITY).is_err());
```

## Inspecting Values

```rust
let x = SqlSingle::new(3.14)?;
assert!(!x.is_null());
assert_eq!(x.value()?, 3.14);

let null = SqlSingle::NULL;
assert!(null.is_null());
assert!(null.value().is_err()); // Err(NullValue)
```

## Arithmetic

```rust
let a = SqlSingle::new(10.0)?;
let b = SqlSingle::new(3.0)?;

let sum = (a + b)?;           // Ok(SqlSingle(13.0))
let diff = (a - b)?;          // Ok(SqlSingle(7.0))
let prod = (a * b)?;          // Ok(SqlSingle(30.0))
let quot = (a / b)?;          // Ok(SqlSingle(3.3333..))

// Overflow detected
let max = SqlSingle::MAX_VALUE;
assert!((max + max).is_err()); // Err(Overflow)

// Division by zero
let zero = SqlSingle::ZERO;
assert!((a / zero).is_err()); // Err(DivideByZero)

// NULL propagates
let null = SqlSingle::NULL;
let result = (a + null)?;
assert!(result.is_null());
```

## Negation

```rust
let x = SqlSingle::new(5.0)?;
let neg = -x; // SqlSingle(-5.0)
assert_eq!(neg.value()?, -5.0);

let null_neg = -SqlSingle::NULL;
assert!(null_neg.is_null());
```

## SQL Comparisons

```rust
use mssqltypes::{SqlSingle, SqlBoolean};

let a = SqlSingle::new(1.0)?;
let b = SqlSingle::new(2.0)?;

assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
assert_eq!(a.sql_equals(&a), SqlBoolean::TRUE);
assert_eq!(a.sql_greater_than(&b), SqlBoolean::FALSE);

// NULL comparison → SqlBoolean::NULL
let cmp = a.sql_equals(&SqlSingle::NULL);
assert!(cmp.is_null());
```

## Display & Parsing

```rust
use std::str::FromStr;

let x = SqlSingle::new(3.14)?;
assert_eq!(format!("{x}"), "3.14");
assert_eq!(format!("{}", SqlSingle::NULL), "Null");

let parsed: SqlSingle = "3.14".parse()?;
assert_eq!(parsed.value()?, 3.14);

let null_parsed: SqlSingle = "Null".parse()?;
assert!(null_parsed.is_null());

// NaN/Infinity rejected in parsing
assert!("NaN".parse::<SqlSingle>().is_err());
assert!("Infinity".parse::<SqlSingle>().is_err());
```

## Type Conversions

```rust
use mssqltypes::*;

// Widening from integer types
let from_byte = SqlSingle::from_sql_byte(SqlByte::new(42));
assert_eq!(from_byte.value()?, 42.0);

let from_i32 = SqlSingle::from_sql_int32(SqlInt32::new(100_000));
assert_eq!(from_i32.value()?, 100_000.0);

// From SqlBoolean
let from_true = SqlSingle::from_sql_boolean(SqlBoolean::TRUE);
assert_eq!(from_true.value()?, 1.0);

// From SqlMoney
let money = SqlMoney::from_parts(42, 5000); // 42.50
let from_money = SqlSingle::from_sql_money(money);
assert_eq!(from_money.value()?, 42.5);

// To SqlDouble (widening, lossless)
let x = SqlSingle::new(3.14)?;
let d = x.to_sql_double();
assert!(!d.is_null());

// From SqlDouble (narrowing, may fail)
let big = SqlDouble::new(f64::MAX)?;
assert!(SqlSingle::from_sql_double(big).is_err()); // Overflow

// To SqlBoolean
let x = SqlSingle::new(42.0)?;
assert_eq!(x.to_sql_boolean(), SqlBoolean::TRUE);
let z = SqlSingle::ZERO;
assert_eq!(z.to_sql_boolean(), SqlBoolean::FALSE);
```

## Validation Tests

These tests verify the quickstart examples compile and behave correctly:

```rust
#[cfg(test)]
mod quickstart_tests {
    use super::*;

    #[test]
    fn create_and_inspect() {
        let x = SqlSingle::new(3.14).unwrap();
        assert_eq!(x.value().unwrap(), 3.14);
        assert!(!x.is_null());

        assert!(SqlSingle::NULL.is_null());
        assert!(SqlSingle::NULL.value().is_err());
        assert!(SqlSingle::new(f32::NAN).is_err());
    }

    #[test]
    fn arithmetic_basics() {
        let a = SqlSingle::new(10.0).unwrap();
        let b = SqlSingle::new(3.0).unwrap();
        let sum = (a + b).unwrap();
        assert_eq!(sum.value().unwrap(), 13.0);

        let result = (a + SqlSingle::NULL).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn sql_comparisons() {
        let a = SqlSingle::new(1.0).unwrap();
        let b = SqlSingle::new(2.0).unwrap();
        assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);
        assert!(a.sql_equals(&SqlSingle::NULL).is_null());
    }

    #[test]
    fn display_and_parse() {
        let x = SqlSingle::new(3.14).unwrap();
        assert_eq!(format!("{x}"), "3.14");
        assert_eq!(format!("{}", SqlSingle::NULL), "Null");

        let parsed: SqlSingle = "3.14".parse().unwrap();
        assert_eq!(parsed.value().unwrap(), 3.14);
    }

    #[test]
    fn to_sql_double_widening() {
        let x = SqlSingle::new(3.14).unwrap();
        let d = x.to_sql_double();
        assert!(!d.is_null());

        assert!(SqlSingle::NULL.to_sql_double().is_null());
    }
}
```
