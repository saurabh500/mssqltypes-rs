# Data Model: SqlGuid

## Entity: SqlGuid

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Option<[u8; 16]>` | Internal byte storage. `None` = SQL NULL. Bytes use .NET mixed-endian layout. |

### Invariants

- `value: None` represents SQL NULL
- `value: Some([0u8; 16])` represents the all-zeros GUID — valid, not NULL
- All-zeros GUID and NULL are distinct states
- Bytes are stored in .NET mixed-endian format (first 3 groups little-endian, last 2 big-endian)
- No maximum/minimum range — any 16-byte combination is valid

### Construction

| Constructor | Input | Result |
|-------------|-------|--------|
| `SqlGuid::NULL` | — | NULL constant (`value: None`) |
| `SqlGuid::new(bytes: [u8; 16])` | 16-byte array | Non-null SqlGuid |
| `FromStr` | Hyphenated or bare hex string | Parsed SqlGuid (mixed-endian conversion) |
| `From<[u8; 16]>` | 16-byte array | Non-null SqlGuid |
| `SqlGuid::from_sql_binary(SqlBinary)` | SqlBinary | SqlGuid if 16 bytes, error otherwise |

### Byte Layout (.NET Mixed-Endian)

When parsing the string `6F9619FF-8B86-D011-B42D-00CF4FC964FF`:

```
String groups:   6F9619FF  8B86  D011  B4  2D  00CF4FC964FF
                 --------  ----  ----  --  --  ------------
Byte indices:    [0..3]    [4,5] [6,7] [8] [9] [10..15]
Endianness:      LE        LE    LE    BE  BE  BE

Stored bytes:    [FF,19,96,6F, 86,8B, 11,D0, B4,2D, 00,CF,4F,C9,64,FF]
Byte index:       0  1  2  3   4  5   6  7   8  9  10 11 12 13 14 15
```

### SQL Server Comparison Order

Bytes are compared in this order: `[10, 11, 12, 13, 14, 15, 8, 9, 6, 7, 4, 5, 0, 1, 2, 3]`

This means:
1. **Node bytes** (10–15) are compared first — highest priority
2. **Clock sequence** (8–9) next
3. **Time high** (6–7) next
4. **Time mid** (4–5) next
5. **Time low** (0–3) last — lowest priority

### Equality vs Ordering

| Operation | Semantics |
|-----------|-----------|
| `PartialEq` / `Eq` | Standard byte equality (all 16 bytes, natural order). NULL == NULL. |
| `sql_equals` | SQL semantics — NULL returns `SqlBoolean::NULL`. Non-null uses byte equality. |
| `PartialOrd` / `Ord` | SQL Server byte ordering. NULL < any non-NULL. |
| `sql_less_than` etc. | SQL Server byte ordering. NULL returns `SqlBoolean::NULL`. |

### State Transitions

SqlGuid is immutable (`Copy`). No state transitions after construction.

### Relationships

| Relationship | Target | Description |
|-------------|--------|-------------|
| Uses | `SqlTypeError` | Returned by `value()`, `to_byte_array()`, `from_sql_binary()` on error |
| Produces | `SqlBoolean` | Returned by all SQL comparison methods |
| Converts to | `SqlBinary` | Via `to_sql_binary()` — 16-byte binary |
| Converts from | `SqlBinary` | Via `from_sql_binary()` — requires exactly 16 bytes |
