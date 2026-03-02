# Tasks: SqlString

**Input**: Design documents from `/specs/013-sql-string/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥95% code coverage (SC-006) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Implementation spans two new files: `src/sql_compare_options.rs` and `src/sql_string.rs`

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [x] T001 Create `src/sql_compare_options.rs` with module-level doc comment and empty `SqlCompareOptions` enum definition
- [x] T002 [P] Create `src/sql_string.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_compare_options::SqlCompareOptions`, `std::fmt`, `std::str::FromStr`, `std::ops::Add`, `std::hash`, `std::cmp`), and empty struct definition
- [x] T003 Register modules in `src/lib.rs`: add `pub mod sql_compare_options;` and `pub mod sql_string;`, add `pub use sql_compare_options::SqlCompareOptions;` and `pub use sql_string::SqlString;`

**Checkpoint**: `cargo build` compiles with empty `SqlCompareOptions` enum and `SqlString` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types, constants, and constructors that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (enum, struct, constants, constructors) before their tests in Phase 3. This is an intentional deviation from strict TDD — the types must exist for test code to compile. Tests in Phase 3 (T008–T012) validate all Phase 2 work before any further phases proceed. This matches the established pattern from SqlInt32/SqlInt64.

- [x] T004 Define `SqlCompareOptions` enum with variants `None`, `IgnoreCase`, `BinarySort`, `BinarySort2`; derive `Copy, Clone, Debug, PartialEq, Eq, Hash, Default` (with `#[default] IgnoreCase`) in `src/sql_compare_options.rs`
- [x] T005 Define `SqlString` struct with `value: Option<String>` and `compare_options: SqlCompareOptions` fields; derive `Clone, Debug` in `src/sql_string.rs`
- [x] T006 Implement `SqlString::NULL` constant (`value: None, compare_options: IgnoreCase`), `new(s: &str) -> Self` (with default `IgnoreCase` options), `with_options(s: &str, options: SqlCompareOptions) -> Self` in `src/sql_string.rs`
- [x] T007 Implement `is_null() -> bool`, `value() -> Result<&str, SqlTypeError>`, `len() -> Result<usize, SqlTypeError>`, `compare_options() -> SqlCompareOptions` in `src/sql_string.rs`

**Checkpoint**: Foundation ready — `SqlCompareOptions` enum, `SqlString::new()`, `with_options()`, `is_null()`, `value()`, `len()`, `compare_options()`, `NULL` constant all work. `cargo build` compiles.

---

## Phase 3: User Story 1 — Create and Inspect String Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlString values from Rust strings, inspect their contents, check for NULL, and retrieve the stored string value and its length.

**Independent Test**: `SqlString::new("hello").value()` returns `Ok("hello")`, `SqlString::NULL.is_null()` returns `true`, empty string is not NULL, default options are `IgnoreCase`.

### Tests for User Story 1

- [x] T008 [P] [US1] Write tests for `SqlCompareOptions` — all 4 variants exist, `Default` returns `IgnoreCase`, `Copy`/`Clone`/`Debug`/`PartialEq`/`Eq`/`Hash` work in `src/sql_compare_options.rs`
- [x] T009 [P] [US1] Write tests for `new()`, `is_null()`, `value()` — create "hello" and verify value, NULL access returns `Err(NullValue)`, empty string is not NULL, `is_null()` returns false for non-null in `src/sql_string.rs`
- [x] T010 [P] [US1] Write tests for `len()` — "hello" returns 5, empty string returns 0, multi-byte UTF-8 ("🦀") returns 4, NULL returns `Err(NullValue)` in `src/sql_string.rs`
- [x] T011 [P] [US1] Write tests for `with_options()` — create with each of the 4 `SqlCompareOptions` variants, verify `compare_options()` returns the correct variant in `src/sql_string.rs`
- [x] T012 [P] [US1] Write tests for default options — `SqlString::new("hello").compare_options()` returns `IgnoreCase`, `SqlString::NULL` has `IgnoreCase` options in `src/sql_string.rs`

**Checkpoint**: All US1 acceptance scenarios (7 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Concatenation (Priority: P1)

**Goal**: Users can concatenate two SqlString values using the `+` operator. NULL propagation applies. Result inherits the left operand's compare options. Concatenation is infallible (returns `SqlString` directly, not `Result`).

**Independent Test**: `SqlString("hello") + SqlString(" world")` returns `SqlString("hello world")`, concatenation with NULL returns NULL, empty string concatenation works.

### Tests for User Story 2

- [x] T013 [P] [US2] Write tests for `Add` operator — "hello" + " world" = "hello world", "" + "hello" = "hello", "hello" + "" = "hello" in `src/sql_string.rs`
- [x] T014 [P] [US2] Write tests for NULL propagation in concatenation — non-null + NULL = NULL, NULL + non-null = NULL, NULL + NULL = NULL in `src/sql_string.rs`
- [x] T015 [P] [US2] Write tests for compare options inheritance in concatenation — left operand's options govern result, verify with `IgnoreCase` + `BinarySort` and `BinarySort` + `IgnoreCase` in `src/sql_string.rs`

### Implementation for User Story 2

- [x] T016 [US2] Implement `Add<SqlString> for SqlString` with NULL propagation, left operand's options inheritance in `src/sql_string.rs`

**Checkpoint**: All US2 acceptance scenarios (5 scenarios) pass. `cargo test` green.

---

## Phase 5: User Story 3 — Comparison with Configurable Options (Priority: P2)

**Goal**: Users can compare SqlString values using SQL comparison methods that respect configurable comparison options. Default comparison is case-insensitive. Binary sort compares bytes directly. Left operand's options govern mixed-option comparisons.

**Independent Test**: `SqlString("ABC").sql_equals(&SqlString("abc"))` returns `SqlBoolean::TRUE` (IgnoreCase), `SqlString::with_options("ABC", BinarySort).sql_equals(&SqlString("abc"))` returns `FALSE`, comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 3

- [x] T017 [P] [US3] Write tests for IgnoreCase comparisons — "ABC" eq "abc" = TRUE, "apple" lt "banana" = TRUE, "Hello" eq "hello" = TRUE, trailing spaces ignored ("hello" eq "hello   " = TRUE) in `src/sql_string.rs`
- [x] T018 [P] [US3] Write tests for BinarySort comparisons — "ABC" eq "abc" = FALSE, "A" lt "a" = TRUE (0x41 < 0x61), "abc" eq "abc" = TRUE in `src/sql_string.rs`
- [x] T019 [P] [US3] Write tests for None (ordinal) comparisons — "Hello" eq "hello" = FALSE (case-sensitive), "abc" eq "abc" = TRUE, "A" lt "B" = TRUE in `src/sql_string.rs`
- [x] T020 [P] [US3] Write tests for NULL propagation — all 6 comparison methods with NULL on left, NULL on right, both NULL all return `SqlBoolean::NULL` in `src/sql_string.rs`
- [x] T021 [P] [US3] Write tests for left-operand-options-govern — `IgnoreCase("hello")` compared with `BinarySort("HELLO")` uses IgnoreCase (equals TRUE), `BinarySort("hello")` compared with `IgnoreCase("HELLO")` uses BinarySort (equals FALSE) in `src/sql_string.rs`

### Implementation for User Story 3

- [x] T022 [US3] Implement private helper `compare_strings(&self, other: &SqlString) -> Option<std::cmp::Ordering>` — trim trailing spaces, dispatch on `self.compare_options` to ordinal/case-insensitive/byte comparison in `src/sql_string.rs`
- [x] T023 [US3] Implement `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` using `compare_strings` helper, NULL propagation in `src/sql_string.rs`

**Checkpoint**: All US3 acceptance scenarios (6 scenarios) pass. SC-001 (5 case-insensitive test pairs) and SC-002 (binary vs case-insensitive differs) verified. `cargo test` green.

---

## Phase 6: User Story 4 — Construction with Explicit Compare Options (Priority: P2)

**Goal**: Users can create SqlString values with explicit comparison options to control how the string participates in comparisons. (Note: `with_options()` constructor was implemented in Phase 2; this phase adds focused acceptance tests and validates the full constructor-to-comparison workflow.)

**Independent Test**: `SqlString::with_options("hello", SqlCompareOptions::None).sql_equals(&SqlString::new("HELLO"))` returns `FALSE`, `SqlString::with_options("hello", IgnoreCase).sql_equals(&SqlString::new("HELLO"))` returns `TRUE`.

### Tests for User Story 4

- [x] T024 [P] [US4] Write end-to-end tests for each compare option variant — create with `None`, compare case-sensitive; create with `IgnoreCase`, compare case-insensitive; create with `BinarySort`, compare bytes; create with `BinarySort2`, verify same as `BinarySort` in `src/sql_string.rs`

**Checkpoint**: All US4 acceptance scenarios (3 scenarios) pass. `cargo test` green.

---

## Phase 7: User Story 5 — Display and Parsing (Priority: P2)

**Goal**: Users can display string values for output and parse strings from input. NULL displays as "Null". Parsing "Null" (case-insensitive) returns `SqlString::NULL`.

**Independent Test**: `format!("{}", SqlString::new("hello"))` returns `"hello"`, `format!("{}", SqlString::NULL)` returns `"Null"`, `"hello".parse::<SqlString>()` returns `Ok(SqlString("hello"))`, `"Null".parse::<SqlString>()` returns `Ok(SqlString::NULL)`.

### Tests for User Story 5

- [x] T025 [P] [US5] Write tests for `Display` — "hello" displays as "hello", empty string displays as "", NULL displays as "Null" in `src/sql_string.rs`
- [x] T026 [P] [US5] Write tests for `FromStr` — "hello" parses to `SqlString("hello")`, "Null"/"null"/"NULL"/"nUlL" all parse to NULL, parsed value has default `IgnoreCase` options in `src/sql_string.rs`

### Implementation for User Story 5

- [x] T027 [US5] Implement `Display` for SqlString (NULL → "Null", non-null → raw value) in `src/sql_string.rs`
- [x] T028 [US5] Implement `FromStr` for SqlString ("Null" case-insensitive → NULL, else `new(input)`) in `src/sql_string.rs`

**Checkpoint**: All US5 acceptance scenarios (4 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits, From conversions, and final quality gates that span all user stories

### Tests

- [x] T029 [P] Write tests for `From<&str>` and `From<String>` — `"hello".into()` creates SqlString with value "hello" and IgnoreCase options, `String::from("world").into()` creates SqlString, verify no unnecessary clone for `From<String>` in `src/sql_string.rs`
- [x] T030 [P] Write tests for `PartialEq`/`Eq` — case-insensitive equality ("Hello" == "hello"), trailing-space-trimmed ("hello" == "hello   "), NULL == NULL, NULL != non-null, different compare options same value are equal in `src/sql_string.rs`
- [x] T031 [P] Write tests for `Hash` — equal values (case-different) hash equal, NULL hashes consistently, can insert into `HashSet` and find case-insensitive in `src/sql_string.rs`
- [x] T032 [P] Write tests for `PartialOrd`/`Ord` — NULL < any non-null, case-insensitive ordering ("apple" < "Banana"), equal values with different case in `src/sql_string.rs`

### Implementation

- [x] T033 Implement `From<&str> for SqlString` and `From<String> for SqlString` (both with default `IgnoreCase` options; `From<String>` avoids clone) in `src/sql_string.rs`
- [x] T034 Implement `PartialEq`, `Eq` for SqlString (case-insensitive ASCII, trailing-space-trimmed; NULL == NULL) in `src/sql_string.rs`
- [x] T035 Implement `Hash` for SqlString (hash of lowercased + trimmed value; NULL hashes as empty string) in `src/sql_string.rs`
- [x] T036 Implement `PartialOrd`, `Ord` for SqlString (case-insensitive; NULL < any non-NULL) in `src/sql_string.rs`
- [x] T037 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass
- [x] T038 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥95% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants
- **US2 (Phase 4)**: Depends on Phase 2 — concatenation needs constructors
- **US3 (Phase 5)**: Depends on Phase 2 — comparisons need constructors + `SqlBoolean` (already exists)
- **US4 (Phase 6)**: Depends on Phase 5 — end-to-end option validation needs comparison methods
- **US5 (Phase 7)**: Depends on Phase 2 — Display/FromStr need constructors
- **Polish (Phase 8)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants/constructors
- **US2 (P1)**: Standalone — only needs foundational struct/constants
- **US3 (P2)**: Standalone — only needs foundational struct + `SqlBoolean` (already exists)
- **US4 (P2)**: Depends on US3 — validates constructor-to-comparison workflow
- **US5 (P2)**: Standalone — only needs foundational struct/constants
- US1, US2, US3, US5 can be implemented in parallel after Phase 2
- US4 should be done after US3

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T001–T002 (Phase 1 setup files) can run in parallel — different files
- T008–T012 (US1 tests) can run in parallel — independent test functions
- T013–T015 (US2 tests) can run in parallel
- T017–T021 (US3 tests) can run in parallel
- T025–T026 (US5 tests) can run in parallel
- T029–T032 (Phase 8 tests) can run in parallel
- US1, US2, US3, US5 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 3

```text
# Write all US3 tests in parallel (T017-T021):
T017: tests for IgnoreCase comparisons
T018: tests for BinarySort comparisons
T019: tests for None (ordinal) comparisons
T020: tests for NULL propagation across all 6 methods
T021: tests for left-operand-options-govern behavior

# Then implement sequentially (T022-T023):
T022: compare_strings helper (trim + dispatch)
T023: 6 SQL comparison methods using helper
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T007)
3. Complete Phase 3: US1 tests + validation (T008–T012)
4. Complete Phase 4: US2 concatenation (T013–T016)
5. **STOP and VALIDATE**: Struct, constructors, concatenation all work

### Incremental Delivery

1. Setup + Foundational → modules compile
2. US1 → values can be created and inspected
3. US2 → concatenation works with NULL propagation
4. US3 → SQL comparisons with configurable options
5. US4 → constructor-to-comparison end-to-end validation
6. US5 → Display and FromStr
7. Polish → standard traits (Eq, Hash, Ord, From), quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 38 |
| Phase 1 (Setup) | 3 |
| Phase 2 (Foundational) | 4 |
| Phase 3 (US1) | 5 |
| Phase 4 (US2) | 4 |
| Phase 5 (US3) | 7 |
| Phase 6 (US4) | 1 |
| Phase 7 (US5) | 4 |
| Phase 8 (Polish) | 10 |
| Parallelizable tasks | 19 |
| Test tasks | 17 |
| Implementation tasks | 21 |
