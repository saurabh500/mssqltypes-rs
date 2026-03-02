# Tasks: SqlMoney

**Input**: Design documents from `/specs/007-sql-money/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-003) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_money.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [X] T001 Create `src/sql_money.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_byte::SqlByte`, `crate::sql_int16::SqlInt16`, `crate::sql_int32::SqlInt32`, `crate::sql_int64::SqlInt64`, `crate::sql_decimal::SqlDecimal`, std traits), and empty struct definition
- [X] T002 Register module in `src/lib.rs`: add `pub mod sql_money;` and `pub use sql_money::SqlMoney;`

**Checkpoint**: `cargo build` compiles with empty `SqlMoney` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core struct, constants, constructors, and accessors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, constructors, accessors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the struct must exist for test code to compile. Tests in Phase 3 (T007–T013) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt64.

- [X] T003 Define `SqlMoney` struct with `value: Option<i64>`, derive `Copy, Clone, Debug`, add `const SCALE: i64 = 10_000` in `src/sql_money.rs`
- [X] T004 Implement constants `NULL`, `ZERO` (internal 0), `MIN_VALUE` (internal `i64::MIN`), `MAX_VALUE` (internal `i64::MAX`) in `src/sql_money.rs`
- [X] T005 Implement constructors: `from_i32(i32) -> Self`, `from_i64(i64) -> Result` (range-checked: `value * 10_000` must fit in i64), `from_f64(f64) -> Result` (reject NaN/Infinity, round to 4dp, range-check), `from_scaled(i64) -> Self` (direct internal value) in `src/sql_money.rs`
- [X] T006 Implement accessors: `is_null() -> bool`, `scaled_value() -> Result<i64>`, `to_i64() -> Result<i64>` (round-half-away-from-zero), `to_i32() -> Result<i32>` (round then range-check), `to_f64() -> Result<f64>` in `src/sql_money.rs`

**Checkpoint**: Foundation ready — `SqlMoney::from_i32()`, `from_i64()`, `from_f64()`, `from_scaled()`, `is_null()`, `scaled_value()`, `to_i64()`, `to_i32()`, `to_f64()`, constants all work. `cargo test` passes.

---

## Phase 3: User Story 1 — Create and Inspect Currency Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlMoney values from integers, floating-point numbers, and raw scaled representations. They can inspect the stored value, check for NULL, and access boundary constants.

**Independent Test**: `SqlMoney::from_i32(100)` creates a value representing 100.0000. `SqlMoney::NULL.is_null()` returns `true`. `SqlMoney::MAX_VALUE` and `SqlMoney::MIN_VALUE` are accessible and have correct internal representations. `SqlMoney::ZERO` represents 0.0000.

### Tests for User Story 1

- [X] T007 [P] [US1] Write tests for `from_i32()` — positive (100 → internal 1_000_000), negative (-50), zero, boundary i32 values (i32::MAX, i32::MIN both fit) in `src/sql_money.rs`
- [X] T008 [P] [US1] Write tests for `from_i64()` — in-range value (922_337_203_685_477 → internal 9_223_372_036_854_770_000), overflow when `value * 10_000` exceeds i64 range, boundary at `i64::MAX / 10_000` and `i64::MIN / 10_000` in `src/sql_money.rs`
- [X] T009 [P] [US1] Write tests for `from_f64()` — exact 4dp (123.4567 → internal 1_234_567), rounding beyond 4dp (123.45678 → 123.4568), NaN → `Err(OutOfRange)`, Infinity → `Err(OutOfRange)`, range overflow in `src/sql_money.rs`
- [X] T010 [P] [US1] Write tests for `from_scaled()` — any i64 accepted without validation, `from_scaled(i64::MAX)`, `from_scaled(i64::MIN)`, `from_scaled(0)` in `src/sql_money.rs`
- [X] T011 [P] [US1] Write tests for constants — `NULL.is_null()` true, `ZERO.scaled_value() == 0`, `MIN_VALUE.scaled_value() == i64::MIN`, `MAX_VALUE.scaled_value() == i64::MAX` in `src/sql_money.rs`
- [X] T012 [P] [US1] Write tests for `is_null()`, `scaled_value()` — non-null returns false, NULL returns true; `scaled_value()` on NULL returns `Err(NullValue)` in `src/sql_money.rs`
- [X] T013 [P] [US1] Write tests for `to_i64()`, `to_i32()`, `to_f64()` — `to_i64()` round-half-away-from-zero (42.5 → 43, -42.5 → -43), `to_i32()` in-range and overflow, `to_f64()` approximate value, NULL → `Err(NullValue)` for all in `src/sql_money.rs`

**Checkpoint**: All US1 acceptance scenarios (11 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Arithmetic with Overflow Detection (Priority: P1)

**Goal**: Users can perform checked addition, subtraction, multiplication, division, and negation on SqlMoney values. Overflow beyond the i64 range returns an error. Division by zero returns an error. NULL propagates through all operations.

**Independent Test**: `SqlMoney(100.00) + SqlMoney(50.25)` returns `SqlMoney(150.25)`. `SqlMoney(MAX) + SqlMoney(0.0001)` returns `Err(Overflow)`. `SqlMoney(100.00) / SqlMoney(0.00)` returns `Err(DivideByZero)`. Any operation with NULL returns NULL.

### Tests for User Story 2

- [X] T014 [P] [US2] Write tests for `checked_add` — normal addition (100.0000 + 50.2500 = 150.2500), exact i64 arithmetic (no rounding), overflow at MAX+0.0001, NULL propagation (both sides) in `src/sql_money.rs`
- [X] T015 [P] [US2] Write tests for `checked_sub` — normal subtraction (100.0000 - 200.0000 = -100.0000), exact i64 arithmetic, underflow at MIN-0.0001, NULL propagation in `src/sql_money.rs`
- [X] T016 [P] [US2] Write tests for `checked_mul` — normal multiply (100.0000 * 2.5000 = 250.0000), multiply by zero gives zero, i128 intermediate overflow back to i64, NULL propagation in `src/sql_money.rs`
- [X] T017 [P] [US2] Write tests for `checked_div` — normal division (100.0000 / 3.0000 = 33.3333), divide-by-zero → `Err(DivideByZero)`, i128 intermediate overflow, NULL propagation in `src/sql_money.rs`
- [X] T018 [P] [US2] Write tests for `checked_neg` — normal negation (-100.0000 → 100.0000), MIN_VALUE → `Err(Overflow)` (i64::MIN cannot be negated), NULL → `Ok(NULL)` in `src/sql_money.rs`

### Implementation for User Story 2

- [X] T019 [US2] Implement `checked_add`, `checked_sub` using `i64::checked_add/sub` on raw ticks (exact, no rounding), NULL propagation in `src/sql_money.rs`
- [X] T020 [US2] Implement `checked_mul` using i128 intermediate: `(a as i128) * (b as i128) / 10_000` with rounding, range check back to i64, NULL propagation in `src/sql_money.rs`
- [X] T021 [US2] Implement `checked_div` — check `rhs==0` → `DivideByZero`, then `((a as i128) * 10_000) / (b as i128)` with rounding, range check back to i64, NULL propagation in `src/sql_money.rs`
- [X] T022 [US2] Implement `checked_neg` using `i64::checked_neg()` (fixes C# bug — correctly detects i64::MIN overflow), NULL propagation in `src/sql_money.rs`
- [X] T023 [US2] Implement operator traits `Add`, `Sub`, `Mul`, `Div`, `Neg` for owned and all borrowed combinations, `Output = Result<SqlMoney, SqlTypeError>`, delegating to `checked_*` methods in `src/sql_money.rs`

**Checkpoint**: All US2 acceptance scenarios (11 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Comparison Returning SqlBoolean (Priority: P1)

**Goal**: Users can compare SqlMoney values using SQL three-valued logic. Comparisons return SqlBoolean. NULL compared with anything returns SqlBoolean::NULL.

**Independent Test**: `SqlMoney(100.00).sql_equals(&SqlMoney(100.00))` returns `SqlBoolean::TRUE`. `SqlMoney(100.00).sql_less_than(&SqlMoney(200.00))` returns `SqlBoolean::TRUE`. Any comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 3

- [X] T024 [P] [US3] Write tests for 6 SQL comparison methods — `sql_equals` (equal, not-equal), `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal` (equal case, less case), `sql_greater_than_or_equal` (equal case, greater case), NULL propagation on both sides for all methods in `src/sql_money.rs`

### Implementation for User Story 3

- [X] T025 [US3] Implement 6 SQL comparison methods (`sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`) comparing raw i64 ticks, returning `SqlBoolean` with NULL propagation in `src/sql_money.rs`

**Checkpoint**: All US3 acceptance scenarios (8 scenarios) pass. `cargo test` green.

---

## Phase 6: User Story 4 — Display and Parsing (Priority: P2)

**Goal**: Users can convert SqlMoney to and from string representations. Display shows format `"#0.00##"` (minimum 2 decimal places, maximum 4, trimming trailing zeros beyond 2nd place). NULL displays as `"Null"`. Parsing supports decimal notation with optional sign.

**Independent Test**: `format!("{}", SqlMoney(123.4567))` produces `"123.4567"`. `format!("{}", SqlMoney(100.0000))` produces `"100.00"`. `"123.45".parse::<SqlMoney>()` returns a valid SqlMoney.

### Tests for User Story 4

- [X] T026 [P] [US4] Write tests for `Display` — all 4dp significant (123.4567 → "123.4567"), trailing zeros trimmed to 2dp (123.4500 → "123.45"), minimum 2dp (100.0000 → "100.00"), 3dp case (123.4560 → "123.456"), negative with trimming (-50.1000 → "-50.10"), NULL → "Null" in `src/sql_money.rs`
- [X] T027 [P] [US4] Write tests for `FromStr` — valid decimal ("123.4567"), negative ("-50.10"), integer without decimal ("100"), "Null" → NULL, invalid ("abc") → `Err(ParseError)`, more than 4dp rounds, string exceeding range → `Err(Overflow)` in `src/sql_money.rs`

### Implementation for User Story 4

- [X] T028 [US4] Implement `Display` for SqlMoney — NULL → "Null", otherwise format with `abs(ticks) / 10_000` integer part and `abs(ticks) % 10_000` fractional part, trim trailing zeros to minimum 2dp, handle negative sign in `src/sql_money.rs`
- [X] T029 [US4] Implement `FromStr` for SqlMoney — "Null" → NULL, parse decimal string to scaled i64 (handle optional sign, integer/fractional parts, round beyond 4dp), reject invalid with `ParseError` in `src/sql_money.rs`

**Checkpoint**: All US4 acceptance scenarios (10 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 7: User Story 5 — Conversions To and From Other SqlTypes (Priority: P2)

**Goal**: Users can convert between SqlMoney and other SqlTypes. Widening conversions from integers always succeed. Narrowing conversions check range. SqlInt64 is fallible (range-checked). Output conversions round and range-check.

**Independent Test**: `SqlMoney::from(SqlInt32::new(42))` returns `SqlMoney(42.0000)`. `SqlMoney(42.9999).to_sql_int64()` returns `SqlInt64(43)` (rounded). `SqlMoney(42.0000).to_sql_decimal()` returns a SqlDecimal with scale=4.

### Tests for User Story 5

- [X] T030 [P] [US5] Write tests for widening `From` — `From<SqlBoolean>` (NULL→NULL, FALSE→0.0000, TRUE→1.0000), `From<SqlByte>` (NULL→NULL, 255→255.0000), `From<SqlInt16>` (NULL→NULL, 1000→1000.0000), `From<SqlInt32>` (NULL→NULL, 42→42.0000, i32::MAX, i32::MIN) in `src/sql_money.rs`
- [X] T031 [P] [US5] Write tests for `from_sql_int64()` — in-range (100→100.0000), overflow when value × 10,000 exceeds i64, NULL→NULL in `src/sql_money.rs`
- [X] T032 [P] [US5] Write tests for `to_sql_int64()` — round (42.9999→43, -42.5→-43), NULL→`Err(NullValue)` in `src/sql_money.rs`
- [X] T033 [P] [US5] Write tests for `to_sql_int32()` — in-range (42.0000→42), rounding (42.5→43), overflow (value exceeds i32 range after rounding), NULL→`Err(NullValue)` in `src/sql_money.rs`
- [X] T034 [P] [US5] Write tests for `to_sql_int16()`, `to_sql_byte()` — in-range, overflow, negative for byte, NULL propagation in `src/sql_money.rs`
- [X] T035 [P] [US5] Write tests for `to_sql_boolean()` — zero→FALSE, non-zero→TRUE, NULL→NULL in `src/sql_money.rs`
- [X] T036 [P] [US5] Write tests for `to_sql_decimal()` — exact value with scale=4, NULL→NULL in `src/sql_money.rs`

### Implementation for User Story 5

- [X] T037 [US5] Implement `From<SqlBoolean>`, `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>` for SqlMoney (widening: value × 10,000, NULL→NULL) in `src/sql_money.rs`
- [X] T038 [US5] Implement `from_sql_int64(SqlInt64) -> Result<Self, SqlTypeError>` (range-checked: value × 10,000 must fit i64, NULL→NULL) in `src/sql_money.rs`
- [X] T039 [US5] Implement `to_sql_int64()`, `to_sql_int32()`, `to_sql_int16()`, `to_sql_byte()` — round-half-away-from-zero then range-check for narrowing types, NULL→`Err(NullValue)` in `src/sql_money.rs`
- [X] T040 [US5] Implement `to_sql_boolean()` — zero→FALSE, non-zero→TRUE, NULL→NULL in `src/sql_money.rs`
- [X] T041 [US5] Implement `to_sql_decimal()` — convert to SqlDecimal with exact value and scale=4, NULL→NULL in `src/sql_money.rs`

**Checkpoint**: All US5 acceptance scenarios (15 scenarios) pass. `cargo test` green.

---

## Phase 8: User Story 6 — Standard Rust Traits (Priority: P3)

**Goal**: Users can use SqlMoney with standard Rust trait operations: equality, hashing, and ordering. This enables use in HashMaps, BTreeMaps, sorting, and pattern matching.

**Independent Test**: Two equal SqlMoney values compare as equal and produce the same hash. NULL == NULL per Rust semantics. SqlMoney values can be sorted with NULL < any value.

### Tests for User Story 6

- [X] T042 [P] [US6] Write tests for `PartialEq`/`Eq` — value equality (100.0000 == 100.0000), inequality (100.0000 != 200.0000), NULL == NULL (Rust semantics), NULL != non-NULL in `src/sql_money.rs`
- [X] T043 [P] [US6] Write tests for `Hash` — equal values hash equal, NULL hashes consistently in `src/sql_money.rs`
- [X] T044 [P] [US6] Write tests for `PartialOrd`/`Ord` — NULL < any non-NULL value, negative < positive, MIN < MAX, equal values in `src/sql_money.rs`

### Implementation for User Story 6

- [X] T045 [US6] Implement `PartialEq`, `Eq`, `Hash` for SqlMoney (NULL == NULL per Rust semantics, NULL hashes as `0i64`) in `src/sql_money.rs`
- [X] T046 [US6] Implement `PartialOrd`, `Ord` for SqlMoney (NULL < any non-NULL value, total ordering) in `src/sql_money.rs`

**Checkpoint**: All US6 acceptance scenarios (5 scenarios) pass. `cargo test` green.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final quality gates and validation across all user stories

- [X] T047 Run `cargo fmt` and `cargo clippy -- -D warnings` — all must pass with zero warnings
- [X] T048 Run `cargo test` — all tests pass, verify ≥95% code coverage
- [X] T049 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants/accessors
- **US2 (Phase 4)**: Depends on Phase 2 — arithmetic needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US4 (Phase 6)**: Depends on Phase 2 — Display/FromStr need constructors
- **US5 (Phase 7)**: Depends on Phase 2 — conversions need constructors + `SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlDecimal` (all exist)
- **US6 (Phase 8)**: Depends on Phase 2 — traits need constructors
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants/accessors
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P1)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US4 (P2)**: Standalone — only needs foundational struct/constants
- **US5 (P2)**: Standalone — only needs foundational struct + existing types (`SqlBoolean`, `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlDecimal`)
- **US6 (P3)**: Standalone — only needs foundational struct/constants
- All user stories can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T007–T013 (US1 tests) can all run in parallel — independent test functions
- T014–T018 (US2 tests) can all run in parallel
- T026–T027 (US4 tests) can run in parallel
- T030–T036 (US5 tests) can all run in parallel
- T042–T044 (US6 tests) can all run in parallel
- US1–US6 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 2

```text
# Write all US2 tests in parallel (T014-T018):
T014: tests for checked_add
T015: tests for checked_sub
T016: tests for checked_mul
T017: tests for checked_div
T018: tests for checked_neg

# Then implement sequentially (T019-T023):
T019: checked_add, checked_sub (exact i64)
T020: checked_mul (i128 intermediate)
T021: checked_div (i128 intermediate)
T022: checked_neg (checked_neg fixes C# bug)
T023: operator trait wiring (Add, Sub, Mul, Div, Neg — owned + borrowed)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 + 3)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: US1 tests + validation (T007–T013)
4. Complete Phase 4: US2 arithmetic (T014–T023)
5. Complete Phase 5: US3 comparisons (T024–T025)
6. **STOP and VALIDATE**: Struct, constants, arithmetic, comparisons all work

### Incremental Delivery

1. Setup + Foundational → module compiles
2. US1 → values can be created and inspected (MVP!)
3. US2 → arithmetic works with overflow detection
4. US3 → SQL three-valued comparisons
5. US4 → Display and FromStr
6. US5 → cross-type conversions
7. US6 → standard Rust traits (PartialEq, Hash, Ord)
8. Polish → quality gates, coverage

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 49 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 7 |
| Phase 4 (US2) | 10 |
| Phase 5 (US3) | 2 |
| Phase 6 (US4) | 4 |
| Phase 7 (US5) | 12 |
| Phase 8 (US6) | 5 |
| Phase 9 (Polish) | 3 |
| Parallelizable tasks | 22 |
| Test tasks | 22 |
| Implementation tasks | 27 |
