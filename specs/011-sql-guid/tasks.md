# Tasks: SqlGuid

**Input**: Design documents from `/specs/011-sql-guid/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-006) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- All implementation in single file `src/sql_guid.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [x] T001 Create `src/sql_guid.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_binary::SqlBinary`, `std::fmt`, `std::str::FromStr`, `std::hash`), and empty struct definition
- [x] T002 Register module in `src/lib.rs`: add `pub mod sql_guid;` and `pub use sql_guid::SqlGuid;`

**Checkpoint**: `cargo build` compiles with empty `SqlGuid` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, constructors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the struct must exist for test code to compile. Tests in Phase 3 (T005–T008) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt32/SqlInt64.

- [x] T003 Define `SqlGuid` struct with `value: Option<[u8; 16]>`, derive `Clone, Debug`, manually impl `Copy` in `src/sql_guid.rs`
- [x] T004 Implement `SqlGuid::NULL` const, `new(bytes: [u8; 16]) -> Self`, `is_null() -> bool`, `value() -> Result<[u8; 16], SqlTypeError>`, `to_byte_array() -> Result<[u8; 16], SqlTypeError>`, and `From<[u8; 16]> for SqlGuid` in `src/sql_guid.rs`

**Checkpoint**: Foundation ready — `SqlGuid::new()`, `is_null()`, `value()`, `to_byte_array()`, `NULL` constant all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect GUID Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlGuid values from byte arrays, check for NULL, and retrieve the raw 16-byte representation.

**Independent Test**: `SqlGuid::new(bytes).value()` returns `Ok(bytes)`, `SqlGuid::NULL.is_null()` returns `true`, `SqlGuid::NULL.value()` returns `Err(NullValue)`, `to_byte_array()` returns same bytes, all-zeros GUID is valid (not NULL).

### Tests for User Story 1

- [x] T005 [US1] Write tests for `new()`, `is_null()`, `value()` — non-null GUID returns correct bytes, NULL access returns `Err(NullValue)`, all-zeros GUID is valid not NULL in `src/sql_guid.rs`
- [x] T006 [US1] Write tests for `SqlGuid::NULL` — `is_null()` is true, `value()` returns `Err(NullValue)` in `src/sql_guid.rs`
- [x] T007 [US1] Write tests for `to_byte_array()` — returns same bytes as `value()`, NULL returns `Err(NullValue)` in `src/sql_guid.rs`
- [x] T008 [US1] Write tests for `From<[u8; 16]>` — creates non-null SqlGuid with correct bytes in `src/sql_guid.rs`

**Checkpoint**: All US1 acceptance scenarios pass. `cargo test` green.

---

## Phase 4: User Story 2 — SQL Server Comparison Ordering (Priority: P1)

**Goal**: Users can compare GUIDs using SQL Server's non-standard byte order `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]`. All SQL comparison methods return `SqlBoolean` with NULL propagation.

**Independent Test**: Two GUIDs differing in byte 10 — byte 10 determines order. Two GUIDs differing in byte 0 and byte 10 — byte 10 wins (higher priority). Comparison with NULL returns `SqlBoolean::NULL`. Identical GUIDs return `SqlBoolean::TRUE` for `sql_equals`.

### Tests for User Story 2

- [x] T009 [P] [US2] Write tests for `sql_equals` and `sql_not_equals` — identical GUIDs → TRUE/FALSE, different GUIDs → FALSE/TRUE, NULL on either side → `SqlBoolean::NULL`, both NULL → `SqlBoolean::NULL` in `src/sql_guid.rs`
- [x] T010 [P] [US2] Write tests for `sql_less_than` and `sql_greater_than` — byte 10 determines order (node group first), byte 0 vs byte 10 priority (byte 10 wins), byte 3 determines order in last group, NULL propagation in `src/sql_guid.rs`
- [x] T011 [P] [US2] Write tests for `sql_less_than_or_equal` and `sql_greater_than_or_equal` — equal GUIDs → TRUE, less/greater → TRUE/FALSE, NULL propagation in `src/sql_guid.rs`
- [x] T012 [P] [US2] Write test verifying SQL_GUID_ORDER constant `[10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` — test each byte-group boundary with targeted GUID pairs (6 pairs minimum per SC-001) in `src/sql_guid.rs`

### Implementation for User Story 2

- [x] T013 [US2] Define `const SQL_GUID_ORDER: [usize; 16] = [10,11,12,13,14,15,8,9,6,7,4,5,0,1,2,3]` and implement private `fn sql_compare(&self, other: &SqlGuid) -> Option<std::cmp::Ordering>` helper using SQL byte order in `src/sql_guid.rs`
- [x] T014 [US2] Implement `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` returning `SqlBoolean` — delegate to `sql_compare()`, NULL propagation in `src/sql_guid.rs`

**Checkpoint**: All US2 acceptance scenarios (6 scenarios) pass. SQL byte ordering verified with 6+ boundary test vectors. `cargo test` green.

---

## Phase 5: User Story 3 — Display and Parsing (Priority: P2)

**Goal**: Users can display GUIDs in lowercase hyphenated format `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` and parse GUIDs from hyphenated or bare hex strings. NULL displays as `"Null"` and parses from `"Null"` (case-insensitive).

**Independent Test**: Known GUID → `"6f9619ff-8b86-d011-b42d-00cf4fc964ff"`, round-trip parse→display→parse equality, `"Null"` → NULL, invalid string → `ParseError`.

### Tests for User Story 3

- [x] T015 [P] [US3] Write tests for `Display` — known GUID outputs correct lowercase hex with hyphens matching .NET mixed-endian layout, NULL displays `"Null"`, all-zeros GUID displays `"00000000-0000-0000-0000-000000000000"` in `src/sql_guid.rs`
- [x] T016 [P] [US3] Write tests for `FromStr` — parse hyphenated format (uppercase, lowercase, mixed case), parse bare hex (32 chars), `"Null"`/`"null"`/`"NULL"` → NULL, invalid strings → `ParseError`, wrong length → `ParseError`, non-hex chars → `ParseError` in `src/sql_guid.rs`
- [x] T017 [P] [US3] Write tests for round-trip fidelity — format 5+ distinct GUIDs via Display then parse back via FromStr, verify equality (per SC-002) in `src/sql_guid.rs`

### Implementation for User Story 3

- [x] T018 [US3] Implement `Display` for SqlGuid — `"Null"` for NULL, lowercase hex with hyphens using .NET mixed-endian byte-to-string conversion in `src/sql_guid.rs`
- [x] T019 [US3] Implement `FromStr` for SqlGuid — detect `"Null"` (case-insensitive), validate length (32 or 36 chars), parse hex pairs with .NET mixed-endian string-to-byte conversion, return `ParseError` on invalid input in `src/sql_guid.rs`

**Checkpoint**: All US3 acceptance scenarios (6 scenarios) pass. Round-trip verified for 5+ GUIDs. `cargo test` green.

---

## Phase 6: User Story 4 — Conversions to/from SqlBinary (Priority: P2)

**Goal**: Users can convert between `SqlGuid` and `SqlBinary`. A SqlGuid converts to a 16-byte SqlBinary. A 16-byte SqlBinary converts back. NULL propagates. Wrong-length SqlBinary returns `Err(ParseError)`.

**Independent Test**: `SqlGuid::new(bytes).to_sql_binary()` → 16-byte SqlBinary with matching bytes. `SqlGuid::from_sql_binary(16_byte_binary)` → matching SqlGuid. NULL→NULL both directions. Non-16-byte → `Err(ParseError)`.

### Tests for User Story 4

- [x] T020 [P] [US4] Write tests for `to_sql_binary()` — non-null GUID → 16-byte SqlBinary with matching bytes, NULL → `SqlBinary::NULL` in `src/sql_guid.rs`
- [x] T021 [P] [US4] Write tests for `from_sql_binary()` — 16-byte SqlBinary → correct SqlGuid, NULL SqlBinary → `Ok(SqlGuid::NULL)`, fewer than 16 bytes → `Err(ParseError)`, more than 16 bytes → `Err(ParseError)`, round-trip `SqlGuid→SqlBinary→SqlGuid` equality in `src/sql_guid.rs`

### Implementation for User Story 4

- [x] T022 [US4] Implement `to_sql_binary(&self) -> SqlBinary` — NULL returns `SqlBinary::NULL`, otherwise `SqlBinary::new(bytes.to_vec())` in `src/sql_guid.rs`
- [x] T023 [US4] Implement `from_sql_binary(binary: &SqlBinary) -> Result<SqlGuid, SqlTypeError>` — NULL returns `Ok(SqlGuid::NULL)`, validate exactly 16 bytes, convert to `[u8; 16]` in `src/sql_guid.rs`

**Checkpoint**: All US4 acceptance scenarios (5 scenarios) pass. Round-trip SqlGuid↔SqlBinary verified. `cargo test` green.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Rust standard traits and final quality gates that span all user stories

### Tests

- [x] T024 [P] Write tests for `PartialEq`/`Eq` — matching GUIDs are equal, different GUIDs are not equal, NULL == NULL (Rust trait semantics), all-zeros ≠ NULL in `src/sql_guid.rs`
- [x] T025 [P] Write tests for `Hash` — equal GUIDs hash equal, NULL hashes consistently in `src/sql_guid.rs`
- [x] T026 [P] Write tests for `PartialOrd`/`Ord` — uses SQL Server byte ordering, NULL < any non-NULL, verify byte-group priority matches SQL_GUID_ORDER, equal GUIDs return `Ordering::Equal` in `src/sql_guid.rs`

### Implementation

- [x] T027 Implement `PartialEq`, `Eq` for SqlGuid — direct byte equality on `value` field (NULL == NULL for Rust trait purposes) in `src/sql_guid.rs`
- [x] T028 Implement `Hash` for SqlGuid — hash `value` field (consistent with `Eq`) in `src/sql_guid.rs`
- [x] T029 Implement `PartialOrd`, `Ord` for SqlGuid — SQL Server byte ordering via `SQL_GUID_ORDER`, NULL < any non-NULL in `src/sql_guid.rs`
- [x] T030 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [x] T031 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants
- **US2 (Phase 4)**: Depends on Phase 2 — SQL comparisons need constructors
- **US3 (Phase 5)**: Depends on Phase 2 — Display/FromStr need constructors
- **US4 (Phase 6)**: Depends on Phase 2 — conversions need constructors + `SqlBinary` (already exists)
- **Polish (Phase 7)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants
- **US2 (P1)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US3 (P2)**: Standalone — only needs foundational struct/constants
- **US4 (P2)**: Standalone — only needs foundational struct + `SqlBinary` (already exists)
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T009–T012 (US2 test methods) can run in parallel — independent test functions
- T015–T017 (US3 tests) can run in parallel
- T020–T021 (US4 tests) can run in parallel
- T024–T026 (Phase 7 tests) can run in parallel
- US1–US4 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 2

```text
# Write all US2 tests in parallel (T009-T012):
T009: tests for sql_equals / sql_not_equals
T010: tests for sql_less_than / sql_greater_than
T011: tests for sql_less_than_or_equal / sql_greater_than_or_equal
T012: tests for SQL_GUID_ORDER byte-group boundary verification

# Then implement sequentially (T013-T014):
T013: SQL_GUID_ORDER constant + sql_compare helper
T014: 6 SQL comparison methods wiring
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T004)
3. Complete Phase 3: US1 tests + validation (T005–T008)
4. Complete Phase 4: US2 SQL comparisons (T009–T014)
5. **STOP and VALIDATE**: Struct, constants, SQL byte ordering all work

### Incremental Delivery

1. Setup + Foundational → module compiles
2. US1 → GUIDs can be created and inspected
3. US2 → SQL Server comparison ordering works
4. US3 → Display and FromStr
5. US4 → SqlBinary bidirectional conversion
6. Polish → standard traits (Eq, Hash, Ord), quality gates

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 + User Story 2 (both P1)
   - Developer B: User Story 3 + User Story 4 (both P2)
3. Stories complete and integrate independently

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 31 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 2 |
| Phase 3 (US1) | 4 |
| Phase 4 (US2) | 6 |
| Phase 5 (US3) | 5 |
| Phase 6 (US4) | 4 |
| Phase 7 (Polish) | 8 |
| Parallelizable tasks | 13 |
| Test tasks | 14 |
| Implementation tasks | 17 |
