# Tasks: SqlSingle

**Input**: Design documents from `/specs/008-sql-single/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — TDD is mandatory per project constitution (Section III). Tests are written before implementation in each phase.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files/sections, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- All implementation in `src/sql_single.rs` with inline `#[cfg(test)] mod tests`

---

## Phase 1: Setup

**Purpose**: Create module file and register in lib.rs

- [ ] T001 Create `src/sql_single.rs` with module skeleton (struct, imports, test module)
- [ ] T002 Register module in `src/lib.rs`: add `pub mod sql_single` and `pub use sql_single::SqlSingle`

**Checkpoint**: Project compiles with empty SqlSingle struct

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, constructors, and From<f32> — MUST complete before any user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Define `SqlSingle` struct with `Option<f32>` field, derive `Copy, Clone, Debug` in `src/sql_single.rs`
- [ ] T004 Implement constants `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE` in `src/sql_single.rs`
- [ ] T005 Implement `new(f32) -> Result<SqlSingle, SqlTypeError>` with `is_finite()` validation in `src/sql_single.rs`
- [ ] T006 Implement `From<f32>` (panicking on NaN/Infinity) in `src/sql_single.rs`

**Checkpoint**: Foundation ready — SqlSingle can be constructed and validated

---

## Phase 3: User Story 1 — Create and Inspect Values (Priority: P1) 🎯 MVP

**Goal**: Construct SqlSingle values from f32, inspect with `value()`/`is_null()`, reject NaN/Infinity

**Independent Test**: Construct values with `new()`, constants; call `value()`/`is_null()`; verify NaN/Infinity rejection

### Tests for User Story 1

- [ ] T007 [P] [US1] Tests for `new()` with valid values (positive, negative, zero) in `src/sql_single.rs`
- [ ] T008 [P] [US1] Tests for `new()` rejecting NaN, Infinity, NEG_INFINITY in `src/sql_single.rs`
- [ ] T009 [P] [US1] Tests for `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE` constants in `src/sql_single.rs`
- [ ] T010 [P] [US1] Tests for `value()` and `is_null()` including NULL error case in `src/sql_single.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement `is_null(&self) -> bool` in `src/sql_single.rs`
- [ ] T012 [US1] Implement `value(&self) -> Result<f32, SqlTypeError>` in `src/sql_single.rs`

**Checkpoint**: SqlSingle can be created, inspected, and rejects invalid inputs — all US1 acceptance scenarios pass

---

## Phase 4: User Story 2 — Arithmetic (Priority: P1)

**Goal**: Checked add/sub/mul/div with overflow/NaN/Infinity detection, NULL propagation, divide-by-zero

**Independent Test**: Perform arithmetic ops, verify results, overflow errors, divide-by-zero, NULL propagation

### Tests for User Story 2

- [ ] T013 [P] [US2] Tests for `checked_add` (normal, overflow, NULL) in `src/sql_single.rs`
- [ ] T014 [P] [US2] Tests for `checked_sub` (normal, overflow, NULL) in `src/sql_single.rs`
- [ ] T015 [P] [US2] Tests for `checked_mul` (normal, overflow, NULL) in `src/sql_single.rs`
- [ ] T016 [P] [US2] Tests for `checked_div` (normal, divide-by-zero, zero/zero, NULL) in `src/sql_single.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement `checked_add`, `checked_sub`, `checked_mul` with `is_finite()` post-check in `src/sql_single.rs`
- [ ] T018 [US2] Implement `checked_div` with divisor==0.0 pre-check and `is_finite()` post-check in `src/sql_single.rs`
- [ ] T019 [US2] Implement `Add/Sub/Mul/Div` operator traits (4 ref variants each) delegating to checked methods in `src/sql_single.rs`
- [ ] T020 [US2] Tests for operator trait variants (owned×owned, owned×ref, ref×owned, ref×ref) in `src/sql_single.rs`

**Checkpoint**: All four arithmetic operations work with overflow detection, divide-by-zero, and NULL propagation

---

## Phase 5: User Story 3 — Negation (Priority: P1)

**Goal**: Unary negation that inverts sign and propagates NULL

**Independent Test**: Negate values, verify sign inversion, negative zero, NULL propagation

### Tests for User Story 3

- [ ] T021 [P] [US3] Tests for negation (positive, negative, zero, negative zero, NULL) in `src/sql_single.rs`

### Implementation for User Story 3

- [ ] T022 [US3] Implement `Neg` trait (owned + ref variants, infallible, NULL propagates) in `src/sql_single.rs`

**Checkpoint**: Negation works correctly including IEEE 754 negative zero

---

## Phase 6: User Story 4 — SQL Comparisons (Priority: P2)

**Goal**: Six SQL comparison methods returning SqlBoolean with NULL propagation

**Independent Test**: Compare pairs of values, verify SqlBoolean return, NULL produces NULL

### Tests for User Story 4

- [ ] T023 [P] [US4] Tests for all 6 comparison methods (equal, unequal, less, greater, NULL) in `src/sql_single.rs`

### Implementation for User Story 4

- [ ] T024 [US4] Implement `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` in `src/sql_single.rs`

**Checkpoint**: All six SQL comparison methods work with three-valued NULL logic

---

## Phase 7: User Story 5 — Display and Parsing (Priority: P2)

**Goal**: Display formatting (NULL → "Null") and FromStr parsing with NaN/Infinity rejection

**Independent Test**: Format values, parse strings, verify round-trip, reject invalid inputs

### Tests for User Story 5

- [ ] T025 [P] [US5] Tests for `Display` (values, NULL, zero) in `src/sql_single.rs`
- [ ] T026 [P] [US5] Tests for `FromStr` (valid, invalid, "Null", "NaN", "Infinity", overflow) in `src/sql_single.rs`

### Implementation for User Story 5

- [ ] T027 [US5] Implement `Display` trait (NULL → "Null", value → f32 default) in `src/sql_single.rs`
- [ ] T028 [US5] Implement `FromStr` trait (case-insensitive "Null", validate is_finite, reject NaN/Infinity) in `src/sql_single.rs`

**Checkpoint**: Display and FromStr round-trip correctly, invalid inputs rejected

---

## Phase 8: User Story 6 — Conversions (Priority: P3)

**Goal**: Widening from integer types/SqlBoolean/SqlMoney, narrowing from SqlDouble, widening to SqlDouble/SqlBoolean

**Independent Test**: Convert values between types, verify results, errors, NULL propagation

### Tests for User Story 6

- [ ] T029 [P] [US6] Tests for `from_sql_byte`, `from_sql_int16`, `from_sql_int32`, `from_sql_int64` in `src/sql_single.rs`
- [ ] T030 [P] [US6] Tests for `from_sql_boolean` (TRUE→1.0, FALSE→0.0, NULL→NULL) in `src/sql_single.rs`
- [ ] T031 [P] [US6] Tests for `from_sql_money` (via f64 intermediate, NULL) in `src/sql_single.rs`
- [ ] T032 [P] [US6] Tests for `to_sql_double` (widening lossless, NULL) in `src/sql_single.rs`
- [ ] T033 [P] [US6] Tests for `from_sql_double` (narrowing, overflow for f64::MAX, NULL) in `src/sql_single.rs`
- [ ] T034 [P] [US6] Tests for `to_sql_boolean` (zero→FALSE, non-zero→TRUE, NULL→NULL) in `src/sql_single.rs`

### Implementation for User Story 6

- [ ] T035 [US6] Implement `from_sql_byte`, `from_sql_int16`, `from_sql_int32` (widening via `as f32`) in `src/sql_single.rs`
- [ ] T036 [US6] Implement `from_sql_int64` (widening, may lose precision) in `src/sql_single.rs`
- [ ] T037 [US6] Implement `from_sql_boolean` (TRUE→1.0, FALSE→0.0, NULL→NULL) in `src/sql_single.rs`
- [ ] T038 [US6] Implement `from_sql_money` (via `scaled_value() as f64 / 10_000.0` then `as f32`) in `src/sql_single.rs`
- [ ] T039 [US6] Implement `to_sql_double` (`f32 as f64` widening, lossless, NULL propagates) in `src/sql_single.rs`
- [ ] T040 [US6] Implement `from_sql_double` (`f64 as f32` narrowing, validate `is_finite()`, Err(Overflow)) in `src/sql_single.rs`
- [ ] T041 [US6] Implement `to_sql_boolean` (0.0→FALSE, non-zero→TRUE, NULL→NULL) in `src/sql_single.rs`

**Checkpoint**: All conversions work with proper NULL propagation and precision behavior

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Standard traits (Eq/Hash/Ord), formatting, linting, quickstart validation

- [ ] T042 [P] Tests for `PartialEq`/`Eq` (NULL==NULL, value equality, -0.0==0.0) in `src/sql_single.rs`
- [ ] T043 [P] Tests for `Hash` (consistent with Eq, -0.0 normalization) in `src/sql_single.rs`
- [ ] T044 [P] Tests for `PartialOrd`/`Ord` (NULL < non-NULL, value ordering) in `src/sql_single.rs`
- [ ] T045 Implement `PartialEq`/`Eq` (manual, safe because NaN excluded) in `src/sql_single.rs`
- [ ] T046 Implement `Hash` (f32::to_bits with -0.0→+0.0 normalization) in `src/sql_single.rs`
- [ ] T047 Implement `PartialOrd`/`Ord` (NULL < non-NULL, then f32 ordering) in `src/sql_single.rs`
- [ ] T048 Run `cargo fmt` and `cargo clippy -- -D warnings` — fix any issues
- [ ] T049 Run full test suite `cargo test` — verify zero regressions
- [ ] T050 Run quickstart.md validation tests — verify all examples work

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational
- **US2 (Phase 4)**: Depends on US1 (needs constructors)
- **US3 (Phase 5)**: Depends on US1 (needs constructors)
- **US4 (Phase 6)**: Depends on US1 (needs constructors)
- **US5 (Phase 7)**: Depends on US1 (needs constructors)
- **US6 (Phase 8)**: Depends on US1 (needs constructors); `to_sql_double` depends on SqlDouble being available
- **Polish (Phase 9)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Foundation only — no cross-story dependencies
- **US2 (P1)**: Foundation + US1 constructors
- **US3 (P1)**: Foundation + US1 constructors — can parallel with US2
- **US4 (P2)**: Foundation + US1 constructors — can parallel with US2/US3
- **US5 (P2)**: Foundation + US1 constructors — can parallel with US2/US3/US4
- **US6 (P3)**: Foundation + US1 constructors — can parallel with others

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Implementation follows test structure
- Story complete before moving to next priority (sequential) or in parallel if independent

### Parallel Opportunities

- T007-T010: All US1 tests can run in parallel
- T013-T016: All US2 tests can run in parallel
- US3, US4, US5 can each start after US1 completes (independent of each other)
- T029-T034: All US6 tests can run in parallel
- T042-T044: All Polish tests can run in parallel

---

## Parallel Example: User Story 2

```bash
# Write all arithmetic tests in parallel:
Task T013: "Tests for checked_add (normal, overflow, NULL)"
Task T014: "Tests for checked_sub (normal, overflow, NULL)"
Task T015: "Tests for checked_mul (normal, overflow, NULL)"
Task T016: "Tests for checked_div (normal, divide-by-zero, zero/zero, NULL)"

# Then implement sequentially:
Task T017: "Implement checked_add, checked_sub, checked_mul"
Task T018: "Implement checked_div"
Task T019: "Implement operator traits"
Task T020: "Operator trait variant tests"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T006)
3. Complete Phase 3: User Story 1 (T007-T012)
4. **STOP and VALIDATE**: SqlSingle can be created, inspected, rejects NaN/Infinity
5. Continue to User Story 2+

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1: Create/Inspect → MVP — safe f32 construction ✓
3. US2: Arithmetic → checked add/sub/mul/div ✓
4. US3: Negation → unary minus ✓
5. US4: Comparisons → SQL three-valued logic ✓
6. US5: Display/Parse → string conversion ✓
7. US6: Conversions → cross-type interop ✓
8. Polish → Eq/Hash/Ord, quality gates ✓

### Single-Developer Strategy

1. Work through phases sequentially (Phase 1 → 2 → 3 → ... → 9)
2. Within each phase: write all tests first, then implement
3. Commit after each phase checkpoint
4. Run `cargo test`, `cargo clippy`, `cargo fmt` at each checkpoint

---

## Notes

- All 50 tasks target `src/sql_single.rs` (single-file type implementation)
- SqlDouble (`src/sql_double.rs`) must exist for `to_sql_double()`/`from_sql_double()` — verified available on current branch
- Follow SqlDouble pattern exactly (f64 sibling) — adjust types from f64→f32
- Research decisions R1-R11 from research.md guide all implementation choices
- Use `f32::is_finite()` for all NaN/Infinity checks (R1)
- Division pre-checks `divisor == 0.0` before compute (R2)
- Hash uses `f32::to_bits()` with -0.0 normalization (R3)
