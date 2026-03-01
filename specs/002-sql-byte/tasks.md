# Tasks: SqlByte

**Input**: Design documents from `specs/002-sql-byte/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, contracts/public-api.md ✅

**Tests**: Included — TDD is NON-NEGOTIABLE per constitution principle III and spec SC-003 (≥95% coverage).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths included in all descriptions

---

## Phase 1: Setup

**Purpose**: Register the new module — blocks all user stories.

- [ ] T001 Create empty `src/sql_byte.rs` with `use crate::error::SqlTypeError;` and `use crate::sql_boolean::SqlBoolean;` imports, plus a `#[cfg(test)] mod tests {}` block.
- [ ] T002 Update `src/lib.rs` — add `pub mod sql_byte;` and `pub use sql_byte::SqlByte;`.

**Checkpoint**: `cargo check` passes with the empty module stub.

---

## Phase 2: User Story 1 — Create and Inspect SqlByte Values (Priority: P1) 🎯 MVP

**Goal**: Developers can construct `SqlByte` values, inspect NULL state, and extract the inner `u8`.

**Independent Test**: `cargo test` — all construction/inspection tests pass.

### Tests for User Story 1

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T003 [US1] Write constant and construction tests in `src/sql_byte.rs` `#[cfg(test)] mod tests`:
  - `test_new_value` — `SqlByte::new(42).value()` returns `Ok(42)`
  - `test_null_is_null` — `SqlByte::NULL.is_null()` returns `true`
  - `test_null_value_returns_error` — `SqlByte::NULL.value()` returns `Err(SqlTypeError::NullValue)`
  - `test_zero_constant` — `SqlByte::ZERO.value()` returns `Ok(0)`
  - `test_min_value` — `SqlByte::MIN_VALUE.value()` returns `Ok(0)`
  - `test_max_value` — `SqlByte::MAX_VALUE.value()` returns `Ok(255)`
  - `test_non_null_is_not_null` — `SqlByte::new(100).is_null()` returns `false`
  - `test_from_u8` — `SqlByte::from(42u8).value()` returns `Ok(42)`

- [ ] T004 [US1] Write Copy/Clone/Debug tests in `src/sql_byte.rs`:
  - `test_copy_semantics` — assigning `SqlByte` copies it, both values accessible
  - `test_debug_format` — `format!("{:?}", SqlByte::new(42))` contains `"SqlByte"`

### Implementation for User Story 1

- [ ] T005 [US1] Implement `SqlByte` struct, constants, and constructors in `src/sql_byte.rs`:
  - Define `#[derive(Copy, Clone, Debug)] pub struct SqlByte { value: Option<u8> }`
  - Constants: `NULL`, `ZERO`, `MIN_VALUE`, `MAX_VALUE`
  - `pub fn new(v: u8) -> Self`
  - `pub fn is_null(&self) -> bool`
  - `pub fn value(&self) -> Result<u8, SqlTypeError>`

- [ ] T006 [US1] Implement `From<u8> for SqlByte` in `src/sql_byte.rs`.

**Checkpoint**: `cargo test` — all US1 tests pass. SqlByte can be constructed, inspected, and values extracted.

---

## Phase 3: User Story 2 — Arithmetic Operations with Overflow Detection (Priority: P1) 🎯 MVP

**Goal**: Add, Sub, Mul, Div, Rem work with overflow detection and NULL propagation.

**Independent Test**: `cargo test` — all arithmetic tests pass.

### Tests for User Story 2

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T007 [US2] Write addition tests in `src/sql_byte.rs`:
  - `test_add_normal` — `SqlByte(10) + SqlByte(20)` returns `Ok(SqlByte(30))`
  - `test_add_overflow` — `SqlByte(200) + SqlByte(100)` returns `Err(Overflow)`
  - `test_add_boundary_max` — `SqlByte(255) + SqlByte(1)` returns `Err(Overflow)`
  - `test_add_zero` — `SqlByte(42) + SqlByte(0)` returns `Ok(SqlByte(42))`
  - `test_add_null_left` — `SqlByte::NULL + SqlByte(10)` returns `Ok(SqlByte::NULL)`
  - `test_add_null_right` — `SqlByte(10) + SqlByte::NULL` returns `Ok(SqlByte::NULL)`
  - `test_add_null_both` — `SqlByte::NULL + SqlByte::NULL` returns `Ok(SqlByte::NULL)`

- [ ] T008 [US2] Write subtraction tests in `src/sql_byte.rs`:
  - `test_sub_normal` — `SqlByte(20) - SqlByte(10)` returns `Ok(SqlByte(10))`
  - `test_sub_negative_overflow` — `SqlByte(5) - SqlByte(10)` returns `Err(Overflow)`
  - `test_sub_boundary_zero` — `SqlByte(0) - SqlByte(1)` returns `Err(Overflow)`
  - `test_sub_to_zero` — `SqlByte(10) - SqlByte(10)` returns `Ok(SqlByte(0))`
  - `test_sub_null` — `SqlByte(10) - SqlByte::NULL` returns `Ok(SqlByte::NULL)`

- [ ] T009 [US2] Write multiplication tests in `src/sql_byte.rs`:
  - `test_mul_normal` — `SqlByte(10) * SqlByte(5)` returns `Ok(SqlByte(50))`
  - `test_mul_overflow` — `SqlByte(15) * SqlByte(20)` returns `Err(Overflow)`
  - `test_mul_boundary` — `SqlByte(128) * SqlByte(2)` returns `Err(Overflow)`
  - `test_mul_by_zero` — `SqlByte(255) * SqlByte(0)` returns `Ok(SqlByte(0))`
  - `test_mul_by_one` — `SqlByte(42) * SqlByte(1)` returns `Ok(SqlByte(42))`
  - `test_mul_null` — `SqlByte(10) * SqlByte::NULL` returns `Ok(SqlByte::NULL)`

- [ ] T010 [US2] Write division and remainder tests in `src/sql_byte.rs`:
  - `test_div_normal` — `SqlByte(20) / SqlByte(5)` returns `Ok(SqlByte(4))`
  - `test_div_by_zero` — `SqlByte(10) / SqlByte(0)` returns `Err(DivideByZero)`
  - `test_div_truncates` — `SqlByte(10) / SqlByte(3)` returns `Ok(SqlByte(3))`
  - `test_div_null` — `SqlByte(10) / SqlByte::NULL` returns `Ok(SqlByte::NULL)`
  - `test_rem_normal` — `SqlByte(10) % SqlByte(3)` returns `Ok(SqlByte(1))`
  - `test_rem_by_zero` — `SqlByte(10) % SqlByte(0)` returns `Err(DivideByZero)`
  - `test_rem_null` — `SqlByte(10) % SqlByte::NULL` returns `Ok(SqlByte::NULL)`
  - `test_rem_even` — `SqlByte(10) % SqlByte(5)` returns `Ok(SqlByte(0))`

### Implementation for User Story 2

- [ ] T011 [US2] Implement `checked_add`, `checked_sub`, `checked_mul` in `src/sql_byte.rs`:
  - Widen both operands to `i32`, compute, check `(result & !0xFF) != 0` for overflow
  - NULL propagation: if either is NULL, return `Ok(SqlByte::NULL)`

- [ ] T012 [US2] Implement `checked_div` and `checked_rem` in `src/sql_byte.rs`:
  - Check divisor == 0 → `Err(DivideByZero)`
  - NULL propagation: if either is NULL, return `Ok(SqlByte::NULL)`
  - No overflow possible for div/rem

- [ ] T013 [US2] Implement `Add`, `Sub`, `Mul`, `Div`, `Rem` operator traits in `src/sql_byte.rs`:
  - Each delegates to the corresponding `checked_*` method
  - `type Output = Result<SqlByte, SqlTypeError>`

**Checkpoint**: `cargo test` — all arithmetic tests pass including overflow, underflow, divide-by-zero, and NULL propagation.

---

## Phase 4: User Story 3 — Bitwise Operations (Priority: P2)

**Goal**: BitAnd, BitOr, BitXor, Not (ones complement) work with NULL propagation.

**Independent Test**: `cargo test` — all bitwise tests pass.

### Tests for User Story 3

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T014 [US3] Write bitwise operation tests in `src/sql_byte.rs`:
  - `test_bitand` — `SqlByte(0xFF) & SqlByte(0x0F)` returns `SqlByte(0x0F)`
  - `test_bitor` — `SqlByte(0xF0) | SqlByte(0x0F)` returns `SqlByte(0xFF)`
  - `test_bitxor` — `SqlByte(0xFF) ^ SqlByte(0x0F)` returns `SqlByte(0xF0)`
  - `test_not` — `!SqlByte(0x0F)` returns `SqlByte(0xF0)`
  - `test_ones_complement` — `SqlByte::new(0x0F).ones_complement()` returns `SqlByte(0xF0)`
  - `test_bitand_null` — `SqlByte(0xFF) & SqlByte::NULL` returns `SqlByte::NULL`
  - `test_bitor_null` — `SqlByte::NULL | SqlByte(0x0F)` returns `SqlByte::NULL`
  - `test_bitxor_null` — `SqlByte::NULL ^ SqlByte::NULL` returns `SqlByte::NULL`
  - `test_not_null` — `!SqlByte::NULL` returns `SqlByte::NULL`
  - `test_not_zero` — `!SqlByte(0x00)` returns `SqlByte(0xFF)`
  - `test_not_max` — `!SqlByte(0xFF)` returns `SqlByte(0x00)`

### Implementation for User Story 3

- [ ] T015 [US3] Implement `ones_complement`, `BitAnd`, `BitOr`, `BitXor`, `Not` in `src/sql_byte.rs`:
  - `ones_complement(self) -> SqlByte` — NULL→NULL, else `!value` (truncated to u8)
  - `BitAnd`: NULL propagation, then `a & b`
  - `BitOr`: NULL propagation, then `a | b`
  - `BitXor`: NULL propagation, then `a ^ b`
  - `Not`: delegates to `ones_complement`

**Checkpoint**: `cargo test` — all bitwise tests pass including NULL propagation.

---

## Phase 5: User Story 4 — Comparison, Display, Parsing & Conversions (Priority: P2)

**Goal**: SQL comparisons return `SqlBoolean`. Display/FromStr round-trip. Rust ordering traits for collections. Conversions to/from `SqlBoolean`.

**Independent Test**: `cargo test` — all comparison, display, parse, and conversion tests pass.

### Tests for User Story 4

> **Write these tests FIRST. They MUST FAIL before implementation.**

- [ ] T016 [US4] Write SQL comparison tests in `src/sql_byte.rs`:
  - `test_sql_equals_same` — `SqlByte(10).sql_equals(&SqlByte(10))` → `SqlBoolean::TRUE`
  - `test_sql_equals_different` — `SqlByte(10).sql_equals(&SqlByte(20))` → `SqlBoolean::FALSE`
  - `test_sql_equals_null` — `SqlByte(10).sql_equals(&SqlByte::NULL)` → `SqlBoolean::NULL`
  - `test_sql_not_equals` — `SqlByte(10).sql_not_equals(&SqlByte(20))` → `SqlBoolean::TRUE`
  - `test_sql_less_than` — `SqlByte(10).sql_less_than(&SqlByte(20))` → `SqlBoolean::TRUE`
  - `test_sql_less_than_with_equal_values` — `SqlByte(10).sql_less_than(&SqlByte(10))` → `SqlBoolean::FALSE`
  - `test_sql_greater_than` — `SqlByte(20).sql_greater_than(&SqlByte(10))` → `SqlBoolean::TRUE`
  - `test_sql_less_than_or_equal` — `SqlByte(10).sql_less_than_or_equal(&SqlByte(10))` → `SqlBoolean::TRUE`
  - `test_sql_greater_than_or_equal` — `SqlByte(10).sql_greater_than_or_equal(&SqlByte(20))` → `SqlBoolean::FALSE`
  - `test_sql_comparison_null_propagation` — all 6 comparisons with NULL return `SqlBoolean::NULL`

- [ ] T017 [US4] Write PartialEq, Eq, Hash, Ord tests in `src/sql_byte.rs`:
  - `test_partialeq_same` — `SqlByte(42) == SqlByte(42)` returns `true`
  - `test_partialeq_different` — `SqlByte(42) != SqlByte(43)` returns `true`
  - `test_partialeq_null_null` — `SqlByte::NULL == SqlByte::NULL` returns `true`
  - `test_partialeq_null_value` — `SqlByte::NULL != SqlByte(0)` returns `true`
  - `test_hash_consistent` — same value produces same hash
  - `test_hash_null_is_zero` — NULL hashes to same value as `0u8.hash()` equivalent
  - `test_ord_null_less_than_value` — `SqlByte::NULL < SqlByte(0)` returns `true`
  - `test_ord_values` — `SqlByte(10) < SqlByte(20)` returns `true`
  - `test_sorting` — `[SqlByte(20), SqlByte::NULL, SqlByte(10)]` sorts to `[NULL, 10, 20]`

- [ ] T018 [US4] Write Display and FromStr tests in `src/sql_byte.rs`:
  - `test_display_value` — `format!("{}", SqlByte::new(42))` returns `"42"`
  - `test_display_zero` — `format!("{}", SqlByte::ZERO)` returns `"0"`
  - `test_display_max` — `format!("{}", SqlByte::MAX_VALUE)` returns `"255"`
  - `test_display_null` — `format!("{}", SqlByte::NULL)` returns `"Null"`
  - `test_parse_valid` — `"123".parse::<SqlByte>()` returns `Ok(SqlByte(123))`
  - `test_parse_zero` — `"0".parse::<SqlByte>()` returns `Ok(SqlByte(0))`
  - `test_parse_max` — `"255".parse::<SqlByte>()` returns `Ok(SqlByte(255))`
  - `test_parse_null` — `"Null".parse::<SqlByte>()` returns `Ok(SqlByte::NULL)`
  - `test_parse_null_case_insensitive` — `"NULL".parse::<SqlByte>()` returns `Ok(SqlByte::NULL)`
  - `test_parse_overflow` — `"256".parse::<SqlByte>()` returns `Err(ParseError(...))`
  - `test_parse_negative` — `"-1".parse::<SqlByte>()` returns `Err(ParseError(...))`
  - `test_parse_invalid` — `"abc".parse::<SqlByte>()` returns `Err(ParseError(...))`
  - `test_parse_empty` — `"".parse::<SqlByte>()` returns `Err(ParseError(...))`
  - `test_parse_whitespace` — `" 42 ".parse::<SqlByte>()` returns `Ok(SqlByte(42))`
  - `test_display_parse_roundtrip` — `SqlByte::new(42).to_string().parse::<SqlByte>()` returns `Ok(SqlByte(42))`

- [ ] T019 [US4] Write conversion tests in `src/sql_byte.rs`:
  - `test_to_sql_boolean_nonzero` — `SqlByte::new(42).to_sql_boolean()` → `SqlBoolean::TRUE`
  - `test_to_sql_boolean_zero` — `SqlByte::new(0).to_sql_boolean()` → `SqlBoolean::FALSE`
  - `test_to_sql_boolean_null` — `SqlByte::NULL.to_sql_boolean()` → `SqlBoolean::NULL`
  - `test_from_sql_boolean_true` — `SqlByte::from(SqlBoolean::TRUE).value()` → `Ok(1)`
  - `test_from_sql_boolean_false` — `SqlByte::from(SqlBoolean::FALSE).value()` → `Ok(0)`
  - `test_from_sql_boolean_null` — `SqlByte::from(SqlBoolean::NULL).is_null()` → `true`

### Implementation for User Story 4

- [ ] T020 [US4] Implement SQL comparison methods in `src/sql_byte.rs`:
  - `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal`
  - All return `SqlBoolean::NULL` if either operand is NULL

- [ ] T021 [US4] Implement `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord` in `src/sql_byte.rs`:
  - `PartialEq`: NULL == NULL, otherwise compare inner value
  - `Hash`: NULL → hash 0u8, else hash the value
  - `Ord`: NULL < any non-null, otherwise compare by value

- [ ] T022 [US4] Implement `Display` and `FromStr` in `src/sql_byte.rs`:
  - `Display`: NULL → `"Null"`, else `value.to_string()`
  - `FromStr`: trim, check `"Null"` (case-insensitive), else `u8::from_str`, return `ParseError` on failure

- [ ] T023 [US4] Implement `to_sql_boolean()` and `From<SqlBoolean> for SqlByte` in `src/sql_byte.rs`:
  - `to_sql_boolean`: NULL→NULL, 0→FALSE, non-zero→TRUE
  - `From<SqlBoolean>`: NULL→NULL, TRUE→1, FALSE→0

**Checkpoint**: `cargo test` — all comparison, display, parse, and conversion tests pass.

---

## Phase 6: Polish & Validation

**Purpose**: Final quality gates per constitution.

- [ ] T024 Run `cargo fmt --check` and fix any formatting issues in `src/sql_byte.rs`.
- [ ] T025 Run `cargo clippy -- -D warnings` and fix all warnings.
- [ ] T026 Run `cargo test` — verify all tests pass.
- [ ] T027 Run coverage tool (`cargo tarpaulin` or `llvm-cov`) — verify ≥95% coverage for `src/sql_byte.rs`.
- [ ] T028 Review public API surface — ensure only spec'd methods/traits are `pub`. No leaking of internals.

**Checkpoint**: All quality gates pass. SqlByte is complete and ready for PR.

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)         → No dependencies — start immediately
Phase 2 (US1 - Core)    → Depends on Phase 1 (needs module registered)
Phase 3 (US2 - Arith)   → Depends on Phase 2 (needs SqlByte struct + constants)
Phase 4 (US3 - Bitwise) → Depends on Phase 2 (needs SqlByte struct)
Phase 5 (US4 - Compare) → Depends on Phase 2 (needs SqlByte struct)
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
  → Test tasks can be written in parallel
  → Implementation tasks within a phase are sequential
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

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: US1 — construction and inspection
3. Complete Phase 3: US2 — arithmetic with overflow detection
4. **STOP and VALIDATE**: Test US1 + US2 independently
5. Continue to US3 + US4

### Incremental Delivery

1. Setup → Module registered
2. US1 → SqlByte can be created, inspected (MVP core)
3. US2 → Arithmetic works with overflow detection (MVP complete)
4. US3 → Bitwise operations
5. US4 → Comparisons, Display, Parse, conversions
6. Polish → Quality gates

---

## Summary

| Phase | Tasks | Est. Tests | Description |
|-------|-------|------------|-------------|
| 1 — Setup | T001–T002 | 0 | Module registration |
| 2 — US1 Core | T003–T006 | ~10 | Construction, inspection, From<u8> |
| 3 — US2 Arithmetic | T007–T013 | ~27 | Add, Sub, Mul, Div, Rem with overflow |
| 4 — US3 Bitwise | T014–T015 | ~11 | BitAnd, BitOr, BitXor, Not |
| 5 — US4 Compare/Display | T016–T023 | ~37 | SQL comparisons, Rust traits, Display, FromStr, conversions |
| 6 — Polish | T024–T028 | — | fmt, clippy, coverage, API review |
| **Total** | **28 tasks** | **~85 tests** | |
