# Contract: Cargo.toml Package Metadata

**Feature**: 015-crates-io-publish | **Date**: 2026-03-02

This is not a public API contract — the crate's public API is unchanged. This documents the **packaging contract** between this crate and crates.io/docs.rs.

---

## Target Cargo.toml `[package]` Section

```toml
[package]
name = "mssqltypes"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Faithful Rust equivalents of C# System.Data.SqlTypes with SQL NULL semantics, checked arithmetic, and three-valued logic"
license = "MIT"
repository = "https://github.com/<owner>/mssqltypes"
readme = "README.md"
keywords = ["sql-server", "sql-types", "mssql", "database", "tds"]
categories = ["database", "data-structures"]
exclude = [".github/", ".specify/", "specs/"]
```

> **Note**: `<owner>` must be replaced with the actual GitHub username/org before publish.

---

## Crate-Level Documentation Contract

The `//!` block at the top of `src/lib.rs` must contain:

1. **Crate title** — `# mssqltypes`
2. **One-paragraph description** — What the library does, key capabilities
3. **Type overview table** — All 14 types with SQL Server / Rust / C# mapping
4. **Features list** — NULL semantics, checked arithmetic, zero dependencies, etc.
5. **Quick Start code example** — Must compile as a doc-test using `SqlInt32` and `SqlBoolean`
6. **Feature flags section** — Document `serde` (even if not yet implemented)

---

## Validation Contract

The following commands must all succeed without warnings or failures:

| Command | Expected Result |
|---------|----------------|
| `cargo publish --dry-run --allow-dirty` | Zero metadata warnings |
| `cargo package --list --allow-dirty` | No `.github/`, `.specify/`, `specs/` files |
| `cargo test` | All 1,469+ tests pass |
| `cargo test --doc` | ≥1 doc-test passes |
| `cargo doc` | No warnings; crate root page populated |
| `cargo clippy -- -D warnings` | Zero warnings |
| `cargo fmt --check` | No formatting issues |
