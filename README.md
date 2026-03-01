# mssqltypes

Rust implementations of SQL Server data types, providing behavioral equivalents of the C#
`System.Data.SqlTypes` namespace.

## Overview

`mssqltypes` is a Rust library that faithfully replicates the behavior of SQL Server's native
data types as defined in the .NET `System.Data.SqlTypes` namespace. Every type supports
**SQL NULL semantics** (three-valued logic), checked arithmetic, and idiomatic Rust trait
implementations.

This library is designed for use in Rust-based SQL Server drivers, ORMs, and any application
that needs to work with SQL Server types with full fidelity to the server's behavior.

## Types

| Rust Type | SQL Server Type | C# Equivalent | Description |
|-----------|----------------|---------------|-------------|
| `SqlBoolean` | `BIT` | `SqlBoolean` | Three-state boolean (True / False / Null) |
| `SqlByte` | `TINYINT` | `SqlByte` | Unsigned 8-bit integer (0–255) |
| `SqlInt16` | `SMALLINT` | `SqlInt16` | Signed 16-bit integer |
| `SqlInt32` | `INT` | `SqlInt32` | Signed 32-bit integer |
| `SqlInt64` | `BIGINT` | `SqlInt64` | Signed 64-bit integer |
| `SqlSingle` | `REAL` | `SqlSingle` | 32-bit floating point (no NaN/Infinity) |
| `SqlDouble` | `FLOAT` | `SqlDouble` | 64-bit floating point (no NaN/Infinity) |
| `SqlDecimal` | `DECIMAL` / `NUMERIC` | `SqlDecimal` | Fixed-point decimal (up to 38 digits) |
| `SqlMoney` | `MONEY` | `SqlMoney` | Currency with 4 decimal places |
| `SqlBinary` | `BINARY` / `VARBINARY` | `SqlBinary` | Variable-length byte sequence |
| `SqlString` | `CHAR` / `VARCHAR` / `NVARCHAR` | `SqlString` | String with comparison options |
| `SqlDateTime` | `DATETIME` | `SqlDateTime` | Date/time (1753–9999, ~3.33ms precision) |
| `SqlGuid` | `UNIQUEIDENTIFIER` | `SqlGuid` | 128-bit GUID with SQL Server sort order |

## Features

- **SQL NULL semantics**: All types support NULL with three-valued logic propagation
- **Checked arithmetic**: Overflow, divide-by-zero, and out-of-range errors returned as `Result`
- **Idiomatic Rust**: Standard trait implementations (`Display`, `FromStr`, `PartialEq`, `Clone`, `Copy`, operator overloading)
- **Zero unsafe code**: Entirely safe Rust
- **Zero required dependencies**: Core library uses only the Rust standard library
- **Optional serde support**: Enable the `serde` feature flag for serialization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mssqltypes = "0.1"
```

With serde support:

```toml
[dependencies]
mssqltypes = { version = "0.1", features = ["serde"] }
```

## Quick Start

```rust
use mssqltypes::{SqlInt32, SqlBoolean};

// Create values
let a = SqlInt32::new(42);
let b = SqlInt32::new(58);
let null_val = SqlInt32::NULL;

// Arithmetic (returns Result)
let sum = (a + b).unwrap(); // SqlInt32(100)

// NULL propagation
let null_sum = a + null_val; // SqlInt32::NULL

// Comparisons return SqlBoolean (three-valued)
let cmp = a.sql_less_than(&b); // SqlBoolean::TRUE
let null_cmp = a.sql_equals(&null_val); // SqlBoolean::NULL

// Three-valued boolean logic
let and_result = SqlBoolean::TRUE & SqlBoolean::NULL; // SqlBoolean::NULL
let or_result = SqlBoolean::TRUE | SqlBoolean::NULL;  // SqlBoolean::TRUE (short-circuit)
```

## Error Handling

All fallible operations return `Result<T, SqlTypeError>`:

```rust
use mssqltypes::{SqlByte, SqlTypeError};

let a = SqlByte::new(200);
let b = SqlByte::new(100);

match a + b {
    Ok(result) => println!("Sum: {}", result),
    Err(SqlTypeError::Overflow) => println!("Overflow!"),
    Err(e) => println!("Error: {}", e),
}
```

Error variants:
- `SqlTypeError::NullValue` — attempted to access value of NULL
- `SqlTypeError::Overflow` — arithmetic overflow
- `SqlTypeError::DivideByZero` — division by zero
- `SqlTypeError::ParseError` — string parsing failure
- `SqlTypeError::OutOfRange` — value outside valid range

## SQL NULL Semantics

All types implement SQL three-valued logic:

- **Arithmetic with NULL** → NULL
- **Comparison with NULL** → SqlBoolean::NULL
- **AND**: `FALSE & NULL = FALSE` (short-circuit), `TRUE & NULL = NULL`
- **OR**: `TRUE | NULL = TRUE` (short-circuit), `FALSE | NULL = NULL`

## Building & Testing

```bash
# Build
cargo build

# Run tests
cargo test

# Run with all features
cargo test --all-features

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## Code Coverage

Coverage is automatically generated on every PR via GitHub Actions using `grcov` with
LLVM source-based coverage. Reports are published as PR comments and downloadable artifacts.

## Project Structure

```
mssqltypes/
├── .github/
│   └── workflows/
│       └── ci.yml              # CI: build, test, lint, coverage
├── .specify/
│   ├── memory/
│   │   └── constitution.md     # Project constitution & principles
│   └── specs/                  # Feature specifications per type
├── src/
│   └── lib.rs                  # Library source
├── Cargo.toml
└── README.md
```

## Spec-Driven Development

This project uses [GitHub Spec Kit](https://github.com/github/spec-kit) for spec-driven
development. Specifications for each type are in `.specify/specs/`. The project constitution
defining core principles is in `.specify/memory/constitution.md`.

## Contributing

1. Fork the repository
2. Create a feature branch (`feature/sql-<type>`)
3. Write specs first, then tests, then implementation (TDD)
4. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass
5. Submit a PR — CI will run tests and generate coverage

## License

[MIT](LICENSE)
