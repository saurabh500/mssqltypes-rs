# Tasks: SqlInt32

**Input**: Design documents from `/specs/004-sql-int32/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-002) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_int32.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [X] T001 Create `src/sql_int32.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_byte::SqlByte`, `crate::sql_int16::SqlInt16`, std traits), and empty struct definition
- [X] T002 Register module in `src/lib.rs`: add `pub mod sql_int32;` and `pub use sql_int32::SqlInt32;`

**Checkpoint**: `cargo build` compiles with empty `SqlInt32` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Define `SqlInt32` struct with `value: Option<i32>`, derive `Copy, Clone, Debug` in `src/sql_int32.rs`
- [X] T004 Implement constants `NULL`, `ZERO`, `MIN_VALUE` (-2_147_483_648), `MAX_VALUE` (2_147_483_647) in `src/sql_int32.rs`
- [X] T005 Implement `new(v: i32) -> Self`, `is_null() -> bool`, `value() -> Result<i32, SqlTypeError>` in `src/sql_int32.rs`
- [X] T006 Implement `From<i32> for SqlInt32` in `src/sql_int32.rs`

**Checkpoint**: Foundation ready — `SqlInt32::new()`, `is_null()`, `value()`, constants all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlInt32 values (including NULL, boundary values) and inspect them.

**Independent Test**: `SqlInt32::new(100_000).value()` returns `Ok(100_000)`, `SqlInt32::NULL.is_null()` returns `true`, boundary values (-2_147_483_648, 2_147_483_647) round-trip correctly.

### Tests for User Story 1

- [X] T007 [US1] Write tests for `new()`, `is_null()`, `value()` — positive value (100_000), negative value (-200_000), zero, NULL access returns `Err(NullValue)` in `src/sql_int32.rs`
- [X] T008 [US1] Write tests for constants — `NULL.is_null()`, `ZERO.value() == 0`, `MIN_VALUE.value() == -2_147_483_648`, `MAX_VALUE.value() == 2_147_483_647` in `src/sql_int32.rs`
- [X] T009 [US1] Write tests for `From<i32>` — `SqlInt32::from(42).value() == 42`, `SqlInt32::from(i32::MIN)`, `SqlInt32::from(i32::MAX)` in `src/sql_int32.rs`

**Checkpoint**: All US1 acceptance scenarios (7 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Arithmetic with Overflow Detection (Priority: P1)

**Goal**: Users can perform checked arithmetic (+, -, *, /, %, negation) with proper overflow and divide-by-zero detection. NULL propagates through all operations.

**Independent Test**: `SqlInt32::new(i32::MAX) + SqlInt32::new(1)` returns `Err(Overflow)`, `SqlInt32::new(7) % SqlInt32::new(3)` returns `Ok(SqlInt32(1))`, `-SqlInt32::new(i32::MIN)` returns `Err(Overflow)`, `SqlInt32::new(i32::MIN) / SqlInt32::new(-1)` returns `Err(Overflow)`.

### Tests for User Story 2

- [X] T010 [P] [US2] Write tests for `checked_add` — normal addition (100+200=300), overflow at MAX+1, underflow at MIN+(-1), NULL propagation (both sides) in `src/sql_int32.rs`
- [X] T011 [P] [US2] Write tests for `checked_sub` — normal subtraction, overflow at MIN-1, NULL propagation in `src/sql_int32.rs`
- [X] T012 [P] [US2] Write tests for `checked_mul` — normal multiply, overflow `100_000*100_000`, NULL propagation in `src/sql_int32.rs`
- [X] T013 [P] [US2] Write tests for `checked_div` — normal division, divide-by-zero error, MIN/-1 overflow, NULL propagation in `src/sql_int32.rs`
- [X] T014 [P] [US2] Write tests for `checked_rem` — normal remainder `7%3=1`, divide-by-zero error, MIN%-1 overflow, NULL propagation in `src/sql_int32.rs`
- [X] T015 [P] [US2] Write tests for `checked_neg` — normal negation, MIN_VALUE overflow, NULL returns NULL in `src/sql_int32.rs`

### Implementation for User Story 2

- [X] T016 [US2] Implement `checked_add`, `checked_sub` using `i32::checked_add/sub`, NULL propagation in `src/sql_int32.rs`
- [X] T017 [US2] Implement `checked_mul` using `i32::checked_mul`, NULL propagation in `src/sql_int32.rs`
- [X] T018 [US2] Implement `checked_div` — check `rhs==0` → `DivideByZero`, then `checked_div` → `None` means `Overflow`, NULL propagation in `src/sql_int32.rs`
- [X] T019 [US2] Implement `checked_rem` — check `rhs==0` → `DivideByZero`, then `checked_rem` → `None` means `Overflow`, NULL propagation in `src/sql_int32.rs`
- [X] T020 [US2] Implement `checked_neg` using `i32::checked_neg`, NULL propagation in `src/sql_int32.rs`
- [X] T021 [US2] Implement operator traits `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` delegating to `checked_*` methods in `src/sql_int32.rs`

**Checkpoint**: All US2 acceptance scenarios (10 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Bitwise Operations (Priority: P2)

**Goal**: Users can perform bitwise operations (AND, OR, XOR, NOT) on SqlInt32 values. Bitwise operations are infallible but propagate NULL.

**Independent Test**: `SqlInt32(0xFF00) & SqlInt32(0x0FF0)` returns `SqlInt32(0x0F00)`, `!SqlInt32(0)` returns `SqlInt32(-1)`, bitwise op with NULL returns NULL.

### Tests for User Story 3

- [X] T022 [P] [US3] Write tests for `BitAnd`, `BitOr`, `BitXor` — normal ops (0xFF00 & 0x0FF0, 0xFF00 | 0x00FF, 0xFF ^ 0x0F), negative values, NULL propagation in `src/sql_int32.rs`
- [X] T023 [P] [US3] Write tests for `Not` (ones complement) — `!0 == -1`, `!(-1) == 0`, `ones_complement()` method, NULL returns NULL in `src/sql_int32.rs`

### Implementation for User Story 3

- [X] T024 [US3] Implement `BitAnd`, `BitOr`, `BitXor` traits with NULL propagation in `src/sql_int32.rs`
- [X] T025 [US3] Implement `Not` trait and `ones_complement()` method with NULL propagation in `src/sql_int32.rs`

**Checkpoint**: All US3 acceptance scenarios (5 scenarios) pass. `cargo test` green.

---

## Phase 6: User Story 4 — Comparison Returning SqlBoolean (Priority: P2)

**Goal**: Users can compare SqlInt32 values using SQL three-valued logic. Comparisons return `SqlBoolean`, and any comparison involving NULL returns `SqlBoolean::NULL`.

**Independent Test**: `SqlInt32(10).sql_equals(&SqlInt32(10))` returns `SqlBoolean::TRUE`, `SqlInt32(10).sql_less_than(&SqlInt32(20))` returns `SqlBoolean::TRUE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 4

- [X] T026 [P] [US4] Write tests for 6 SQL comparison methods — `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`, NULL propagation on both sides in `src/sql_int32.rs`

### Implementation for User Story 4

- [X] T027 [US4] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean` in `src/sql_int32.rs`

**Checkpoint**: All US4 acceptance scenarios (8 scenarios) pass. `cargo test` green.

---

## Phase 7: User Story 5 — Display and Parsing (Priority: P2)

**Goal**: Users can convert SqlInt32 to and from string representations. NULL displays as `"Null"`. Parsing invalid strings returns a parse error.

**Independent Test**: `format!("{}", SqlInt32::new(42))` returns `"42"`, `format!("{}", SqlInt32::NULL)` returns `"Null"`, `"42".parse::<SqlInt32>()` returns `Ok(SqlInt32(42))`, `"abc".parse::<SqlInt32>()` returns error.

### Tests for User Story 5

- [X] T028 [P] [US5] Write tests for `Display` — positive (42), negative (-100), zero, NULL displays `"Null"` in `src/sql_int32.rs`
- [X] T029 [P] [US5] Write tests for `FromStr` — valid i32 (`"42"`), `"Null"` → NULL, out-of-range string (`"99999999999"`), non-numeric string (`"abc"`) → ParseError in `src/sql_int32.rs`

### Implementation for User Story 5

- [X] T030 [US5] Implement `Display` for SqlInt32 (`"Null"` for NULL, value string otherwise) in `src/sql_int32.rs`
- [X] T031 [US5] Implement `FromStr` for SqlInt32 (`"Null"` → NULL, parse i32, else `ParseError`) in `src/sql_int32.rs`

**Checkpoint**: All US5 acceptance scenarios (5 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 8: User Story 6 — Conversions to and from Other SqlTypes (Priority: P3)

**Goal**: Users can convert between SqlInt32 and other SQL types. Narrowing conversions check for range overflow. Widening from SqlBoolean follows C# semantics (TRUE=1, FALSE=0).

**Independent Test**: `SqlInt32::from(SqlBoolean::TRUE).value() == 1`, `SqlInt32(100).to_sql_int16()` returns `Ok(SqlInt16(100))`, `SqlInt32(100_000).to_sql_int16()` returns `Err(Overflow)`, `SqlInt32(200).to_sql_byte()` returns `Ok(SqlByte(200))`, `SqlInt32(300).to_sql_byte()` returns `Err(Overflow)`.

### Tests for User Story 6

- [X] T032 [P] [US6] Write tests for `From<SqlBoolean>` — NULL→NULL, FALSE→0, TRUE→1 in `src/sql_int32.rs`
- [X] T033 [P] [US6] Write tests for `to_sql_int16()` — in-range (100), overflow (100_000 > 32767), underflow (-100_000 < -32768), NULL→`Err(NullValue)` or NULL propagation in `src/sql_int32.rs`
- [X] T034 [P] [US6] Write tests for `to_sql_byte()` — in-range (200), overflow (300 > 255), negative (-1 < 0), NULL propagation in `src/sql_int32.rs`

### Implementation for User Story 6

- [X] T035 [US6] Implement `From<SqlBoolean> for SqlInt32` (widening: NULL→NULL, FALSE→0, TRUE→1) in `src/sql_int32.rs`
- [X] T036 [US6] Implement `to_sql_int16(&self) -> Result<SqlInt16, SqlTypeError>` (narrowing: overflow if < -32768 or > 32767) in `src/sql_int32.rs`
- [X] T037 [US6] Implement `to_sql_byte(&self) -> Result<SqlByte, SqlTypeError>` (narrowing: overflow if < 0 or > 255) in `src/sql_int32.rs`

**Checkpoint**: All US6 acceptance scenarios (9 scenarios) pass. `cargo test` green.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits and final quality gates that span all user stories

### Tests

- [X] T038 [P] Write tests for `PartialEq`/`Eq` — value equality, NULL==NULL (Rust semantics), different values not equal in `src/sql_int32.rs`
- [X] T039 [P] Write tests for `Hash` — equal values hash equal, NULL hashes consistently in `src/sql_int32.rs`
- [X] T040 [P] Write tests for `PartialOrd`/`Ord` — NULL < any value, MIN < MAX, negative < positive, equal values in `src/sql_int32.rs`

### Implementation

- [X] T041 Implement `PartialEq`, `Eq`, `Hash` for SqlInt32 (NULL hashes as `0i32`) in `src/sql_int32.rs`
- [X] T042 Implement `PartialOrd`, `Ord` for SqlInt32 (NULL < any value) in `src/sql_int32.rs`
- [X] T043 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [X] T044 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants
- **US2 (Phase 4)**: Depends on Phase 2 — arithmetic needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — bitwise needs constructors
- **US4 (Phase 6)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US5 (Phase 7)**: Depends on Phase 2 — Display/FromStr need constructors
- **US6 (Phase 8)**: Depends on Phase 2 — conversions need constructors + `SqlBoolean`, `SqlByte`, `SqlInt16` (all exist)
- **Polish (Phase 9)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P2)**: Standalone — only needs foundational struct/constants
- **US4 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US5 (P2)**: Standalone — only needs foundational struct/constants
- **US6 (P3)**: Standalone — only needs foundational struct + `SqlBoolean`, `SqlByte`, `SqlInt16` (all exist)
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T010–T015 (US2 test methods) can run in parallel — independent test functions
- T022–T023 (US3 tests) can run in parallel
- T028–T029 (US5 tests) can run in parallel
- T032–T034 (US6 tests) can run in parallel
- T038–T040 (Phase 9 tests) can run in parallel
- US1–US6 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 2

```text
# Write all US2 tests in parallel (T010-T015):
T010: tests for checked_add
T011: tests for checked_sub
T012: tests for checked_mul
T013: tests for checked_div
T014: tests for checked_rem
T015: tests for checked_neg

# Then implement sequentially (T016-T021):
T016: checked_add, checked_sub
T017: checked_mul
T018: checked_div
T019: checked_rem
T020: checked_neg
T021: operator trait wiring
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: US1 tests + validation (T007–T009)
4. Complete Phase 4: US2 arithmetic (T010–T021)
5. **STOP and VALIDATE**: Struct, constants, arithmetic all work

### Incremental Delivery

1. Setup + Foundational → module compiles
2. US1 → values can be created and inspected
3. US2 → arithmetic works with overflow detection
4. US3 → bitwise operations
5. US4 → SQL three-valued comparisons
6. US5 → Display and FromStr
7. US6 → cross-type conversions
8. Polish → standard traits, quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 44 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 3 |
| Phase 4 (US2) | 12 |
| Phase 5 (US3) | 4 |
| Phase 6 (US4) | 2 |
| Phase 7 (US5) | 4 |
| Phase 8 (US6) | 6 |
| Phase 9 (Polish) | 7 |
| Parallelizable tasks | 16 |
| Test tasks | 17 |
| Implementation tasks | 27 |
