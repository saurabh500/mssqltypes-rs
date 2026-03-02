# Research: SqlBinary

## R1: Internal Representation — `Option<Vec<u8>>` vs `Option<Box<[u8]>>`

- **Decision**: Use `Option<Vec<u8>>`
- **Rationale**: `Vec<u8>` is the idiomatic Rust equivalent of C#'s `byte[]`. Concatenation (`Add`) produces a new `Vec`, so growable semantics are natural. `Box<[u8]>` would save one `usize` (capacity) but adds friction for concat.
- **Alternatives considered**: `Option<Box<[u8]>>` — rejected because concatenation would require converting back to `Vec` anyway.

## R2: Constructor Ownership — `new(Vec<u8>)` vs `new(&[u8])`

- **Decision**: `new(v: Vec<u8>)` takes ownership; add `From<&[u8]>` for convenience
- **Rationale**: C# copies the input array. In Rust, taking ownership of `Vec<u8>` avoids unnecessary copies when the caller already has a `Vec`. `From<&[u8]>` handles the borrowed case. Also provide `From<Vec<u8>>`.
- **Alternatives considered**: `new(s: &[u8])` — rejected because it forces allocation even when the caller already has a `Vec`.

## R3: `value()` Return Type — `&[u8]` vs `Vec<u8>`

- **Decision**: Return `Result<&[u8], SqlTypeError>` (borrowed slice)
- **Rationale**: C# returns a copy via `.Value`, but Rust idiom is to return a borrowed slice for zero-copy access. The caller can `.to_vec()` if they need ownership. This matches the spec's FR-004.
- **Alternatives considered**: Returning `Vec<u8>` (cloned) — rejected as wasteful; callers rarely need ownership.

## R4: Trailing-Zero-Padded Comparison Algorithm

- **Decision**: Compare byte-by-byte up to the shorter length, then check if remaining bytes in the longer array are all zero.
- **Rationale**: Direct port of C#'s `PerformCompareByte`. When arrays have equal length, standard byte comparison. When unequal, after comparing the common prefix, iterate the longer tail — any non-zero byte makes it greater. If all remaining are zero, they're equal.
- **Alternatives considered**: Normalizing by trimming trailing zeros before comparison — rejected because it would require allocation or complex iterator logic. The C# approach is simpler and allocation-free.

## R5: Hash Normalization for Trailing-Zero Consistency

- **Decision**: Trim trailing zeros before hashing. NULL hashes as 0 (empty slice hash).
- **Rationale**: C# trims trailing zeros in `GetHashCode()` to ensure `[1,2]` and `[1,2,0,0]` hash identically (consistent with `Eq`). Rust's `Hash` trait requires: `a == b ⟹ hash(a) == hash(b)`. Since our `Eq` uses trailing-zero padding, `Hash` must normalize.
- **Alternatives considered**: Hashing the full bytes — rejected because it violates the Hash/Eq consistency contract.

## R6: Display Format — Hex vs C# `SqlBinary({len})`

- **Decision**: Use lowercase hex with no separators (e.g., `"0aff"`)
- **Rationale**: C# uses `"SqlBinary({len})"` for `ToString()`, which is not particularly useful for debugging. The spec explicitly requires lowercase hex (FR-011). Hex display is more useful in a Rust context where binary data inspection is common. This is a deliberate deviation from C# for improved Rust ergonomics.
- **Alternatives considered**: C# format `"SqlBinary(4)"` — rejected per spec requirement and because it provides less useful information.

## R7: `SqlBinary::NULL` — `const` vs `static`

- **Decision**: Cannot use `const` because `Vec<u8>` is not `const`-constructible. Use an associated function or `static` with lazy initialization — or simply let callers use a factory function.
- **Rationale**: `Vec<u8>` allocates on the heap, so `const` is impossible. However, since NULL has `value: None`, no allocation occurs. We can use a factory function `SqlBinary::null()` or use a pattern like `SqlBinary { value: None }` directly. Looking at existing codebase patterns: fixed-size types use `const NULL`. For `SqlBinary`, since the internal `Option<Vec<u8>>` where `None` doesn't allocate, we can still define a `const NULL` because `Option::<Vec<u8>>::None` is `const`-constructible.
- **Alternatives considered**: `static NULL: SqlBinary` with `lazy_static` — rejected because `const` works for `None` variant.

## R8: Indexed Access — `get(usize)` vs `Index` trait

- **Decision**: Use `get(usize) -> Result<u8, SqlTypeError>` method, not `Index` trait
- **Rationale**: The `Index` trait returns a reference and panics on out-of-bounds. Our spec requires returning `Err(SqlTypeError::OutOfRange)` for invalid indices and `Err(SqlTypeError::NullValue)` for NULL. A method returning `Result` is the appropriate Rust pattern for fallible access.
- **Alternatives considered**: `impl Index<usize>` — rejected because it would panic instead of returning errors, violating constitution principle (no panics in library code).

## R9: `len()` Behavior for NULL

- **Decision**: `len() -> Result<usize, SqlTypeError>` — returns `Err(NullValue)` for NULL
- **Rationale**: C# throws `SqlNullValueException` when accessing `.Length` on a NULL `SqlBinary`. Our Rust equivalent returns `Err`. Also add `is_empty() -> Result<bool, SqlTypeError>` to satisfy clippy's `len_without_is_empty` lint.
- **Alternatives considered**: `len() -> usize` returning 0 for NULL — rejected because it conflates empty binary with NULL.

## R10: `PartialEq`/`Eq` — NULL == NULL

- **Decision**: In Rust's `PartialEq`, NULL == NULL returns `true`
- **Rationale**: C#'s `Equals` method returns `true` for two NULLs (needed for .NET collection semantics). Rust's `Eq` trait has the same requirement — reflexivity mandates `x == x`. SQL comparison operators return `SqlBoolean::NULL` for NULL comparisons, but `PartialEq` is a different contract. Two NULLs are equal in `PartialEq` but not in `sql_equals`.
- **Alternatives considered**: NULL != NULL — rejected because it violates `Eq` reflexivity.

## R11: `Ord`/`PartialOrd` — NULL Ordering

- **Decision**: NULL < any non-NULL value. Two NULLs are equal.
- **Rationale**: Matches C#'s `CompareTo` implementation and is consistent with other types in the library (SqlString, SqlInt32, etc.). This enables sorted collections.
- **Alternatives considered**: Return `None` from `partial_cmp` for NULL — rejected because `Ord` requires total ordering, and consistency with existing types.

## R12: FromStr / Parsing

- **Decision**: Do NOT implement `FromStr` for SqlBinary
- **Rationale**: C# does not provide string-to-binary parsing. Hex parsing would be a useful addition but is outside the scope of C# behavioral fidelity. Binary data is typically constructed from byte vectors, not parsed from strings. If needed later, can add `from_hex()` as a convenience method.
- **Alternatives considered**: Implement hex parsing — deferred to a future enhancement.
