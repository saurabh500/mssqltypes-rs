# Tasks: SqlDouble

**Input**: Design documents from `/specs/009-sql-double/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-002) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_double.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [x] T001 Create `src/sql_double.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_byte::SqlByte`, `crate::sql_int16::SqlInt16`, `crate::sql_int32::SqlInt32`, `crate::sql_int64::SqlInt64`, `crate::sql_money::SqlMoney`, std traits), and empty struct definition
- [x] T002 Register module in `src/lib.rs`: add `pub mod sql_double;` and `pub use sql_double::SqlDouble;`

**Checkpoint**: `cargo build` compiles with empty `SqlDouble` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, constructors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the struct must exist for test code to compile. Tests in Phase 3 (T007–T010) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt64.

- [x] T003 Define `SqlDouble` struct with `value: Option<f64>`, derive `Copy, Clone, Debug` in `src/sql_double.rs`
- [x] T004 Implement constants `NULL`, `ZERO`, `MIN_VALUE` (`f64::MIN`), `MAX_VALUE` (`f64::MAX`) in `src/sql_double.rs`
- [x] T005 Implement `new(value: f64) -> Result<SqlDouble, SqlTypeError>` (reject NaN/Infinity via `is_finite()`), `is_null() -> bool`, `value() -> Result<f64, SqlTypeError>` in `src/sql_double.rs`
- [x] T006 Implement `From<f64> for SqlDouble` (panics on NaN/Infinity, per R8) in `src/sql_double.rs`

**Checkpoint**: Foundation ready — `SqlDouble::new()`, `is_null()`, `value()`, constants all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlDouble values from f64, inspect them, and rely on NaN/Infinity rejection at construction time.

**Independent Test**: `SqlDouble::new(3.14159265358979).unwrap().value()` returns `Ok(3.14159265358979)`, `SqlDouble::NULL.is_null()` returns `true`, `SqlDouble::new(f64::NAN)` returns `Err(Overflow)`, boundary values (`f64::MIN`, `f64::MAX`) round-trip correctly.

### Tests for User Story 1

- [x] T007 [US1] Write tests for `new()`, `is_null()`, `value()` — positive value (3.14159265358979), negative value (-2.718281828), zero (0.0), NULL access returns `Err(NullValue)` in `src/sql_double.rs`
- [x] T008 [US1] Write tests for constants — `NULL.is_null()`, `ZERO.value() == 0.0`, `MIN_VALUE.value() == f64::MIN`, `MAX_VALUE.value() == f64::MAX` in `src/sql_double.rs`
- [x] T009 [US1] Write tests for NaN/Infinity rejection — `new(f64::NAN)` returns `Err(Overflow)`, `new(f64::INFINITY)` returns `Err(Overflow)`, `new(f64::NEG_INFINITY)` returns `Err(Overflow)` in `src/sql_double.rs`
- [x] T010 [US1] Write tests for `From<f64>` — `SqlDouble::from(42.0).value() == 42.0`, `SqlDouble::from(f64::MIN)`, `SqlDouble::from(f64::MAX)`, and `#[should_panic]` tests for `From<f64>` with NaN and Infinity in `src/sql_double.rs`

**Checkpoint**: All US1 acceptance scenarios (11 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Arithmetic with NaN/Infinity Detection (Priority: P1)

**Goal**: Users can perform checked arithmetic (+, -, *, /) with proper NaN/Infinity rejection after each operation. Division by zero is detected before computing. NULL propagates through all operations.

**Independent Test**: `SqlDouble(2.5) + SqlDouble(3.5)` returns `Ok(SqlDouble(6.0))`, `SqlDouble(f64::MAX) + SqlDouble(f64::MAX)` returns `Err(Overflow)` (result is Infinity), `SqlDouble(1.0) / SqlDouble(0.0)` returns `Err(DivideByZero)`, `SqlDouble(0.0) / SqlDouble(0.0)` returns `Err(DivideByZero)`, arithmetic with NULL returns NULL.

### Tests for User Story 2

- [x] T011 [P] [US2] Write tests for `checked_add` — normal addition (2.5+3.5=6.0), overflow (MAX+MAX→Infinity→`Err(Overflow)`), NULL propagation (both sides) in `src/sql_double.rs`
- [x] T012 [P] [US2] Write tests for `checked_sub` — normal subtraction (10.0-3.0=7.0), overflow (-MAX-MAX→-Infinity→`Err(Overflow)`), NULL propagation in `src/sql_double.rs`
- [x] T013 [P] [US2] Write tests for `checked_mul` — normal multiply (4.0*2.5=10.0), overflow (MAX*2.0→Infinity→`Err(Overflow)`), NULL propagation in `src/sql_double.rs`
- [x] T014 [P] [US2] Write tests for `checked_div` — normal division (10.0/4.0=2.5), divide-by-zero (1.0/0.0→`Err(DivideByZero)`), 0.0/0.0→`Err(DivideByZero)`, NULL propagation in `src/sql_double.rs`

### Implementation for User Story 2

- [x] T015 [US2] Implement `checked_add`, `checked_sub` — perform f64 arithmetic, check `is_finite()` on result, NULL propagation in `src/sql_double.rs`
- [x] T016 [US2] Implement `checked_mul` — perform f64 arithmetic, check `is_finite()` on result, NULL propagation in `src/sql_double.rs`
- [x] T017 [US2] Implement `checked_div` — check divisor `== 0.0` → `DivideByZero` (per R2), then compute and check `is_finite()`, NULL propagation in `src/sql_double.rs`
- [x] T018 [US2] Implement operator traits `Add`, `Sub`, `Mul`, `Div` with `Output = Result<SqlDouble, SqlTypeError>` delegating to `checked_*` methods, also for `&SqlDouble` references in `src/sql_double.rs`

**Checkpoint**: All US2 acceptance scenarios (10 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Negation (Priority: P1)

**Goal**: Users can negate SqlDouble values. Negation is infallible (finite f64 negation always produces a finite result). NULL propagates.

**Independent Test**: `-SqlDouble(5.0)` returns `SqlDouble(-5.0)`, `-SqlDouble(-3.14)` returns `SqlDouble(3.14)`, `-SqlDouble(0.0)` returns `SqlDouble(-0.0)` (IEEE 754), `-SqlDouble::NULL` returns `SqlDouble::NULL`.

### Tests for User Story 3

- [x] T019 [P] [US3] Write tests for `Neg` — positive→negative, negative→positive, zero→-0.0 (verify via `f64::is_sign_negative()`), NULL→NULL in `src/sql_double.rs`

### Implementation for User Story 3

- [x] T020 [US3] Implement `Neg` trait for `SqlDouble` (infallible, per R5) — negate inner f64, propagate NULL, also for `&SqlDouble` in `src/sql_double.rs`

**Checkpoint**: All US3 acceptance scenarios (4 scenarios) pass. `cargo test` green.

---

## Phase 6: User Story 4 — Comparison Returning SqlBoolean (Priority: P2)

**Goal**: Users can compare SqlDouble values using SQL three-valued logic. Comparisons return `SqlBoolean`, and any comparison involving NULL returns `SqlBoolean::NULL`.

**Independent Test**: `SqlDouble(1.0).sql_equals(&SqlDouble(1.0))` returns `SqlBoolean::TRUE`, `SqlDouble(1.0).sql_less_than(&SqlDouble(2.0))` returns `SqlBoolean::TRUE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 4

- [x] T021 [P] [US4] Write tests for 6 SQL comparison methods — `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` with equal values, unequal values, and NULL propagation on both sides in `src/sql_double.rs`

### Implementation for User Story 4

- [x] T022 [US4] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) returning `SqlBoolean`, using `f64::partial_cmp()` for ordering in `src/sql_double.rs`

**Checkpoint**: All US4 acceptance scenarios (8 scenarios) pass. `cargo test` green.

---

## Phase 7: User Story 5 — Display and Parsing (Priority: P2)

**Goal**: Users can convert SqlDouble to and from string representations. NULL displays as `"Null"`. Parsing rejects NaN/Infinity strings and invalid inputs.

**Independent Test**: `format!("{}", SqlDouble(3.14159265358979))` returns `"3.14159265358979"`, `format!("{}", SqlDouble::NULL)` returns `"Null"`, `"3.14".parse::<SqlDouble>()` returns `Ok(SqlDouble(3.14))`, `"NaN".parse::<SqlDouble>()` returns error, `"abc".parse::<SqlDouble>()` returns error.

### Tests for User Story 5

- [x] T023 [P] [US5] Write tests for `Display` — positive (3.14159265358979), negative (-2.718), zero (0), NULL displays `"Null"` in `src/sql_double.rs`
- [x] T024 [P] [US5] Write tests for `FromStr` — valid number (`"3.14159265358979"`), `"Null"` → NULL (case-insensitive), `"NaN"` → error, `"Infinity"` → error, `"-Infinity"` → error, non-numeric string (`"abc"`) → `ParseError`, extremely large string → `Overflow` in `src/sql_double.rs`

### Implementation for User Story 5

- [x] T025 [US5] Implement `Display` for SqlDouble (`"Null"` for NULL, default f64 Display otherwise, per R10) in `src/sql_double.rs`
- [x] T026 [US5] Implement `FromStr` for SqlDouble — `"Null"` (case-insensitive) → NULL, parse f64, then validate `is_finite()`, else `ParseError` or `Overflow` in `src/sql_double.rs`

**Checkpoint**: All US5 acceptance scenarios (8 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 8: User Story 6 — Conversions to and from Other SqlTypes (Priority: P3)

**Goal**: Users can convert between SqlDouble and other SQL types. Widening conversions from integer types, SqlMoney, and SqlBoolean are infallible. `to_sql_boolean()` maps zero→FALSE, non-zero→TRUE, NULL→NULL. SqlSingle conversions are DEFERRED (per R7).

**Independent Test**: `SqlDouble::from_sql_byte(SqlByte(42)).value() == 42.0`, `SqlDouble::from_sql_int64(SqlInt64(1_000_000_000)).value() == 1_000_000_000.0`, `SqlDouble::from_sql_money(SqlMoney(42.5)).value() == 42.5`, `SqlDouble::from_sql_boolean(SqlBoolean::TRUE).value() == 1.0`, `SqlDouble(42.0).to_sql_boolean() == SqlBoolean::TRUE`, `SqlDouble(0.0).to_sql_boolean() == SqlBoolean::FALSE`, NULL propagates in all conversions.

### Tests for User Story 6

- [x] T027 [P] [US6] Write tests for `from_sql_byte`, `from_sql_int16`, `from_sql_int32` — representative values (42, 1000, 100_000), NULL propagation in `src/sql_double.rs`
- [x] T028 [P] [US6] Write tests for `from_sql_int64` — representative values (1_000_000_000), large values near i64::MAX (precision loss expected), NULL propagation in `src/sql_double.rs`
- [x] T029 [P] [US6] Write tests for `from_sql_money` — representative values (42.5, -100.1234), NULL propagation in `src/sql_double.rs`
- [x] T030 [P] [US6] Write tests for `from_sql_boolean` — TRUE→1.0, FALSE→0.0, NULL→NULL in `src/sql_double.rs`
- [x] T031 [P] [US6] Write tests for `to_sql_boolean` — non-zero (42.0)→TRUE, zero (0.0)→FALSE, negative (-1.5)→TRUE, NULL→NULL in `src/sql_double.rs`

### Implementation for User Story 6

- [x] T032 [US6] Implement `from_sql_byte`, `from_sql_int16`, `from_sql_int32` — widening via `as f64`, NULL propagation in `src/sql_double.rs`
- [x] T033 [US6] Implement `from_sql_int64` — widening via `i64 as f64` (may lose precision for large values, per R9/data-model), NULL propagation in `src/sql_double.rs`
- [x] T034 [US6] Implement `from_sql_money` — extract inner i64, divide by 10_000.0 (per R9), NULL propagation in `src/sql_double.rs`
- [x] T035 [US6] Implement `from_sql_boolean` — TRUE=1.0, FALSE=0.0, NULL=NULL in `src/sql_double.rs`
- [x] T036 [US6] Implement `to_sql_boolean` — 0.0→FALSE, non-zero→TRUE, NULL→NULL in `src/sql_double.rs`

**Checkpoint**: All US6 acceptance scenarios (13 scenarios, excluding SqlSingle which is DEFERRED) pass. `cargo test` green.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits and final quality gates that span all user stories

### Tests

- [x] T037 [P] Write tests for `PartialEq`/`Eq` — value equality, NULL==NULL (Rust semantics), -0.0==0.0 (IEEE 754), different values not equal in `src/sql_double.rs`
- [x] T038 [P] Write tests for `Hash` — equal values hash equal, 0.0 and -0.0 hash equal (per R3: `-0.0` normalized to `0.0` before `to_bits()`), NULL hashes consistently in `src/sql_double.rs`
- [x] T039 [P] Write tests for `PartialOrd`/`Ord` — NULL < any value, negative < positive, MIN < MAX, equal values in `src/sql_double.rs`

### Implementation

- [x] T040 Implement `PartialEq`, `Eq` for SqlDouble — manual impl, safe because NaN excluded (per R4), `Option<f64>` inner equality in `src/sql_double.rs`
- [x] T041 Implement `Hash` for SqlDouble — `to_bits()` with `-0.0` normalization to `0.0` (per R3), NULL hashes as `0u64` in `src/sql_double.rs`
- [x] T042 Implement `PartialOrd`, `Ord` for SqlDouble — NULL < any non-NULL, then f64 total ordering in `src/sql_double.rs`
- [x] T043 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [x] T044 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants/NaN rejection
- **US2 (Phase 4)**: Depends on Phase 2 — arithmetic needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — negation needs constructors
- **US4 (Phase 6)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US5 (Phase 7)**: Depends on Phase 2 — Display/FromStr need constructors
- **US6 (Phase 8)**: Depends on Phase 2 — conversions need constructors + `SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlMoney` (all exist)
- **Polish (Phase 9)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P1)**: Standalone — only needs foundational struct/constants
- **US4 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US5 (P2)**: Standalone — only needs foundational struct/constants
- **US6 (P3)**: Standalone — only needs foundational struct + existing types (`SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlMoney` — all exist)
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T011–T014 (US2 test methods) can run in parallel — independent test functions
- T023–T024 (US5 tests) can run in parallel
- T027–T031 (US6 tests) can run in parallel
- T037–T039 (Phase 9 tests) can run in parallel
- US1–US6 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 2

```text
# Write all US2 tests in parallel (T011-T014):
T011: tests for checked_add
T012: tests for checked_sub
T013: tests for checked_mul
T014: tests for checked_div

# Then implement sequentially (T015-T018):
T015: checked_add, checked_sub
T016: checked_mul
T017: checked_div
T018: operator trait wiring (Add, Sub, Mul, Div + &SqlDouble refs)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 + 3)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: US1 tests + validation (T007–T010)
4. Complete Phase 4: US2 arithmetic (T011–T018)
5. Complete Phase 5: US3 negation (T019–T020)
6. **STOP and VALIDATE**: Struct, constants, NaN/Infinity rejection, arithmetic, negation all work

### Incremental Delivery

1. Setup + Foundational → module compiles
2. US1 → values can be created and inspected with NaN/Infinity rejection
3. US2 → arithmetic works with overflow/infinity detection
4. US3 → negation works (infallible)
5. US4 → SQL three-valued comparisons
6. US5 → Display and FromStr
7. US6 → cross-type conversions (SqlSingle DEFERRED)
8. Polish → standard traits (Eq, Hash, Ord), quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 44 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 4 |
| Phase 4 (US2) | 8 |
| Phase 5 (US3) | 2 |
| Phase 6 (US4) | 2 |
| Phase 7 (US5) | 4 |
| Phase 8 (US6) | 10 |
| Phase 9 (Polish) | 8 |
| Parallelizable tasks | 15 |
| Test tasks | 18 |
| Implementation tasks | 26 |
