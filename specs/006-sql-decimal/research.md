# Research: SqlDecimal

## R1: Internal Representation — 4×u32 vs u128

**Decision**: Store the mantissa as `[u32; 4]` (four `u32` components in little-endian order, `data[0]` = least significant), with a separate `bool` sign flag, `u8` precision (1–38), and `u8` scale (0–precision). Do NOT use Rust's native `u128`.

**Rationale**: C# SQLDecimal.cs stores the mantissa as four separate `uint` fields (`_data1`–`_data4`) with `_bLen` tracking the number of active components (1–4). Although Rust has a native `u128` type, using `[u32; 4]` provides:
1. **Behavioral fidelity**: The multi-precision arithmetic algorithms (add, subtract, multiply, divide) operate on individual `u32` words with carry propagation via `u64`. Matching the C# layout ensures identical intermediate results and overflow detection.
2. **Precision calculation**: C# uses lookup tables (`DecimalHelpersLo/Mid/Hi/HiHi`) indexed by `u32` word to efficiently calculate the number of decimal digits. These tables assume a 4×u32 decomposition.
3. **Data interchange**: The `data()` accessor returns the four components directly, matching the C# `Data` property (returns `int[4]`).

The `_bLen` field from C# (number of active u32s) can be derived from the array rather than stored explicitly — scan from the most significant word down to find the first non-zero. This simplifies the struct.

**Alternatives considered**:
- Use Rust `u128`: Simpler arithmetic but breaks fidelity with C# multi-precision algorithms, makes precision calculation harder, and loses the `Data` property compatibility. Rejected per Constitution I.
- Use `Vec<u32>`: Unnecessary heap allocation for a fixed 4-element array. Rejected per Constitution performance constraints.

**Struct layout** (~20 bytes):
```rust
struct InnerDecimal {
    precision: u8,      // 1–38
    scale: u8,          // 0–precision
    positive: bool,     // true = positive, false = negative
    data: [u32; 4],     // 128-bit unsigned mantissa, little-endian
}

pub struct SqlDecimal {
    inner: Option<InnerDecimal>,  // None = SQL NULL
}
```

## R2: Multi-Precision Arithmetic Approach

**Decision**: Implement schoolbook (O(n²)) multi-precision arithmetic for addition, subtraction, and multiplication on `[u32; 4]` arrays using `u64` for intermediate results and carry propagation.

**Rationale**: C# SQLDecimal.cs implements:
- **Addition**: Word-by-word addition with carry using `ulong` intermediate (`dwlAccum`). For different signs, uses subtraction with complement approach (lines 1232–1370).
- **Subtraction**: Implemented as `x + (-y)` (line 1373).
- **Multiplication**: Schoolbook O(n²) — iterates multiplier words, for each multiplies against all multiplicand words, accumulating into a 9-element result buffer. Uses `ulong` for intermediate products (lines 1486–1529).

In Rust, the same approach maps directly:
- `u64` widening: `(a as u64) * (b as u64)` for carry-free multiplication of two `u32` values
- Carry propagation: `result[i] = (accum & 0xFFFF_FFFF) as u32; carry = accum >> 32;`
- The 9-element intermediate buffer for multiplication handles the maximum product of two 4-word numbers (up to 8 words, plus 1 for overflow detection)

**Alternatives considered**:
- Use `u128` for intermediate multiplication: Rust supports `u128` natively, so `(a as u128) * (b as u128)` could be used for pairs. However, the schoolbook algorithm still needs to iterate words for the full 4×4 multiplication. Using `u128` for intermediate values adds complexity without benefit.

## R3: Division Algorithm

**Decision**: Implement division using Knuth's Algorithm D (The Art of Computer Programming, Vol. 2, pg 272) for multi-digit division, with a fast path for single-word divisors using simple long division.

**Rationale**: C# SQLDecimal.cs uses:
- `MpDiv1` (lines 2530–2562): Fast path for dividing by a single `uint`. Simple long division using `ulong` intermediate.
- `MpDiv` (lines 2585–2653): Full multi-digit division using Knuth's Algorithm D. Includes normalization, trial quotient estimation, correction step, and remainder extraction.

The division operator (lines 1735–1749) first scales the dividend by `AdjustScale` to achieve the target result scale, then calls `MpDiv` for the actual division.

**Alternatives considered**:
- Use f64 approximation for division: Loses precision for large values. Rejected.
- Convert to u128 and use native division: Only works when both operands fit in u128, doesn't handle the full 4×u32 × scale-factor case. Rejected.

## R4: Precision/Scale Propagation Rules

**Decision**: Follow C# SQLDecimal.cs formulas exactly for result precision and scale.

### Addition/Subtraction (C# lines 1232–1269):
```
ResInteger = max(p1 - s1, p2 - s2)
ResScale   = max(s1, s2)
ResPrec    = ResInteger + ResScale + 1     // +1 for carry
ResPrec    = min(38, ResPrec)

// If capped, reduce scale to preserve integer part:
if (ResPrec - ResInteger < ResScale)
    ResScale = ResPrec - ResInteger
```

### Multiplication (C# lines 1451–1478):
```
ActualScale = s1 + s2
ResInteger  = (p1 - s1) + (p2 - s2) + 1
ResPrec     = ActualScale + ResInteger

if (ResPrec > 38) ResPrec = 38
if (ActualScale > 38) ActualScale = 38

ResScale    = min(ResPrec - ResInteger, ActualScale)
ResScale    = max(ResScale, min(ActualScale, 6))   // min scale = 6

ScaleAdjust = ResScale - ActualScale
```

### Division (C# lines 1663–1725):
```
ResScale   = max(s1 + p2 + 1, 6)          // 6 = minimum division scale
ResInteger = (p1 - s1) + s2
ResPrec    = ResScale + p1 + p2 + 1
MinScale   = min(ResScale, 6)

ResInteger = min(ResInteger, 38)
ResPrec    = ResInteger + ResScale
if (ResPrec > 38) ResPrec = 38

ResScale   = min(ResPrec - ResInteger, ResScale)
ResScale   = max(ResScale, MinScale)

ScaleAdjust = ResScale - s1 + s2
```

**Rationale**: These formulas ensure SQL Server compatibility. The minimum division scale of 6 (`s_cNumeDivScaleMin`) prevents loss of fractional digits in common division operations.

**Alternatives considered**: None — behavioral fidelity requires following these exact formulas.

## R5: Rounding Mode

**Decision**: Use round-half-up (away from zero in magnitude) for `AdjustScale` and arithmetic truncation.

**Rationale**: C# SQLDecimal.cs (lines 2271–2362) uses:
```csharp
fNeedRound = (ulRem >= ulShiftBase / 2);
if (fNeedRound && fRound)
    AddULong(1);
```

Since `AddULong(1)` always increases the absolute (unsigned) mantissa value, and the sign is stored separately, this is round-half-away-from-zero in magnitude. For powers of 10 as the divisor, `ulShiftBase / 2` is exact (e.g., 10/2=5), so remainder ≥ 5 rounds up.

The `fRound` parameter distinguishes between rounding (`adjust_scale(_, true)`) and truncation (`adjust_scale(_, false)`).

**Alternatives considered**:
- Round-half-to-even (banker's rounding): Not used by C# SqlDecimal. Rejected per Constitution I.

## R6: Copy vs Clone Semantics

**Decision**: Implement `Clone` but NOT `Copy` for `SqlDecimal`.

**Rationale**: The Constitution's Performance section classifies SqlDecimal as "heap-allocated only for variable-size types (SqlBinary, SqlString, SqlDecimal)" — explicitly listing SqlDecimal in the non-Copy category. While the struct is fixed-size (~20 bytes) and _could_ technically be Copy, the Constitution mandates otherwise. The C# struct is 20 bytes and is value-typed, but the Rust API design follows the Constitution classification.

Note: Since the inner representation uses `Option<InnerDecimal>` where `InnerDecimal` contains only fixed-size fields (`u8`, `bool`, `[u32; 4]`), `InnerDecimal` itself is `Copy`-eligible. But wrapping in `Option` and the Constitution's explicit classification means `SqlDecimal` as a whole should be `Clone` only.

**Alternatives considered**:
- Implement `Copy`: Constitution explicitly lists SqlDecimal in the heap-allocated category. Rejected per Constitution compliance.

## R7: Negative Zero Normalization

**Decision**: Normalize negative zero to positive zero in all constructors and arithmetic operations. When the mantissa is zero, the sign flag MUST be `true` (positive).

**Rationale**: C# SQLDecimal.cs normalizes negative zero in every constructor and operation:
```csharp
if (FZero()) SetPositive();
```

This appears in at least 12 locations (constructors from int/long/double/bytes, arithmetic results, AdjustScale, Parse). `FZero()` checks `_data1 == 0 && _bLen <= 1`. In our Rust implementation, `FZero()` equivalent checks `data == [0, 0, 0, 0]`.

**Alternatives considered**: None — required for behavioral fidelity and correct comparison semantics.

## R8: Power Implementation

**Decision**: Implement `power(exponent)` by converting `SqlDecimal` to `f64`, using `f64::powi()`, then converting back to `SqlDecimal`. Set result precision to `MAX_PRECISION` (38) and scale to the original operand's scale.

**Rationale**: C# SQLDecimal.cs `Power(n, exp)` (lines 3256–3270) does exactly this:
```csharp
double x = n.ToDouble();
x = Math.Pow(x, exp);
SqlDecimal ret = new SqlDecimal(x);
ret.SetPrecision(MaxPrecision);
ret.AdjustScale(n.m_bScale - ret.m_bScale, true);
```

This loses precision for very large values but matches C# behavior exactly. The C# approach is pragmatic — implementing arbitrary-precision exponentiation would be extremely complex with diminishing returns.

**Alternatives considered**:
- Implement integer exponentiation via repeated multiplication: More precise for integer exponents but diverges from C# behavior. Could be added as an optimization for small exponents while falling back to f64 for large ones. Rejected for initial implementation — behavioral fidelity first.

## R9: Available Conversions (Scoped to Existing Types)

**Decision**: Implement conversions with existing types only:

**From other types to SqlDecimal** (widening/infallible):
1. `From<i32> for SqlDecimal` — precision 10, scale 0
2. `From<i64> for SqlDecimal` — precision 19, scale 0
3. `From<SqlBoolean> for SqlDecimal` — NULL→NULL, FALSE→0, TRUE→1
4. `From<SqlByte> for SqlDecimal` — precision 3, scale 0
5. `From<SqlInt16> for SqlDecimal` — precision 5, scale 0
6. `From<SqlInt32> for SqlDecimal` — precision 10, scale 0
7. `From<SqlInt64> for SqlDecimal` — precision 19, scale 0

**From SqlDecimal to other types** (narrowing/fallible):
1. `SqlDecimal::to_f64()` — lossy conversion to closest double
2. `SqlDecimal::to_sql_int32() -> Result` — truncate fractional, range check
3. `SqlDecimal::to_sql_int64() -> Result` — truncate fractional, range check
4. `SqlDecimal::to_sql_int16() -> Result` — truncate fractional, range check
5. `SqlDecimal::to_sql_byte() -> Result` — truncate fractional, range check
6. `SqlDecimal::to_sql_boolean() -> SqlBoolean` — 0→FALSE, non-zero→TRUE, NULL→NULL

**Rationale**: Only `SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, and `SqlInt64` exist. Conversions to/from `SqlSingle`, `SqlDouble`, `SqlMoney`, `SqlString` deferred until those types are implemented.

**Alternatives considered**: None — scoping to existing types follows the established pattern.

## R10: Precision Calculation

**Decision**: Implement `calculate_precision()` using iterative division by 10, counting digits. Optionally use lookup tables for O(1) bounds checking.

**Rationale**: C# uses lookup tables (`DecimalHelpersLo/Mid/Hi/HiHi`) containing powers of 10 decomposed by u32 word for O(1) precision calculation. The Rust implementation can start with a simpler iterative approach (divide mantissa by 10, counting digits) and optimize with lookup tables later if performance is critical.

The C# `CalculatePrecision()` (lines 1848–1889) estimates the precision from `_bLen` and then checks against the stored power-of-10 table. The `RgCLenFromPrec` array maps precision to the required number of u32 elements:
- Precision 1–9: 1 u32
- Precision 10–19: 2 u32s
- Precision 20–28: 3 u32s
- Precision 29–38: 4 u32s

**Alternatives considered**:
- Direct `u128` conversion and `ilog10`: Simpler but breaks the 4×u32 representation principle. Rejected.

## R11: C# Named Static Methods

**Decision**: Do NOT implement C#'s named static methods (`Add`, `Subtract`, `Multiply`, `Divide`, `Mod`, etc.). These are CLS-compliance wrappers redundant with operator overloading.

**Rationale**: Same pattern as SqlInt32/SqlInt64 — in Rust, operators are the idiomatic API. Mathematical functions (`Abs`, `Floor`, `Ceiling`, `Round`, `Truncate`, `Sign`, `Power`) are implemented as static/associated functions or methods since they don't map to Rust operators.

**Alternatives considered**: None — follows established pattern.

## R12: Remainder (Modulus) Operation

**Decision**: Implement remainder as `a - (a / b).truncate() * b`, using the quotient truncated to integer. Division by zero returns `Err(DivideByZero)`.

**Rationale**: C# SQLDecimal.cs `operator %` (lines 1753–1780) computes remainder by:
1. Dividing `x / y` to get quotient
2. Truncating quotient to integer via scale adjustment
3. Computing `x - truncated_quotient * y`

This ensures `a == (a / b) * b + (a % b)` holds.

**Alternatives considered**: None — must match C# behavior.
