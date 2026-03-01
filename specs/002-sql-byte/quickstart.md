# Quickstart: SqlByte

## Creating Values

```rust
use mssqltypes::{SqlByte, SqlBoolean};

let a = SqlByte::new(42);
assert_eq!(a.value().unwrap(), 42);

let null = SqlByte::NULL;
assert!(null.is_null());

let b: SqlByte = 100u8.into();
let max = SqlByte::MAX_VALUE; // 255
```

## Arithmetic (Returns Result)

```rust
let sum = (SqlByte::new(10) + SqlByte::new(20))?;
assert_eq!(sum.value()?, 30);

// Overflow
assert!((SqlByte::new(200) + SqlByte::new(100)).is_err());

// NULL propagation
let r = (SqlByte::new(42) + SqlByte::NULL)?;
assert!(r.is_null());
```

## Bitwise

```rust
let a = SqlByte::new(0xFF) & SqlByte::new(0x0F); // 0x0F
let b = !SqlByte::new(0x0F);                      // 0xF0
```

## SQL Comparisons

```rust
let lt = SqlByte::new(10).sql_less_than(&SqlByte::new(20)); // TRUE
let eq = SqlByte::new(10).sql_equals(&SqlByte::NULL);       // NULL
```

## Parsing

```rust
let p: SqlByte = "123".parse().unwrap();
let n: SqlByte = "Null".parse().unwrap();
assert!(n.is_null());
```

## Conversions

```rust
let b = SqlByte::new(1).to_sql_boolean(); // TRUE
let s: SqlByte = SqlBoolean::TRUE.into(); // SqlByte(1)
```

## Source File

New: `src/sql_byte.rs` — register in `src/lib.rs` as `pub mod sql_byte; pub use sql_byte::SqlByte;`
