# Data Model: SqlBinary

## Entity: SqlBinary

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<Vec<u8>>` | Internal byte storage. `None` = SQL NULL. |

### Invariants

- `value: None` represents SQL NULL
- `value: Some(vec![])` represents an empty (zero-length) binary — valid, not NULL
- Empty binary and NULL are distinct states
- No maximum length enforced (mirrors C# SqlBinary which has no intrinsic limit)

### Construction

| Constructor | Input | Result |
|-------------|-------|--------|
| `SqlBinary::NULL` | — | NULL constant (`value: None`) |
| `SqlBinary::new(v: Vec<u8>)` | Owned byte vector | Non-null SqlBinary taking ownership |
| `From<&[u8]>` | Borrowed byte slice | Non-null SqlBinary (cloned) |
| `From<Vec<u8>>` | Owned byte vector | Non-null SqlBinary (moved) |

### Comparison Semantics (Trailing-Zero Padding)

Two `SqlBinary` values are compared byte-by-byte. When lengths differ, the shorter value is logically padded with trailing zeros:

```
[1, 2]      == [1, 2, 0, 0]   → Equal (trailing zeros)
[1, 2]      <  [1, 3]          → Less (byte at index 1 differs)
[1, 2, 1]   >  [1, 2]          → Greater (extra non-zero byte)
[0]         == []               → Equal (trailing zero padding)
[]          == []               → Equal
```

Algorithm:
1. Compare bytes at indices `0..min(a.len, b.len)`
2. If mismatch found, shorter byte < longer byte → ordering determined
3. If prefix matches and lengths equal → Equal
4. If prefix matches and lengths differ → check remaining bytes of longer array:
   - All zeros → Equal
   - Any non-zero → longer array is Greater

### Hash Normalization

To maintain `Hash`/`Eq` consistency, trailing zeros are trimmed before hashing:
- `[1, 2, 0, 0]` hashes the same as `[1, 2]`
- NULL hashes as empty slice (deterministic value)

### State Transitions

SqlBinary is immutable. No state transitions occur after construction. All operations (`Add`, comparisons) produce new instances.

### Relationships

| Relationship | Target | Description |
|-------------|--------|-------------|
| Uses | `SqlTypeError` | Returned by `value()`, `len()`, `get()`, `is_empty()` on NULL or out-of-bounds |
| Produces | `SqlBoolean` | Returned by all SQL comparison methods |
| Produces | `SqlBinary` | Returned by `Add` operator (concatenation) |
