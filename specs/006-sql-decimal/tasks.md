# Tasks: SqlDecimal

**Input**: Design documents from `/specs/006-sql-decimal/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api.md, quickstart.md

**Tests**: Included — the spec requires ≥90% code coverage (SC-003) and the constitution mandates TDD (Principle III).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All implementation in single file `src/sql_decimal.rs` unless noted

---

## Phase 1: Setup

**Purpose**: Module registration and project wiring

- [X] T001 Create `src/sql_decimal.rs` with module-level doc comment, imports (`crate::error::SqlTypeError`, `crate::sql_boolean::SqlBoolean`, `crate::sql_byte::SqlByte`, `crate::sql_int16::SqlInt16`, `crate::sql_int32::SqlInt32`, `crate::sql_int64::SqlInt64`, std traits), and empty struct definitions for `InnerDecimal` (private) and `SqlDecimal` (public)
- [X] T002 Register module in `src/lib.rs`: add `pub mod sql_decimal;` and `pub use sql_decimal::SqlDecimal;`

**Checkpoint**: `cargo build` compiles with empty `SqlDecimal` struct.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core structs, constants, constructors, multi-precision arithmetic helpers, and scale adjustment that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Phase 2 implements foundational scaffolding (struct, constants, constructors, multi-precision helpers) before their public API tests in Phase 3. This is an intentional deviation from strict TDD — the struct and helpers must exist for test code to compile. Private multi-precision helpers are unit-tested within Phase 2 (T014–T015) because they are complex algorithms that need validation before higher-level code uses them. Public API tests in Phase 3 (US1) validate all remaining Phase 2 work. This matches the established pattern from SqlInt32/SqlInt64.

- [X] T003 Define `InnerDecimal` struct (private, `Copy`, `Clone`, `Debug`) with fields `precision: u8`, `scale: u8`, `positive: bool`, `data: [u32; 4]` and `SqlDecimal` struct (public, `Clone`, `Debug`, NOT `Copy`) with field `inner: Option<InnerDecimal>` in `src/sql_decimal.rs`
- [X] T004 Implement constants: `NULL` (const, `inner: None`), `MAX_PRECISION: u8 = 38`, `MAX_SCALE: u8 = 38` in `src/sql_decimal.rs`
- [X] T005 Implement private helpers: `is_zero(data: &[u32; 4]) -> bool`, `normalize_zero(inner: &mut InnerDecimal)` (sets `positive = true` when mantissa is zero) in `src/sql_decimal.rs`
- [X] T006 Implement private multi-precision comparison: `mp_cmp(a: &[u32; 4], b: &[u32; 4]) -> Ordering` (compares from most-significant word down) in `src/sql_decimal.rs`
- [X] T007 Implement private multi-precision single-word ops: `mp_mul1(a: &[u32; 4], b: u32) -> ([u32; 4], u32)` (multiply array by scalar, return result + carry), `mp_div1(a: &[u32; 4], b: u32) -> ([u32; 4], u32)` (divide array by scalar, return quotient + remainder) in `src/sql_decimal.rs`
- [X] T008 Implement private multi-precision array ops: `mp_add(a: &[u32; 4], b: &[u32; 4]) -> ([u32; 4], u32)` (add with carry), `mp_sub(a: &[u32; 4], b: &[u32; 4]) -> ([u32; 4], u32)` (subtract with borrow) in `src/sql_decimal.rs`
- [X] T009 Implement private multi-precision multiply: `mp_mul(a: &[u32; 4], b: &[u32; 4]) -> ([u32; 5], bool)` (schoolbook O(n²), returns extended result + overflow flag) in `src/sql_decimal.rs`
- [X] T010 Implement private multi-precision divide: `mp_div(a: &[u32; 4], b: &[u32; 4]) -> Option<([u32; 4], [u32; 4])>` (Knuth Algorithm D with single-word fast path via `mp_div1`, returns `None` for divide-by-zero) in `src/sql_decimal.rs`
- [X] T011 Implement private `calculate_precision(data: &[u32; 4]) -> u8` — count decimal digits in mantissa via iterative division by 10 using `mp_div1` in `src/sql_decimal.rs`
- [X] T012 Implement `new(precision, scale, positive, data1, data2, data3, data4) -> Result<SqlDecimal, SqlTypeError>` with validation (precision 1–38, scale 0–precision, mantissa fits within precision), `max_value() -> SqlDecimal`, `min_value() -> SqlDecimal` in `src/sql_decimal.rs`
- [X] T013 Implement accessors: `is_null() -> bool`, `precision() -> Result<u8, SqlTypeError>`, `scale() -> Result<u8, SqlTypeError>`, `is_positive() -> Result<bool, SqlTypeError>`, `data() -> Result<[u32; 4], SqlTypeError>`, `value() -> Result<InnerDecimal, SqlTypeError>` in `src/sql_decimal.rs`
- [X] T014 [P] Write private unit tests for multi-precision helpers: `mp_cmp` (equal, less, greater), `mp_add` (no carry, with carry), `mp_sub` (no borrow, with borrow), `mp_mul1` (scalar multiply, carry), `mp_div1` (scalar divide, remainder) in `src/sql_decimal.rs`
- [X] T015 [P] Write private unit tests for `mp_mul` (small values, large values, overflow detection), `mp_div` (exact division, with remainder, single-word fast path, divide-by-zero returns None), `calculate_precision` (1-digit through 38-digit values), `is_zero` in `src/sql_decimal.rs`
- [X] T016 Implement `adjust_scale(&self, new_scale: u8, round: bool) -> Result<SqlDecimal, SqlTypeError>` — scale increase via `mp_mul1` by powers of 10, scale decrease via `mp_div1` with round-half-up (remainder ≥ divisor/2) or truncate, NULL returns `Ok(NULL)`, overflow returns `Err(Overflow)` in `src/sql_decimal.rs`

**Checkpoint**: Foundation ready — `SqlDecimal::new()`, `is_null()`, accessors, constants, multi-precision helpers, `adjust_scale` all work. `cargo test` passes with private helper tests green.

---

## Phase 3: User Story 1 — Create and Inspect Values (Priority: P1) 🎯 MVP

**Goal**: Users can create SqlDecimal values from components and inspect precision, scale, sign, and NULL status.

**Independent Test**: `SqlDecimal::new(10, 2, true, 12345, 0, 0, 0)` represents `123.45` with `precision()` returning `10` and `scale()` returning `2`. `SqlDecimal::NULL.is_null()` returns `true`. Boundary values (`max_value()`, `min_value()`) round-trip correctly.

### Tests for User Story 1

- [X] T017 [US1] Write tests for `new()` — valid construction `(10, 2, true, 12345, 0, 0, 0)`, negative value `(10, 2, false, 12345, 0, 0, 0)`, invalid precision 0 and 39 → `OutOfRange`, scale > precision → `OutOfRange`, mantissa exceeding declared precision → `Overflow`, all four `u32` components populated (large 128-bit value), trailing fractional zeros preserved (`100.00` keeps scale=2) in `src/sql_decimal.rs`
- [X] T018 [US1] Write tests for constants — `NULL.is_null()` is `true`, `MAX_PRECISION == 38`, `MAX_SCALE == 38`, `max_value()` has precision=38 scale=0 positive=true and data=`[0x098A2240, 0x5A86C47A, 0x4B3B4CA8, 0x4EE2D6D4]`, `min_value()` has positive=false with same mantissa in `src/sql_decimal.rs`
- [X] T019 [US1] Write tests for accessors — `precision()`, `scale()`, `is_positive()`, `data()` return correct values on valid `SqlDecimal`, `value()` on NULL returns `Err(NullValue)`, `precision()` on NULL returns `Err(NullValue)` in `src/sql_decimal.rs`
- [X] T020 [US1] Write tests for negative zero normalization — constructing with `(p, s, false, 0, 0, 0, 0)` produces `is_positive() == true` in `src/sql_decimal.rs`

**Checkpoint**: All US1 acceptance scenarios (10 scenarios) pass. `cargo test` green.

---

## Phase 4: User Story 2 — Arithmetic with Precision/Scale Propagation (Priority: P1)

**Goal**: Users can perform checked arithmetic (+, -, *, /, %, negation) with SQL Server precision/scale propagation rules. Overflow beyond 38 digits returns error. NULL propagates through all operations.

**Independent Test**: `SqlDecimal(123.45) + SqlDecimal(678.90)` returns `SqlDecimal(802.35)` with correct result precision/scale. `SqlDecimal(10.00) / SqlDecimal(0)` returns `Err(DivideByZero)`. Any op with NULL returns NULL. Overflow at precision 38 returns `Err(Overflow)`.

### Tests for User Story 2

- [X] T021 [P] [US2] Write tests for `checked_add` — same scale `(123.45 + 678.90 = 802.35)`, different scales, carry propagation across u32 boundary, result precision/scale per SQL Server rules, overflow at max precision, NULL propagation (both sides) in `src/sql_decimal.rs`
- [X] T022 [P] [US2] Write tests for `checked_sub` — same scale `(100.00 - 200.00 = -100.00)`, different scales, negative result, sign handling (negative - negative), overflow at max precision, NULL propagation in `src/sql_decimal.rs`
- [X] T023 [P] [US2] Write tests for `checked_mul` — normal multiply, precision/scale propagation per SQL Server rules `(p1+p2+1, s1+s2)`, overflow at max precision, multiply by zero, NULL propagation in `src/sql_decimal.rs`
- [X] T024 [P] [US2] Write tests for `checked_div` — normal division `(10.00 / 3.00)` with appropriate result scale, exact division, divide-by-zero → `Err(DivideByZero)`, overflow, minimum division scale = 6, NULL propagation in `src/sql_decimal.rs`
- [X] T025 [P] [US2] Write tests for `checked_rem` — normal remainder `(10.00 % 3.00 = 1.00)`, divide-by-zero → `Err(DivideByZero)`, NULL propagation in `src/sql_decimal.rs`
- [X] T026 [P] [US2] Write tests for `checked_neg` — positive to negative, negative to positive, zero stays zero (positive), NULL returns NULL in `src/sql_decimal.rs`

### Implementation for User Story 2

- [X] T027 [US2] Implement private precision/scale computation helpers: `add_sub_result_prec_scale(p1, s1, p2, s2)`, `mul_result_prec_scale(p1, s1, p2, s2)`, `div_result_prec_scale(p1, s1, p2, s2)` per SQL Server rules (see research.md R4) in `src/sql_decimal.rs`
- [X] T028 [US2] Implement `checked_add` — normalize operands to same scale via `adjust_scale`, magnitude comparison for different-sign addition, `mp_add`/`mp_sub` on mantissas, precision/scale propagation, overflow check, NULL propagation in `src/sql_decimal.rs`
- [X] T029 [US2] Implement `checked_sub` — delegate to `checked_add` with negated rhs (flip sign), NULL propagation in `src/sql_decimal.rs`
- [X] T030 [US2] Implement `checked_mul` — `mp_mul` on mantissas, result sign = XOR of operand signs, precision/scale propagation, `adjust_scale` for result scale adjustment, overflow check, NULL propagation in `src/sql_decimal.rs`
- [X] T031 [US2] Implement `checked_div` — check divisor not zero → `Err(DivideByZero)`, scale dividend via `adjust_scale` to achieve target result scale, `mp_div` for quotient, result sign = XOR of operand signs, precision/scale propagation, overflow check, NULL propagation in `src/sql_decimal.rs`
- [X] T032 [US2] Implement `checked_rem` — check divisor not zero → `Err(DivideByZero)`, compute `a - truncate(a/b) * b`, NULL propagation in `src/sql_decimal.rs`
- [X] T033 [US2] Implement `checked_neg` — flip sign flag, normalize negative zero, NULL propagation in `src/sql_decimal.rs`
- [X] T034 [US2] Implement operator traits `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` for both `SqlDecimal` (by value) and `&SqlDecimal` (by reference), all delegating to `checked_*` methods in `src/sql_decimal.rs`

**Checkpoint**: All US2 acceptance scenarios (11 scenarios) pass. Precision/scale propagation verified. `cargo test` green.

---

## Phase 5: User Story 3 — Comparison Returning SqlBoolean (Priority: P1)

**Goal**: Users can compare SqlDecimal values using SQL three-valued logic. Comparisons return `SqlBoolean`. Values with different scales but same mathematical value compare as equal.

**Independent Test**: `SqlDecimal(100.00).sql_equals(&SqlDecimal(100.0000))` returns `SqlBoolean::TRUE`. `SqlDecimal(100.00).sql_less_than(&SqlDecimal(200.00))` returns `SqlBoolean::TRUE`. Any comparison with NULL returns `SqlBoolean::NULL`.

### Tests for User Story 3

- [X] T035 [US3] Write tests for 6 SQL comparison methods — equal values `(100.00 == 100.00)`, equal values with different scales `(100.00 == 100.0000)`, unequal values, `sql_less_than` positive cases, `sql_greater_than` positive cases, `sql_less_than_or_equal` at equality, `sql_greater_than_or_equal` at equality, `sql_not_equals`, negative vs positive values (negative < positive), NULL propagation on both operand sides in `src/sql_decimal.rs`

### Implementation for User Story 3

- [X] T036 [US3] Implement private scale-normalized comparison helper (normalize both operands to same scale via `adjust_scale`, compare signs then `mp_cmp` on mantissas) and 6 SQL comparison methods: `sql_equals`, `sql_not_equals`, `sql_less_than`, `sql_greater_than`, `sql_less_than_or_equal`, `sql_greater_than_or_equal` — all return `SqlBoolean`, NULL propagation in `src/sql_decimal.rs`

**Checkpoint**: All US3 acceptance scenarios (10 scenarios) pass. `cargo test` green.

---

## Phase 6: User Story 4 — Display and Parsing (Priority: P2)

**Goal**: Users can convert SqlDecimal to and from string representations. NULL displays as `"Null"`. Parsing supports decimal notation with optional sign. Invalid strings return parse error.

**Independent Test**: `format!("{}", SqlDecimal(123.45))` produces `"123.45"`. `"123.45".parse::<SqlDecimal>()` returns `Ok` with correct precision/scale. `"abc".parse::<SqlDecimal>()` returns `Err(ParseError)`.

### Tests for User Story 4

- [X] T037 [P] [US4] Write tests for `Display` — positive decimal `(123.45 → "123.45")`, negative `(-123.45 → "-123.45")`, integer-like with preserved scale `(100.00 → "100.00")`, zero `(0.00 → "0.00")`, NULL displays `"Null"`, large value using all four u32 words in `src/sql_decimal.rs`
- [X] T038 [P] [US4] Write tests for `FromStr` — `"123.45"` → precision=5 scale=2, `"-0.001"` → negative scale=3, `"42"` no decimal point → scale=0, `"abc"` → `ParseError`, string with >38 significant digits → error, `"Null"` → `NULL`, leading zeros `"007.50"` → correct value, trailing zeros `"1.0"` vs `"1.00"` different scale in `src/sql_decimal.rs`

### Implementation for User Story 4

- [X] T039 [US4] Implement `Display` — convert 4×u32 mantissa to decimal digit string by repeated `mp_div1` by 10 to extract digits, insert decimal point at scale position, prepend sign for negative, output `"Null"` for NULL in `src/sql_decimal.rs`
- [X] T040 [US4] Implement `FromStr` — parse optional `"-"`/`"+"` sign, integer part and optional `"."` + fractional part, count digits for precision/scale, convert digit characters to 4×u32 mantissa via repeated `mp_mul1` by 10 + add digit, handle `"Null"` → NULL, return `ParseError` for invalid input in `src/sql_decimal.rs`

**Checkpoint**: All US4 acceptance scenarios (9 scenarios) pass. Display/FromStr round-trip verified. `cargo test` green.

---

## Phase 7: User Story 5 — Scale Adjustment and Rounding (Priority: P2)

**Goal**: Users can adjust the scale of a SqlDecimal, with rounding or truncation. Needed for combining values with different scales or storing into a column with specific scale.

**Independent Test**: `SqlDecimal(123.456).adjust_scale(2, true)` returns `SqlDecimal(123.46)` (rounded). `SqlDecimal(123.456).adjust_scale(2, false)` returns `SqlDecimal(123.45)` (truncated). `SqlDecimal(123.45).adjust_scale(4, true)` returns `SqlDecimal(123.4500)` (zero-padded).

### Tests for User Story 5

- [X] T041 [US5] Write tests for `adjust_scale` — increase scale zero-pad `(123.45 → 123.4500 at scale=4)`, decrease with round-half-up `(123.456 → 123.46 at scale=2)`, decrease with truncate `(123.456 → 123.45 at scale=2)`, round at midpoint `(123.455 → 123.46 with round=true, away from zero)`, round-half-up for negative values `(-123.455 → -123.46)`, scale to 0 (integer conversion), already-at-target-scale (no-op), NULL propagation → `Ok(NULL)`, overflow when resulting precision exceeds 38 → `Err(Overflow)` in `src/sql_decimal.rs`

**Checkpoint**: All US5 acceptance scenarios (7 scenarios) pass. `cargo test` green.

---

## Phase 8: User Story 6 — Conversions To and From Other Types (Priority: P3)

**Goal**: Users can convert between SqlDecimal and integer/boolean/float types. Widening conversions (from integers) always succeed. Narrowing conversions (to integers) truncate fractional parts and check range.

**Independent Test**: `SqlDecimal::from(SqlInt32::new(42)).precision() == 10` and `scale() == 0`. `SqlDecimal(42.99).to_sql_int32()` returns `Ok(SqlInt32(42))`. `SqlDecimal(value > i64::MAX).to_sql_int64()` returns `Err(Overflow)`. `SqlDecimal::from(SqlBoolean::TRUE)` returns value `1`.

### Tests for User Story 6

- [X] T042 [P] [US6] Write tests for widening conversions: `From<i32>` (42 → precision=10, scale=0), `From<i64>` (9_000_000_000 → precision=19, scale=0), `From<SqlBoolean>` (NULL→NULL, FALSE→0, TRUE→1), `From<SqlByte>` (200 → precision=3, scale=0), `From<SqlInt16>` (1000 → precision=5, scale=0), `From<SqlInt32>` (42 → precision=10, scale=0), `From<SqlInt64>` (9_000_000_000 → precision=19, scale=0), NULL propagation for all SqlType conversions in `src/sql_decimal.rs`
- [X] T043 [P] [US6] Write tests for narrowing conversions: `to_f64()` (42.99 → closest double, NULL → Err(NullValue)), `to_sql_int32()` (42.99 → 42 truncated, overflow beyond i32 range, NULL propagation), `to_sql_int64()` (42.99 → 42, overflow beyond i64 range, NULL propagation), `to_sql_int16()` (100 → 100, overflow beyond i16 range, NULL propagation), `to_sql_byte()` (200 → 200, overflow >255 and <0, NULL propagation), `to_sql_boolean()` (0→FALSE, non-zero→TRUE, NULL→NULL) in `src/sql_decimal.rs`

### Implementation for User Story 6

- [X] T044 [US6] Implement `From<i32>`, `From<i64>` for SqlDecimal — convert absolute value to `data[0..1]`, set precision (10 for i32, 19 for i64), scale=0 in `src/sql_decimal.rs`
- [X] T045 [US6] Implement `From<SqlBoolean>`, `From<SqlByte>`, `From<SqlInt16>`, `From<SqlInt32>`, `From<SqlInt64>` for SqlDecimal — extract inner value, convert via `From<i32>` or `From<i64>`, propagate NULL in `src/sql_decimal.rs`
- [X] T046 [US6] Implement `to_f64(&self) -> Result<f64, SqlTypeError>` — convert 4×u32 mantissa to f64 via `(data[3] as f64 * 2^96 + data[2] as f64 * 2^64 + data[1] as f64 * 2^32 + data[0] as f64) / 10^scale`, apply sign, NULL → `Err(NullValue)` in `src/sql_decimal.rs`
- [X] T047 [US6] Implement `to_sql_int32`, `to_sql_int64`, `to_sql_int16`, `to_sql_byte` (truncate fractional via `adjust_scale(0, false)`, extract integer value, range check against target type bounds → `Err(Overflow)`, NULL propagation) and `to_sql_boolean` (zero→FALSE, non-zero→TRUE, NULL→NULL) in `src/sql_decimal.rs`

**Checkpoint**: All US6 acceptance scenarios (8 scenarios) pass. `cargo test` green.

---

## Phase 9: User Story 7 — Mathematical Functions (Priority: P3)

**Goal**: Users can call mathematical helper functions: `abs()`, `floor()`, `ceiling()`, `round()`, `truncate()`, `sign()`, `power()`.

**Independent Test**: `SqlDecimal(-123.45).abs()` returns `SqlDecimal(123.45)`. `SqlDecimal(123.45).floor()` returns `SqlDecimal(123)`. `SqlDecimal(123.456).round(2)` returns `SqlDecimal(123.46)`. `SqlDecimal(5.00).power(3)` returns `SqlDecimal(125.000000)`.

**Dependency Note**: `power()` requires `to_f64()` (T046) and an internal f64-to-SqlDecimal helper. Other math functions have no cross-story dependencies.

### Tests for User Story 7

- [X] T048 [P] [US7] Write tests for `abs` (negative→positive, positive unchanged, zero, NULL→NULL), `floor` (123.45→123, -123.45→-124, integer value no change), `ceiling` (123.45→124, -123.45→-123, integer value no change), `sign` (positive→1, negative→-1, zero→0, NULL→SqlInt32::NULL) in `src/sql_decimal.rs`
- [X] T049 [P] [US7] Write tests for `round` (123.456 round to 2 → 123.46, round to 0 → 123, negative position, NULL→NULL), `truncate` (123.456 truncate to 2 → 123.45, truncate to 0 → 123, NULL→NULL), `power` (5^3=125, 2^10=1024, 0^5=0, x^0=1, NULL→NULL) in `src/sql_decimal.rs`

### Implementation for User Story 7

- [X] T050 [US7] Implement `abs(&self) -> SqlDecimal` (set positive=true, NULL→NULL), `floor(&self) -> Result<SqlDecimal, SqlTypeError>` (truncate toward negative infinity via `adjust_scale(0, false)` then subtract 1 if negative with fractional part), `ceiling(&self) -> Result<SqlDecimal, SqlTypeError>` (truncate toward positive infinity), `sign(&self) -> SqlInt32` (return -1/0/1 as SqlInt32, NULL→SqlInt32::NULL) in `src/sql_decimal.rs`
- [X] T051 [US7] Implement `round(&self, position: i32) -> Result<SqlDecimal, SqlTypeError>` (delegate to `adjust_scale` with round=true), `truncate(&self, position: i32) -> Result<SqlDecimal, SqlTypeError>` (delegate to `adjust_scale` with round=false), `power(&self, exponent: i32) -> Result<SqlDecimal, SqlTypeError>` (convert to f64 via `to_f64`, call `f64::powi`, convert back, set precision=38, adjust scale to original, NULL→NULL) in `src/sql_decimal.rs`

**Checkpoint**: All US7 acceptance scenarios (9 scenarios) pass. `cargo test` green.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Standard Rust traits and final quality gates that span all user stories

### Tests

- [X] T052 [P] Write tests for `PartialEq`/`Eq` — value equality (`100.00 == 100.00`), `NULL == NULL` (Rust semantics), different values not equal, equal values with different scales in `src/sql_decimal.rs`
- [X] T053 [P] Write tests for `Hash` (equal values hash equal, `NULL` hashes consistently, equal values with different scale produce same hash) and `PartialOrd`/`Ord` (`NULL < any value`, `MIN_VALUE < MAX_VALUE`, negative < positive, equal values) in `src/sql_decimal.rs`

### Implementation

- [X] T054 Implement `PartialEq`, `Eq` (scale-normalized comparison, NULL==NULL per Rust semantics) and `Hash` (normalize to canonical form before hashing, NULL hashes consistently) for SqlDecimal in `src/sql_decimal.rs`
- [X] T055 Implement `PartialOrd`, `Ord` for SqlDecimal (NULL < any non-NULL value, then sign-aware scale-normalized comparison) in `src/sql_decimal.rs`
- [X] T056 Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` — all must pass in `src/sql_decimal.rs`
- [X] T057 Run quickstart.md scenarios as validation smoke test

**Checkpoint**: All quality gates pass. ≥90% coverage. `cargo fmt`, `cargo clippy`, `cargo test` all green.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — tests for constructors/constants/accessors
- **US2 (Phase 4)**: Depends on Phase 2 — arithmetic uses mp_* helpers and adjust_scale
- **US3 (Phase 5)**: Depends on Phase 2 — comparisons use mp_cmp and adjust_scale
- **US4 (Phase 6)**: Depends on Phase 2 — Display/FromStr use mp_div1/mp_mul1
- **US5 (Phase 7)**: Depends on Phase 2 — tests validate adjust_scale (already implemented in T016)
- **US6 (Phase 8)**: Depends on Phase 2 — conversions use constructors and mp_* helpers
- **US7 (Phase 9)**: Depends on Phase 2 + T046 (to_f64 from US6, needed by power())
- **Polish (Phase 10)**: Depends on Phase 2 — traits need constructors; can overlap with US phases

### User Story Independence

- **US1 (P1)**: Standalone — only needs foundational struct/constants/accessors
- **US2 (P1)**: Standalone — only needs foundational struct + mp_* helpers + adjust_scale
- **US3 (P1)**: Standalone — only needs foundational struct + mp_cmp + adjust_scale
- **US4 (P2)**: Standalone — only needs foundational struct + mp_div1/mp_mul1
- **US5 (P2)**: Standalone — only needs foundational adjust_scale (Phase 2)
- **US6 (P3)**: Standalone — only needs foundational struct + existing SqlTypes (all already exist)
- **US7 (P3)**: Mostly standalone — `power()` depends on `to_f64()` from US6 (T046). Other math functions are independent.
- All user stories except US7 can be implemented in parallel after Phase 2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD per Constitution III)
- Implementation follows test order
- Story complete = all tests green

### Parallel Opportunities

- T014–T015 (Phase 2 helper tests) can run in parallel — independent test groups
- T021–T026 (US2 test methods) can run in parallel — independent test functions
- T037–T038 (US4 tests) can run in parallel — Display and FromStr are independent
- T042–T043 (US6 tests) can run in parallel — widening and narrowing test groups
- T048–T049 (US7 tests) can run in parallel — abs/floor/ceiling/sign vs round/truncate/power
- T052–T053 (Phase 10 tests) can run in parallel — PartialEq/Eq vs Hash/Ord
- US1–US6 can all be worked on in parallel after Phase 2

---

## Parallel Example: User Story 2

```text
# Write all US2 tests in parallel (T021-T026):
T021: tests for checked_add
T022: tests for checked_sub
T023: tests for checked_mul
T024: tests for checked_div
T025: tests for checked_rem
T026: tests for checked_neg

# Then implement sequentially (T027-T034):
T027: precision/scale computation helpers
T028: checked_add
T029: checked_sub
T030: checked_mul
T031: checked_div
T032: checked_rem
T033: checked_neg
T034: operator trait wiring
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 + 3)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T016)
3. Complete Phase 3: US1 tests — construction and inspection validated (T017–T020)
4. Complete Phase 4: US2 arithmetic (T021–T034)
5. Complete Phase 5: US3 comparisons (T035–T036)
6. **STOP and VALIDATE**: Core type works — values can be created, inspected, computed, and compared

### Incremental Delivery

1. Setup + Foundational → module compiles with helper tests
2. US1 → values can be created and inspected
3. US2 → arithmetic works with precision/scale propagation and overflow detection
4. US3 → SQL three-valued comparisons
5. US4 → Display and FromStr (string conversion)
6. US5 → scale adjustment validated
7. US6 → cross-type conversions
8. US7 → mathematical functions (abs, floor, ceiling, round, truncate, sign, power)
9. Polish → standard Rust traits, quality gates

---

## Summary

| Metric | Count |
|--------|-------|
| Total tasks | 57 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 14 |
| Phase 3 (US1) | 4 |
| Phase 4 (US2) | 14 |
| Phase 5 (US3) | 2 |
| Phase 6 (US4) | 4 |
| Phase 7 (US5) | 1 |
| Phase 8 (US6) | 6 |
| Phase 9 (US7) | 4 |
| Phase 10 (Polish) | 6 |
| Parallelizable tasks | 16 |
| Test tasks | 21 |
| Implementation tasks | 36 |
