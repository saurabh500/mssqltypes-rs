# Research: SqlInt64

## R1: Overflow Detection for Signed i64

**Decision**: Use Rust's `i64::checked_add()`, `checked_sub()`, `checked_mul()`, `checked_neg()` for arithmetic overflow detection. For division and remainder, check `rhs == 0` first (→ `DivideByZero`), then use `checked_div`/`checked_rem` (→ `None` means `MIN_VALUE / -1` overflow).

**Rationale**: C# uses three different overflow strategies depending on the operation:
- **Addition/Subtraction**: Sign-bit check via `SameSignLong()` — tests `(x ^ y) & 0x8000000000000000 == 0` to detect sign change. Same pattern as SqlInt32's `SameSignInt()` but for 64-bit.
- **Multiplication**: Splits each operand into high/low 32-bit halves, cross-multiplies, and checks intermediate results for overflow. This is significantly more complex than SqlInt32 (which simply widened to `long`), because there is no 128-bit type in C# to widen into. The C# code is ~40 lines of careful bit manipulation.
- **Division/Remainder**: Explicit guard `(x == long.MinValue) && (y == -1)`.

Rust's `checked_*` methods detect identical overflow conditions for all cases via compiler intrinsics. For multiplication, `i64::checked_mul()` uses intrinsics that map to the platform's widening-multiply instruction (e.g., x86's `IMUL r/m64` sets OF on overflow), producing the same results as C#'s manual split-half approach. This is the primary advantage of Rust for SqlInt64 — the most complex C# overflow detection (40 lines of split-half multiplication) becomes a single `checked_mul()` call.

**Alternatives considered**:
- Mirror C# split-half multiplication: Correct but extremely non-idiomatic in Rust; adds ~40 lines of unnecessary complexity. Rejected per Constitution II (Idiomatic Rust).
- Use `i128` widening for multiplication: Works (`(a as i128) * (b as i128)` then range-check), but `checked_mul` is simpler and equally correct. Rejected as unnecessary.

## R2: Negation of MIN_VALUE

**Decision**: Return `Err(SqlTypeError::Overflow)` when negating `i64::MIN` (-9,223,372,036,854,775,808). Use `i64::checked_neg()`.

**Rationale**: C#'s `operator -(SqlInt64 x)` (line 92 of SQLInt64.cs) simply does `new SqlInt64(-x.m_value)` with **no overflow check**. This is the same bug as SqlInt32 — `-long.MinValue` silently wraps back to `long.MinValue` in C#'s unchecked context. This is inconsistent with all other arithmetic operators in the same class, which explicitly check for overflow. The spec explicitly requires overflow detection for negation. Rust's `checked_neg()` correctly returns `None` for `i64::MIN`. Per Constitution I, we follow the principled behavior (detect overflow), not the C# bug.

**Alternatives considered**:
- Match C# wraparound: Violates the spec's edge case requirement and is mathematically incorrect. Rejected.

## R3: Remainder MIN_VALUE % -1

**Decision**: Return `Err(SqlTypeError::Overflow)` for `i64::MIN % -1`. Check `rhs == 0` first for `DivideByZero`, then use `i64::checked_rem()` — if it returns `None`, return `Overflow`.

**Rationale**: C# explicitly checks `(x.m_value == long.MinValue) && (y.m_value == -1)` and throws `OverflowException` (line 211 of SQLInt64.cs). Rust's `i64::checked_rem(-1)` also returns `None` for `i64::MIN`, giving us the same behavior for free. Mathematically the remainder is 0, but the hardware division instruction traps on this case, and C# treats it as overflow.

**Alternatives considered**:
- Return `Ok(SqlInt64::new(0))`: Mathematically correct but diverges from C# and would be inconsistent with division (which overflows for the same inputs). Rejected.

## R4: Available Conversions (Scoped to Existing Types)

**Decision**: Implement four conversions with existing types:
1. `From<SqlBoolean> for SqlInt64` — NULL→NULL, FALSE→0, TRUE→1 (per C# `explicit operator SqlInt64(SqlBoolean x)`)
2. `SqlInt64::to_sql_int32() -> Result<SqlInt32, SqlTypeError>` — narrowing, overflow if value < i32::MIN or > i32::MAX
3. `SqlInt64::to_sql_int16() -> Result<SqlInt16, SqlTypeError>` — narrowing, overflow if value < i16::MIN or > i16::MAX
4. `SqlInt64::to_sql_byte() -> Result<SqlByte, SqlTypeError>` — narrowing, overflow if value < 0 or > 255

C# also defines `implicit operator SqlInt64(SqlByte x)`, `implicit operator SqlInt64(SqlInt16 x)`, and `implicit operator SqlInt64(SqlInt32 x)` — these are widening conversions FROM smaller types. Per the established pattern (SqlInt32 deferred `From<SqlByte>` and `From<SqlInt16>`), these widening-in conversions are deferred. Widening conversions TO `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney` are deferred until those types exist.

**Rationale**: Only `SqlBoolean`, `SqlByte`, `SqlInt16`, and `SqlInt32` exist in the codebase. Following SqlInt32's precedent, we implement narrowing-out conversions and widening-from-SqlBoolean, deferring widening-from-smaller-integers.

**Alternatives considered**:
- Add `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>`: Matches C# exactly but deferred per established pattern. Will be added in follow-up.

## R5: Bitwise Operations — No Casting Needed in Rust

**Decision**: Use `a & b`, `a | b`, `a ^ b` directly on `i64` values. No casting or widening needed.

**Rationale**: C# bitwise operators on `long` work directly — `x.m_value & y.m_value` with no casting. Rust's `i64 & i64` returns `i64` directly with correct two's complement semantics. No special handling needed. Identical to SqlInt32 pattern.

**Alternatives considered**: None.

## R6: Hash and Equality Semantics

**Decision**: NULL hashes as `0i64` (matching C# and SqlInt32 pattern). `PartialEq` / `Eq` compare `Option<i64>` directly (two NULLs are equal for Rust purposes). This is distinct from SQL semantics where NULL ≠ NULL.

**Rationale**: C# `GetHashCode()` returns `IsNull ? 0 : Value.GetHashCode()` (line 577). C# `Equals(object)` treats two Nulls as equal (line 572). This matches the SqlInt32 implementation pattern already established.

**Alternatives considered**: None — must match both C# behavior and existing Rust pattern.

## R7: C# Named Static Methods (Add, Subtract, etc.)

**Decision**: Do NOT implement C#'s named static methods (`SqlInt64.Add()`, `SqlInt64.Subtract()`, etc.). These are CLS-compliance wrappers redundant with operator overloading.

**Rationale**: C# has `Add`, `Subtract`, `Multiply`, `Divide`, `Mod`, `Modulus`, `BitwiseAnd`, `BitwiseOr`, `Xor`, `OnesComplement`, `Equals`, `NotEquals`, `LessThan`, `GreaterThan`, `LessThanOrEqual`, `GreaterThanOrEqual` as static methods that simply delegate to operators. Identical to SqlInt32. In Rust, operators are the idiomatic API. Adding static method wrappers would bloat the API with no value.

**Alternatives considered**:
- Implement all named methods: Unnecessary API bloat. Rejected per Constitution II (Idiomatic Rust).

## R8: Multiplication — C# Split-Half vs Rust checked_mul

**Decision**: Use `i64::checked_mul()` instead of C#'s split-half multiplication approach.

**Rationale**: C#'s SqlInt64 multiplication (lines 130-188 of SQLInt64.cs) is the most complex arithmetic operation in the type. Since C# has no 128-bit integer, it manually:
1. Takes absolute values of both operands
2. Splits each into low-32 and high-32 bit words
3. If both high words are non-zero → overflow
4. Cross-multiplies high×low and low×low
5. Checks intermediate results for overflow at each step
6. Combines results and re-applies sign

This is ~40 lines of careful bit manipulation. In Rust, `i64::checked_mul()` achieves identical overflow detection in a single call, using the compiler's intrinsic (which typically maps to the CPU's widening multiply instruction). The behavioral result is identical — overflow is detected for all the same inputs.

Note: C#'s approach has a subtle edge case — negating `long.MinValue` at lines 144/150 before the split would overflow, but since both high words would be non-zero in that case, the overflow is caught at line 163. Rust's `checked_mul` handles this correctly without the manual workaround.

**Alternatives considered**:
- Mirror C# split-half approach: Correct but 40 lines vs 1 line, non-idiomatic. Rejected.
- Widen to `i128`: `(a as i128) * (b as i128)` then range-check. Works but more complex than `checked_mul`. Rejected.
