# Tasks: Cross-Type Conversions

**Input**: Design documents from `/specs/014-cross-type-conversions/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/public-api.md ✅

**Tests**: Required — constitution mandates TDD; NFR-006 requires ≥3 tests per conversion (normal value, NULL, edge case).

**Organization**: Tasks grouped by user story (7 stories). Each story is independently testable. No new files — all 43 methods added to existing source files.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm all existing code compiles and tests pass before adding conversions

- [X] T001 Verify baseline: run `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — all must pass

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Not applicable — all 13 SQL types are already implemented, `SqlTypeError` already has all needed variants, and `lib.rs` already registers all modules. No blocking infrastructure work needed.

**⚠️ Proceed directly to User Story phases after T001 passes.**

---

## Phase 3: User Story 1 — Widening Integer Conversions (Priority: P1) 🎯 MVP

**Goal**: Implement 5 infallible `From` trait impls so smaller integer types widen into larger ones without data loss.

**Independent Test**: `SqlByte(200).into::<SqlInt32>()` returns `SqlInt32(200)`. `SqlInt32(2_000_000).into::<SqlInt64>()` returns `SqlInt64(2_000_000)`. NULL → NULL.

**Requirements**: FR-001 through FR-005

### Tests for User Story 1

> **Write these tests FIRST — they must FAIL before implementation (no `From` impls exist yet)**

- [X] T002 [P] [US1] Write tests for `From<SqlByte>` and `From<SqlInt16>` for `SqlInt32` in src/sql_int32.rs — test normal values, boundary values (0, u8::MAX, i16::MAX), and NULL propagation
- [X] T003 [P] [US1] Write tests for `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>` for `SqlInt64` in src/sql_int64.rs — test normal values, boundary values (0, u8::MAX, i16::MAX, i32::MAX, i32::MIN), and NULL propagation

### Implementation for User Story 1

- [X] T004 [P] [US1] Implement `From<SqlByte>` and `From<SqlInt16>` for `SqlInt32` in src/sql_int32.rs — NULL input → `SqlInt32::NULL`, otherwise widen via `i32::from(value)`
- [X] T005 [P] [US1] Implement `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>` for `SqlInt64` in src/sql_int64.rs — NULL input → `SqlInt64::NULL`, otherwise widen via `i64::from(value)`

**Checkpoint**: `cargo test` — all widening conversions pass, no regressions

---

## Phase 4: User Story 2 — Boolean ↔ Numeric Conversions (Priority: P1)

**Goal**: Implement `to_sql_boolean()` on `SqlInt32` and `SqlInt64` — non-zero → TRUE, zero → FALSE, NULL → NULL.

**Independent Test**: `SqlInt32(42).to_sql_boolean()` returns `SqlBoolean::TRUE`. `SqlInt64(0).to_sql_boolean()` returns `SqlBoolean::FALSE`. NULL → NULL.

**Requirements**: FR-006, FR-007

### Tests for User Story 2

- [X] T006 [P] [US2] Write tests for `SqlInt32::to_sql_boolean()` in src/sql_int32.rs — test zero→FALSE, positive→TRUE, negative→TRUE, i32::MAX→TRUE, NULL→NULL
- [X] T007 [P] [US2] Write tests for `SqlInt64::to_sql_boolean()` in src/sql_int64.rs — test zero→FALSE, positive→TRUE, negative→TRUE, i64::MAX→TRUE, NULL→NULL

### Implementation for User Story 2

- [X] T008 [P] [US2] Implement `SqlInt32::to_sql_boolean()` in src/sql_int32.rs — add `use crate::sql_boolean::SqlBoolean;`, return `SqlBoolean::NULL` if null, `SqlBoolean::FALSE` if 0, `SqlBoolean::TRUE` otherwise
- [X] T009 [P] [US2] Implement `SqlInt64::to_sql_boolean()` in src/sql_int64.rs — add `use crate::sql_boolean::SqlBoolean;`, return `SqlBoolean::NULL` if null, `SqlBoolean::FALSE` if 0, `SqlBoolean::TRUE` otherwise

**Checkpoint**: `cargo test` — boolean conversions pass, no regressions

---

## Phase 5: User Story 3 — Float ↔ Float Conversions (Priority: P1)

**Goal**: Implement `SqlDouble::from_sql_single()` (widening) and `SqlDouble::to_sql_single()` (narrowing with overflow check).

**Independent Test**: `SqlDouble::from_sql_single(SqlSingle(3.14))` returns `SqlDouble` with widened value. `SqlDouble(1e300).to_sql_single()` returns `Err(Overflow)`.

**Requirements**: FR-008, FR-009

### Tests for User Story 3

- [X] T010 [US3] Write tests for `SqlDouble::from_sql_single()` and `SqlDouble::to_sql_single()` in src/sql_double.rs — test widening normal value, narrowing normal value, narrowing overflow (1e300→f32 = infinity), NULL propagation for both, edge case f32::MAX round-trip

### Implementation for User Story 3

- [X] T011 [US3] Implement `SqlDouble::from_sql_single()` and `SqlDouble::to_sql_single()` in src/sql_double.rs — `from_sql_single`: widen f32→f64, NULL→NULL; `to_sql_single`: cast f64 as f32, check if result is infinite when input was finite → `Err(Overflow)`, NULL→`Ok(SqlSingle::NULL)` (per R5)

**Checkpoint**: `cargo test` — float conversions pass, no regressions

---

## Phase 6: User Story 4 — SqlString ↔ All Types (Priority: P2)

**Goal**: Add `to_sql_string()` on all 11 non-SqlString types (formatting via `Display`) and 11 `SqlString::to_sql_*()` parsing methods (via `FromStr`). 22 methods total.

**Independent Test**: `SqlInt32(42).to_sql_string()` returns `SqlString("42")`. `SqlString("42").to_sql_int32()` returns `Ok(SqlInt32(42))`. `SqlString("bad").to_sql_int32()` returns `Err(ParseError)`. NULL ↔ NULL.

**Requirements**: FR-020 through FR-041

### to_sql_string() — Formatting (11 methods across 11 files)

> **Each task writes tests first, then implements. All tasks are parallel (different files).**
> **Pattern**: If null → `SqlString::NULL`, else → `SqlString::new(format!("{}", value))`. Add `use crate::sql_string::SqlString;` import.
> **Tests**: Normal value, NULL→NULL, edge case (boundary value or special format).

- [X] T012 [P] [US4] Write tests and implement `SqlBoolean::to_sql_string()` in src/sql_boolean.rs — TRUE→"True", FALSE→"False", NULL→NULL (per R3)
- [X] T013 [P] [US4] Write tests and implement `SqlByte::to_sql_string()` in src/sql_byte.rs — normal value, 0, 255, NULL
- [X] T014 [P] [US4] Write tests and implement `SqlInt16::to_sql_string()` in src/sql_int16.rs — normal value, i16::MIN, i16::MAX, NULL
- [X] T015 [P] [US4] Write tests and implement `SqlInt32::to_sql_string()` in src/sql_int32.rs — normal value, i32::MIN, i32::MAX, NULL
- [X] T016 [P] [US4] Write tests and implement `SqlInt64::to_sql_string()` in src/sql_int64.rs — normal value, i64::MIN, i64::MAX, NULL
- [X] T017 [P] [US4] Write tests and implement `SqlSingle::to_sql_string()` in src/sql_single.rs — normal value (3.14), negative, NULL
- [X] T018 [P] [US4] Write tests and implement `SqlDouble::to_sql_string()` in src/sql_double.rs — normal value (3.14), negative, NULL
- [X] T019 [P] [US4] Write tests and implement `SqlDecimal::to_sql_string()` in src/sql_decimal.rs — normal value, zero, negative, NULL
- [X] T020 [P] [US4] Write tests and implement `SqlMoney::to_sql_string()` in src/sql_money.rs — normal value, zero, negative, NULL
- [X] T021 [P] [US4] Write tests and implement `SqlDateTime::to_sql_string()` in src/sql_datetime.rs — normal date, boundary dates (1753-01-01, 9999-12-31), NULL
- [X] T022 [P] [US4] Write tests and implement `SqlGuid::to_sql_string()` in src/sql_guid.rs — normal GUID, all-zeros, NULL

### SqlString::to_sql_*() — Parsing Hub (11 methods in src/sql_string.rs)

> **Pattern**: If null → `Ok(T::NULL)`, else → `self.value().unwrap().parse::<InnerType>()` mapped to target type and `SqlTypeError::ParseError` on failure.
> **Imports**: Add `use crate::sql_*::Sql*;` for each target type.

- [X] T023 [US4] Write tests for all 11 `SqlString::to_sql_*()` parsing methods in src/sql_string.rs — for each: valid parse, invalid parse→ParseError, NULL→Ok(NULL), edge cases (overflow for integers, NaN/Infinity rejection for floats)
- [X] T024 [US4] Implement `SqlString::to_sql_boolean()`, `to_sql_byte()`, `to_sql_int16()`, `to_sql_int32()`, `to_sql_int64()` in src/sql_string.rs — delegate to target type's `FromStr` impl
- [X] T025 [US4] Implement `SqlString::to_sql_single()`, `to_sql_double()`, `to_sql_decimal()`, `to_sql_money()` in src/sql_string.rs — delegate to target type's `FromStr` impl, reject NaN/Infinity for float results
- [X] T026 [US4] Implement `SqlString::to_sql_date_time()` and `to_sql_guid()` in src/sql_string.rs — delegate to target type's `FromStr` impl

**Checkpoint**: `cargo test` — all string conversions pass. Verify round-trip: `value.to_sql_string() → SqlString::to_sql_*() → original value` for each type.

---

## Phase 7: User Story 5 — SqlDecimal ↔ Float/Money Conversions (Priority: P2)

**Goal**: Implement `From<SqlSingle>`, `From<SqlDouble>`, `From<SqlMoney>` for `SqlDecimal`, plus `SqlDecimal::to_sql_single()`, `to_sql_double()`, `to_sql_money()`. 6 methods total.

**Independent Test**: `SqlDecimal::from(SqlDouble(100.50)).to_sql_double()` returns `SqlDouble(100.50)`. `SqlDecimal::from(SqlMoney)` preserves 4-decimal scale. `to_sql_money()` returns `Err(Overflow)` for out-of-range values.

**Requirements**: FR-010 through FR-015

### Tests for User Story 5

- [X] T027 [US5] Write tests for `From<SqlSingle>`, `From<SqlDouble>`, `From<SqlMoney>` for `SqlDecimal` in src/sql_decimal.rs — normal values, NULL propagation, NaN/Infinity inputs must panic (per contract: `From` is infallible, NaN/Inf panics match C# OverflowException), SqlMoney preserves 4-decimal scale
- [X] T028 [US5] Write tests for `SqlDecimal::to_sql_single()`, `to_sql_double()`, `to_sql_money()` in src/sql_decimal.rs — normal values, NULL→NULL, to_sql_money overflow for values outside ±922,337,203,685,477.5807

### Implementation for User Story 5

- [X] T029 [US5] Implement `From<SqlSingle>`, `From<SqlDouble>`, `From<SqlMoney>` for `SqlDecimal` in src/sql_decimal.rs — add imports, NULL→NULL, float→decimal via string intermediary or direct conversion, money→decimal preserves 4 decimal places, panic on NaN/Infinity (per R4 contract note)
- [X] T030 [US5] Implement `SqlDecimal::to_sql_single()`, `to_sql_double()`, `to_sql_money()` in src/sql_decimal.rs — to_sql_single/double: convert to f32/f64 (precision loss acceptable), NULL→NULL; to_sql_money: convert to i64×10000, range-check, `Err(Overflow)` if out of range

**Checkpoint**: `cargo test` — all decimal↔float/money conversions pass

---

## Phase 8: User Story 6 — SqlMoney ↔ Float Conversions (Priority: P2)

**Goal**: Implement `SqlMoney::from_sql_single()`, `from_sql_double()`, `to_sql_single()`, `to_sql_double()`. 4 methods total.

**Independent Test**: `SqlMoney::from_sql_double(SqlDouble(100.50))` returns `Ok(SqlMoney)` with value `100.5000`. `SqlMoney::from_sql_double(SqlDouble(1e18))` returns `Err(Overflow)`. NULL ↔ NULL.

**Requirements**: FR-016 through FR-019

### Tests for User Story 6

- [X] T031 [US6] Write tests for `SqlMoney::from_sql_single()`, `from_sql_double()`, `to_sql_single()`, `to_sql_double()` in src/sql_money.rs — normal values, NULL→NULL/Ok(NULL), overflow on from_sql_double(1e18), precision edge cases, to_sql_single/double divides internal i64 by 10,000 (per R6)

### Implementation for User Story 6

- [X] T032 [US6] Implement `SqlMoney::from_sql_single()` and `from_sql_double()` in src/sql_money.rs — NULL→Ok(NULL), convert via `(f64 * 10_000.0).round() as i64`, range-check against money bounds, `Err(Overflow)` if out of range (per R6)
- [X] T033 [US6] Implement `SqlMoney::to_sql_single()` and `to_sql_double()` in src/sql_money.rs — NULL→NULL, convert internal i64 to f32/f64 by dividing by 10_000.0

**Checkpoint**: `cargo test` — all money↔float conversions pass

---

## Phase 9: User Story 7 — SqlDateTime ↔ SqlString (Priority: P3)

**Goal**: Implement `SqlDateTime::from_sql_string()` — parse date string into `SqlDateTime`. (`to_sql_string()` already covered by US4/T021.)

**Independent Test**: `SqlDateTime::from_sql_string(&SqlString("2025-01-15 10:30:00"))` returns valid `SqlDateTime`. Invalid string → `Err(ParseError)`. NULL → `Ok(NULL)`.

**Requirements**: FR-042

### Tests for User Story 7

- [X] T034 [US7] Write tests for `SqlDateTime::from_sql_string()` in src/sql_datetime.rs — valid date string, invalid string→ParseError, NULL→Ok(NULL), boundary dates (1753-01-01, 9999-12-31), out-of-range dates

### Implementation for User Story 7

- [X] T035 [US7] Implement `SqlDateTime::from_sql_string()` in src/sql_datetime.rs — NULL→Ok(NULL), delegate to `FromStr` on the inner string value, map parse errors to `SqlTypeError::ParseError`

**Checkpoint**: `cargo test` — SqlDateTime↔SqlString conversions pass

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories

- [X] T036 Run `cargo fmt` to format all modified files
- [X] T037 Run `cargo clippy -- -D warnings` and fix any warnings
- [X] T038 Run `cargo test` — verify all 43 conversion methods pass with no regressions
- [X] T039 Run quickstart.md validation scenarios end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — verify baseline first
- **Foundational (Phase 2)**: N/A — all infrastructure already exists
- **US1–US3 (Phases 3–5)**: Depend on Phase 1 only. All three P1 stories can run in parallel.
- **US4 (Phase 6)**: Depends on Phase 1 only. Can run in parallel with US1–US3. The to_sql_string() tasks (T012–T022) are independent of SqlString parsing tasks (T023–T026).
- **US5 (Phase 7)**: Depends on Phase 1 only. Can run in parallel with other stories.
- **US6 (Phase 8)**: Depends on Phase 1 only. Can run in parallel with other stories.
- **US7 (Phase 9)**: Depends on Phase 1 only. Can run in parallel with other stories.
- **Polish (Phase 10)**: Depends on ALL user stories being complete

### User Story Dependencies

- **US1 (P1)**: Independent — no cross-story dependencies
- **US2 (P1)**: Independent — no cross-story dependencies
- **US3 (P1)**: Independent — no cross-story dependencies
- **US4 (P2)**: Independent — delegates to existing `Display`/`FromStr`, does NOT depend on new conversions from US1–US3
- **US5 (P2)**: Independent — does NOT depend on US4's `to_sql_string()` or US6's money↔float
- **US6 (P2)**: Independent — does NOT depend on US5's decimal↔money
- **US7 (P3)**: Independent — `from_sql_string()` delegates to existing `FromStr` on `SqlDateTime`

**All stories are independent** — they can be implemented in any order or in parallel.

### Within Each User Story

1. Tests MUST be written and FAIL before implementation (TDD)
2. Implementation makes tests pass
3. Checkpoint verification before moving to next story

### Parallel Opportunities

Within each phase, tasks marked [P] can run in parallel:

- **US1**: T002 ∥ T003 (tests), then T004 ∥ T005 (impl) — different files
- **US2**: T006 ∥ T007 (tests), then T008 ∥ T009 (impl) — different files
- **US4**: T012–T022 all parallel (11 different files for to_sql_string)
- **Cross-story**: US1 ∥ US2 ∥ US3 ∥ US4 ∥ US5 ∥ US6 ∥ US7 — all independent

---

## Parallel Example: User Story 4

```text
# Batch 1: All to_sql_string() tasks in parallel (11 different files)
T012 ∥ T013 ∥ T014 ∥ T015 ∥ T016 ∥ T017 ∥ T018 ∥ T019 ∥ T020 ∥ T021 ∥ T022

# Batch 2: SqlString parsing tests (single file, sequential)
T023

# Batch 3: SqlString parsing implementations (single file, sequential)
T024 → T025 → T026
```

---

## Implementation Strategy

### MVP First (P1 Stories Only)

1. Complete Phase 1: Baseline verification
2. Complete Phase 3: US1 — Widening integer conversions (5 methods)
3. Complete Phase 4: US2 — Boolean↔numeric conversions (2 methods)
4. Complete Phase 5: US3 — Float↔float conversions (2 methods)
5. **STOP and VALIDATE**: 9 methods implemented, `cargo test` passes
6. This is a shippable increment — most common conversions work

### Incremental Delivery

1. P1 stories (US1+US2+US3) → 9 methods → Test independently → MVP ✓
2. Add US4 (String hub) → 22 methods → Test independently → 31 methods total
3. Add US5+US6 (Decimal/Money) → 10 methods → Test independently → 41 methods total
4. Add US7 (DateTime↔String) → 1 method → Test independently → 42 unique methods total
5. Polish phase → Final validation → Feature complete

### Sequential Agent Strategy

For a single LLM agent implementing all tasks sequentially:

1. Complete T001 (baseline)
2. US1: T002 → T003 → T004 → T005 (tests first, then impl, across files)
3. US2: T006 → T007 → T008 → T009
4. US3: T010 → T011
5. US4: T012–T022 (one file at a time), then T023 → T024 → T025 → T026
6. US5: T027 → T028 → T029 → T030
7. US6: T031 → T032 → T033
8. US7: T034 → T035
9. Polish: T036 → T037 → T038 → T039

---

## Summary

| Metric | Value |
|--------|-------|
| Total tasks | 39 |
| User stories | 7 |
| Requirements covered | FR-001 through FR-043 (42 unique, FR-043 = FR-040) |
| Source files modified | 11 (no new files) |
| Parallel opportunities | 19 tasks marked [P] across 4 phases |
| Estimated LOC | ~400–600 impl + ~800–1200 tests |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All conversions follow NULL propagation: NULL input → NULL output (or `Ok(T::NULL)` for fallible)
- `From` impls go in **target** type's file; `to_sql_*()` goes on **source** type (per R9)
- `SqlString::to_sql_*()` parsing delegates to target type's `FromStr` (per R2)
- `to_sql_string()` delegates to type's `Display` impl (per R2)
- NaN/Infinity → `SqlDecimal` panics in `From` impl (matches C# OverflowException behavior)
- SqlDouble→SqlSingle overflow: post-cast infinity check (per R5)
- SqlMoney↔float: direct `i64 × 10,000` conversion, no decimal intermediary (per R6)
- Commit after each user story checkpoint
