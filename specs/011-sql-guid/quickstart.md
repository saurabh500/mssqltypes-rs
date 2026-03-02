# SqlGuid Quickstart

## Create & Inspect

```rust
use mssqltypes::{SqlGuid, SqlBoolean};

// Create from 16-byte array
let guid = SqlGuid::new([
    0xFF, 0x19, 0x96, 0x6F,  // time_low (LE)
    0x86, 0x8B,              // time_mid (LE)
    0x11, 0xD0,              // time_hi_and_version (LE)
    0xB4, 0x2D,              // clock_seq
    0x00, 0xCF, 0x4F, 0xC9, 0x64, 0xFF, // node
]);

// Check NULL
assert!(!guid.is_null());
assert!(SqlGuid::NULL.is_null());

// Get inner bytes
let bytes = guid.value().unwrap();
assert_eq!(bytes.len(), 16);
```

## Display & Parsing

```rust
use std::str::FromStr;

// Display prints lowercase hyphenated format (matches .NET "D" format)
let guid = SqlGuid::new([
    0xFF, 0x19, 0x96, 0x6F,
    0x86, 0x8B, 0x11, 0xD0,
    0xB4, 0x2D, 0x00, 0xCF,
    0x4F, 0xC9, 0x64, 0xFF,
]);
assert_eq!(guid.to_string(), "6f9619ff-8b86-d011-b42d-00cf4fc964ff");

// Parse hyphenated format
let parsed = SqlGuid::from_str("6f9619ff-8b86-d011-b42d-00cf4fc964ff").unwrap();
assert_eq!(guid, parsed);

// Parse bare hex (no hyphens)
let bare = SqlGuid::from_str("6f9619ff8b86d011b42d00cf4fc964ff").unwrap();
assert_eq!(guid, bare);

// NULL displays as "Null"
assert_eq!(SqlGuid::NULL.to_string(), "Null");

// Parse "Null" string
let null_guid = SqlGuid::from_str("Null").unwrap();
assert!(null_guid.is_null());
```

## SQL Comparisons (using SQL Server byte order)

```rust
use mssqltypes::{SqlGuid, SqlBoolean};

let a = SqlGuid::new([0; 16]);
let b = SqlGuid::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

// Comparisons return SqlBoolean, not bool
let eq = a.sql_equals(&a);
assert_eq!(eq.value().unwrap(), true);

let lt = a.sql_less_than(&b);
assert_eq!(lt.value().unwrap(), true);

// NULL propagation
let cmp = a.sql_equals(&SqlGuid::NULL);
assert!(cmp.is_null());
```

## SqlBinary Conversions

```rust
use mssqltypes::{SqlGuid, SqlBinary};

let guid = SqlGuid::new([1u8; 16]);

// Convert to SqlBinary (16 bytes)
let binary = guid.to_sql_binary();
assert!(!binary.is_null());

// Convert back from SqlBinary
let roundtrip = SqlGuid::from_sql_binary(&binary).unwrap();
assert_eq!(guid, roundtrip);

// NULL propagation
let null_binary = SqlGuid::NULL.to_sql_binary();
assert!(null_binary.is_null());

let null_guid = SqlGuid::from_sql_binary(&SqlBinary::NULL).unwrap();
assert!(null_guid.is_null());
```
