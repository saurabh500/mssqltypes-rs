# Quickstart: SqlString

## Create values

```rust
use mssqltypes::{SqlString, SqlCompareOptions};

// Default compare options (IgnoreCase)
let hello = SqlString::new("hello");
let null = SqlString::NULL;

assert_eq!(hello.value().unwrap(), "hello");
assert!(null.is_null());
assert!(!hello.is_null());

// Empty string is valid (not NULL)
let empty = SqlString::new("");
assert!(!empty.is_null());
assert_eq!(empty.len().unwrap(), 0);
```

## Explicit compare options

```rust
let case_sensitive = SqlString::with_options("Hello", SqlCompareOptions::None);
let binary = SqlString::with_options("Hello", SqlCompareOptions::BinarySort);
let ignore_case = SqlString::with_options("Hello", SqlCompareOptions::IgnoreCase);

assert_eq!(case_sensitive.compare_options(), SqlCompareOptions::None);
assert_eq!(binary.compare_options(), SqlCompareOptions::BinarySort);
assert_eq!(ignore_case.compare_options(), SqlCompareOptions::IgnoreCase);
```

## Byte length

```rust
let s = SqlString::new("hello");
assert_eq!(s.len().unwrap(), 5);

// Multi-byte UTF-8: len() returns byte count, not char count
let emoji = SqlString::new("🦀");
assert_eq!(emoji.len().unwrap(), 4);

// NULL has no length
assert!(SqlString::NULL.len().is_err());
```

## Concatenation

```rust
let a = SqlString::new("hello");
let b = SqlString::new(" world");
let result = a + b;
assert_eq!(result.value().unwrap(), "hello world");

// NULL propagation
let null_concat = SqlString::new("hello") + SqlString::NULL;
assert!(null_concat.is_null());

// Empty string concat
let with_empty = SqlString::new("hello") + SqlString::new("");
assert_eq!(with_empty.value().unwrap(), "hello");
```

## SQL Comparisons (return SqlBoolean)

```rust
use mssqltypes::SqlBoolean;

// Default: case-insensitive
let a = SqlString::new("Hello");
let b = SqlString::new("hello");
assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);  // IgnoreCase

// Case-sensitive with SqlCompareOptions::None
let a = SqlString::with_options("Hello", SqlCompareOptions::None);
let b = SqlString::new("hello");
assert_eq!(a.sql_equals(&b), SqlBoolean::FALSE);  // ordinal

// Binary sort
let a = SqlString::with_options("A", SqlCompareOptions::BinarySort);
let b = SqlString::new("a");
assert_eq!(a.sql_less_than(&b), SqlBoolean::TRUE);  // 'A' (0x41) < 'a' (0x61)

// NULL comparisons return NULL
let cmp = SqlString::new("hello").sql_equals(&SqlString::NULL);
assert!(cmp.is_null());

// Ordering
let apple = SqlString::new("apple");
let banana = SqlString::new("banana");
assert_eq!(apple.sql_less_than(&banana), SqlBoolean::TRUE);
```

## Trailing space handling

```rust
// Trailing spaces are trimmed in comparisons
let a = SqlString::new("hello");
let b = SqlString::new("hello   ");
assert_eq!(a.sql_equals(&b), SqlBoolean::TRUE);
```

## From conversions

```rust
// From &str
let a: SqlString = "hello".into();
assert_eq!(a.value().unwrap(), "hello");

// From String
let owned = String::from("world");
let b: SqlString = owned.into();
assert_eq!(b.value().unwrap(), "world");
```

## Display & Parse

```rust
// Display
assert_eq!(SqlString::new("hello").to_string(), "hello");
assert_eq!(SqlString::NULL.to_string(), "Null");

// Parse
let parsed: SqlString = "hello".parse().unwrap();
assert_eq!(parsed.value().unwrap(), "hello");

let null_parsed: SqlString = "Null".parse().unwrap();
assert!(null_parsed.is_null());

// "null", "NULL", "nUlL" all parse as NULL
let null2: SqlString = "NULL".parse().unwrap();
assert!(null2.is_null());
```

## Rust equality (case-insensitive, always)

```rust
// Rust == uses case-insensitive comparison regardless of compare_options
let a = SqlString::new("Hello");
let b = SqlString::new("hello");
assert_eq!(a, b);  // Eq is always case-insensitive

// Trailing spaces ignored in Eq too
let c = SqlString::new("hello   ");
assert_eq!(b, c);

// Can be used in HashSet/HashMap
use std::collections::HashSet;
let mut set = HashSet::new();
set.insert(SqlString::new("Hello"));
assert!(set.contains(&SqlString::new("hello")));
```
