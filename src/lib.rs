// Licensed under the MIT License. See LICENSE file in the project root for full license information.

//! # mssqltypes
//!
//! Faithful Rust equivalents of C# `System.Data.SqlTypes` with full SQL NULL
//! semantics, checked arithmetic, and three-valued logic.
//!
//! This library replicates the behavior of SQL Server's native data types as
//! defined in the .NET `System.Data.SqlTypes` namespace. Every type supports
//! SQL NULL propagation, overflow-checked arithmetic, and idiomatic Rust trait
//! implementations. It is designed for Rust-based SQL Server drivers, ORMs, and
//! any application that needs full fidelity to SQL Server type behavior.
//!
//! ## Types
//!
//! | Rust Type | SQL Server Type | C# Equivalent |
//! |-----------|----------------|---------------|
//! | [`SqlBoolean`] | `BIT` | `SqlBoolean` |
//! | [`SqlByte`] | `TINYINT` | `SqlByte` |
//! | [`SqlInt16`] | `SMALLINT` | `SqlInt16` |
//! | [`SqlInt32`] | `INT` | `SqlInt32` |
//! | [`SqlInt64`] | `BIGINT` | `SqlInt64` |
//! | [`SqlSingle`] | `REAL` | `SqlSingle` |
//! | [`SqlDouble`] | `FLOAT` | `SqlDouble` |
//! | [`SqlDecimal`] | `DECIMAL` / `NUMERIC` | `SqlDecimal` |
//! | [`SqlMoney`] | `MONEY` | `SqlMoney` |
//! | [`SqlBinary`] | `BINARY` / `VARBINARY` | `SqlBinary` |
//! | [`SqlString`] | `CHAR` / `VARCHAR` / `NVARCHAR` | `SqlString` |
//! | [`SqlDateTime`] | `DATETIME` | `SqlDateTime` |
//! | [`SqlGuid`] | `UNIQUEIDENTIFIER` | `SqlGuid` |
//!
//! ## Key Features
//!
//! - **SQL NULL semantics**: Three-valued logic propagation across all operations
//! - **Checked arithmetic**: Overflow, divide-by-zero, and out-of-range errors
//!   returned as [`Result<T, SqlTypeError>`](SqlTypeError)
//! - **Idiomatic Rust**: `Display`, `FromStr`, `PartialEq`, `Clone`, `Copy`,
//!   and operator overloading
//! - **Zero unsafe code**: Entirely safe Rust
//! - **Zero required dependencies**: Core library uses only `std`
//!
//! ## Quick Start
//!
//! ```
//! use mssqltypes::{SqlInt32, SqlBoolean};
//!
//! // Create values
//! let a = SqlInt32::new(42);
//! let b = SqlInt32::new(58);
//!
//! // Arithmetic returns Result (checked for overflow)
//! let sum = (a + b).unwrap();
//! assert_eq!(sum.value().unwrap(), 100);
//!
//! // NULL propagation
//! let null_sum = a + SqlInt32::NULL;
//! assert!(null_sum.unwrap().is_null());
//!
//! // Comparisons return SqlBoolean (three-valued)
//! let cmp = a.sql_less_than(&b);
//! assert_eq!(cmp, SqlBoolean::TRUE);
//!
//! // Three-valued boolean logic
//! let result = SqlBoolean::TRUE | SqlBoolean::NULL;
//! assert_eq!(result, SqlBoolean::TRUE); // short-circuit
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `serde` | Enable `Serialize` / `Deserialize` for all types (optional) |
//!
//! By default, no features are enabled and the crate has zero dependencies.

pub mod error;
pub mod sql_binary;
pub mod sql_boolean;
pub mod sql_byte;
pub mod sql_compare_options;
pub mod sql_datetime;
pub mod sql_decimal;
pub mod sql_double;
pub mod sql_guid;
pub mod sql_int16;
pub mod sql_int32;
pub mod sql_int64;
pub mod sql_money;
pub mod sql_single;
pub mod sql_string;

pub use error::SqlTypeError;
pub use sql_binary::SqlBinary;
pub use sql_boolean::SqlBoolean;
pub use sql_byte::SqlByte;
pub use sql_compare_options::SqlCompareOptions;
pub use sql_datetime::SqlDateTime;
pub use sql_decimal::SqlDecimal;
pub use sql_double::SqlDouble;
pub use sql_guid::SqlGuid;
pub use sql_int16::SqlInt16;
pub use sql_int32::SqlInt32;
pub use sql_int64::SqlInt64;
pub use sql_money::SqlMoney;
pub use sql_single::SqlSingle;
pub use sql_string::SqlString;
