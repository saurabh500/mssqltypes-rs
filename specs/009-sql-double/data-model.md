# Data Model: SqlDouble

**Feature**: 009-sql-double
**Date**: 2026-03-02

## Entity: SqlDouble

A nullable 64-bit IEEE 754 floating-point number, equivalent to C# `System.Data.SqlTypes.SqlDouble` / SQL Server `FLOAT`.

### Internal Representation

```text
SqlDouble {
    value: Option<f64>    // None = SQL NULL, Some(v) = finite f64 value
}
```

**Invariant**: When `value` is `Some(v)`, `v` is always finite (`v.is_finite() == true`). NaN and Infinity are never stored.

### Traits

- `Copy`, `Clone`, `Debug` (derived)
- `PartialEq`, `Eq` (manual — safe because NaN excluded)
- `Hash` (manual — uses `f64::to_bits()` with `-0.0` normalization)
- `PartialOrd`, `Ord` (manual — NULL < any non-NULL)
- `Add`, `Sub`, `Mul`, `Div` (operator traits, `Output = Result<SqlDouble, SqlTypeError>`)
- `Neg` (unary negation, infallible)
- `Display`, `FromStr`
- `From<f64>` (panics on NaN/Infinity)

### Constants

| Name | Type | Value | Description |
|------|------|-------|-------------|
| `NULL` | `SqlDouble` | `SqlDouble { value: None }` | SQL NULL sentinel |
| `ZERO` | `SqlDouble` | `SqlDouble { value: Some(0.0) }` | Zero value |
| `MIN_VALUE` | `SqlDouble` | `SqlDouble { value: Some(f64::MIN) }` | Minimum finite f64 |
| `MAX_VALUE` | `SqlDouble` | `SqlDouble { value: Some(f64::MAX) }` | Maximum finite f64 |

### Validation Rules

| Rule | Scope | Error |
|------|-------|-------|
| Value must be finite | Construction (`new`, `From<f64>`) | `SqlTypeError::Overflow` |
| Arithmetic result must be finite | `Add`, `Sub`, `Mul`, `Div` | `SqlTypeError::Overflow` |
| Divisor must not be zero | `Div` | `SqlTypeError::DivideByZero` |
| Parsed string must not be "NaN"/"Infinity" | `FromStr` | `SqlTypeError::Overflow` |
| Parsed string must be valid number | `FromStr` | `SqlTypeError::ParseError` |

### State Transitions

```text
                 ┌─────────────────────────────────┐
                 │                                  │
   new(finite) ──►  Some(v)  ◄──arithmetic────►  Some(v')
                 │   (valid)     (if finite)      (valid)
                 │                                  │
                 └──────────────────────────────────┘

   SqlDouble::NULL ──► None (SQL NULL)
                        │
                        ├── arithmetic → None (NULL propagation)
                        ├── comparison → SqlBoolean::NULL
                        ├── negation → None
                        └── value() → Err(NullValue)
```

### Relationships to Other Types

| Direction | Target Type | Conversion | Notes |
|-----------|-------------|------------|-------|
| From | `SqlByte` | Widening | `u8 as f64` — always exact |
| From | `SqlInt16` | Widening | `i16 as f64` — always exact |
| From | `SqlInt32` | Widening | `i32 as f64` — always exact |
| From | `SqlInt64` | Widening | `i64 as f64` — may lose precision for large values |
| From | `SqlMoney` | Widening | `i64 / 10_000.0` — always fits in f64 range |
| From | `SqlBoolean` | Explicit | TRUE=1.0, FALSE=0.0, NULL=NULL |
| To | `SqlBoolean` | Explicit | 0.0→FALSE, non-zero→TRUE, NULL→NULL |
| From | `SqlSingle` | Widening | **DEFERRED** — SqlSingle not yet implemented |
| To | `SqlSingle` | Narrowing | **DEFERRED** — SqlSingle not yet implemented |
