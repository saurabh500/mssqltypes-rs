# Implementation Plan: SqlDateTime

**Branch**: `010-sql-datetime` | **Date**: 2025-07-17 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/010-sql-datetime/spec.md`

## Summary

Implement `SqlDateTime` — a Rust equivalent of C# `System.Data.SqlTypes.SqlDateTime`. A date/time value stored internally as `Option<(i32, i32)>` where `(day_ticks, time_ticks)` counts days since 1900-01-01 and 1/300-second ticks since midnight. Valid date range: 1753-01-01 (day_ticks = -53690) to 9999-12-31 (day_ticks = 2958463). Time range: 0 to 25919999 (TicksPerDay - 1). Construction from calendar components (year, month, day, hour, minute, second, millisecond) with Gregorian calendar math and millisecond rounding via `(int)(ms * 0.3 + 0.5)`. Duration arithmetic via `checked_add(day_delta, time_delta)` / `checked_sub(day_delta, time_delta)` with `i64` intermediate and `div_euclid`/`rem_euclid` normalization. Calendar component extraction via manual 400/100/4/1-year-cycle decomposition. ISO 8601-like Display/FromStr. Comparisons return `SqlBoolean` with lexicographic `(day_ticks, time_ticks)` ordering.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)
**Primary Dependencies**: None (std only)
**Storage**: N/A
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)
**Target Platform**: All platforms (pure Rust)
**Project Type**: Library
**Performance Goals**: Stack-allocated (`Copy + Clone`), zero-allocation arithmetic/comparison
**Constraints**: No `unsafe`, no external deps, no panics
**Scale/Scope**: Single module (~1200 lines including tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Pre-Design | Post-Design | Notes |
|---|-----------|-----------|-------------|-------|
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLDateTime.cs. Calendar math (day_ticks from YMD) uses identical formula. Millisecond rounding matches C# exactly: `(int)(ms * 0.3 + 0.5)`. Comparison is lexicographic `(m_day, m_time)` — identical. Duration arithmetic uses direct tick manipulation (equivalent to C# round-trip through DateTime but without precision loss). Display/FromStr use deterministic ISO 8601 format (C# uses culture-sensitive — justified deviation for Rust library). |
| II | Idiomatic Rust | PASS | PASS | `Option<(i32, i32)>` repr; `Result` for fallible ops; `checked_add`/`checked_sub` naming; `div_euclid`/`rem_euclid` for time normalization; standard traits |
| III | Test-First Development | PASS | PASS | Spec defines 62 acceptance scenarios + edge cases across 8 user stories |
| IV | Comprehensive Coverage | PASS | PASS | SqlDateTime is a required date/time type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All calendar math and tick manipulation via safe integer arithmetic |
| VI | No External Deps | PASS | PASS | std only — no chrono, no time crate |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **Duration as `(i32, i32)` parameters** (not a custom Duration type) — see [research.md](research.md#r1-duration-representation)
2. **Direct calendar formula port** from C# for day_ticks from YMD — see [research.md](research.md#r2-calendar-computation-day_ticks-from-ymd)
3. **Manual 400/100/4/1-year-cycle algorithm** for YMD from day_ticks (no chrono dependency) — see [research.md](research.md#r3-reverse-computation-ymd-from-day_ticks)
4. **Integer division for time extraction** (TICKS_PER_HOUR, etc.) — see [research.md](research.md#r4-time-extraction-from-time_ticks)
5. **Lexicographic `(day_ticks, time_ticks)` comparison** — trivial port from C# — see [research.md](research.md#r5-comparison-operators)
6. **ISO 8601 deterministic format** for Display/FromStr (justified deviation from C# culture-sensitive) — see [research.md](research.md#r6-parse--tostring-implementation)
7. **No public DateTime interop** in core (defer chrono behind feature flag) — see [research.md](research.md#r7-datetime-interop)
8. **Direct tick manipulation for arithmetic** with `i64` intermediate and `div_euclid`/`rem_euclid` — see [research.md](research.md#r8-arithmetic-implementation)

## Project Structure

### Documentation (this feature)

```text
specs/010-sql-datetime/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── public-api.md    # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # MODIFY: add sql_datetime module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons)
├── sql_byte.rs          # NO CHANGES
├── sql_int16.rs         # NO CHANGES
├── sql_int32.rs         # NO CHANGES
├── sql_int64.rs         # NO CHANGES
└── sql_datetime.rs      # NEW: SqlDateTime implementation + inline tests
```

## Complexity Tracking

No violations — nothing to justify.
