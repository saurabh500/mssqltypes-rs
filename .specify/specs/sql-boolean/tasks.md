# Tasks: SqlBoolean

**Input**: Design documents from `.specify/specs/sql-boolean/`
**Prerequisites**: plan.md ✅, spec.md (sql-boolean.md) ✅, research.md ✅

**Tests**: Included — TDD is NON-NEGOTIABLE per constitution principle III and spec SC-003 (≥95% coverage).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths included in all descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create `SqlTypeError` enum and module structure — blocks all user stories.

- [x] T001 [P] Create `src/error.rs` — define `SqlTypeError` enum with variants: `NullValue`, `Overflow`, `DivideByZero`, `ParseError(String)`, `OutOfRange(String)`. Implement `Display`, `Debug`, `Clone`, `PartialEq`, `std::error::Error`. Add unit tests for Display output of each variant.
- [x] T002 Update `src/lib.rs` — remove placeholder `add` function, declare modules (`mod error; mod sql_boolean;`), re-export `pub use error::SqlTypeError;` and `pub use sql_boolean::SqlBoolean;`.

**Checkpoint**: `cargo check` passes with empty `sql_boolean.rs` stub.

---

## Phase 2: User Story 1 — Create and Inspect SqlBoolean Values (Priority: P1) 🎯 MVP

**Goal**: Developers can construct `SqlBoolean` values (True, False, Null) and inspect their state.

**Independent Test**: `cargo test us1_` — all construction and inspection tests pass.

### Tests for User Story 1

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [x] T003 [US1] Write tests in `src/sql_boolean.rs` `#[cfg(test)] mod tests` for constants and construction:
  - `test_true_constant_is_true` — `SqlBoolean::TRUE.is_true()` returns `true`
  - `test_false_constant_is_false` — `SqlBoolean::FALSE.is_false()` returns `true`
  - `test_null_constant_is_null` — `SqlBoolean::NULL.is_null()` returns `true`
  - `test_zero_equals_false` — `SqlBoolean::ZERO` has same behavior as `SqlBoolean::FALSE`
  - `test_one_equals_true` — `SqlBoolean::ONE` has same behavior as `SqlBoolean::TRUE`
  - `test_new_true` — `SqlBoolean::new(true).is_true()` returns `true`
  - `test_new_false` — `SqlBoolean::new(false).is_false()` returns `true`
  - `test_from_int_zero` — `SqlBoolean::from_int(0).is_false()` returns `true`
  - `test_from_int_positive` — `SqlBoolean::from_int(42).is_true()` returns `true`
  - `test_from_int_negative` — `SqlBoolean::from_int(-1).is_true()` returns `true`
  - `test_from_bool_trait` — `SqlBoolean::from(true).is_true()` returns `true`

- [x] T004 [US1] Write tests for value access:
  - `test_value_true` — `SqlBoolean::TRUE.value()` returns `Ok(true)`
  - `test_value_false` — `SqlBoolean::FALSE.value()` returns `Ok(false)`
  - `test_value_null_returns_error` — `SqlBoolean::NULL.value()` returns `Err(SqlTypeError::NullValue)`
  - `test_byte_value_true` — `SqlBoolean::TRUE.byte_value()` returns `Ok(1)`
  - `test_byte_value_false` — `SqlBoolean::FALSE.byte_value()` returns `Ok(0)`
  - `test_byte_value_null_returns_error` — `SqlBoolean::NULL.byte_value()` returns `Err(SqlTypeError::NullValue)`

- [x] T005 [US1] Write tests for Copy/Clone/Debug:
  - `test_copy_semantics` — assigning `SqlBoolean` to another variable copies it
  - `test_debug_format` — `format!("{:?}", SqlBoolean::TRUE)` produces meaningful output

### Implementation for User Story 1

- [x] T006 [US1] Implement `SqlBoolean` struct and constants in `src/sql_boolean.rs`:
  - Define `pub struct SqlBoolean { m_value: u8 }` with `Copy`, `Clone`, `Debug` derives
  - Internal constants: `X_NULL: u8 = 0`, `X_FALSE: u8 = 1`, `X_TRUE: u8 = 2`
  - Public constants: `NULL`, `TRUE`, `FALSE`, `ZERO`, `ONE`

- [x] T007 [US1] Implement constructors and inspectors in `src/sql_boolean.rs`:
  - `pub fn new(value: bool) -> Self`
  - `pub fn from_int(value: i32) -> Self`
  - `pub fn is_null(&self) -> bool`
  - `pub fn is_true(&self) -> bool`
  - `pub fn is_false(&self) -> bool`

- [x] T008 [US1] Implement value access in `src/sql_boolean.rs`:
  - `pub fn value(&self) -> Result<bool, SqlTypeError>`
  - `pub fn byte_value(&self) -> Result<u8, SqlTypeError>`

- [x] T009 [US1] Implement `From<bool> for SqlBoolean` in `src/sql_boolean.rs`.

**Checkpoint**: `cargo test` — all US1 tests pass. `SqlBoolean` can be constructed, inspected, and values extracted.

---

## Phase 3: User Story 2 — Three-Valued Logical Operations (Priority: P1) 🎯 MVP

**Goal**: AND, OR, XOR, NOT work with SQL three-valued NULL propagation and short-circuit rules.

**Independent Test**: `cargo test us2_` — all logic operator tests pass.

### Tests for User Story 2

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T010 [US2] Write NOT tests in `src/sql_boolean.rs`:
  - `test_not_true` — `!SqlBoolean::TRUE` is `FALSE`
  - `test_not_false` — `!SqlBoolean::FALSE` is `TRUE`
  - `test_not_null` — `!SqlBoolean::NULL` is `NULL`

- [ ] T011 [US2] Write AND truth table tests in `src/sql_boolean.rs` (all 9 combinations):
  - `test_and_true_true` → `TRUE`
  - `test_and_true_false` → `FALSE`
  - `test_and_true_null` → `NULL`
  - `test_and_false_true` → `FALSE`
  - `test_and_false_false` → `FALSE`
  - `test_and_false_null` → `FALSE` *(short-circuit)*
  - `test_and_null_true` → `NULL`
  - `test_and_null_false` → `FALSE` *(short-circuit)*
  - `test_and_null_null` → `NULL`

- [ ] T012 [US2] Write OR truth table tests in `src/sql_boolean.rs` (all 9 combinations):
  - `test_or_true_true` → `TRUE`
  - `test_or_true_false` → `TRUE`
  - `test_or_true_null` → `TRUE` *(short-circuit)*
  - `test_or_false_true` → `TRUE`
  - `test_or_false_false` → `FALSE`
  - `test_or_false_null` → `NULL`
  - `test_or_null_true` → `TRUE` *(short-circuit)*
  - `test_or_null_false` → `NULL`
  - `test_or_null_null` → `NULL`

- [ ] T013 [US2] Write XOR truth table tests in `src/sql_boolean.rs` (all 9 combinations):
  - `test_xor_true_true` → `FALSE`
  - `test_xor_true_false` → `TRUE`
  - `test_xor_true_null` → `NULL`
  - `test_xor_false_true` → `TRUE`
  - `test_xor_false_false` → `FALSE`
  - `test_xor_false_null` → `NULL`
  - `test_xor_null_true` → `NULL`
  - `test_xor_null_false` → `NULL`
  - `test_xor_null_null` → `NULL`

### Implementation for User Story 2

- [ ] T014 [US2] Implement `Not` trait for `SqlBoolean` in `src/sql_boolean.rs`. True→False, False→True, Null→Null.

- [ ] T015 [US2] Implement `BitAnd` trait for `SqlBoolean` in `src/sql_boolean.rs`. FALSE short-circuit: if either operand is False, result is False. Both True→True. Otherwise Null.

- [ ] T016 [US2] Implement `BitOr` trait for `SqlBoolean` in `src/sql_boolean.rs`. TRUE short-circuit: if either operand is True, result is True. Both False→False. Otherwise Null.

- [ ] T017 [US2] Implement `BitXor` trait for `SqlBoolean` in `src/sql_boolean.rs`. If either is Null, result is Null. Otherwise `m_value != m_value`.

**Checkpoint**: `cargo test` — all 30 truth-table tests pass (3 NOT + 9 AND + 9 OR + 9 XOR).

---

## Phase 4: User Story 3 — Comparison Operations (Priority: P2)

**Goal**: SQL comparison methods return `SqlBoolean` with NULL propagation. Rust `PartialEq`/`Ord`/`Hash` implemented for collection use.

**Independent Test**: `cargo test us3_` — all comparison tests pass.

### Tests for User Story 3

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T018 [US3] Write SQL comparison tests in `src/sql_boolean.rs`:
  - `test_sql_equals_true_true` → `TRUE`
  - `test_sql_equals_true_false` → `FALSE`
  - `test_sql_equals_null_any` → `NULL` (test both operand positions)
  - `test_sql_not_equals_true_false` → `TRUE`
  - `test_sql_not_equals_true_true` → `FALSE`
  - `test_sql_less_than_false_true` → `TRUE` (False < True per m_value)
  - `test_sql_less_than_true_false` → `FALSE`
  - `test_sql_greater_than_true_false` → `TRUE`
  - `test_sql_less_than_or_equal_true_true` → `TRUE`
  - `test_sql_greater_than_or_equal_false_false` → `TRUE`
  - `test_sql_comparison_with_null` → `NULL` for all operators

- [ ] T019 [US3] Write `PartialEq`, `Eq`, `Hash`, `Ord` tests in `src/sql_boolean.rs`:
  - `test_partialeq_true_true` → `true` (Rust `==`)
  - `test_partialeq_true_false` → `false`
  - `test_partialeq_null_null` → `true` (two Nulls are equal per C# Equals)
  - `test_partialeq_null_true` → `false`
  - `test_hash_true_consistent` — same hash for two TRUE values
  - `test_hash_null_is_zero` — hash of NULL maps to deterministic value
  - `test_ord_null_less_than_false` — `NULL < FALSE` per CompareTo
  - `test_ord_false_less_than_true` — `FALSE < TRUE` per CompareTo
  - `test_ord_null_equal_null` — `NULL.cmp(&NULL) == Equal`
  - `test_sorting` — `[TRUE, NULL, FALSE]` sorts to `[NULL, FALSE, TRUE]`

### Implementation for User Story 3

- [ ] T020 [US3] Implement SQL comparison methods in `src/sql_boolean.rs`:
  - `pub fn sql_equals(&self, other: &SqlBoolean) -> SqlBoolean`
  - `pub fn sql_not_equals(&self, other: &SqlBoolean) -> SqlBoolean`
  - `pub fn sql_less_than(&self, other: &SqlBoolean) -> SqlBoolean`
  - `pub fn sql_greater_than(&self, other: &SqlBoolean) -> SqlBoolean`
  - `pub fn sql_less_than_or_equal(&self, other: &SqlBoolean) -> SqlBoolean`
  - `pub fn sql_greater_than_or_equal(&self, other: &SqlBoolean) -> SqlBoolean`
  - All return `SqlBoolean::NULL` if either operand is Null.

- [ ] T021 [US3] Implement `PartialEq`, `Eq`, `Hash` for `SqlBoolean` in `src/sql_boolean.rs`:
  - `PartialEq`: Two Nulls are equal. Otherwise compare `m_value`.
  - `Hash`: Use 0 for Null, otherwise delegate to `bool::hash()`.

- [ ] T022 [US3] Implement `PartialOrd` and `Ord` for `SqlBoolean` in `src/sql_boolean.rs`:
  - CompareTo semantics: Null < False < True.
  - Both Nulls are Equal.

**Checkpoint**: `cargo test` — all comparison and ordering tests pass.

---

## Phase 5: User Story 4 — Display, Parsing & Conversions (Priority: P2)

**Goal**: `SqlBoolean` displays as "True"/"False"/"Null" and parses from strings and primitives.

**Independent Test**: `cargo test us4_` — all display/parse tests pass.

### Tests for User Story 4

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T023 [US4] Write `Display` tests in `src/sql_boolean.rs`:
  - `test_display_true` — `format!("{}", SqlBoolean::TRUE)` → `"True"`
  - `test_display_false` — `format!("{}", SqlBoolean::FALSE)` → `"False"`
  - `test_display_null` — `format!("{}", SqlBoolean::NULL)` → `"Null"`

- [ ] T024 [US4] Write `FromStr` tests in `src/sql_boolean.rs`:
  - `test_parse_true_lowercase` — `"true".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_true_uppercase` — `"True".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_true_mixed_case` — `"TRUE".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_false_lowercase` — `"false".parse::<SqlBoolean>()` → `Ok(FALSE)`
  - `test_parse_one` — `"1".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_zero` — `"0".parse::<SqlBoolean>()` → `Ok(FALSE)`
  - `test_parse_positive_int` — `"42".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_negative_int` — `"-1".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_null_string` — `"Null".parse::<SqlBoolean>()` → `Ok(NULL)`
  - `test_parse_with_leading_whitespace` — `" true".parse::<SqlBoolean>()` → `Ok(TRUE)`
  - `test_parse_invalid` — `"maybe".parse::<SqlBoolean>()` → `Err(SqlTypeError::ParseError(...))`
  - `test_parse_empty` — `"".parse::<SqlBoolean>()` → `Err(SqlTypeError::ParseError(...))`

### Implementation for User Story 4

- [ ] T025 [US4] Implement `Display` for `SqlBoolean` in `src/sql_boolean.rs`:
  - True → `"True"`, False → `"False"`, Null → `"Null"`

- [ ] T026 [US4] Implement `FromStr` for `SqlBoolean` in `src/sql_boolean.rs`:
  1. If input is `"Null"` (case-insensitive) → `Ok(SqlBoolean::NULL)`
  2. Trim leading whitespace. If first char is digit, `-`, or `+` → parse as `i32` → `from_int()`
  3. Otherwise parse as bool (`"true"`/`"false"`, case-insensitive)
  4. Invalid input → `Err(SqlTypeError::ParseError(...))`

**Checkpoint**: `cargo test` — all display and parse tests pass. `SqlBoolean` round-trips through `Display`/`FromStr`.

---

## Phase 6: Polish & Validation

**Purpose**: Final quality gates per constitution.

- [ ] T027 Run `cargo fmt --check` and fix any formatting issues.
- [ ] T028 Run `cargo clippy -- -D warnings` and fix all warnings.
- [ ] T029 Run `cargo test` — verify all tests pass (expected: 60+ tests).
- [ ] T030 Run coverage tool (`cargo tarpaulin` or `llvm-cov`) — verify ≥95% coverage for `src/sql_boolean.rs`.
- [ ] T031 Review public API surface — ensure only spec'd methods/traits are `pub`. No leaking of internal constants or helper functions.

**Checkpoint**: All quality gates pass. SqlBoolean is complete and ready for PR.

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)         → No dependencies — start immediately
Phase 2 (US1 - Core)    → Depends on Phase 1 (needs SqlTypeError)
Phase 3 (US2 - Logic)   → Depends on Phase 2 (needs SqlBoolean struct + constants)
Phase 4 (US3 - Compare) → Depends on Phase 2 (needs SqlBoolean struct)
Phase 5 (US4 - Display) → Depends on Phase 2 (needs SqlBoolean struct)
Phase 6 (Polish)        → Depends on all previous phases
```

### Parallel Opportunities

```
After Phase 1 completes:
  → Phase 2 (US1) starts immediately

After Phase 2 completes:
  → Phase 3 (US2), Phase 4 (US3), Phase 5 (US4) can all run in parallel
    (all operate on same file but different impl blocks / test groups)

Within each phase:
  → Test tasks (T003-T005, T010-T013, T018-T019, T023-T024) can be written in parallel
  → Implementation tasks within a phase are sequential (later tasks may depend on earlier ones)
```

### TDD Workflow Per Phase

```
1. Write all test functions for the phase (they compile but FAIL)
2. Implement code to make tests pass one by one
3. Refactor if needed
4. Run full test suite to verify no regressions
5. Move to next phase
```

---

## Summary

| Phase | Tasks | Tests | Description |
|-------|-------|-------|-------------|
| 1 — Setup | T001–T002 | ~5 | SqlTypeError + module structure |
| 2 — US1 Core | T003–T009 | ~17 | Construction, inspection, value access |
| 3 — US2 Logic | T010–T017 | ~30 | NOT, AND, OR, XOR with truth tables |
| 4 — US3 Compare | T018–T022 | ~21 | SQL comparisons + PartialEq/Ord/Hash |
| 5 — US4 Display | T023–T026 | ~15 | Display + FromStr |
| 6 — Polish | T027–T031 | — | fmt, clippy, coverage, API review |
| **Total** | **31 tasks** | **~88 tests** | |
