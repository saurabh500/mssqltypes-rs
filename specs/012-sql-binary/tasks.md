# Tasks: SqlBinary

**Input**: Design documents from `/specs/012-sql-binary/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-006) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_binary.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [x] T001 Create `src/sql_binary.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `std::fmt`, `std::hash`, `std::ops::Add`, `std::cmp::Ordering`), and empty struct definition
- [x] T002 Register module in `src/lib.rs`: add `pub mod sql_binary;` and `pub use sql_binary::SqlBinary;`

**Checkpoint**: `cargo build` compiles with empty `SqlBinary` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, constructors, and accessors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, constructors, accessors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the struct must exist for test code to compile. Tests in Phase 3 (T007–T010) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt32/SqlInt64.

- [x] T003 Define `SqlBinary` struct with `value: Option<Vec<u8>>`, derive `Clone, Debug` in `src/sql_binary.rs`
- [x] T004 Implement `SqlBinary::NULL` constant (`const NULL: SqlBinary = SqlBinary { value: None }`) in `src/sql_binary.rs`
- [x] T005 Implement `new(v: Vec<u8>) -> Self`, `is_null() -> bool`, `value() -> Result<&[u8], SqlTypeError>` in `src/sql_binary.rs`
- [x] T006 Implement `len() -> Result<usize, SqlTypeError>`, `is_empty() -> Result<bool, SqlTypeError>`, `get(index: usize) -> Result<u8, SqlTypeError>` in `src/sql_binary.rs`

**Checkpoint**: Foundation ready — `SqlBinary::new()`, `is_null()`, `value()`, `len()`, `is_empty()`, `get()`, `NULL` constant all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect Binary Values (Priority: P1) 🎯 MVP

**Goal**: Users can create `SqlBinary` values from byte vectors, inspect contents, check for NULL, retrieve length and individual bytes.

**Independent Test**: `SqlBinary::new(vec![1,2,3]).value()` returns `Ok(&[1,2,3])`, `SqlBinary::NULL.is_null()` returns `true`, `SqlBinary::new(vec![]).is_null()` returns `false` (empty ≠ NULL), `SqlBinary::new(vec![10,20,30]).get(1)` returns `Ok(20)`.

### Tests for User Story 1

- [x] T007 [P] [US1] Write tests for `new()`, `is_null()`, `value()` — create from byte vector `vec![1,2,3]`, empty binary `vec![]` is not NULL, NULL access returns `Err(NullValue)`, NULL constant `is_null()` is true in `src/sql_binary.rs`
- [x] T008 [P] [US1] Write tests for `len()`, `is_empty()` — normal binary has len 3, empty binary has len 0 and `is_empty()` returns `Ok(true)`, non-empty returns `Ok(false)`, NULL returns `Err(NullValue)` for both in `src/sql_binary.rs`
- [x] T009 [P] [US1] Write tests for `get()` — valid index 0/1/2 returns correct bytes, out-of-bounds index 5 returns `Err(OutOfRange)`, NULL returns `Err(NullValue)`, empty binary `get(0)` returns `Err(OutOfRange)` in `src/sql_binary.rs`
- [x] T010 [P] [US1] Write tests for `From<&[u8]>` and `From<Vec<u8>>` — slice `&[10,20,30]` creates non-null SqlBinary, `Vec<u8>` into SqlBinary preserves bytes, both roundtrip via `value()` in `src/sql_binary.rs`

### Implementation for User Story 1

- [x] T011 [US1] Implement `From<&[u8]> for SqlBinary` (clones slice) and `From<Vec<u8>> for SqlBinary` (takes ownership) in `src/sql_binary.rs`

**Checkpoint**: All US1 acceptance scenarios (8 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Concatenation (Priority: P1)

**Goal**: Users can concatenate two `SqlBinary` values using the `+` operator. NULL propagation applies — if either operand is NULL, the result is NULL.

**Independent Test**: `SqlBinary([1,2]) + SqlBinary([3,4])` returns `SqlBinary([1,2,3,4])`, `SqlBinary([1,2]) + SqlBinary::NULL` returns NULL, empty concat works correctly.

### Tests for User Story 2

- [x] T012 [US2] Write tests for `Add` operator — `[1,2] + [3,4] = [1,2,3,4]`, left NULL returns NULL, right NULL returns NULL, both NULL returns NULL, empty `[] + [1,2] = [1,2]`, `[1,2] + [] = [1,2]`, `[] + [] = []` in `src/sql_binary.rs`

### Implementation for User Story 2

- [x] T013 [US2] Implement `Add` trait for SqlBinary — if either `is_null()` return `SqlBinary::NULL`, otherwise concatenate both byte vectors into new `SqlBinary` in `src/sql_binary.rs`

**Checkpoint**: All US2 acceptance scenarios (5 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Comparison with Trailing-Zero Padding (Priority: P2)

**Goal**: Users can compare binary values using SQL comparison methods. Shorter values are logically padded with trailing zeros before comparison, matching C# `PerformCompareByte` behavior. Any comparison involving NULL returns `SqlBoolean::NULL`.

**Independent Test**: `SqlBinary([1,2]).sql_equals(&SqlBinary([1,2,0,0]))` returns `SqlBoolean::TRUE` (trailing zeros), `SqlBinary([1,2]).sql_less_than(&SqlBinary([1,3]))` returns `SqlBoolean::TRUE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 3

- [x] T014 [P] [US3] Write tests for trailing-zero-padded comparison edge cases — `[1,2] == [1,2,0,0]`, `[0] == []`, `[] == []`, `[1,2,1] > [1,2]` (extra non-zero byte), `[1,2] < [1,3]`, `[1,2,0] == [1,2]`, `[0,0,0] == []` in `src/sql_binary.rs`
- [x] T015 [P] [US3] Write tests for 6 SQL comparison methods — `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` with equal values, less/greater values, NULL propagation on left, right, and both sides in `src/sql_binary.rs`

### Implementation for User Story 3

- [x] T016 [US3] Implement internal `compare_bytes(a: &[u8], b: &[u8]) -> Ordering` helper — compare byte-by-byte up to `min(len)`, then check remaining bytes in longer array (any non-zero → longer is Greater, all zero → Equal) in `src/sql_binary.rs`
- [x] T017 [US3] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean`, NULL propagation, delegating to `compare_bytes` in `src/sql_binary.rs`

**Checkpoint**: All US3 acceptance scenarios (6 scenarios) pass. Trailing-zero padding verified with ≥5 test pairs per SC-001. `cargo test` green.

---

## Phase 6: User Story 4 — Display (Priority: P2)

**Goal**: Users can display binary values as lowercase hex strings for debugging and logging. NULL displays as `"Null"`, empty binary as `""`.

**Independent Test**: `format!("{}", SqlBinary([0x0A, 0xFF]))` returns `"0aff"`, `format!("{}", SqlBinary::NULL)` returns `"Null"`, `format!("{}", SqlBinary([]))` returns `""`.

### Tests for User Story 4

- [x] T018 [US4] Write tests for `Display` — `[0x0A, 0xFF]` → `"0aff"`, `[0x00]` → `"00"`, `[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]` → `"0123456789abcdef"`, NULL → `"Null"`, empty `[]` → `""` in `src/sql_binary.rs`

### Implementation for User Story 4

- [x] T019 [US4] Implement `Display` for SqlBinary — NULL outputs `"Null"`, otherwise write each byte as `"{:02x}"` (lowercase hex, zero-padded to 2 digits) in `src/sql_binary.rs`

**Checkpoint**: All US4 acceptance scenarios (3 scenarios) pass. `cargo test` green.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits (`Eq`, `Hash`, `Ord`) and final quality gates that span all user stories

### Tests

- [x] T020 [P] Write tests for `PartialEq`/`Eq` — trailing-zero-padded equality (`[1,2] == [1,2,0,0]`), NULL == NULL (Rust semantics), empty == empty, different values not equal, `[1,2] != [1,3]` in `src/sql_binary.rs`
- [x] T021 [P] Write tests for `Hash` — equal values hash equal (`[1,2]` and `[1,2,0,0]` produce same hash), NULL hashes consistently, `[0]` and `[]` hash equal in `src/sql_binary.rs`
- [x] T022 [P] Write tests for `PartialOrd`/`Ord` — NULL < any non-NULL value, NULL == NULL, trailing-zero ordering (`[1,2] == [1,2,0,0]`), `[1,2] < [1,3]`, `[1,2,1] > [1,2]`, empty < non-empty-non-zero in `src/sql_binary.rs`

### Implementation

- [x] T023 Implement `PartialEq`, `Eq` for SqlBinary using trailing-zero-padded comparison (NULL == NULL for Rust `Eq` reflexivity) in `src/sql_binary.rs`
- [x] T024 Implement `Hash` for SqlBinary — trim trailing zeros from byte slice before hashing, NULL hashes as empty slice in `src/sql_binary.rs`
- [x] T025 Implement `PartialOrd`, `Ord` for SqlBinary using trailing-zero-padded byte ordering (NULL < any non-NULL, NULL == NULL) in `src/sql_binary.rs`
- [x] T026 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [x] T027 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/accessors
- **US2 (Phase 4)**: Depends on Phase 2 — concatenation needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US4 (Phase 6)**: Depends on Phase 2 — Display needs constructors
- **Polish (Phase 7)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants/accessors
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US4 (P2)**: Standalone — only needs foundational struct
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T007–T010 (US1 tests) can run in parallel — independent test functions
- T014–T015 (US3 tests) can run in parallel
- T020–T022 (Phase 7 tests) can run in parallel
- US1–US4 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 3

```text
# Write US3 tests in parallel (T014-T015):
T014: tests for trailing-zero-padded comparison edge cases
T015: tests for 6 SQL comparison methods + NULL propagation

# Then implement sequentially (T016-T017):
T016: compare_bytes internal helper
T017: 6 SQL comparison methods
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: US1 tests + From conversions (T007–T011)
4. Complete Phase 4: US2 concatenation (T012–T013)
5. **STOP and VALIDATE**: Struct, accessors, conversions, concatenation all work

### Incremental Delivery

1. Setup + Foundational → module compiles
2. US1 → values can be created, inspected, and converted
3. US2 → concatenation works with NULL propagation
4. US3 → trailing-zero-padded SQL comparisons
5. US4 → hex Display formatting
6. Polish → standard traits (Eq, Hash, Ord), quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 27 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 5 |
| Phase 4 (US2) | 2 |
| Phase 5 (US3) | 4 |
| Phase 6 (US4) | 2 |
| Phase 7 (Polish) | 8 |
| Parallelizable tasks | 9 |
| Test tasks | 11 |
| Implementation tasks | 16 |
