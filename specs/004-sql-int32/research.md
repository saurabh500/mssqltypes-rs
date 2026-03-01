# Research: SqlInt32

## R1: Overflow Detection for Signed i32

**Decision**: Use Rust's `i32::checked_add()`, `checked_sub()`, `checked_mul()`, `checked_neg()` for arithmetic overflow detection. For division and remainder, check `rhs == 0` first (→ `DivideByZero`), then use `checked_div`/`checked_rem` (→ `None` means `MIN_VALUE / -1` overflow).

**Rationale**: C# uses three different overflow strategies depending on the operation:
- **Addition/Subtraction**: Sign-bit check via `SameSignInt()` — tests `(x ^ y) & 0x80000000 == 0` to detect sign change.
- **Multiplication**: Widens to `long`, multiplies, then checks if upper bits fit via `result & ~(long)int.MaxValue` bitmask.
- **Division/Remainder**: Explicit guard `(x == s_iIntMin) && (y == -1)`.

Rust's `checked_*` methods detect identical overflow conditions for all cases via compiler intrinsics. They are idiomatic, bug-free, and produce the same results without needing sign-bit manipulation or widening. This follows the same pattern established by SqlByte and SqlInt16.

**Alternatives considered**:
- Mirror C# sign-bit patterns: Correct but non-idiomatic in Rust; adds unnecessary complexity. Rejected per Constitution II (Idiomatic Rust).
- Widen to i64 for multiplication: Works but `checked_mul` is simpler and equally correct.

## R2: Negation of MIN_VALUE

**Decision**: Return `Err(SqlTypeError::Overflow)` when negating `i32::MIN` (-2,147,483,648). Use `i32::checked_neg()`.

**Rationale**: C#'s `operator -(SqlInt32 x)` simply does `new SqlInt32(-x.m_value)` with **no overflow check**. This is a confirmed deficiency — `-int.MinValue` silently wraps back to `int.MinValue` in C#'s unchecked context. This is inconsistent with all other arithmetic operators in the same class, which explicitly check for overflow. The spec explicitly requires overflow detection for negation. Rust's `checked_neg()` correctly returns `None` for `i32::MIN`. Per Constitution I, we follow the principled behavior (detect overflow), not the C# bug.

**Alternatives considered**:
- Match C# wraparound: Violates the spec's edge case requirement and is mathematically incorrect. Rejected.

## R3: Remainder MIN_VALUE % -1

**Decision**: Return `Err(SqlTypeError::Overflow)` for `i32::MIN % -1`. Check `rhs == 0` first for `DivideByZero`, then use `i32::checked_rem()` — if it returns `None`, return `Overflow`.

**Rationale**: C# explicitly checks `(x.m_value == s_iIntMin) && (y.m_value == -1)` and throws `OverflowException`. Rust's `i32::checked_rem(-1)` also returns `None` for `i32::MIN`, giving us the same behavior for free. Mathematically the remainder is 0, but the hardware division instruction traps on this case, and C# treats it as overflow.

**Alternatives considered**:
- Return `Ok(SqlInt32::new(0))`: Mathematically correct but diverges from C# and would be inconsistent with division (which overflows for the same inputs). Rejected.

## R4: Available Conversions (Scoped to Existing Types)

**Decision**: Implement three conversions with existing types:
1. `From<SqlBoolean> for SqlInt32` — NULL→NULL, FALSE→0, TRUE→1 (per C# `explicit operator SqlInt32(SqlBoolean x)`)
2. `SqlInt32::to_sql_int16() -> Result<SqlInt16, SqlTypeError>` — narrowing, overflow if value < -32768 or > 32767
3. `SqlInt32::to_sql_byte() -> Result<SqlByte, SqlTypeError>` — narrowing, overflow if value < 0 or > 255

Widening conversions FROM `SqlByte`/`SqlInt16` and narrowing `to_sql_boolean()` are explicitly deferred per clarification session (2026-03-01). Widening conversions TO `SqlInt64`, `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney` are deferred until those types exist.

**Rationale**: Only `SqlBoolean`, `SqlByte`, and `SqlInt16` exist in the codebase. The user explicitly chose to defer `From<SqlByte>`, `From<SqlInt16>`, and `to_sql_boolean()` in the clarification session. The three conversions above match the minimum scope defined in the spec's FR-007 and FR-008.

**Alternatives considered**:
- Add `From<SqlByte>`, `From<SqlInt16>`, `to_sql_boolean()`: Matches C# exactly and follows SqlInt16's pattern, but explicitly deferred by user. Will be added in follow-up.

## R5: OnesComplement (Bitwise NOT)

**Decision**: Implement `Not` trait returning `SqlInt32` (not `Result`). NULL → NULL, otherwise `!value`. Provide `ones_complement()` method as named alternative.

**Rationale**: C#'s `~` operator on `SqlInt32` returns `SqlInt32` directly (no exception). Bitwise NOT on any `i32` always produces a valid `i32`. `!i32::MIN == i32::MAX` and vice versa, both valid. Matches SqlInt16 pattern exactly.

**Alternatives considered**: None — this is the only correct approach.

## R6: Bitwise Operations — No Casting Needed in Rust

**Decision**: Use `a & b`, `a | b`, `a ^ b` directly on `i32` values. No casting or widening needed.

**Rationale**: C# bitwise operators on `int` don't require the `ushort` casting that `SqlInt16` does (since `int` is already the native arithmetic width in C#). In Rust, `i32 & i32` returns `i32` directly with correct two's complement semantics. No special handling needed.

**Alternatives considered**: None.

## R7: Hash and Equality Semantics

**Decision**: NULL hashes as `0i32` (matching SqlInt16 pattern). `PartialEq` / `Eq` compare `Option<i32>` directly (two NULLs are equal for Rust purposes). This is distinct from SQL semantics where NULL ≠ NULL.

**Rationale**: C# `GetHashCode()` returns 0 for Null. C# `Equals(object)` treats two Nulls as equal (returns `true`). This matches the SqlInt16 implementation pattern already established.

**Alternatives considered**: None — must match both C# behavior and existing Rust pattern.

## R8: C# Named Static Methods (Add, Subtract, etc.)

**Decision**: Do NOT implement C#'s named static methods (`SqlInt32.Add()`, `SqlInt32.Subtract()`, etc.). These are CLS-compliance wrappers redundant with operator overloading.

**Rationale**: C# has `Add`, `Subtract`, `Multiply`, `Divide`, `Mod`, `Modulus`, `BitwiseAnd`, `BitwiseOr`, `Xor`, `OnesComplement`, `Equals`, `NotEquals`, `LessThan`, `GreaterThan`, `LessThanOrEqual`, `GreaterThanOrEqual` as static methods that simply delegate to operators. In Rust, operators are the idiomatic API. Adding static method wrappers would bloat the API with no value. Note: C# even has both `Mod()` and `Modulus()` as duplicates — no reason to replicate this.

**Alternatives considered**:
- Implement all named methods: Unnecessary API bloat. Rejected per Constitution II (Idiomatic Rust).
