# Copilot Instructions for mssqltypes

## Project Overview

This is a Rust library that provides faithful Rust equivalents of C# `System.Data.SqlTypes` from the .NET runtime. The reference C# implementations live at `~/work/runtime/src/libraries/System.Data.Common/src/System/Data/SQLTypes/`.

## Architecture

- **One Rust type per C# SqlType**: `SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney`, `SqlBinary`, `SqlString`, `SqlDateTime`, `SqlGuid`
- **Shared error type**: `SqlTypeError` enum with variants: `NullValue`, `Overflow`, `DivideByZero`, `ParseError`, `OutOfRange`
- **All types implement**: `Display`, `FromStr`, `Clone`, `Debug`; fixed-size types also implement `Copy`
- **Comparisons return `SqlBoolean`** (not `bool`) to support SQL three-valued NULL logic
- **Arithmetic returns `Result<T, SqlTypeError>`** — never panics

## Key Behavioral Rules

When implementing or modifying any type, these rules are NON-NEGOTIABLE:

1. **NULL propagation**: Any operation involving a NULL operand MUST return NULL (except `SqlBoolean` short-circuit: `FALSE & NULL = FALSE`, `TRUE | NULL = TRUE`)
2. **Overflow detection**: All integer and money arithmetic MUST check for overflow and return `Err(SqlTypeError::Overflow)` — never wrap
3. **NaN/Infinity rejection**: `SqlSingle` and `SqlDouble` MUST reject NaN and Infinity on construction and after arithmetic
4. **SqlGuid comparison order**: MUST use SQL Server's non-standard byte order: `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`
5. **SqlDateTime range**: 1753-01-01 to 9999-12-31, time stored as 1/300-second ticks
6. **SqlDecimal precision**: Up to 38 digits, 4×u32 internal representation
7. **SqlMoney scale**: Always 4 decimal places, stored as `i64 × 10,000`

## Coding Standards

- **No `unsafe` code** — the entire library must be safe Rust
- **No required external dependencies** — core library uses only `std`; optional `serde` behind feature flag
- **Run `cargo fmt` and `cargo clippy -- -D warnings`** before committing
- **TDD**: Write tests before implementation. Every public method needs test coverage.
- **Target ≥90% code coverage**

## Testing Patterns

```rust
// Test NULL propagation
#[test]
fn add_with_null_returns_null() {
    let a = SqlInt32::new(42);
    let result = a + SqlInt32::NULL;
    assert!(result.is_null());
}

// Test overflow detection
#[test]
fn add_overflow_returns_error() {
    let result = SqlInt32::new(i32::MAX).checked_add(SqlInt32::new(1));
    assert!(matches!(result, Err(SqlTypeError::Overflow)));
}

// Test comparison returns SqlBoolean
#[test]
fn comparison_with_null_returns_null() {
    let cmp = SqlInt32::new(1).sql_equals(&SqlInt32::NULL);
    assert!(cmp.is_null());
}
```

## Project Resources

- **Constitution**: `.specify/memory/constitution.md` — core principles and governance
- **Type specs**: `.specify/specs/sql-*.md` — detailed specifications per type
- **CI**: `.github/workflows/ci.yml` — build, test, lint, coverage on PRs
- **Reference C# code**: `~/work/runtime/src/libraries/System.Data.Common/src/System/Data/SQLTypes/`

## Common Type Patterns

All nullable SQL types follow this pattern:

```rust
pub struct SqlFoo {
    value: Option<InnerType>, // None = SQL NULL
}

impl SqlFoo {
    pub const NULL: SqlFoo = SqlFoo { value: None };
    pub fn new(v: InnerType) -> Self { Self { value: Some(v) } }
    pub fn is_null(&self) -> bool { self.value.is_none() }
    pub fn value(&self) -> Result<InnerType, SqlTypeError> {
        self.value.ok_or(SqlTypeError::NullValue)
    }
}
```

Comparisons:

```rust
impl SqlFoo {
    pub fn sql_equals(&self, other: &SqlFoo) -> SqlBoolean { ... }
    pub fn sql_less_than(&self, other: &SqlFoo) -> SqlBoolean { ... }
    pub fn sql_greater_than(&self, other: &SqlFoo) -> SqlBoolean { ... }
    // etc. — always return SqlBoolean, NULL if either operand is NULL
}
```

## When in Doubt

- Check the C# reference implementation for exact behavior
- Prefer returning `Result` over panicking
- Prefer `Copy` types for fixed-size structs
- Keep the public API surface minimal — only expose what C# exposes
