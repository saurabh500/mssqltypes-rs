# Data Model: Cross-Type Conversions

**Feature**: 014-cross-type-conversions
**Date**: 2026-03-02

## Overview

This feature adds no new entities or types. It adds conversion methods and trait implementations to 11 existing types. The data model describes the conversion relationships between types.

## Existing Entities (unchanged)

All 13 SQL types are already implemented with their internal representations:

| Entity | Internal Representation | Copy | Nullable |
|--------|------------------------|------|----------|
| SqlBoolean | `m_value: u8` (0=Null, 1=False, 2=True) | Yes | Yes |
| SqlByte | `value: Option<u8>` | Yes | Yes |
| SqlInt16 | `value: Option<i16>` | Yes | Yes |
| SqlInt32 | `value: Option<i32>` | Yes | Yes |
| SqlInt64 | `value: Option<i64>` | Yes | Yes |
| SqlSingle | `value: Option<f32>` | Yes | Yes |
| SqlDouble | `value: Option<f64>` | Yes | Yes |
| SqlDecimal | `inner: Option<InnerDecimal>` (precision, scale, sign, [u32; 4]) | No | Yes |
| SqlMoney | `value: Option<i64>` (×10,000) | Yes | Yes |
| SqlBinary | `value: Option<Vec<u8>>` | No | Yes |
| SqlString | `value: Option<String>`, `compare_options: SqlCompareOptions` | No | Yes |
| SqlDateTime | `value: Option<(i32, i32)>` (day_ticks, time_ticks) | Yes | Yes |
| SqlGuid | `value: Option<[u8; 16]>` | Yes | Yes |

## Conversion Relationships

### Widening Conversions (infallible, `From` trait)

Widening conversions are lossless — the source type's full value range fits within the target type.

```text
SqlByte ──From──> SqlInt32
SqlByte ──From──> SqlInt64
SqlInt16 ─From──> SqlInt32
SqlInt16 ─From──> SqlInt64
SqlInt32 ─From──> SqlInt64
```

These augment existing widening chains:
- SqlBoolean → {SqlByte, SqlInt16, SqlInt32, SqlInt64} (already exist)
- SqlByte → SqlInt16 (already exists)
- SqlByte/SqlInt16/SqlInt32/SqlInt64 → {SqlSingle, SqlDouble, SqlDecimal, SqlMoney} (already exist)

### Narrowing Numeric Conversions (fallible, `Result`)

Narrowing conversions may lose data or overflow.

```text
SqlInt32 ──to_sql_boolean()──> SqlBoolean    (zero→FALSE, nonzero→TRUE)
SqlInt64 ──to_sql_boolean()──> SqlBoolean    (zero→FALSE, nonzero→TRUE)

SqlDouble ─to_sql_single()──> Result<SqlSingle>   (overflow if out of f32 range)

SqlDecimal ─to_sql_single()──> SqlSingle     (precision loss, no overflow possible)
SqlDecimal ─to_sql_double()──> SqlDouble     (precision loss, no overflow possible)
SqlDecimal ─to_sql_money()───> Result<SqlMoney>    (range check)

SqlMoney ──to_sql_single()──> SqlSingle      (precision loss, no overflow)
SqlMoney ──to_sql_double()──> SqlDouble      (precision loss, no overflow)
```

### Widening Float Conversion

```text
SqlSingle ─from_sql_single()──> SqlDouble    (placed on SqlDouble, widening)
```

### Float/Money → Decimal/Money (fallible construction)

```text
SqlSingle ──From──> SqlDecimal    (reject NaN/Infinity)
SqlDouble ──From──> SqlDecimal    (reject NaN/Infinity)
SqlMoney  ──From──> SqlDecimal    (preserves 4-decimal scale)

SqlSingle ─from_sql_single()──> Result<SqlMoney>   (range check)
SqlDouble ─from_sql_double()──> Result<SqlMoney>   (range check)
```

### String Hub Conversions

```text
All 11 non-SqlBinary types ──to_sql_string()──> SqlString   (via Display)

SqlString ──to_sql_boolean()──> Result<SqlBoolean>
SqlString ──to_sql_byte()────> Result<SqlByte>
SqlString ──to_sql_int16()───> Result<SqlInt16>
SqlString ──to_sql_int32()───> Result<SqlInt32>
SqlString ──to_sql_int64()───> Result<SqlInt64>
SqlString ──to_sql_single()──> Result<SqlSingle>
SqlString ──to_sql_double()──> Result<SqlDouble>
SqlString ──to_sql_decimal()─> Result<SqlDecimal>
SqlString ──to_sql_money()───> Result<SqlMoney>
SqlString ──to_sql_date_time()> Result<SqlDateTime>
SqlString ──to_sql_guid()────> Result<SqlGuid>
```

## Validation Rules

1. **NULL propagation**: Every conversion checks source `is_null()` first; if NULL, returns target's NULL
2. **Overflow detection**: Narrowing conversions that may exceed target range return `Err(SqlTypeError::Overflow)`
3. **NaN/Infinity rejection**: Float-to-decimal conversions reject non-finite values with `Err(SqlTypeError::OutOfRange)`
4. **Parse error handling**: String parsing failures return `Err(SqlTypeError::ParseError(message))`
5. **Boolean semantics**: Zero → FALSE, any non-zero → TRUE (matches C# exactly)

## No State Transitions

Conversions are stateless transformations — they create new instances of the target type. No entity undergoes state changes.
