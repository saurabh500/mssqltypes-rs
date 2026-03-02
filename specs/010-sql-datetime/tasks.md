# Tasks: SqlDateTime

**Input**: Design documents from `/specs/010-sql-datetime/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-005) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_datetime.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [X] T001 Create `src/sql_datetime.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `std::fmt`, `std::str::FromStr`, `std::hash`, `std::cmp`), and empty struct definition
- [X] T002 Register module in `src/lib.rs`: add `pub mod sql_datetime;` and `pub use sql_datetime::SqlDateTime;`

**Checkpoint**: `cargo build` compiles with empty `SqlDateTime` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, internal calendar helpers, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, internal helpers, constructors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the struct must exist for test code to compile. Tests in Phase 3 (T007–T012) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt64.

- [X] T003 Define `SqlDateTime` struct with `value: Option<(i32, i32)>`, derive `Copy, Clone, Debug` in `src/sql_datetime.rs`
- [X] T004 Implement public tick-rate constants `TICKS_PER_SECOND` (300), `TICKS_PER_MINUTE` (18_000), `TICKS_PER_HOUR` (1_080_000), `TICKS_PER_DAY` (25_920_000) and sentinel constants `NULL`, `MIN_VALUE` ((-53690, 0)), `MAX_VALUE` ((2958463, 25919999)) in `src/sql_datetime.rs`
- [X] T005 Implement private constants `DAY_BASE` (693_595), `MIN_DAY` (-53_690), `MAX_DAY` (2_958_463), `MAX_TIME` (25_919_999), `DAYS_TO_MONTH_365`, `DAYS_TO_MONTH_366` lookup tables, and private helper `fn is_leap_year(year: i32) -> bool` in `src/sql_datetime.rs`
- [X] T006 Implement private calendar helpers: `fn date_to_day_ticks(year, month, day) -> Result<i32, SqlTypeError>` (Gregorian formula from R2), `fn time_to_ticks(hour, minute, second, millisecond) -> Result<(i32, i32), SqlTypeError>` returning `(time_ticks, day_carry)` with ms rounding formula `(ms * 0.3 + 0.5) as i32` and midnight overflow detection, and `fn day_ticks_to_ymd(day_ticks) -> (i32, i32, i32)` (400/100/4/1-year cycle decomposition from R3) in `src/sql_datetime.rs`
- [X] T007 Implement `from_ticks(day_ticks: i32, time_ticks: i32) -> Result<SqlDateTime, SqlTypeError>` with range validation (day ∈ [MIN_DAY, MAX_DAY], time ∈ [0, MAX_TIME]) in `src/sql_datetime.rs`
- [X] T008 Implement `new(year, month, day, hour, minute, second, millisecond: f64) -> Result<SqlDateTime, SqlTypeError>` using `date_to_day_ticks` + `time_to_ticks` with midnight rollover handling and final range validation in `src/sql_datetime.rs`
- [X] T009 Implement `is_null() -> bool`, `value() -> Result<(i32, i32), SqlTypeError>`, `day_ticks() -> Result<i32, SqlTypeError>`, `time_ticks() -> Result<i32, SqlTypeError>` in `src/sql_datetime.rs`

**Checkpoint**: Foundation ready — `SqlDateTime::new()`, `from_ticks()`, `is_null()`, `value()`, constants all work. `cargo build` passes.

---

## Phase 3: User Story 1 — Create and Inspect Values from Calendar Components (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlDateTime values from (year, month, day, hour, minute, second, millisecond) and inspect them via `day_ticks()` and `time_ticks()`.

**Independent Test**: `SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0)?.day_ticks()` returns correct value, `SqlDateTime::new(1900, 1, 1, 0, 0, 0, 0.0)?.day_ticks()` returns `0`, boundary values at MIN/MAX succeed, out-of-range components fail.

### Tests for User Story 1

- [X] T010 [US1] Write tests for `new()` with valid calendar components — epoch date (1900, 1, 1) → day_ticks=0, date before epoch (1899, 12, 31) → day_ticks=-1, representative date (2025, 7, 17, 12, 30, 0, 0.0), MIN_VALUE date (1753, 1, 1) → day_ticks=-53690, MAX_VALUE date (9999, 12, 31, 23, 59, 59, 997.0) → day_ticks=2958463 and time_ticks=25919999 in `src/sql_datetime.rs`
- [X] T011 [US1] Write tests for `new()` with invalid components — year 1752 (too low), year 10000 (too high), month 0, month 13, day 0, day 32, hour 24, minute 60, second 60, millisecond 1000.0, millisecond negative — all return `Err(OutOfRange)` in `src/sql_datetime.rs`
- [X] T012 [US1] Write tests for day-of-month validation — April 31 fails (30-day month), Feb 29 in non-leap year fails, valid month-end dates succeed (Jan 31, Jun 30, Feb 28) in `src/sql_datetime.rs`

**Checkpoint**: All US1 acceptance scenarios (11 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Create and Inspect Values from Raw Ticks (Priority: P1)

**Goal**: Users can create SqlDateTime from raw (day_ticks, time_ticks), access NULL sentinel, and use MIN_VALUE/MAX_VALUE constants.

**Independent Test**: `SqlDateTime::from_ticks(0, 0)?.day_ticks()` returns `0`, `SqlDateTime::NULL.is_null()` returns `true`, `SqlDateTime::NULL.value()` returns `Err(NullValue)`, out-of-range ticks fail.

### Tests for User Story 2

- [X] T013 [P] [US2] Write tests for `from_ticks()` — epoch (0, 0), MIN_VALUE (-53690, 0), MAX_VALUE (2958463, 25919999), out-of-range day_ticks (-53691 and 2958464), out-of-range time_ticks (-1 and 25920000) in `src/sql_datetime.rs`
- [X] T014 [P] [US2] Write tests for NULL and constants — `NULL.is_null()` true, `NULL.value()` returns `Err(NullValue)`, `MIN_VALUE.day_ticks()` returns `-53690`, `MAX_VALUE.time_ticks()` returns `25919999`, tick-rate constants are correct in `src/sql_datetime.rs`

**Checkpoint**: All US2 acceptance scenarios (11 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Millisecond Rounding to 1/300-Second Precision (Priority: P1)

**Goal**: Millisecond values are rounded to the nearest SQL tick using the C# formula `(int)(ms * 0.3 + 0.5)`. Time overflow at midnight rolls the day forward.

**Independent Test**: `SqlDateTime::new(2025, 1, 1, 0, 0, 0, 3.33)?.time_ticks()` returns `1`, midnight rollover at 23:59:59.998 increments day and resets time to 0.

### Tests for User Story 3

- [X] T015 [US3] Write tests for millisecond rounding — ms=0.0 → 0 ticks, ms=3.33 → 1 tick, ms=500.0 → correct per formula, ms=997.0 → expected ticks, at least 10 representative values per SC-002 in `src/sql_datetime.rs`
- [X] T016 [US3] Write tests for midnight rollover — construction with (2025, 1, 1, 23, 59, 59, 998.0) → time resets to 0 and day increments by 1, and (9999, 12, 31, 23, 59, 59, 998.0) → returns `Err(OutOfRange)` due to day overflow past MAX_DAY in `src/sql_datetime.rs`

**Checkpoint**: All US3 acceptance scenarios (5 scenarios) pass. SC-002 and SC-004 verified. `cargo test` green.

---

## Phase 6: User Story 4 — Duration Arithmetic (Priority: P1)

**Goal**: Users can add/subtract durations (day and time tick offsets) to SqlDateTime values with range validation and NULL propagation. Time overflow/underflow normalizes into day carry.

**Independent Test**: `SqlDateTime::new(2025, 1, 15, 12, 0, 0, 0.0)?.checked_add_days(1)?.day()` returns `16`, `MAX_VALUE.checked_add_ticks(1)` returns `Err(OutOfRange)`, `NULL.checked_add_days(1)` returns `Ok(NULL)`.

### Tests for User Story 4

- [X] T017 [P] [US4] Write tests for `checked_add` and `checked_add_days` — add 1 day, add negative days, day rollover from time overflow (23:00 + 2 hours), range error at MAX_VALUE + 1 tick, NULL propagation in `src/sql_datetime.rs`
- [X] T018 [P] [US4] Write tests for `checked_sub` — subtract 1 day, day rollback from time underflow (01:00 - 2 hours), range error at MIN_VALUE - 1 tick, NULL propagation in `src/sql_datetime.rs`
- [X] T019 [P] [US4] Write tests for `checked_add_ticks` — add ticks within same day, add ticks causing day rollover, subtract ticks (negative), convenience method equivalence with `checked_add(0, ticks)` in `src/sql_datetime.rs`

### Implementation for User Story 4

- [X] T020 [US4] Implement `checked_add(day_delta: i32, time_delta: i32)` — NULL propagation, i64 intermediate for time, `div_euclid`/`rem_euclid` normalization, range check on result day_ticks per R8 algorithm in `src/sql_datetime.rs`
- [X] T021 [US4] Implement `checked_sub(day_delta: i32, time_delta: i32)` — negate deltas and delegate to `checked_add` in `src/sql_datetime.rs`
- [X] T022 [US4] Implement convenience methods `checked_add_days(days: i32)` and `checked_add_ticks(ticks: i32)` delegating to `checked_add` in `src/sql_datetime.rs`

**Checkpoint**: All US4 acceptance scenarios (7 scenarios) pass. SC-007 verified. `cargo test` green.

---

## Phase 7: User Story 5 — Comparison Returning SqlBoolean (Priority: P2)

**Goal**: Users can compare SqlDateTime values using SQL three-valued logic. Comparisons return `SqlBoolean` with lexicographic (day_ticks, time_ticks) ordering. NULL propagation on either operand.

**Independent Test**: `SqlDateTime::new(2025, 1, 1, ..).sql_less_than(&SqlDateTime::new(2025, 1, 2, ..))` returns `SqlBoolean::TRUE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 5

- [X] T023 [US5] Write tests for 6 SQL comparison methods — `sql_equals` (same date/time → TRUE), `sql_not_equals` (different → TRUE), `sql_less_than` (earlier date → TRUE, same date earlier time → TRUE), `sql_greater_than`, `sql_less_than_or_equal` (equal → TRUE), `sql_greater_than_or_equal` (equal → TRUE), NULL propagation on both sides for all 6 methods in `src/sql_datetime.rs`

### Implementation for User Story 5

- [X] T024 [US5] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) with lexicographic (day_ticks, time_ticks) comparison and NULL propagation in `src/sql_datetime.rs`

**Checkpoint**: All US5 acceptance scenarios (9 scenarios) pass. SC-006 verified. `cargo test` green.

---

## Phase 8: User Story 6 — Display and Parsing (Priority: P2)

**Goal**: Users can convert SqlDateTime to and from string representations using ISO 8601-like format. NULL displays as `"Null"`. Parsing supports multiple format variants.

**Independent Test**: `format!("{}", SqlDateTime::new(2025, 7, 17, 12, 30, 0, 0.0)?)` returns `"2025-07-17 12:30:00.000"`, `"2025-07-17 12:30:00".parse::<SqlDateTime>()` returns correct value, `"abc".parse::<SqlDateTime>()` returns `ParseError`.

### Tests for User Story 6

- [X] T025 [P] [US6] Write tests for `Display` — representative date/time → `"YYYY-MM-DD HH:MM:SS.fff"` format, epoch → `"1900-01-01 00:00:00.000"`, NULL → `"Null"`, leading-zero padding for month/day/hour/minute/second in `src/sql_datetime.rs`
- [X] T026 [P] [US6] Write tests for `FromStr` — `"Null"` → NULL, `"2025-07-17 12:30:00"` → correct value, `"2025-07-17T12:30:00.000"` (T separator), `"2025-07-17"` (date only → midnight), `"abc"` → `ParseError`, `"1752-01-01 00:00:00"` → out-of-range error, `"10000-01-01 00:00:00"` → out-of-range error in `src/sql_datetime.rs`

### Implementation for User Story 6

- [X] T027 [US6] Implement `Display` for SqlDateTime — NULL → `"Null"`, non-NULL → extract year/month/day/hour/minute/second via internal helpers and format as `"YYYY-MM-DD HH:MM:SS.fff"` with zero-padded components in `src/sql_datetime.rs`
- [X] T028 [US6] Implement `FromStr` for SqlDateTime — handle `"Null"` → NULL; parse `"YYYY-MM-DD HH:MM:SS.fff"`, `"YYYY-MM-DD HH:MM:SS"`, `"YYYY-MM-DDTHH:MM:SS.fff"`, `"YYYY-MM-DDTHH:MM:SS"`, `"YYYY-MM-DD"` (time defaults to midnight); return `ParseError` for invalid format, `OutOfRange` for valid format but out-of-range date in `src/sql_datetime.rs`

**Checkpoint**: All US6 acceptance scenarios (6 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 9: User Story 7 — Leap Year and Calendar Correctness (Priority: P2)

**Goal**: Calendar calculations are correct around leap year boundaries. Feb 29 is valid in leap years, invalid in non-leap years.

**Independent Test**: `SqlDateTime::new(2024, 2, 29, ..)` succeeds (leap year), `SqlDateTime::new(2023, 2, 29, ..)` fails (not leap), `SqlDateTime::new(1900, 2, 29, ..)` fails (century rule), `SqlDateTime::new(2000, 2, 29, ..)` succeeds (400-year rule).

### Tests for User Story 7

- [X] T029 [US7] Write tests for leap year construction — 2024 Feb 29 succeeds (divisible by 4), 2023 Feb 29 fails (not divisible by 4), 2000 Feb 29 succeeds (divisible by 400), 1900 Feb 29 fails (divisible by 100 not 400), arithmetic across leap boundary (2024 Feb 28 + 1 day → Feb 29, 2023 Feb 28 + 1 day → Mar 1) in `src/sql_datetime.rs`

**Checkpoint**: All US7 acceptance scenarios (6 scenarios) pass. SC-003 verified. `cargo test` green.

---

## Phase 10: User Story 8 — Accessors and Component Extraction (Priority: P3)

**Goal**: Users can extract individual date/time components (year, month, day, hour, minute, second) from a SqlDateTime value. NULL returns `Err(NullValue)`.

**Independent Test**: `SqlDateTime::new(2025, 7, 17, 14, 30, 45, 333.0)?.year()` returns `2025`, `SqlDateTime::NULL.year()` returns `Err(NullValue)`.

### Tests for User Story 8

- [X] T030 [P] [US8] Write tests for date component extraction — `year()`, `month()`, `day()` for representative date (2025, 7, 17), epoch (1900, 1, 1), MIN_VALUE (1753, 1, 1), MAX_VALUE (9999, 12, 31), NULL → `Err(NullValue)` in `src/sql_datetime.rs`
- [X] T031 [P] [US8] Write tests for time component extraction — `hour()`, `minute()`, `second()` for representative time (14, 30, 45), midnight (0, 0, 0), max time (23, 59, 59), NULL → `Err(NullValue)` in `src/sql_datetime.rs`

### Implementation for User Story 8

- [X] T032 [US8] Implement `year()`, `month()`, `day()` using `day_ticks_to_ymd()` internal helper with NULL check in `src/sql_datetime.rs`
- [X] T033 [US8] Implement `hour()`, `minute()`, `second()` using integer division by tick-rate constants per R4 algorithm with NULL check in `src/sql_datetime.rs`

**Checkpoint**: All US8 acceptance scenarios (7 scenarios) pass. SC-008 verified (round-trip calendar components). `cargo test` green.

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits and final quality gates that span all user stories

### Tests

- [X] T034 [P] Write tests for `PartialEq`/`Eq` — same values equal, different values not equal, NULL==NULL (Rust semantics), NULL≠non-NULL in `src/sql_datetime.rs`
- [X] T035 [P] Write tests for `Hash` — equal values hash equal, NULL hashes consistently in `src/sql_datetime.rs`
- [X] T036 [P] Write tests for `PartialOrd`/`Ord` — NULL < any non-NULL value, earlier date < later date, same date earlier time < later time, equal values compare equal in `src/sql_datetime.rs`

### Implementation

- [X] T037 Implement `PartialEq`, `Eq`, `Hash` for SqlDateTime (NULL hashes as `(0i32, 0i32)`) in `src/sql_datetime.rs`
- [X] T038 Implement `PartialOrd`, `Ord` for SqlDateTime (NULL < any non-NULL value, lexicographic tuple ordering) in `src/sql_datetime.rs`
- [X] T039 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [X] T040 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for calendar construction
- **US2 (Phase 4)**: Depends on Phase 2 — tests for tick construction and constants
- **US3 (Phase 5)**: Depends on Phase 2 — tests for millisecond rounding (uses `new()`)
- **US4 (Phase 6)**: Depends on Phase 2 — arithmetic needs constructors
- **US5 (Phase 7)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US6 (Phase 8)**: Depends on Phase 2 + US8 (Phase 10) — Display needs component extraction helpers
- **US7 (Phase 9)**: Depends on Phase 2 — leap year tests use `new()` + optional US4 for arithmetic tests
- **US8 (Phase 10)**: Depends on Phase 2 — accessors need internal calendar helpers
- **Polish (Phase 11)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constructors
- **US2 (P1)**: Standalone — only needs foundational struct/constructors
- **US3 (P1)**: Standalone — only needs foundational `new()` constructor
- **US4 (P1)**: Standalone — only needs foundational struct/constructors
- **US5 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US6 (P2)**: Needs internal calendar helpers from Phase 2 (same file, available)
- **US7 (P2)**: Standalone — only needs foundational `new()` constructor
- **US8 (P3)**: Standalone — only needs foundational struct + internal helpers
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T013–T014 (US2 tests) can run in parallel — independent test functions
- T017–T019 (US4 tests) can run in parallel — independent test functions
- T025–T026 (US6 tests) can run in parallel — independent test functions
- T030–T031 (US8 tests) can run in parallel — independent test functions
- T034–T036 (Phase 11 tests) can run in parallel — independent test functions
- US1–US8 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 4

```text
# Write all US4 tests in parallel (T017-T019):
T017: tests for checked_add and checked_add_days
T018: tests for checked_sub
T019: tests for checked_add_ticks

# Then implement sequentially (T020-T022):
T020: checked_add (core algorithm)
T021: checked_sub (delegates to checked_add)
T022: checked_add_days, checked_add_ticks (convenience wrappers)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 + 3)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T009)
3. Complete Phase 3: US1 calendar construction tests (T010–T012)
4. Complete Phase 4: US2 raw tick construction tests (T013–T014)
5. Complete Phase 5: US3 millisecond rounding tests (T015–T016)
6. **STOP and VALIDATE**: Construction surface complete, rounding verified

### Incremental Delivery

1. Setup + Foundational → module compiles, constructors work
2. US1 → calendar component construction validated
3. US2 → raw tick construction + constants validated
4. US3 → millisecond rounding fidelity verified
5. US4 → duration arithmetic works with range checking
6. US5 → SQL three-valued comparisons
7. US6 → Display and FromStr with ISO 8601 format
8. US7 → leap year calendar correctness verified
9. US8 → component extraction (year, month, day, hour, minute, second)
10. Polish → standard traits, quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 40 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 7 |
| Phase 3 (US1) | 3 |
| Phase 4 (US2) | 2 |
| Phase 5 (US3) | 2 |
| Phase 6 (US4) | 6 |
| Phase 7 (US5) | 2 |
| Phase 8 (US6) | 4 |
| Phase 9 (US7) | 1 |
| Phase 10 (US8) | 4 |
| Phase 11 (Polish) | 7 |
| Parallelizable tasks | 14 |
| Test tasks | 17 |
| Implementation tasks | 23 |
