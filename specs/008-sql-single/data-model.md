# Data Model: SqlSingle

**Feature**: 008-sql-single | **Date**: 2026-03-02

## Entity: SqlSingle

### Definition

A nullable 32-bit IEEE 754 floating-point number equivalent to C# `System.Data.SqlTypes.SqlSingle` / SQL Server `REAL`.

### Internal Representation

```rust
#[derive(Copy, Clone, Debug)]
pub struct SqlSingle {
    value: Option<f32>,  // None = SQL NULL, Some(v) = finite f32
}
```

### Invariant

The contained `f32` is **always finite** — never NaN, Infinity, or NEG_INFINITY. This is enforced at construction and after every arithmetic operation.

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `NULL` | `SqlSingle { value: None }` | SQL NULL sentinel |
| `ZERO` | `SqlSingle { value: Some(0.0) }` | Zero value |
| `MIN_VALUE` | `SqlSingle { value: Some(f32::MIN) }` | Minimum finite f32 (≈ -3.4028235 × 10^38) |
| `MAX_VALUE` | `SqlSingle { value: Some(f32::MAX) }` | Maximum finite f32 (≈ 3.4028235 × 10^38) |

### Fields

| Field | Type | Nullable | Description |
|-------|------|----------|-------------|
| `value` | `Option<f32>` | Yes (Option) | `None` = SQL NULL; `Some(v)` where v is always finite |

### Validation Rules

| Rule | Condition | Error |
|------|-----------|-------|
| Finite check | `!v.is_finite()` | `SqlTypeError::Overflow` |
| Division by zero | divisor `== 0.0` | `SqlTypeError::DivideByZero` |
| Arithmetic overflow | result `!is_finite()` | `SqlTypeError::Overflow` |
| Parse failure | invalid string | `SqlTypeError::ParseError` |
| Parse NaN/Infinity | string is "NaN", "Infinity", etc. | `SqlTypeError::ParseError` |
| NULL access | `value()` on NULL | `SqlTypeError::NullValue` |

### State Transitions

```text
                 new(finite_f32)
    [Uninitialized] ────────────────► [Valid(f32)]
                                          │
                  NULL constant           │ arithmetic / conversion
                      │                   ▼
                      ▼              [Valid(f32)] ◄──── if result is_finite()
                   [Null]                 │
                      │                   │ if result !is_finite()
                      │                   ▼
                      │            Err(Overflow / DivideByZero)
                      │
                      └──── propagates through all operations ──► [Null]
```

### Relationships

| Related Type | Direction | Conversion | Notes |
|---|---|---|---|
| `SqlBoolean` | from | `from_sql_boolean()` | TRUE→1.0, FALSE→0.0, NULL→NULL |
| `SqlBoolean` | to | `to_sql_boolean()` | 0.0→FALSE, non-zero→TRUE, NULL→NULL |
| `SqlByte` | from | `from_sql_byte()` | Widening, infallible |
| `SqlInt16` | from | `from_sql_int16()` | Widening, infallible |
| `SqlInt32` | from | `from_sql_int32()` | Widening, may lose precision |
| `SqlInt64` | from | `from_sql_int64()` | Widening, may lose precision |
| `SqlMoney` | from | `from_sql_money()` | Via f64 intermediate, may lose precision |
| `SqlDouble` | to | `to_sql_double()` | Widening, lossless, infallible |
| `SqlDouble` | from | `from_sql_double()` | Narrowing, may overflow → Err(Overflow) |

### Trait Implementations

| Trait | Behavior |
|-------|----------|
| `Copy`, `Clone` | Bitwise copy (fixed-size stack type) |
| `Debug` | Derived |
| `Display` | NULL → "Null", value → f32 default display |
| `FromStr` | "Null" → NULL, parse f32, reject NaN/Infinity |
| `PartialEq`, `Eq` | NULL == NULL (Rust semantics); safe because NaN excluded |
| `Hash` | `f32::to_bits()` with -0.0 normalization |
| `PartialOrd`, `Ord` | NULL < any non-NULL; then f32 ordering |
| `Add`, `Sub`, `Mul`, `Div` | `Output = Result<SqlSingle, SqlTypeError>` |
| `Neg` | `Output = SqlSingle` (infallible) |
| `From<f32>` | Panics on NaN/Infinity |
