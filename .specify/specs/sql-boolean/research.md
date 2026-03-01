# Research: SqlBoolean — C# Reference Analysis

**Date**: 2026-03-01
**Source**: `~/work/runtime/src/libraries/System.Data.Common/src/System/Data/SQLTypes/SQLBoolean.cs` (532 lines)

## Internal Layout

```csharp
private byte m_value;
// 0 = Null (unknown)
// 1 = False
// 2 = True
```

This non-obvious encoding (False=1, True=2 rather than False=0, True=1) is deliberate — it reserves 0 for Null and allows direct comparison of `m_value` to work correctly for ordering (False < True).

### Rust Mapping

```rust
#[derive(Clone, Copy, Debug)]
pub struct SqlBoolean {
    m_value: u8,
}

const X_NULL: u8 = 0;
const X_FALSE: u8 = 1;
const X_TRUE: u8 = 2;
```

## Constructors

| C# Signature | Behavior | Rust Equivalent |
|--------------|----------|-----------------|
| `SqlBoolean(bool value)` | `true→2, false→1` | `SqlBoolean::new(value: bool)` |
| `SqlBoolean(int value)` | `0→False, nonzero→True` | `SqlBoolean::from_int(value: i32)` |
| `SqlBoolean(int, bool fNull)` | Private, used for Null construction | `SqlBoolean::NULL` constant |

## Properties → Methods

| C# Property | Type | Null Behavior | Rust Method |
|-------------|------|---------------|-------------|
| `IsNull` | `bool` | Returns `true` | `is_null(&self) -> bool` |
| `IsTrue` | `bool` | Returns `false` | `is_true(&self) -> bool` |
| `IsFalse` | `bool` | Returns `false` | `is_false(&self) -> bool` |
| `Value` | `bool` | Throws `SqlNullValueException` | `value(&self) -> Result<bool, SqlTypeError>` |
| `ByteValue` | `byte` | Throws `SqlNullValueException` | `byte_value(&self) -> Result<u8, SqlTypeError>` |

## Constants

| C# Field | Value | Rust Constant |
|----------|-------|---------------|
| `True` | `m_value = 2` | `SqlBoolean::TRUE` |
| `False` | `m_value = 1` | `SqlBoolean::FALSE` |
| `Null` | `m_value = 0` | `SqlBoolean::NULL` |
| `Zero` | `m_value = 1` (same as False) | `SqlBoolean::ZERO` |
| `One` | `m_value = 2` (same as True) | `SqlBoolean::ONE` |

## Operator Truth Tables

### NOT (`!`)

| Input | Output |
|-------|--------|
| True  | False  |
| False | True   |
| Null  | Null   |

### AND (`&`) — *FALSE short-circuits*

|  & | True | False | Null  |
|----|------|-------|-------|
| **True** | True | False | Null |
| **False** | False | False | **False** |
| **Null** | Null | **False** | Null |

Key: `FALSE & NULL = FALSE` (short-circuit, not standard NULL propagation)

### OR (`|`) — *TRUE short-circuits*

| \| | True | False | Null |
|----|------|-------|------|
| **True** | True | True | **True** |
| **False** | True | False | Null |
| **Null** | **True** | Null | Null |

Key: `TRUE | NULL = TRUE` (short-circuit, not standard NULL propagation)

### XOR (`^`)

| ^ | True | False | Null |
|---|------|-------|------|
| **True** | False | True | Null |
| **False** | True | False | Null |
| **Null** | Null | Null | Null |

XOR does NOT short-circuit — any Null yields Null.

### Bitwise complement (`~`)

Identical to NOT. In Rust, there is no separate `~` operator; `Not` trait covers both.

## Comparison Operators

All return `SqlBoolean`. If either operand is Null, result is Null.

Comparisons use the raw `m_value` encoding (False=1 < True=2):
- `x == y` → `m_value == m_value`
- `x < y` → `m_value < m_value` (so False < True ✓)
- `x != y` → `!(x == y)`

## CompareTo (IComparable)

```
Both Null → 0 (equal)
x is Null, y is not → -1 (Null sorts first)
x is not Null, y is Null → 1
Otherwise → compare ByteValue (0 for False, 1 for True)
```

## Equals / GetHashCode

```csharp
// Equals: two Nulls ARE equal (unlike SQL comparison)
public bool Equals(SqlBoolean other) =>
    other.IsNull || IsNull ? other.IsNull && IsNull :
    (this == other).Value;

// Hash: 0 for Null, otherwise bool.GetHashCode()
public override int GetHashCode() => IsNull ? 0 : Value.GetHashCode();
```

**Important**: `Equals` treats two Nulls as equal (for use in collections), while `sql_equals` returns Null for Null comparisons (SQL semantics). These are distinct behaviors.

## Display / Parse

### ToString
- Null → `"Null"` (from `SQLResource.NullString`)
- True → `"True"` (from `bool.ToString()`)
- False → `"False"` (from `bool.ToString()`)

### Parse
```
1. If input == "Null" → return SqlBoolean.Null
2. If first char is digit, '-', or '+' → parse as int → 0=False, nonzero=True
3. Otherwise → parse as bool ("true"/"false", case-insensitive)
4. Invalid → throw FormatException
```

**Rust Note**: `FromStr` should handle "Null" → `Ok(SqlBoolean::NULL)`, numeric strings, and boolean strings. Invalid input returns `Err(SqlTypeError::ParseError)`.

## Type Conversions (Deferred)

These will be added when the target types are implemented:

| To/From | Behavior |
|---------|----------|
| `SqlByte` → `SqlBoolean` | `0→False, nonzero→True, Null→Null` |
| `SqlBoolean` → `SqlByte` | `False→0, True→1, Null→Null` |
| `SqlInt16` → `SqlBoolean` | Same pattern |
| `SqlInt32` → `SqlBoolean` | Same pattern |
| `SqlInt64` → `SqlBoolean` | Same pattern |
| `SqlSingle` → `SqlBoolean` | `0.0→False, nonzero→True, Null→Null` |
| `SqlDouble` → `SqlBoolean` | Same as SqlSingle |
| `SqlMoney` → `SqlBoolean` | `Zero→False, otherwise True, Null→Null` |
| `SqlDecimal` → `SqlBoolean` | All data words zero→False, otherwise True, Null→Null |
| `SqlString` → `SqlBoolean` | Parse string value, Null→Null |

## Design Decisions

1. **`u8` internal repr, not `Option<bool>`**: Using `u8` directly matches C# layout and avoids the overhead of `Option` discriminant + bool. Three states pack into one byte.

2. **Constants as `const`, not `static`**: Rust `const` items are inlined at use sites, giving zero-cost access. C# uses `static readonly` because it lacks `const struct`.

3. **`sql_equals` vs `PartialEq`**: We need both. `PartialEq` (Rust `==`) follows C# `Equals()` — two Nulls are equal. `sql_equals` follows SQL semantics — Null comparison yields Null. This matches the C# distinction between `Equals()` and `operator ==`.

4. **`Ord` implementation**: Follows `CompareTo` — Null < False < True. This implements total ordering for sorting and collection use.

5. **`FromStr` for "Null"**: Returns `Ok(SqlBoolean::NULL)` rather than `Err`, matching C#'s `Parse("Null")` behavior.
