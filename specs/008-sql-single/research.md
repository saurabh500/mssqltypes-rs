# Research: SqlSingle

**Feature**: 008-sql-single | **Date**: 2026-03-02

## R1: NaN/Infinity Rejection Strategy

**Decision**: Use `f32::is_finite()` — returns `false` for NaN, Infinity, NEG_INFINITY; `true` for all valid values including `-0.0` and subnormals.

**Rationale**: Mirrors the C# pattern where `SqlSingle` constructor calls `float.IsFinite(value)` (SQLSingle.cs L36-43). Identical approach to `SqlDouble` which uses `f64::is_finite()`. Single function call, idiomatic Rust.

**Alternatives considered**: `!v.is_nan() && !v.is_infinite()` — functionally equivalent but more verbose.

## R2: Division by Zero Detection

**Decision**: Check `divisor == 0.0` **before** computing, then **also** check result with `is_finite()`. Return `DivideByZero` for zero-divisor, `Overflow` for infinite results.

**Rationale**: C# reference does exactly this (SQLSingle.cs L137-148): pre-check `y._value == 0.0` → `DivideByZeroException`, then post-check `float.IsInfinity(value)` → `OverflowException`. Pre-check is necessary because `0.0 / 0.0 = NaN` (not Infinity), and we want `DivideByZero`, not `Overflow`.

**Alternatives considered**: Compute-then-check only — would conflate `0.0/0.0` (NaN) with overflow (Infinity).

## R3: Hash Implementation

**Decision**: Use `f32::to_bits()` for hashing, with `-0.0` normalized to `+0.0` before calling `to_bits()`.

**Rationale**: IEEE 754 says `0.0 == -0.0`, so they must hash identically per `Hash`/`Eq` contract. `f32::to_bits()` returns `u32` representation. Normalization: `if v == 0.0 { 0.0_f32.to_bits() } else { v.to_bits() }`.

**Alternatives considered**: Using `to_bits()` without normalization — violates `Hash`/`Eq` contract for `-0.0` vs `0.0`.

## R4: Eq/PartialEq Safety

**Decision**: Safe to implement `Eq` because NaN is excluded at construction and after every arithmetic operation.

**Rationale**: The only reason `f32` doesn't implement `Eq` is `NaN != NaN`. `SqlSingle` maintains the invariant that the contained `f32` is always finite, making `==` reflexive on all representable values. Two `SqlSingle::NULL` values are equal (Rust `Option::None == None`), distinct from `sql_equals()` which returns `SqlBoolean::NULL`.

**Alternatives considered**: Skipping `Eq` — would prevent `HashSet`/`HashMap` usage.

## R5: Neg Trait

**Decision**: Negation is infallible (`Output = SqlSingle`, not `Result`). NULL propagates.

**Rationale**: IEEE 754 negation flips the sign bit — cannot produce NaN or Infinity from a finite input. C# confirms (SQLSingle.cs L99-102): `return x.IsNull ? Null : new SqlSingle(-x._value)` — no exception possible.

**Alternatives considered**: Returning `Result` — rejected because negation truly cannot fail.

## R6: Operator Trait Pattern

**Decision**: Follow `SqlDouble`/`SqlMoney` pattern: `Add/Sub/Mul/Div` with `Output = Result<SqlSingle, SqlTypeError>`, 4 variants each (owned×owned, owned×ref, ref×owned, ref×ref). `Neg` with `Output = SqlSingle`, 2 variants.

**Rationale**: Established project pattern. Define `checked_add/sub/mul/div` methods first, then implement operator traits by delegation. Each checked method handles NULL propagation and validates result with `is_finite()`.

**Alternatives considered**: None — consistency with existing types is mandatory.

## R7: SqlDouble Conversion (to_sql_double)

**Decision**: `to_sql_double()` performs `f32 as f64` widening — lossless and always finite. Returns `SqlDouble::NULL` for NULL input.

**Rationale**: Every finite `f32` is exactly representable as `f64` (24-bit vs 53-bit mantissa). C# has implicit `SqlSingle → SqlDouble` conversion (SQLDouble.cs L196-199). Infallible — no error case possible.

**Alternatives considered**: None — only correct approach for a lossless widening.

## R8: From\<f32\> Trait

**Decision**: `From<f32>` panics on NaN/Infinity, matching spec FR-011. Use `new()` which returns `Result`, and `From<f32>` calls `new().expect()`.

**Rationale**: C#'s implicit `float → SqlSingle` throws on non-finite (SQLSingle.cs L72-75). Spec explicitly states panic behavior for ergonomic construction. `TryFrom<f32>` or `new()` available for fallible path.

**Alternatives considered**: Making `From<f32>` accept NaN — violates core invariant.

## R9: SqlMoney Conversion

**Decision**: Convert via `scaled_value() as f64 / 10_000.0` then narrow `as f32`. Validate result with `is_finite()`.

**Rationale**: C# does `new SqlSingle(x.ToDouble())` (SQLSingle.cs L177-179). Going through `f64` preserves more precision. The `f64 → f32` narrowing can lose precision but `SqlMoney` max value (≈9.22 × 10^14) is well within `f32::MAX` (≈3.4 × 10^38), so the result is always finite.

**Alternatives considered**: Direct `scaled_value() as f32 / 10_000.0_f32` — loses more precision than the f64 intermediate.

## R10: Display Format

**Decision**: Use Rust's default `Display` for `f32`. NULL displays as `"Null"`.

**Rationale**: Default f32 Display produces expected output: `3.14` → `"3.14"`, `0.0` → `"0"`, `1.0` → `"1"`. Negative zero displays as `"-0"`, matching C# `float.ToString()` behavior.

**Alternatives considered**: Custom formatting — rejected because default matches C# behavior.

## R11: SqlDouble → SqlSingle Narrowing (from_sql_double)

**Decision**: Provide `from_sql_double()` — narrowing conversion, `f64 as f32`, validate result with `is_finite()`. Returns `Result<SqlSingle, SqlTypeError>` (Overflow if non-finite).

**Rationale**: C# has explicit `SqlDouble → SqlSingle` conversion (SQLSingle.cs). A `f64` value outside `f32` range becomes Infinity when narrowed, which must be rejected. This conversion completes the bidirectional SqlSingle ↔ SqlDouble relationship.

**Alternatives considered**: Omitting — but spec FR-015 implies bidirectional support, and the C# reference includes it.
