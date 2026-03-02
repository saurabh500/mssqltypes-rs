# Quickstart: Cross-Type Conversions

**Feature**: 014-cross-type-conversions

## Widening Integer Conversions

```rust
use mssqltypes::*;

// SqlByte → SqlInt32 (lossless, via From trait)
let byte_val = SqlByte::new(200);
let int32_val: SqlInt32 = byte_val.into();
assert_eq!(int32_val.value().unwrap(), 200);

// SqlInt32 → SqlInt64 (lossless)
let int32_val = SqlInt32::new(2_000_000);
let int64_val: SqlInt64 = int32_val.into();
assert_eq!(int64_val.value().unwrap(), 2_000_000);

// NULL propagation
let null_byte = SqlByte::NULL;
let null_int32: SqlInt32 = null_byte.into();
assert!(null_int32.is_null());
```

## Boolean ↔ Numeric

```rust
// Non-zero → TRUE
let val = SqlInt32::new(42);
let b = val.to_sql_boolean();
assert_eq!(b, SqlBoolean::TRUE);

// Zero → FALSE
let zero = SqlInt64::new(0);
let b = zero.to_sql_boolean();
assert_eq!(b, SqlBoolean::FALSE);

// NULL → NULL
let null_val = SqlInt32::NULL;
assert!(null_val.to_sql_boolean().is_null());
```

## Float ↔ Float

```rust
// Widen: SqlSingle → SqlDouble (lossless)
let single = SqlSingle::new(3.14);
let double = SqlDouble::from_sql_single(single);
assert!(!double.is_null());

// Narrow: SqlDouble → SqlSingle (may overflow)
let big = SqlDouble::new(1e300);
let result = big.to_sql_single();
assert!(result.is_err()); // Overflow
```

## String Conversions

```rust
// Any type → SqlString
let val = SqlInt32::new(42);
let s = val.to_sql_string();
assert_eq!(s.value().unwrap(), "42");

// SqlString → any type (parsing)
let s = SqlString::new("42");
let parsed = s.to_sql_int32().unwrap();
assert_eq!(parsed.value().unwrap(), 42);

// Parse errors
let bad = SqlString::new("not_a_number");
let result = bad.to_sql_int32();
assert!(result.is_err());

// NULL round-trip
let null_str = SqlString::NULL;
let result = null_str.to_sql_int64().unwrap();
assert!(result.is_null());
```

## Decimal ↔ Float/Money

```rust
// SqlDecimal → SqlDouble
let dec = SqlDecimal::new(10, 2, true, &[1050, 0, 0, 0]); // represents 10.50
let dbl = dec.to_sql_double();
assert!(!dbl.is_null());

// SqlDouble → SqlDecimal (via From, rejects NaN/Infinity)
let dbl = SqlDouble::new(3.14);
let dec: SqlDecimal = SqlDecimal::from(dbl);

// SqlDecimal → SqlMoney (range-checked)
let result = dec.to_sql_money();
// Ok if within money range, Err(Overflow) if not

// SqlMoney → SqlDecimal (via From, preserves 4 decimals)
let money = SqlMoney::new(100.50);
let dec: SqlDecimal = money.into();
```

## DateTime ↔ String

```rust
// SqlDateTime → SqlString
let dt = SqlDateTime::new(2025, 1, 15, 10, 30, 0, 0).unwrap();
let s = dt.to_sql_string();
assert!(!s.is_null());

// SqlString → SqlDateTime
let s = SqlString::new("2025-01-15 10:30:00");
let dt = SqlDateTime::from_sql_string(&s);
// Ok if valid date, Err(ParseError) if not
```
