# Research: SqlDateTime

## R1: Duration Representation

**Decision**: Use `(i32, i32)` parameters on `checked_add(day_delta: i32, time_delta: i32)` and `checked_sub(day_delta: i32, time_delta: i32)` rather than a custom Duration type. Provide convenience constructors: `checked_add_days(days: i32)`, `checked_add_ticks(ticks: i32)`.

**Rationale**: C# uses `TimeSpan` for duration arithmetic (`SqlDateTime + TimeSpan`), round-tripping through `DateTime`. Rust has no standard date/time type to mirror this pattern. `std::time::Duration` is unsigned-only and unsuitable for subtraction (can't represent negative intervals). A custom `SqlDuration` struct would be over-engineered for two integers. The `(i32, i32)` parameter approach is minimal, explicit, and allows the caller to express any combination of day and time offsets without allocating an intermediate struct.

**Alternatives considered**:
- `std::time::Duration`: Unsigned-only, cannot represent negative intervals for subtraction. Rejected.
- Custom `SqlDuration` struct: Over-engineered — introduces a new type for what is fundamentally two integer parameters. Rejected.
- Single `i64` total-ticks parameter: Requires callers to compute `days * TICKS_PER_DAY + ticks`, error-prone. Rejected.

## R2: Calendar Computation (day_ticks from YMD)

**Decision**: Direct port of C# formula: `y = year - 1; day_ticks = y*365 + y/4 - y/100 + y/400 + DAYS_TO_MONTH[month-1] + day - 1 - DAY_BASE` where `DAY_BASE = 693595` (Jan 1, 1900 in absolute days). Use `DAYS_TO_MONTH_365` and `DAYS_TO_MONTH_366` lookup tables for month-day offsets.

**Rationale**: C#'s `SqlDateTime` constructor delegates to `DateTime`'s internal `DateToTicks()` method, which uses the standard Gregorian calendar formula with cumulative month-day lookup tables. The formula has been stable for 20+ years in .NET. Direct porting avoids any dependency on external date libraries while maintaining behavioral fidelity.

Lookup tables:
- `DAYS_TO_MONTH_365: [i32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365]`
- `DAYS_TO_MONTH_366: [i32; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366]`

Leap year check: `year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)`

**Alternatives considered**:
- Use `chrono` crate for calendar math: Rejected per Constitution VI (no external deps).
- Custom algorithm without lookup tables: More complex (nested conditionals), slower. Rejected.

## R3: Reverse Computation (YMD from day_ticks)

**Decision**: Implement the standard 400-year/100-year/4-year/1-year cycle decomposition algorithm manually. Convert `day_ticks + DAY_BASE` to absolute days, then decompose via the cycle constants: 146097 (days/400yr), 36524 (days/100yr), 1461 (days/4yr), 365 (days/yr).

**Rationale**: C# `SqlDateTime.ToString()` delegates to `DateTime`, which internally extracts year/month/day from a tick count using this exact algorithm. Since we don't have a standard `DateTime` type, we must implement the reverse computation ourselves. The algorithm is well-known, deterministic, and handles all edge cases (century boundaries, leap year cycles).

Algorithm outline:
1. `abs_day = day_ticks + DAY_BASE` (convert to absolute day number from year 1)
2. Decompose into 400-year cycles: `n400 = abs_day / 146097`
3. Remainder into 100-year cycles: `n100 = min(remainder / 36524, 3)` (clamped for century correction)
4. Remainder into 4-year cycles: `n4 = remainder / 1461`
5. Remainder into single years: `n1 = min(remainder / 365, 3)` (clamped for leap year correction)
6. `year = n400*400 + n100*100 + n4*4 + n1 + 1`
7. Use `DAYS_TO_MONTH` table to find month and day from remaining days

**Alternatives considered**:
- Depend on `chrono` for extraction: Rejected per Constitution VI.
- Iterative scanning of years: O(n) in year count, unacceptable for years near 9999. Rejected.

## R4: Time Extraction from time_ticks

**Decision**: Use integer division by tick-rate constants for each component:
- `hour = time_ticks / TICKS_PER_HOUR` (1,080,000)
- `minute = (time_ticks % TICKS_PER_HOUR) / TICKS_PER_MINUTE` (18,000)
- `second = (time_ticks % TICKS_PER_MINUTE) / TICKS_PER_SECOND` (300)
- `millisecond = (remaining_ticks * 1000 + 150) / 300` (rounded conversion from SQL ticks to ms)

**Rationale**: C# extracts time components by converting to `DateTime` (which uses 100ns ticks). Since our internal representation already uses 1/300-second ticks, we extract directly via integer division. The millisecond formula uses the rounding expression `(ticks * 1000 + 150) / 300` to convert fractional SQL ticks to the nearest millisecond, matching C#'s `ToTimeSpan` rounding approach.

**Alternatives considered**:
- Convert to milliseconds first, then extract: Adds an intermediate conversion step with potential precision loss. Rejected.
- Floating-point division: Unnecessary — integer division is exact for these constants. Rejected.

## R5: Comparison Operators

**Decision**: Direct lexicographic `(day_ticks, time_ticks)` comparison. NULL propagation: if either operand is NULL, return `SqlBoolean::NULL`. For `PartialOrd`/`Ord`: `None < Some`, then natural tuple ordering.

**Rationale**: C#'s comparison operators compare `m_day` first, then `m_time` on equality — this is standard lexicographic tuple comparison. In Rust, for `sql_*` methods, implement manually with the NULL check. For `PartialOrd`/`Ord` traits, the `Option<(i32, i32)>` natural ordering (`None < Some`, then tuple ordering) gives exactly the right behavior. This is a trivial port — no design decisions needed.

**Alternatives considered**: None — this is the only correct approach.

## R6: Parse / ToString Implementation

**Decision**: Use a deterministic ISO 8601-like format for both `Display` and `FromStr`:
- **Display**: `"YYYY-MM-DD HH:MM:SS.fff"` (e.g., `"2025-07-17 12:30:00.000"`); NULL displays as `"Null"`
- **FromStr**: Accept `"Null"` → NULL; `"YYYY-MM-DD HH:MM:SS.fff"`, `"YYYY-MM-DD HH:MM:SS"`, `"YYYY-MM-DDTHH:MM:SS.fff"`, `"YYYY-MM-DDTHH:MM:SS"`, `"YYYY-MM-DD"` (time defaults to midnight). Invalid input → `ParseError`; valid format but out-of-range → `OutOfRange`.

**Rationale**: C#'s `ToString()` uses culture-sensitive formatting via `DateTime.ToString((IFormatProvider)null!)` — producing locale-dependent output like `"7/17/2025 12:30:00 PM"`. C#'s `Parse()` leans on .NET's `DateTime.Parse()` ecosystem with fallback to SQL Server-specific formats. Replicating culture-sensitive formatting in Rust without external dependencies is impractical and undesirable for a library targeting data interchange. ISO 8601 is unambiguous, widely supported, and explicitly endorsed by the spec's Assumptions section.

**Alternatives considered**:
- Replicate all C# culture-sensitive formats: Impractical without locale infrastructure, culture-dependent behavior is non-deterministic. Rejected.
- Depend on `chrono` for parsing: Rejected per Constitution VI.
- Support only one strict format: Too restrictive — accepting common variants like `T` separator and optional milliseconds improves usability. Rejected.

## R7: DateTime Interop

**Decision**: No public `ToDateTime` / `FromDateTime` methods in core. Component extraction (year, month, day, hour, minute, second) replaces `ToDateTime` for inspection. Optional `chrono` interop (`From<NaiveDateTime>` / `Into<NaiveDateTime>`) deferred to a feature flag.

**Rationale**: C# has `ToDateTime` / `FromDateTime` because `DateTime` is .NET's lingua franca for date/time. Rust has no standard datetime type. The internal algorithms from C#'s `ToTimeSpan` / `FromTimeSpan` are still relevant for duration arithmetic (R8) but remain internal implementation details. If `chrono` interop is needed, it belongs behind a feature flag per Constitution VI.

**Alternatives considered**:
- Add `chrono` interop now: Rejected — no external deps in core.
- Add a custom `SqlDateTimeComponents` struct: Already covered by individual accessor methods — a struct adds API surface with minimal value. Rejected.

## R8: Arithmetic Implementation

**Decision**: Direct tick manipulation on `(day_ticks, time_ticks)` with `i64` intermediate for time computation and `div_euclid`/`rem_euclid` for normalization.

Algorithm for `checked_add(day_delta, time_delta)`:
1. NULL propagation — if self is NULL, return `Ok(SqlDateTime::NULL)`
2. Compute `new_time: i64 = self.time_ticks as i64 + time_delta as i64`
3. Normalize: `day_carry = new_time.div_euclid(TICKS_PER_DAY as i64)` and `new_time = new_time.rem_euclid(TICKS_PER_DAY as i64)`
4. Compute `new_day: i64 = self.day_ticks as i64 + day_delta as i64 + day_carry`
5. Range-check: `new_day` in `[MIN_DAY, MAX_DAY]` and `new_time` in `[0, MAX_TIME]`
6. Construct result with `new_day as i32` and `new_time as i32`

`checked_sub(day_delta, time_delta)` negates the deltas and calls `checked_add`.

**Rationale**: C# round-trips through `DateTime` for arithmetic (`ToDateTime → DateTime + TimeSpan → FromDateTime`), which introduces unnecessary precision conversions (SQL 1/300s ticks → .NET 100ns ticks → back). Direct tick manipulation is simpler, avoids precision loss, and is more efficient. Using `i64` intermediate prevents overflow when `time_delta` is large. Rust's `div_euclid`/`rem_euclid` correctly handles negative remainders (always produces non-negative remainder), replacing C#'s manual "if negative, borrow a day" logic.

**Alternatives considered**:
- Implement a minimal internal DateTime-like type for round-tripping: Over-engineered for two integer operations. Rejected.
- Stay in `i32` for intermediate computation: Risk of overflow when `time_delta` is large (e.g., expressing hours in ticks). Using `i64` is safer. Rejected.
