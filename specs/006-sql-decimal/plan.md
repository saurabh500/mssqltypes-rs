# Implementation Plan: SqlDecimal

**Branch**: `006-sql-decimal` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/006-sql-decimal/spec.md`

## Summary

Implement `SqlDecimal` — a Rust equivalent of C# `System.Data.SqlTypes.SqlDecimal`. Fixed-point decimal with up to 38 digits of precision and configurable scale, using a 128-bit unsigned mantissa (four `u32` components) plus sign, precision, and scale metadata. Arithmetic returns `Result<SqlDecimal, SqlTypeError>` with precision/scale propagation per SQL Server rules. Comparisons return `SqlBoolean` with three-valued NULL logic. Multi-precision arithmetic implemented via schoolbook algorithms (add/sub/mul) and Knuth's Algorithm D (division). Follows patterns established by `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, and `SqlBoolean`.

## Technical Context

**Language/Version**: Rust (Edition 2024, latest stable)  
**Primary Dependencies**: None (std only)  
**Storage**: N/A  
**Testing**: `cargo test` (inline `#[cfg(test)]` modules)  
**Target Platform**: All platforms (pure Rust)  
**Project Type**: Library  
**Performance Goals**: Heap-free arithmetic using stack-allocated fixed-size struct (~20 bytes). No allocation in arithmetic/comparison operations.  
**Constraints**: No `unsafe`, no external deps, no panics  
**Scale/Scope**: Single module (~2000–3000 lines including tests — significantly larger than integer types due to multi-precision arithmetic)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Pre-Design | Post-Design | Notes |
|---|-----------|-----------|-------------|-------|
| I | Behavioral Fidelity | PASS | PASS | Verified against C# SQLDecimal.cs (~3400 lines). Matches internal representation (4×u32 mantissa + sign + precision + scale), arithmetic precision/scale propagation rules, AdjustScale rounding (round-half-up), comparison via scale normalization, mathematical functions, and negative zero normalization. Power() goes through f64 in C# — same approach. |
| II | Idiomatic Rust | PASS | PASS | `Option<InnerDecimal>` for NULL, `Result` for fallible ops, standard traits, operator overloading. `Clone` but not `Copy` — matches Constitution performance constraint (heap-allocated category for SqlDecimal). |
| III | Test-First Development | PASS | PASS | Spec defines 68 acceptance scenarios + 12 edge cases across 7 user stories |
| IV | Comprehensive Coverage | PASS | PASS | SqlDecimal is a required numeric type per Constitution IV |
| V | Zero Unsafe | PASS | PASS | All arithmetic via safe Rust integer operations. Multi-precision arithmetic uses `u64` widening for carry propagation. |
| VI | No External Deps | PASS | PASS | std only — no `num-bigint` or similar crates |
| VII | Versioning | PASS | PASS | Additive: new module + re-export, no breaking changes |

**Gate Result**: ALL PASS — no violations.

## Key Design Decisions

1. **128-bit mantissa via 4×u32 array** (not u128) — matches C# internal layout, enables identical precision calculation and multi-precision algorithms. See [research.md](research.md#r1-internal-representation-4u32-vs-u128).
2. **Schoolbook multi-precision arithmetic** for add/sub/mul — see [research.md](research.md#r2-multi-precision-arithmetic-approach).
3. **Knuth's Algorithm D for division** — see [research.md](research.md#r3-division-algorithm).
4. **Precision/scale propagation follows C# formulas exactly** — see [research.md](research.md#r4-precisionscale-propagation-rules).
5. **Round-half-up rounding** (away from zero in magnitude) — see [research.md](research.md#r5-rounding-mode).
6. **Clone but not Copy** — Constitution classifies SqlDecimal as heap-allocated category. See [research.md](research.md#r6-copy-vs-clone-semantics).
7. **Negative zero normalized** — see [research.md](research.md#r7-negative-zero-normalization).
8. **Power via f64** — matches C# approach — see [research.md](research.md#r8-power-implementation).
9. **Conversions scoped to existing types** — see [research.md](research.md#r9-available-conversions).
10. **Minimum division scale = 6** — matches C# `s_cNumeDivScaleMin` — see [research.md](research.md#r4-precisionscale-propagation-rules).

## Project Structure

### Documentation (this feature)

```text
specs/006-sql-decimal/
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
├── lib.rs               # MODIFY: add sql_decimal module + re-export
├── error.rs             # NO CHANGES
├── sql_boolean.rs       # NO CHANGES (dependency for comparisons + conversion)
├── sql_byte.rs          # NO CHANGES (dependency for conversion)
├── sql_int16.rs         # NO CHANGES (dependency for conversion)
├── sql_int32.rs         # NO CHANGES (dependency for conversion)
├── sql_int64.rs         # NO CHANGES (dependency for conversion)
└── sql_decimal.rs       # NEW: SqlDecimal implementation + inline tests
```

**Structure Decision**: Single flat module following the established pattern. SqlDecimal is self-contained in one file with inline tests, consistent with all existing SQL type implementations.

## Complexity Tracking

No violations — nothing to justify.
