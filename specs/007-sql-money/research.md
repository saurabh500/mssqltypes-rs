# Research: SqlMoney

## R1: Add/Subtract Strategy

**Decision**: Use `i64::checked_add()` and `i64::checked_sub()` directly on the internal scaled values. No intermediate conversion needed.

**Rationale**: C# SqlMoney addition (line 282) does `checked(x._value + y._value)` — direct i64 addition on the already-scaled tick values, wrapped in C#'s `checked` context for overflow detection. Subtraction (line 293) is identical: `checked(x._value - y._value)`. Since both operands are already scaled by 10,000, adding/subtracting their raw tick values gives an exact result with no rounding. Rust's `i64::checked_add()` / `checked_sub()` provide identical overflow detection in a single call.

**Alternatives considered**:
- Convert to i128  first: Unnecessary — direct i64 checked arithmetic is simpler and equally correct. Both operands share the same scale factor, so the result is already correctly scaled. Rejected.

## R2: Multiply/Divide Strategy

**Decision**: Use `i128` intermediate for both multiplication and division.

**For multiplication**: Convert both i64 tick values to i128, multiply, divide by 10,000 (to correct double-scaling), round, then check if result fits in i64.

**For division**: Convert dividend to i128, multiply by 10,000 (to preserve 4dp in result), divide by divisor, round, then check if result fits in i64.

**Rationale**: C# SqlMoney multiplication (line 303) converts both operands to `decimal` via `ToDecimal()`, then calls `decimal.Multiply()`, then constructs a new `SqlMoney(decimal)` which internally scales and range-checks. C# SqlMoney division (line 308) follows the same pattern with `decimal.Divide()`. This decimal intermediary approach works in C# because `decimal` is a 128-bit type with 28-29 significant digits.

In Rust, we don't have a built-in `decimal` type. Instead, we can use `i128`:
- **Multiply**: `(a as i128) * (b as i128)` gives a 128-bit result (double-scaled by 10,000²). Dividing by 10,000 corrects the scale. The maximum intermediate value is `i64::MAX * i64::MAX ≈ 8.5 × 10^37`, which fits in i128 (max ~1.7 × 10^38). After dividing by 10,000, range-check against i64.
- **Divide**: `((a as i128) * 10_000) / (b as i128)` preserves 4 decimal places. The maximum intermediate is `i64::MAX * 10,000 ≈ 9.2 × 10^22`, easily within i128 range.

For rounding of multiplication: the division by 10,000 may have a remainder. Apply round-half-away-from-zero (consistent with C# decimal behavior):
```
remainder = result_128 % 10_000
if abs(remainder) >= 5_000 → round away from zero
```

**Alternatives considered**:
- Use Rust `SqlDecimal` as intermediary (mirroring C#): Would require non-trivial conversions and the SqlDecimal API. More complex than i128, which is a native Rust type. Rejected.
- Use `f64` intermediary: Loses precision for large values. A money value of 922,337,203,685,477.5807 cannot be exactly represented in f64. Rejected.

## R3: Negation Overflow Detection

**Decision**: Use `i64::checked_neg()` to detect overflow when negating. This correctly catches `i64::MIN`, which cannot be negated.

**Rationale**: C# SQLMoney.cs (lines 271-272) checks `x._value == s_minLong` before negation, where `s_minLong = i64::MIN / 10000 = -922_337_203_685_477`. However, this is a **bug in the C# reference**: `s_minLong` is the minimum *unscaled* monetary value (used for range-checking the `SqlMoney(long)` constructor), not the minimum internal tick value. The internal `_value` field ranges from `i64::MIN` to `i64::MAX` (the `MinValue` and `MaxValue` constants use raw `i64::MIN`/`i64::MAX`). So the C# code only throws for one specific value (`-922_337_203_685_477`) while silently wrapping `-i64::MIN` back to `i64::MIN` in unchecked context (since C# arithmetic is unchecked by default outside of `checked` blocks).

Rust's `i64::checked_neg()` returns `None` for `i64::MIN`, correctly detecting the real overflow case. This is stricter and more correct than C#.

**Alternatives considered**:
- Match C# bug (check against `i64::MIN / 10_000`): Incorrect — would miss the true overflow case and silently wrap. Rejected per Constitution I principle of catching the *correct* edge case.

## R4: Display Format

**Decision**: Format as `"#0.00##"` — minimum 2 decimal places, maximum 4, trimming trailing zeros beyond the 2nd place.

**Rationale**: C# `ToString()` (line 207) converts to decimal first, then formats with `money.ToString("#0.00##", null)`. In C#'s custom format `"#0.00##"`:
- `#0` = at least one digit before decimal point
- `.00` = exactly 2 mandatory decimal digits
- `##` = up to 2 additional optional decimal digits (printed only if non-zero)

So: `100.0000` → `"100.00"`, `123.4500` → `"123.45"`, `123.4567` → `"123.4567"`, `-50.1000` → `"-50.10"`.

Implementation in Rust: extract the 4-digit fractional part from the internal value, then conditionally trim trailing zeros to a minimum of 2 digits.

**Alternatives considered**:
- Always show 4 decimal places: Simpler, but doesn't match C# behavior. The spec explicitly requires `"#0.00##"` format. Rejected.
- Use Rust's `format!("{:.4}", value)`: Always shows 4 decimal places — doesn't trim trailing zeros. Would need post-processing anyway. Rejected as base approach.

## R5: `to_i64()` Rounding Semantics

**Decision**: Use round-half-away-from-zero, matching C# `SqlMoney.ToInt64()`.

**Rationale**: C#'s `ToInt64()` (lines 159-176) works as follows:
1. Divides `_value` by `s_lTickBase / 10` (= 1000), giving a value with one remaining decimal digit
2. Takes `remainder = result % 10`
3. Integer-divides by 10 to remove the last digit
4. If `remainder >= 5`: increment for positive, decrement for negative

This is round-half-away-from-zero: 42.5 → 43, -42.5 → -43, 42.4 → 42. In Rust implementation:
```rust
let div_1000 = value / 1000; // one extra digit of precision
let remainder = (div_1000 % 10).abs();
let result = div_1000 / 10;
if remainder >= 5 {
    if value >= 0 { result + 1 } else { result - 1 }
}
```

**Alternatives considered**:
- Truncation (round toward zero): Doesn't match C# behavior. Rejected.
- Banker's rounding (round-half-to-even): C# explicitly uses round-half-away-from-zero in this implementation. Rejected.

## R6: `from_f64()` Construction

**Decision**: Reject NaN and Infinity with `Err(OutOfRange)`. For valid finite values, multiply by 10,000, round to nearest integer, then check i64 range.

**Rationale**: C# `SqlMoney(double)` (line 105) delegates to `new SqlMoney(new decimal(value))`. The `decimal(double)` constructor in C# rejects NaN and Infinity with `OverflowException`. For finite values, it converts the double to decimal (which has ~28 significant digits), then the `SqlMoney(decimal)` constructor scales by 10,000 and range-checks.

In Rust, the equivalent approach is:
1. Check `value.is_finite()` — reject NaN/Infinity
2. Compute `scaled = value * 10_000.0`
3. Round `scaled` to nearest integer via `f64::round()` (round-half-away-from-zero)
4. Check `scaled` fits in i64 range: `scaled >= i64::MIN as f64 && scaled <= i64::MAX as f64`
5. Cast to i64: `scaled as i64`

Edge case: very large doubles may lose precision in the `* 10_000.0` step, but this is inherent to f64 and matches C# behavior (which also goes through floating-point in the `decimal(double)` constructor).

**Alternatives considered**:
- Route through string parsing: Avoids float precision issues but is much slower and over-engineered. Rejected.
- Use `f64::to_bits()` for exact conversion: Unnecessarily complex for the expected use case. Rejected.

## R7: Conversion Scope

**Decision**: Implement conversions with all currently existing types:

**Widening INTO SqlMoney (infallible, via `From`):**
- `From<SqlBoolean>` — NULL→NULL, FALSE→0.0000, TRUE→1.0000
- `From<SqlByte>` — NULL→NULL, value→scaled by 10,000
- `From<SqlInt16>` — NULL→NULL, value→scaled by 10,000
- `From<SqlInt32>` — NULL→NULL, value→scaled by 10,000 (always fits in i64)

**Widening INTO SqlMoney (fallible, via method):**
- `from_sql_int64(SqlInt64) -> Result` — range-checked (value must fit in `i64::MIN/10000..=i64::MAX/10000`)

**Narrowing OUT of SqlMoney (fallible):**
- `to_sql_int64() -> Result<SqlInt64>` — rounds then returns
- `to_sql_int32() -> Result<SqlInt32>` — rounds then range-checks i32
- `to_sql_int16() -> Result<SqlInt16>` — rounds then range-checks i16
- `to_sql_byte() -> Result<SqlByte>` — rounds then range-checks u8
- `to_sql_boolean() -> SqlBoolean` — zero→FALSE, non-zero→TRUE, NULL→NULL
- `to_sql_decimal() -> SqlDecimal` — exact conversion with scale=4

**Rationale**: C# defines implicit conversions from SqlBoolean, SqlByte, SqlInt16, SqlInt32, SqlInt64 (all via integer constructors), and explicit conversions to SqlBoolean, SqlByte, SqlInt16, SqlInt32, SqlInt64 (via integer casts with checks). SqlDecimal exists in the codebase, so `to_sql_decimal()` can be implemented. SqlSingle/SqlDouble/SqlString/SqlGuid/SqlDateTime don't exist yet — those conversions are deferred.

**Alternatives considered**:
- Defer all conversions until all types exist: Would miss the opportunity to test interop with existing types. Rejected.
- Add `From<SqlDecimal>` for SqlMoney: C# does this (`explicit operator SqlMoney(SqlDecimal x)`) — could be implemented but is a less common direction. Deferred for now to keep scope manageable.

## R8: C# Named Static Methods

**Decision**: Do NOT implement C#'s named static methods (`SqlMoney.Add()`, `SqlMoney.Subtract()`, `SqlMoney.Multiply()`, `SqlMoney.Divide()`, etc.). These are CLS-compliance wrappers redundant with operator overloading.

**Rationale**: C# has `Add`, `Subtract`, `Multiply`, `Divide`, `Equals`, `NotEquals`, `LessThan`, `GreaterThan`, `LessThanOrEqual`, `GreaterThanOrEqual` as static methods that simply delegate to operators (lines 413-479 of SQLMoney.cs). In Rust, operators and `sql_*` comparison methods are the idiomatic API. Adding static method wrappers would bloat the API with no value.

**Alternatives considered**:
- Implement all named methods: Unnecessary API bloat. Rejected per Constitution II (Idiomatic Rust).

## R9: Division Rounding and Truncation

**Decision**: Division result is truncated toward zero after the 4th decimal place, matching C# `decimal.Divide()` → `SqlMoney(decimal)` semantics.

**Rationale**: When C# computes `decimal.Divide(x.ToDecimal(), y.ToDecimal())`, the result is a `decimal` with up to 28 significant digits. This is then passed to `SqlMoney(decimal)` which calls `SqlDecimal.AdjustScale(4 - scale, true)` — the `true` parameter means "round" (not truncate) when adjusting to exactly 4 decimal places.

For i128 implementation in Rust, after computing `(dividend_128 * 10_000) / divisor_128`, we get the correctly scaled result with truncation. To match C# rounding, we check the remainder and round the 4th decimal place:
```
quotient = (dividend * 10_000) / divisor
remainder = (dividend * 10_000) % divisor
if 2 * abs(remainder) >= abs(divisor) → round away from zero
```

**Alternatives considered**:
- Always truncate: Simpler but doesn't match C# behavior for division. Rejected.
