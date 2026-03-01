# Research: SqlInt16

## R1: Overflow Detection for Signed i16

**Decision**: Use Rust's `i16::checked_add()`, `checked_sub()`, `checked_mul()`, `checked_neg()` for arithmetic overflow detection. For division and remainder, check `rhs == 0` first (→ `DivideByZero`), then use `checked_div`/`checked_rem` (→ `None` means `MIN_VALUE / -1` overflow).

**Rationale**: The C# implementation widens to `int` and uses bit-shift checks (`(iResult >> 15) ^ (iResult >> 16) & 1`). Rust's `checked_*` methods detect the identical overflow conditions via compiler intrinsics. They are idiomatic, bug-free, and produce the same results. This differs from SqlByte which mirrored the C# unsigned bitmask verbatim, but for signed types `checked_*` is cleaner and avoids replicating the more complex signed bit logic.

**Alternatives considered**:
- Widen to i32 + range check: Works but adds unnecessary verbosity.
- Mirror C# bit-shift pattern: Correct but non-idiomatic in Rust; `checked_*` is preferred per Constitution II (Idiomatic Rust).

## R2: Negation of MIN_VALUE

**Decision**: Return `Err(SqlTypeError::Overflow)` when negating `i16::MIN` (-32768). Use `i16::checked_neg()`.

**Rationale**: C# `SqlInt16.operator -(SqlInt16 x)` does `(short)-x.m_value` which silently wraps: `-(-32768)` overflows the `int` promotion and `(short)32768` wraps to `-32768`. This is a deficiency in C# — it returns the wrong value without signaling an error. The spec explicitly requires overflow detection. Rust's `checked_neg()` correctly returns `None` for `i16::MIN`. Per Constitution I, we follow SQL Server semantics (which should detect overflow), not the C# bug.

**Alternatives considered**:
- Match C# wraparound: Violates the spec's edge case requirement and Constitution principle of overflow detection. Rejected.

## R3: Remainder MIN_VALUE % -1

**Decision**: Return `Err(SqlTypeError::Overflow)` for `i16::MIN % -1`, matching C# behavior. Check `rhs == 0` first for `DivideByZero`, then use `i16::checked_rem()` — if it returns `None`, return `Overflow`.

**Rationale**: C# explicitly checks `(x == short.MinValue) && (y == -1)` and throws `OverflowException` in the `%` operator. Mathematically the remainder is 0, but the hardware division instruction traps on this case. Rust's `i16::checked_rem(-1)` also returns `None` for `i16::MIN`, so we get the same behavior for free.

**Alternatives considered**:
- Return `Ok(SqlInt16::new(0))`: Mathematically correct but diverges from C# and would behave inconsistently with division (which overflows for the same inputs).

## R4: Available Conversions

**Decision**: Implement four conversions with existing types:
1. `From<SqlBoolean> for SqlInt16` — NULL→NULL, FALSE→0, TRUE→1
2. `From<SqlByte> for SqlInt16` — widening, always safe (u8 → i16 fits)
3. `SqlInt16::to_sql_boolean()` — NULL→NULL, 0→FALSE, nonzero→TRUE
4. `SqlInt16::to_sql_byte() -> Result<SqlByte, SqlTypeError>` — narrowing, overflow if value < 0 or > 255

Widening conversions to SqlInt32, SqlInt64, SqlSingle, SqlDouble, SqlDecimal, SqlMoney: deferred until those types exist.

**Rationale**: Only `SqlBoolean` and `SqlByte` exist in the codebase. Implementing conversions to non-existent types would create compilation errors. The four listed conversions match the C# reference exactly for the available types.

**Alternatives considered**:
- Defer all conversions: Loses useful functionality for the two types we have. Rejected.
- Implement placeholder traits: Over-engineering without benefit.

## R5: OnesComplement (bitwise NOT)

**Decision**: Implement `Not` trait returning `SqlInt16` (not `Result`). NULL → NULL, otherwise `!value`. No overflow possible.

**Rationale**: C#'s `~` operator on `SqlInt16` returns `SqlInt16` directly (not wrapped in exception). Bitwise NOT on any `i16` always produces a valid `i16`. `!i16::MIN = i16::MAX` and vice versa, both valid.

**Alternatives considered**: None — this is the only correct approach.

## R6: Bitwise Operations — No Casting Needed in Rust

**Decision**: Use `a & b`, `a | b`, `a ^ b` directly on `i16` values. No casting or widening needed.

**Rationale**: C#'s `BitOr` uses `unchecked((short)((ushort)x | (ushort)y))` to work around C#'s implicit `int` promotion of `short` arithmetic. Rust has no such promotion — `i16 | i16` returns `i16` directly with correct two's complement semantics. The `ushort` dance is a C#-specific workaround irrelevant to Rust.

**Alternatives considered**:
- Cast to `u16` and back: Unnecessary in Rust; would add confusion.
