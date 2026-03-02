# Research: SqlString

## R1: Internal Representation — Simplified from C#

**Decision**: Store `Option<String>` for the value and `SqlCompareOptions` (a simple Rust enum) for comparison behavior. No LCID field, no `CompareInfo` field.

**Rationale**: C# `SqlString` has 5 fields: `m_value` (string?), `m_cmpInfo` (CompareInfo?), `m_lcid` (int), `m_flag` (SqlCompareOptions), `m_fNotNull` (bool). The `m_lcid` and `m_cmpInfo` fields support culture-aware comparisons via .NET's globalization infrastructure, which has no Rust equivalent without heavy ICU dependencies. Since the Constitution mandates no external dependencies (Principle VI), we drop locale support entirely. The `m_fNotNull` field is replaced by `Option` — `None` = NULL, `Some(_)` = not null.

**Alternatives considered**:
- Store LCID as `u32` for future use: Adds unused complexity. Rejected per YAGNI and Constitution II (Idiomatic Rust).
- Use `icu4x` crate for locale support: Violates Constitution VI (no external dependencies). Rejected.

## R2: SqlCompareOptions — Simple Enum, Not Bitflags

**Decision**: Define `SqlCompareOptions` as a simple Rust enum with 4 variants: `None`, `IgnoreCase`, `BinarySort`, `BinarySort2`. Not a bitflags type.

**Rationale**: C#'s `SqlCompareOptions` is a `[Flags]` enum with 7 values: `None=0`, `IgnoreCase=1`, `IgnoreNonSpace=2`, `IgnoreKanaType=8`, `IgnoreWidth=16`, `BinarySort=0x8000`, `BinarySort2=0x4000`. The flags are combinable — C#'s default is `IgnoreCase | IgnoreKanaType | IgnoreWidth`. However, `IgnoreNonSpace`, `IgnoreKanaType`, and `IgnoreWidth` only make sense in culture-aware (CompareInfo-based) comparisons which we're dropping (R1). With those removed, the remaining values (`None`, `IgnoreCase`, `BinarySort`, `BinarySort2`) are mutually exclusive in practice — you wouldn't combine `BinarySort` with `IgnoreCase`. A simple enum is cleaner than bitflags for 4 exclusive variants.

**Alternatives considered**:
- Use `bitflags` crate: External dependency, violates Constitution VI. Rejected.
- Manual bitflag implementation: Over-engineers for 4 mutually exclusive variants. Rejected.
- Include all 7 C# variants: `IgnoreNonSpace`, `IgnoreKanaType`, `IgnoreWidth` are meaningless without locale. Rejected.

## R3: Default Compare Options — IgnoreCase

**Decision**: Default compare options are `SqlCompareOptions::IgnoreCase`. The `new()` constructor uses this default.

**Rationale**: C# defaults to `IgnoreCase | IgnoreKanaType | IgnoreWidth`, but since we drop `IgnoreKanaType` and `IgnoreWidth` (R2), the effective default is just `IgnoreCase`. This matches the spec's requirement (FR-003, Edge Cases section).

**Alternatives considered**:
- Default to `None` (case-sensitive): Diverges from C# behavior. Rejected per Constitution I (Behavioral Fidelity).

## R4: Comparison Logic — Three Paths

**Decision**: Implement comparison based on `SqlCompareOptions` variant:
1. **`None`** (case-sensitive ordinal): Direct `str::cmp()` on trimmed trailing spaces
2. **`IgnoreCase`**: `to_ascii_lowercase()` on both strings, then `str::cmp()` on trimmed trailing spaces
3. **`BinarySort` / `BinarySort2`**: Direct byte-level comparison (`[u8]::cmp()`) on trimmed trailing spaces

**Rationale**: C# has 3 comparison paths:
- **BinarySort**: Encodes to UTF-16 LE bytes, memcmp with space-padding to equal length
- **BinarySort2**: Char-by-char unsigned comparison with space-padding
- **Culture-aware**: `CompareInfo.Compare()` with trailing space trimming

Our Rust equivalents simplify these:
- We use UTF-8 bytes instead of UTF-16 LE (since Rust strings are natively UTF-8)
- `BinarySort` and `BinarySort2` produce the same result when comparing raw UTF-8 bytes — both are "compare the bytes" semantics
- `IgnoreCase` uses ASCII case folding per spec (FR-010)
- Trailing space trimming is applied before comparison (matching C#'s behavior)

**Alternatives considered**:
- Convert to UTF-16 for BinarySort: Unnecessary allocation, only matters if mixed with C# consumers expecting UTF-16 order. Rejected.
- Full Unicode case folding for IgnoreCase: Would require `unicode-casefold` crate or manual tables. ASCII folding is sufficient per spec assumption.

## R5: Trailing Space Handling

**Decision**: Trim trailing spaces before comparison using `str::trim_end_matches(' ')`.

**Rationale**: C# SqlString comparison trims trailing spaces for both culture-aware and BinarySort2 modes. BinarySort pads with spaces to equal length (equivalent effect for comparison). This ensures `"hello"` equals `"hello   "` in all comparison modes, matching SQL Server semantics where trailing spaces are ignored in comparisons.

**Alternatives considered**:
- No trailing space handling: Diverges from SQL Server / C# behavior. Rejected.
- Pad shorter string with spaces: Equivalent result but allocates. Trimming is allocation-free. Chosen.

## R6: Concatenation — Left Operand's Options Govern

**Decision**: Concatenation via `Add` operator produces a new `SqlString` with the left operand's `SqlCompareOptions`. NULL propagation: if either operand is NULL, result is NULL.

**Rationale**: C# requires both operands to have matching LCID and flags, throwing `SqlTypeException` if they differ. Since we dropped LCID (R1), the "must match" requirement simplifies. The spec (FR-007) says "result inherits left operand's compare options", and the edge cases section says "the left operand's options take precedence." Following this simpler rule avoids errors that would confuse users who don't understand compare options.

**Alternatives considered**:
- Throw error if options differ (matching C# strictly): Overly strict for Rust, where there's no implicit conversion from `string`. Rejected per spec decision.
- Always use default options for result: Loses left operand context. Rejected.

## R7: Eq/Hash — Case-Insensitive

**Decision**: `PartialEq`/`Eq` compare using case-insensitive ASCII comparison with trailing space trimming. Two NULLs are equal. `Hash` hashes the lowercased, trailing-space-trimmed string; NULL hashes as empty string.

**Rationale**: C# `SqlString.Equals(object)` calls `StringCompare` which respects compare options. C# `GetHashCode()` is compare-options-aware: BinarySort hashes UTF-16 bytes, culture-aware hashes `SortKey.KeyData`. Since our `Eq` trait must be consistent for all instances regardless of compare options (Rust's `Eq` contract), we standardize on case-insensitive comparison (the default). This ensures `Eq` is reflexive, symmetric, transitive. `Hash` must be consistent with `Eq`, so we hash the same normalized form.

**Alternatives considered**:
- Case-sensitive `Eq`: Would make `a == b` case-sensitive while `sql_equals` is case-insensitive by default — confusing. Rejected.
- Options-aware `Eq`: Two SqlStrings with different options couldn't be consistently compared. Violates Rust's `Eq` symmetry requirement. Rejected.

## R8: Ord — Case-Insensitive, NULL < Non-NULL

**Decision**: `PartialOrd`/`Ord` use case-insensitive ASCII comparison. NULL compares less than any non-NULL value. Two NULLs are equal in ordering.

**Rationale**: Follows the established pattern from SqlInt32/SqlInt64 etc. where NULL is ordered before all values. Case-insensitive ordering matches Eq (consistency required by Rust's `Ord` contract).

**Alternatives considered**: None — must be consistent with `Eq` and follow established NULL ordering pattern.

## R9: Display and FromStr

**Decision**: `Display` outputs the raw string value for non-NULL, `"Null"` for NULL. `FromStr` parses `"Null"` (case-insensitive) as `SqlString::NULL`, everything else as `SqlString::new(input)` with default compare options.

**Rationale**: Matches the established pattern from all other SQL types. C# `SqlString.ToString()` returns the raw value for non-null and `"Null"` for null. C# has no `Parse` method — construction is via implicit operator from `string`. But Rust's `FromStr` is the idiomatic way to parse, so we provide it.

**Alternatives considered**:
- Quote the string in Display (e.g., `"hello"`): Not what C# does. Rejected.

## R10: Type Conversions — Deferred

**Decision**: Defer all type conversions (`to_sql_boolean`, `from_sql_int32`, etc.) to follow-up work. SqlString does not depend on other SQL types for its core implementation.

**Rationale**: C# provides explicit casts from all numeric SqlTypes, SqlBoolean, SqlDateTime, SqlGuid to SqlString (via ToString), and reverse parsing. These are numerous conversions that can be added incrementally. The spec does not list them as functional requirements — the scope is the core string type with comparison options. Following the project's pattern of deferring cross-type conversions.

**Alternatives considered**:
- Implement all conversions now: Scope creep; not in the spec FRs. Rejected.

## R11: Clone, Not Copy

**Decision**: `SqlString` implements `Clone` but NOT `Copy`. `SqlCompareOptions` implements `Copy + Clone`.

**Rationale**: `SqlString` contains `Option<String>`, which is heap-allocated and not `Copy`. `SqlCompareOptions` is a simple enum with no heap data, so it can be `Copy`. This is explicitly called out in the spec (FR-001, Edge Cases).

**Alternatives considered**: None — dictated by Rust's type system.

## R12: Add Operator Return Type — SqlString (Not Result)

**Decision**: The `Add` operator returns `SqlString` directly (not `Result<SqlString, SqlTypeError>`), because string concatenation cannot fail (no overflow, no divide-by-zero). NULL propagation returns `SqlString::NULL`.

**Rationale**: Unlike numeric types where arithmetic can overflow, string concatenation always succeeds (barring OOM, which Rust handles via panic, not Result). There are no error conditions. Following C#'s `operator +` which returns `SqlString` directly.

**Alternatives considered**:
- Return `Result<SqlString, SqlTypeError>`: No error conditions exist. Unnecessary wrapping. Rejected.

## R13: Constructor Takes `&str`, Not `String`

**Decision**: `new()` and `with_options()` accept `&str` and clone internally to `String`. Also implement `From<&str>` and `From<String>`.

**Rationale**: Taking `&str` is idiomatic Rust for string constructors — callers don't need to pre-allocate a `String`. Internally we store `String` (owned) for the type to own its data. Providing `From<String>` avoids an unnecessary clone when the caller already has an owned `String`.

**Alternatives considered**:
- Accept only `String`: Forces callers to `.to_string()` every literal. Not idiomatic. Rejected.
- Use `Cow<'a, str>`: Adds lifetime parameter, complicating the type. Rejected per simplicity.
