# Research: crates.io Publish Readiness

**Feature**: 015-crates-io-publish | **Date**: 2026-03-02

---

## 1. Cargo.toml Metadata Fields

### Decision
Set `description`, `license`, `repository`, `readme`, `keywords`, `categories`, `exclude`, and `rust-version`. **Omit** `homepage` and `documentation`.

### Rationale
- `description` + `license` are effectively required — `cargo publish` warns without them.
- `repository` is displayed prominently on the crates.io page; essential for discoverability.
- `homepage`: Cargo docs say _"Do not make homepage redundant with either the documentation or repository values."_ No dedicated website exists, so omit.
- `documentation`: crates.io auto-links to docs.rs when unset. Omitting avoids stale URLs.
- `readme = "README.md"`: Explicit is better. Rendered as Markdown on the crates.io page.

### Alternatives Considered
- Setting `homepage` = repo URL: Rejected; redundant with `repository`.
- Setting `documentation` explicitly: Rejected; docs.rs auto-link is superior.

---

## 2. Valid crates.io Category Slugs

### Decision
Use `categories = ["database", "data-structures"]`.

### Rationale
Both are confirmed valid slugs:
- **`database`** — _"Crates to interface with database management systems."_ — Fits; the library provides SQL Server data types.
- **`data-structures`** — _"Rust implementations of particular ways of organizing data."_ — Fits; 14 specialized data types.

### Alternatives Considered
| Slug | Verdict | Reason |
|------|---------|--------|
| `database-implementations` | No | For databases implemented in Rust, not type libraries |
| `encoding` | No | Parsing is incidental (`FromStr`), not the primary purpose |
| `finance` | No | `SqlMoney` is one of 14 types; too narrow |

---

## 3. Package Exclusions & Size

### Decision
Add `exclude = [".github/", ".specify/", "specs/"]` to Cargo.toml. Target: <100 KB compressed.

### Current State vs Target

| Metric | Current | Target |
|--------|---------|--------|
| Files | 170 | ~20 |
| Compressed | 322 KB | <100 KB |
| Spec files | 93 | 0 |
| .github/.specify | 52 | 0 |
| src/ files | 16 | 16 |

### Rationale
- `target/` is already auto-excluded by Cargo.
- `exclude` is safer than `include` — doesn't risk omitting new source files added later.
- Must-include files (`Cargo.toml`, `LICENSE`, `README.md`) are auto-included by Cargo.

### Alternatives Considered
- Using `include` (allowlist): More secure but brittle — adding a new source directory later requires updating the list. Rejected for simplicity.

---

## 4. Edition 2024 vs 2021

### Decision
**Keep edition 2024.** No code changes needed.

### Rationale
- This is an initial 0.1.0 publish with zero downstream dependents.
- Rust 1.85 has been stable for over a year (released Feb 2025). Adoption is high as of March 2026.
- The crate uses **zero** edition-2024-specific language features — a downgrade to 2021 would work without code changes. But upgrading later is also trivial.
- Edition 2024 provides better defaults (stricter lifetime capture, safer environment variable functions).
- The marginal user base on Rust 1.56–1.84 is very small in 2026.

### Key Edition 2024 Changes (none impacting this crate)

| Change | Relevant? |
|--------|-----------|
| RPIT lifetime capture rules | Low — no `impl Trait` returns |
| `unsafe extern` blocks required | N/A — no FFI |
| `unsafe_op_in_unsafe_fn` lint on by default | N/A — no `unsafe` code |
| `gen` keyword reserved | N/A — not used |

### Alternatives Considered
- Downgrade to edition 2021: Would expand to Rust 1.56+ users. Rejected because the marginal user base is tiny and edition 2024 defaults are preferred.

---

## 5. MSRV (`rust-version`)

### Decision
```toml
rust-version = "1.85"
```

### Rationale
- Edition 2024 was introduced in Rust 1.85.0.
- Uses two-component version format (major.minor), not three.
- Provides a clear error at dependency resolution time for users on older toolchains.
- No evidence the crate uses any 1.86+ features, so 1.85 is the true minimum.

### Alternatives Considered
- `rust-version = "1.93"` (current toolchain): Overly restrictive; no 1.86+ features used.
- Omitting entirely: Users would get a confusing "unknown edition 2024" error.

---

## 6. Crate Name Availability

### Decision
The name **`mssqltypes`** is available on crates.io.

### Rationale
- Concise, descriptive, follows naming conventions (all lowercase).
- No trademark concerns — "mssql" is a widely-used abbreviation.

### Alternatives Considered
| Name | Verdict |
|------|---------|
| `mssql-types` | More readable but different from module name |
| `sql-server-types` | Too long (16 chars) |
| `sqltypes` | Too generic; may conflict |
| `tds-types` | Obscure; TDS less recognized than MSSQL |

---

## 7. `warn` vs `deny` for `missing_docs`

### Decision
Use `#![warn(missing_docs)]`.

### Rationale
- `warn` surfaces gaps without breaking the build during initial development.
- ~307 public items need documentation; `deny` would block all other work.
- The project already uses `cargo clippy -- -D warnings` in CI, which effectively promotes warnings to errors in CI while keeping local dev ergonomic.
- Many well-regarded crates (`serde`, `tokio`) use `warn` rather than `deny`.

### Alternatives Considered
- `#![deny(missing_docs)]`: Too strict for a 0.1.0 with incomplete docs. Can upgrade once all items documented.
- `#![forbid(missing_docs)]`: Cannot be overridden with `#[allow]` — too inflexible.
- `[lints.rust]` table in Cargo.toml: Valid alternative but less conventional and less visible to developers.

---

## 8. Suggested `keywords`

### Decision
```toml
keywords = ["sql-server", "sql-types", "mssql", "database", "tds"]
```

### Rationale
- Max 5 keywords (crates.io hard limit), each ≤20 ASCII characters.
- Covers the main search terms users would look for.
- `tds` included because the library's types map directly to TDS protocol wire types.

---

## Summary of Resolved Items

All technical context items are resolved. No NEEDS CLARIFICATION items remain.

| Item | Resolution |
|------|-----------|
| Required metadata fields | `description`, `license`, `repository`, `readme`, `keywords`, `categories` |
| Omitted metadata fields | `homepage`, `documentation` (auto-linked by crates.io/docs.rs) |
| Package exclusions | `.github/`, `.specify/`, `specs/` |
| Edition decision | Keep 2024 |
| MSRV | `rust-version = "1.85"` |
| Crate name | `mssqltypes` — available |
| Doc lint | `#![warn(missing_docs)]` |
