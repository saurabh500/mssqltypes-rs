# Tasks: SqlInt16

**Input**: Design documents from `/specs/003-sql-int16/`
**Prerequisites**: plan.md, spec.md, research.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-002) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_int16.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [X] T001 Create `src/sql_int16.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_byte::SqlByte`, std traits), and empty struct definition
- [X] T002 Register module in `src/lib.rs`: add `pub mod sql_int16;` and `pub use sql_int16::SqlInt16;`

**Checkpoint**: `cargo build` compiles with empty `SqlInt16` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Define `SqlInt16` struct with `value: Option<i16>`, derive `Copy, Clone, Debug` in `src/sql_int16.rs`
- [X] T004 Implement constants `NULL`, `ZERO`, `MIN_VALUE` (-32768), `MAX_VALUE` (32767) in `src/sql_int16.rs`
- [X] T005 Implement `new(v: i16) -> Self`, `is_null() -> bool`, `value() -> Result<i16, SqlTypeError>` in `src/sql_int16.rs`
- [X] T006 Implement `From<i16> for SqlInt16` in `src/sql_int16.rs`

**Checkpoint**: Foundation ready — `SqlInt16::new()`, `is_null()`, `value()`, constants all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlInt16 values (including NULL, boundary values) and inspect them.

**Independent Test**: `SqlInt16::new(1000).value()` returns `Ok(1000)`, `SqlInt16::NULL.is_null()` returns `true`, boundary values (-32768, 32767) round-trip correctly.

### Tests for User Story 1

- [X] T007 [US1] Write tests for `new()`, `is_null()`, `value()` — positive value, negative value, zero, NULL access returns `Err(NullValue)` in `src/sql_int16.rs`
- [X] T008 [US1] Write tests for constants — `NULL.is_null()`, `ZERO.value() == 0`, `MIN_VALUE.value() == -32768`, `MAX_VALUE.value() == 32767` in `src/sql_int16.rs`
- [X] T009 [US1] Write tests for `From<i16>` — `SqlInt16::from(42).value() == 42`, `SqlInt16::from(i16::MIN)` in `src/sql_int16.rs`

**Checkpoint**: All US1 acceptance scenarios pass. `cargo test` green.

---

## Phase 4: User Story 2 — Arithmetic with Overflow (Priority: P1)

**Goal**: Users can perform checked arithmetic (+, -, *, /, %, negation) with proper overflow and divide-by-zero detection. NULL propagates through all operations.

**Independent Test**: `SqlInt16::new(32767) + SqlInt16::new(1)` returns `Err(Overflow)`, `SqlInt16::new(7) % SqlInt16::new(3)` returns `Ok(SqlInt16(1))`, `-SqlInt16::new(i16::MIN)` returns `Err(Overflow)`.

### Tests for User Story 2

- [X] T010 [P] [US2] Write tests for `checked_add` — normal addition, overflow at MAX+1, underflow at MIN-1, NULL propagation (both sides) in `src/sql_int16.rs`
- [X] T011 [P] [US2] Write tests for `checked_sub` — normal subtraction, overflow at MIN-1, underflow at MAX+1-direction, NULL propagation in `src/sql_int16.rs`
- [X] T012 [P] [US2] Write tests for `checked_mul` — normal multiply, overflow `100*400`, overflow `-100*400`, NULL propagation in `src/sql_int16.rs`
- [X] T013 [P] [US2] Write tests for `checked_div` — normal division, divide-by-zero error, MIN/-1 overflow, NULL propagation in `src/sql_int16.rs`
- [X] T014 [P] [US2] Write tests for `checked_rem` — normal remainder `7%3=1`, divide-by-zero error, MIN%-1 overflow, NULL propagation in `src/sql_int16.rs`
- [X] T015 [P] [US2] Write tests for `checked_neg` — normal negation, MIN_VALUE overflow, NULL returns NULL in `src/sql_int16.rs`

### Implementation for User Story 2

- [X] T016 [US2] Implement `checked_add`, `checked_sub` using `i16::checked_add/sub`, NULL propagation in `src/sql_int16.rs`
- [X] T017 [US2] Implement `checked_mul` using `i16::checked_mul`, NULL propagation in `src/sql_int16.rs`
- [X] T018 [US2] Implement `checked_div` — check `rhs==0` → `DivideByZero`, then `checked_div` → `None` means `Overflow`, NULL propagation in `src/sql_int16.rs`
- [X] T019 [US2] Implement `checked_rem` — check `rhs==0` → `DivideByZero`, then `checked_rem` → `None` means `Overflow`, NULL propagation in `src/sql_int16.rs`
- [X] T020 [US2] Implement `checked_neg` using `i16::checked_neg`, NULL propagation in `src/sql_int16.rs`
- [X] T021 [US2] Implement operator traits `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` delegating to `checked_*` methods in `src/sql_int16.rs`

**Checkpoint**: All US2 acceptance scenarios pass. All 7 spec scenarios verified. `cargo test` green.

---

## Phase 5: User Story 3 — Bitwise and Comparison (Priority: P2)

**Goal**: Users can perform bitwise operations (AND, OR, XOR, NOT) and SQL three-valued comparisons. NULL propagates correctly.

**Independent Test**: `SqlInt16(0xFF) & SqlInt16(0x0F)` returns `SqlInt16(0x0F)`, `SqlInt16(10).sql_less_than(&SqlInt16(20))` returns `SqlBoolean::TRUE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 3

- [X] T022 [P] [US3] Write tests for `BitAnd`, `BitOr`, `BitXor` — normal ops, negative values, NULL propagation in `src/sql_int16.rs`
- [X] T023 [P] [US3] Write tests for `Not` (ones complement) — `!0 == -1`, `!MIN == MAX`, `!MAX == MIN`, NULL returns NULL in `src/sql_int16.rs`
- [X] T024 [P] [US3] Write tests for 6 SQL comparison methods — equal, not-equal, less-than, greater-than, less-than-or-equal, greater-than-or-equal, NULL propagation on both sides in `src/sql_int16.rs`

### Implementation for User Story 3

- [X] T025 [US3] Implement `BitAnd`, `BitOr`, `BitXor` traits with NULL propagation in `src/sql_int16.rs`
- [X] T026 [US3] Implement `Not` trait and `ones_complement()` method with NULL propagation in `src/sql_int16.rs`
- [X] T027 [US3] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean` in `src/sql_int16.rs`

**Checkpoint**: All US3 acceptance scenarios pass. `cargo test` green.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Standard traits, conversions, display/parse, and final quality gates

### Tests

- [X] T028 [P] Write tests for `PartialEq`/`Eq` — value equality, NULL==NULL (Rust semantics), different values in `src/sql_int16.rs`
- [X] T029 [P] Write tests for `Hash` — equal values hash equal, NULL hashes consistently in `src/sql_int16.rs`
- [X] T030 [P] Write tests for `PartialOrd`/`Ord` — NULL < any value, MIN < MAX, negative < positive in `src/sql_int16.rs`
- [X] T031 [P] Write tests for `Display` — positive, negative, zero, NULL displays "Null" in `src/sql_int16.rs`
- [X] T032 [P] Write tests for `FromStr` — valid i16, "Null", out-of-range string, non-numeric string in `src/sql_int16.rs`
- [X] T033 [P] Write tests for conversions — `From<SqlBoolean>` (NULL/FALSE/TRUE), `From<SqlByte>` (NULL/0/255), `to_sql_boolean()` (NULL/0/nonzero), `to_sql_byte()` (NULL/in-range/negative/over-255) in `src/sql_int16.rs`

### Implementation

- [X] T034 Implement `PartialEq`, `Eq`, `Hash` for `SqlInt16` in `src/sql_int16.rs`
- [X] T035 Implement `PartialOrd`, `Ord` for `SqlInt16` (NULL < any value) in `src/sql_int16.rs`
- [X] T036 Implement `Display` for `SqlInt16` ("Null" for NULL, value string otherwise) in `src/sql_int16.rs`
- [X] T037 Implement `FromStr` for `SqlInt16` ("Null" → NULL, parse i16, else `ParseError`) in `src/sql_int16.rs`
- [X] T038 Implement `From<SqlBoolean> for SqlInt16` and `From<SqlByte> for SqlInt16` (widening) in `src/sql_int16.rs`
- [X] T039 Implement `to_sql_boolean()` and `to_sql_byte() -> Result<SqlByte, SqlTypeError>` (narrowing) in `src/sql_int16.rs`
- [X] T040 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [X] T041 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants
- **US2 (Phase 4)**: Depends on Phase 2 — arithmetic needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — bitwise/comparison needs constructors
- **Polish (Phase 6)**: Depends on Phase 2 — traits/conversions need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- US2 and US3 can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T011–T015 (US2 test methods) can run in parallel — independent test functions
- T022–T024 (US3 tests) can run in parallel
- T028–T033 (Phase 6 tests) can run in parallel
- US2 and US3 can be worked on in parallel after Phase 2

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
4. US3 → bitwise and comparison operations
5. Polish → traits, conversions, display/parse, quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 41 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 3 |
| Phase 4 (US2) | 12 |
| Phase 5 (US3) | 6 |
| Phase 6 (Polish) | 14 |
| Parallelizable tasks | 18 |
| Test tasks | 18 |
| Implementation tasks | 23 |
