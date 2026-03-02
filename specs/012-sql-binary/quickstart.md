# Quickstart: SqlBinary

## Create values

```rust
use mssqltypes::SqlBinary;

// From a byte vector (takes ownership)
let bin = SqlBinary::new(vec![1, 2, 3]);
let null = SqlBinary::NULL;

assert_eq!(bin.value().unwrap(), &[1, 2, 3]);
assert!(null.is_null());
assert!(!bin.is_null());

// Empty binary is valid (not NULL)
let empty = SqlBinary::new(vec![]);
assert!(!empty.is_null());
assert_eq!(empty.len().unwrap(), 0);
```

## From conversions

```rust
// From a byte slice (clones)
let a: SqlBinary = (&[10u8, 20, 30][..]).into();
assert_eq!(a.value().unwrap(), &[10, 20, 30]);

// From a Vec<u8> (moves, no clone)
let v = vec![0xAB, 0xCD];
let b: SqlBinary = v.into();
assert_eq!(b.value().unwrap(), &[0xAB, 0xCD]);
```

## Indexed access

```rust
let bin = SqlBinary::new(vec![10, 20, 30]);
assert_eq!(bin.get(0).unwrap(), 10);
assert_eq!(bin.get(1).unwrap(), 20);
assert_eq!(bin.get(2).unwrap(), 30);

// Out of bounds → error (no panic)
assert!(bin.get(5).is_err());

// NULL → error
assert!(SqlBinary::NULL.get(0).is_err());
```

## Byte length

```rust
let bin = SqlBinary::new(vec![1, 2, 3]);
assert_eq!(bin.len().unwrap(), 3);

// NULL has no length
assert!(SqlBinary::NULL.len().is_err());
```

## Concatenation

```rust
let a = SqlBinary::new(vec![1, 2]);
let b = SqlBinary::new(vec![3, 4]);
let result = a + b;
assert_eq!(result.value().unwrap(), &[1, 2, 3, 4]);

// NULL propagation
let null_concat = SqlBinary::new(vec![1, 2]) + SqlBinary::NULL;
assert!(null_concat.is_null());

// Empty concat
let with_empty = SqlBinary::new(vec![1, 2]) + SqlBinary::new(vec![]);
assert_eq!(with_empty.value().unwrap(), &[1, 2]);
```

## SQL Comparisons (return SqlBoolean)

```rust
use mssqltypes::SqlBoolean;

// Trailing-zero-padded: [1,2] == [1,2,0,0]
let a = SqlBinary::new(vec![1, 2]);
let b = SqlBinary::new(vec![1, 2, 0, 0]);
assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);

// Byte ordering
let x = SqlBinary::new(vec![1, 2]);
let y = SqlBinary::new(vec![1, 3]);
assert_eq!(x.sql_less_than(&y), SqlBoolean::TRUE);

// Extra non-zero byte → greater
let p = SqlBinary::new(vec![1, 2, 1]);
let q = SqlBinary::new(vec![1, 2]);
assert_eq!(p.sql_greater_than(&q), SqlBoolean::TRUE);

// NULL comparisons return NULL
let cmp = SqlBinary::new(vec![1, 2]).sql_equals(&SqlBinary::NULL);
assert!(cmp.is_null());
```

## Display

```rust
// Lowercase hex, no separators
assert_eq!(SqlBinary::new(vec![0x0A, 0xFF]).to_string(), "0aff");
assert_eq!(SqlBinary::NULL.to_string(), "Null");
assert_eq!(SqlBinary::new(vec![]).to_string(), "");
```

## Rust equality (trailing-zero-padded, always)

```rust
use std::collections::HashSet;

// PartialEq uses trailing-zero-padded comparison
let a = SqlBinary::new(vec![1, 2]);
let b = SqlBinary::new(vec![1, 2, 0, 0]);
assert_eq!(a, b);

// NULL == NULL in Rust Eq
assert_eq!(SqlBinary::NULL, SqlBinary::NULL);

// Can use in HashSet
let mut set = HashSet::new();
set.insert(SqlBinary::new(vec![1, 2]));
assert!(set.contains(&SqlBinary::new(vec![1, 2, 0])));
```
