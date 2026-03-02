# Research: SqlGuid

## R1: Internal Representation — `Option<[u8; 16]>` vs `Option<Uuid>`

- **Decision**: Use `Option<[u8; 16]>`
- **Rationale**: Constitution VI mandates no external dependencies. A `[u8; 16]` is the simplest representation, is `Copy`, and avoids pulling in the `uuid` crate. The C# implementation stores `Guid?` internally, but Rust has no built-in GUID type. A 16-byte array is the natural zero-dependency equivalent.
- **Alternatives considered**: `Option<uuid::Uuid>` — rejected because it adds an external dependency. Custom newtype wrapper around `[u8; 16]` — unnecessary complexity for this use case.

## R2: SQL Server Comparison Byte Order

- **Decision**: Use constant array `[10, 11, 12, 13, 14, 15, 8, 9, 6, 7, 4, 5, 0, 1, 2, 3]`
- **Rationale**: Confirmed from C# reference implementation. The `Compare` method in `SqlGuid.cs` defines this as a local `ReadOnlySpan<byte>` and iterates byte-by-byte in this order. SQL Server compares the "node" bytes first (10–15), then clock_seq (8–9), time_hi (6–7), time_mid (4–5), and time_low (0–3) last.
- **Alternatives considered**: None — this is a fixed specification from SQL Server.

## R3: Equality vs Ordering — Different Semantics

- **Decision**: `PartialEq`/`Eq` uses standard byte equality (all 16 bytes in natural order). `PartialOrd`/`Ord` and SQL comparison methods use the SQL Server byte ordering.
- **Rationale**: C# `Equals()` uses direct `Guid` equality (natural byte order). Only `<`, `>`, `<=`, `>=` use the `rgiGuidOrder` comparison. Our spec FR-011 says "PartialEq/Eq based on byte equality" and FR-013 says "PartialOrd/Ord using SQL Server byte ordering". This matches C#'s split semantics exactly.
- **Alternatives considered**: Using SQL ordering for Eq — rejected because it matches C# behavior and two different GUIDs (byte-for-byte) being "equal" under SQL ordering could violate Hash/Eq consistency for `Eq`.

## R4: Hash Consistency with Eq

- **Decision**: `Hash` uses the raw `[u8; 16]` bytes directly (natural order), consistent with `Eq`.
- **Rationale**: Since `Eq` uses natural byte equality, `Hash` must also use natural byte order. C# delegates to `Guid.GetHashCode()` which hashes the raw bytes. NULL hashes as `0u128.to_ne_bytes()` (or simply hashes `None` discriminant).
- **Alternatives considered**: Custom hash with SQL byte order — rejected because it must be consistent with `Eq`, not `Ord`.

## R5: GUID String Format and Parsing

- **Decision**: Display outputs lowercase hex in format `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`. `FromStr` accepts this format (with or without hyphens), case-insensitive. `"Null"` (case-insensitive) returns `SqlGuid::NULL`.
- **Rationale**: C# `ToString()` outputs the "D" format (lowercase hex with hyphens). C# `Parse` accepts multiple formats via `new Guid(string)`. For simplicity and no-dependency constraint, we support the standard hyphenated format and the no-hyphen format. Case-insensitive parsing matches C# behavior.
- **Alternatives considered**: Supporting braced `{...}` and parenthesized `(...)` formats — deferred for simplicity; the standard hyphenated format covers >99% of use cases.

## R6: GUID Byte Layout — .NET Mixed-Endian Convention

- **Decision**: Store bytes in .NET's mixed-endian layout (same as C# `Guid.ToByteArray()`). When constructing from string, parse using .NET's convention: first 3 groups (data1, data2, data3) are little-endian, last 2 groups (data4, node) are big-endian.
- **Rationale**: For SQL Server compatibility, the bytes must match what SQL Server sends/receives. .NET `Guid` uses a mixed-endian format where the first group `6F9619FF` is stored as `[FF, 19, 96, 6F]` (little-endian), the second `8B86` as `[86, 8B]`, the third `D011` as `[11, D0]`, and the remaining bytes are stored as-is (big-endian). This is the standard COM/Windows GUID layout and matches SQL Server's wire format.
- **Alternatives considered**: RFC 4122 big-endian — rejected because it would produce different bytes than C# `SqlGuid` for the same GUID string, breaking cross-platform compatibility.

## R7: `SqlGuid::NULL` — `const` Feasibility

- **Decision**: Use `const NULL: SqlGuid = SqlGuid { value: None }` — `Option::<[u8; 16]>::None` is const-constructible.
- **Rationale**: `[u8; 16]` is `Copy` and stack-allocated, so `Option<[u8; 16]>` is const-constructible. This matches the pattern used by all other fixed-size types in the library.
- **Alternatives considered**: None — this is straightforward.

## R8: `to_sql_binary` / `from_sql_binary` — Dependency on SqlBinary

- **Decision**: Implement `to_sql_binary()` and `from_sql_binary()` directly since `SqlBinary` exists on the `012-sql-binary` branch and will be in main.
- **Rationale**: The spec requires conversion between `SqlGuid` and `SqlBinary` (FR-014, FR-015). `SqlBinary` is already implemented, so these methods can be included from the start. `to_sql_binary()` returns `SqlBinary::NULL` for NULL, otherwise a 16-byte `SqlBinary`. `from_sql_binary()` accepts NULL (returns `SqlGuid::NULL`) and validates exactly 16 bytes.
- **Alternatives considered**: Deferring conversions — rejected since `SqlBinary` already exists.

## R9: `value()` Return Type — `&[u8; 16]` vs `[u8; 16]`

- **Decision**: Return `Result<[u8; 16], SqlTypeError>` (by value, since it's `Copy`)
- **Rationale**: `[u8; 16]` is 16 bytes on the stack and is `Copy`. Returning by value avoids lifetime complications and matches the pattern of other `Copy` types in the library (e.g., `SqlInt32::value()` returns `i32`, not `&i32`). The spec says `&[u8; 16]` but returning by value is more idiomatic for `Copy` types.
- **Alternatives considered**: `Result<&[u8; 16], SqlTypeError>` — unnecessarily ties the return to `&self` lifetime for a `Copy` type.

## R10: Parsing Formats Supported

- **Decision**: Support two formats: hyphenated `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (32 hex + 4 hyphens = 36 chars) and bare `xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx` (32 hex = 32 chars). Case-insensitive.
- **Rationale**: These are the two most common GUID string formats. C# supports more formats but they're rarely used. Keeping it simple avoids complex parsing logic without external dependencies.
- **Alternatives considered**: Supporting `{...}` braces — deferred; can be added later if needed.
