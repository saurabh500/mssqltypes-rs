# mssqltypes Constitution

## Core Principles

### I. Behavioral Fidelity to SQL Server

Every Rust type in this library MUST faithfully replicate the behavior of its C# `System.Data.SqlTypes` counterpart. This includes: NULL semantics (three-valued logic), overflow detection, precision/scale rules, comparison ordering, and type conversion rules. Deviations from C# behavior are bugs unless explicitly documented and justified.

### II. Idiomatic Rust Design

While maintaining behavioral fidelity, types MUST be expressed as idiomatic Rust. This means:
- Use Rust `Option<T>` semantics to represent SQL NULL where appropriate internally, but expose a consistent `is_null()` API
- Implement standard Rust traits (`Display`, `FromStr`, `PartialEq`, `PartialOrd`, `Clone`, `Copy` where applicable, `Debug`, `Hash`)
- Use Rust operator overloading (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `BitAnd`, `BitOr`, `BitXor`, `Not`, `Neg`) to mirror C# operators
- Return `Result` types for fallible operations instead of throwing exceptions
- Use `#[derive]` macros where possible to reduce boilerplate

### III. Test-First Development (NON-NEGOTIABLE)

TDD is mandatory for all type implementations:
- Tests written BEFORE implementation code
- Red-Green-Refactor cycle strictly enforced
- Every public API method MUST have corresponding test coverage
- Edge cases (NULL propagation, overflow, boundary values, type conversions) MUST be tested
- Target: ≥90% code coverage across the library

### IV. Comprehensive Type Coverage

The library MUST provide Rust equivalents for the following C# `System.Data.SqlTypes`:
- **Numeric**: `SqlByte`, `SqlInt16`, `SqlInt32`, `SqlInt64`, `SqlSingle`, `SqlDouble`, `SqlDecimal`, `SqlMoney`
- **Boolean**: `SqlBoolean`
- **String/Binary**: `SqlString`, `SqlBinary`
- **Date/Time**: `SqlDateTime`
- **GUID**: `SqlGuid`

Each type MUST support: construction, NULL handling, arithmetic/logical operations (where applicable), comparison, type conversions, string parsing, and `Display` formatting.

### V. Zero Unsafe Code

The library MUST NOT use `unsafe` Rust code. All implementations must rely on safe Rust abstractions. This ensures memory safety guarantees without runtime overhead.

### VI. No External Runtime Dependencies

The core library MUST have zero required runtime dependencies beyond the Rust standard library. Optional features (e.g., `serde` serialization) may be gated behind feature flags but are not required for core functionality.

### VII. Versioning & Breaking Changes

The library follows Semantic Versioning (SemVer): `MAJOR.MINOR.PATCH`.
- MAJOR: Breaking API changes
- MINOR: New types or methods, backward-compatible
- PATCH: Bug fixes, behavioral corrections
- Pre-1.0: API is unstable, breaking changes allowed in MINOR versions

## Technical Constraints

### Language & Toolchain
- **Language**: Rust (latest stable edition)
- **Minimum Supported Rust Version (MSRV)**: Documented in `Cargo.toml` and CI
- **Edition**: Rust 2024

### Error Handling
- All fallible operations return `Result<T, SqlTypeError>` with a unified error enum
- Error types MUST be descriptive: overflow, divide-by-zero, null-value-access, parse-failure, out-of-range
- Panics are NOT acceptable in library code

### Performance
- Stack-allocated types (`Copy + Clone`) for all fixed-size types (SqlByte, SqlInt16, SqlInt32, SqlInt64, SqlSingle, SqlDouble, SqlBoolean, SqlDateTime, SqlGuid, SqlMoney)
- Heap-allocated only for variable-size types (SqlBinary, SqlString, SqlDecimal)
- No unnecessary allocations in arithmetic or comparison operations

### Feature Flags
- `serde`: Enable `Serialize`/`Deserialize` for all types
- Default: no features enabled (zero dependencies)

## Development Workflow

### Pull Request Requirements
- All PRs MUST pass CI (build + test + clippy + format check)
- All PRs MUST maintain or improve code coverage (minimum 90%)
- Code coverage report published as PR artifact
- New types or public API changes MUST include spec updates

### Code Quality Gates
- `cargo fmt --check` — formatting enforced
- `cargo clippy -- -D warnings` — no clippy warnings
- `cargo test` — all tests pass
- `cargo tarpaulin` or `llvm-cov` — coverage report generated

### Branch Strategy
- `main`: stable, release-ready code
- Feature branches: `feature/<type-name>` or `feature/<description>`
- All changes via PR with review

## Governance

This constitution supersedes all other development practices for the mssqltypes project. Amendments require:
1. A documented proposal explaining the change
2. Update to this constitution file
3. PR review and approval

All PRs and code reviews MUST verify compliance with these principles. Complexity must be justified against the principle of behavioral fidelity.

**Version**: 1.0.0 | **Ratified**: 2026-03-01 | **Last Amended**: 2026-03-01
