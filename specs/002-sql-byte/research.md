# Research: SqlByte

**Feature**: 002-sql-byte | **Date**: 2026-03-01

## R1: Internal Representation

**Decision**: Use `Option<u8>` (None = NULL).

**Rationale**: C# uses `bool m_fNotNull` + `byte m_value`. Rust's `Option<u8>` is semantically identical, idiomatic, and compiler-optimized (2 bytes). Matches the project's established pattern — `SqlBoolean` uses an internal `u8` state, but for numeric types `Option<T>` is the documented pattern from the constitution and copilot-instructions.

**Alternatives considered**: Two-field struct — rejected as non-idiomatic.

## R2: Arithmetic Overflow Detection

**Decision**: Widen operands to `i32`, compute, check `(result & !0xFF) != 0`. Return `Err(SqlTypeError::Overflow)` on failure.

**Rationale**: Exactly mirrors C#'s `s_iBitNotByteMax = ~0xff` bitmask technique. Widening to `i32` catches both underflow (negative results from subtraction) and overflow (>255). The bitmask is a single bitwise AND.

**Alternatives considered**: `u8::checked_add()` — can't detect subtraction underflow cleanly since `u8` can't represent negative intermediaries.

## R3: Operator Trait Output Types

**Decision**: Arithmetic traits (`Add`, `Sub`, `Mul`, `Div`, `Rem`) use `type Output = Result<SqlByte, SqlTypeError>`. NULL operands return `Ok(SqlByte::NULL)`.

**Rationale**: Spec FR-002 explicitly requires `Result<SqlByte, SqlTypeError>`. C# throws exceptions on overflow — Rust returns `Result`. Cannot panic (Constitution §III). Bitwise ops (`BitAnd`, `BitOr`, `BitXor`, `Not`) return `SqlByte` directly since they never fail.

**Alternatives considered**: `Output = SqlByte` with panic — violates constitution.

## R4: Comparison Return Types

**Decision**: SQL comparison methods (`sql_equals`, `sql_less_than`, etc.) return `SqlBoolean`. Standard Rust traits (`PartialEq`, `Ord`) also implemented for collection/sorting support.

**Rationale**: Matches C# which has both SQL operators (returning `SqlBoolean`) and `IEquatable`/`IComparable` (returning `bool`/`int`). For `PartialEq`: two NULLs are equal (matches C# `Equals`). For `Ord`: NULL < any non-null (matches C# `CompareTo`).

## R5: Type Conversions Scope

**Decision**: Implement only conversions to/from types that exist today: `From<u8>`, `From<SqlBoolean>` → `SqlByte`, and `to_sql_boolean()`. Widening conversions to SqlInt16/32/64 etc. will be added when those types land.

**Rationale**: C# defines widening conversions in the target type's file. Since those types don't exist yet, we only implement what's testable now.

## R6: Display/FromStr Behavior

**Decision**: Display: NULL → `"Null"`, non-null → decimal string. Parse: `"Null"` (case-insensitive) → NULL, valid u8 string → `SqlByte`, else `Err(ParseError)`.

**Rationale**: Matches C#'s `ToString()` (uses `SQLResource.NullString`) and `Parse()` (delegates to `byte.Parse`).
