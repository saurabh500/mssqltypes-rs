# Quickstart: Verifying crates.io Publish Readiness

**Feature**: 015-crates-io-publish | **Date**: 2026-03-02

---

## Prerequisites

- Rust toolchain 1.85+ (`rustup update stable`)
- Git repository with clean working tree

## Verification Steps

### 1. Check for publish warnings

```bash
cargo publish --dry-run --allow-dirty 2>&1
```

**Expected**: No warnings about missing metadata. Output should show:
```
Packaging mssqltypes v0.1.0
Packaged N files, ...
Verifying mssqltypes v0.1.0
Finished ...
Uploading mssqltypes v0.1.0
warning: aborting upload due to dry run
```

No line containing `warning: manifest has no ...`. 

### 2. Check package contents

```bash
cargo package --list --allow-dirty 2>/dev/null
```

**Expected**: Only these files (approximately):
```
.cargo_vcs_info.json
.gitignore
Cargo.lock
Cargo.toml
Cargo.toml.orig
LICENSE
README.md
src/error.rs
src/lib.rs
src/sql_binary.rs
src/sql_boolean.rs
src/sql_byte.rs
src/sql_compare_options.rs
src/sql_datetime.rs
src/sql_decimal.rs
src/sql_double.rs
src/sql_guid.rs
src/sql_int16.rs
src/sql_int32.rs
src/sql_int64.rs
src/sql_money.rs
src/sql_single.rs
src/sql_string.rs
```

**Not expected**: Any files from `.github/`, `.specify/`, or `specs/`.

### 3. Check package size

```bash
cargo package --allow-dirty 2>&1 | grep 'Packaged'
```

**Expected**: Compressed size under 100 KB.

### 4. Check doc-tests

```bash
cargo test --doc
```

**Expected**: At least 1 doc-test passes.

### 5. Check generated docs

```bash
cargo doc --open
```

**Expected**: The crate root page shows a description, type table, and quick-start example.

### 6. Check all tests still pass

```bash
cargo test
```

**Expected**: All 1,469+ tests pass, 0 failures.

### 7. Check Cargo.toml metadata

```bash
grep -E '^(description|license|repository|readme|keywords|categories|rust-version|exclude)' Cargo.toml
```

**Expected**: All fields present with non-empty values.

## Full Validation Script

```bash
#!/bin/bash
set -e

echo "=== 1. Publish dry-run ==="
output=$(cargo publish --dry-run --allow-dirty 2>&1)
if echo "$output" | grep -q 'warning: manifest has no'; then
  echo "FAIL: Missing metadata warnings"
  exit 1
fi
echo "PASS: No metadata warnings"

echo "=== 2. Package contents ==="
if cargo package --list --allow-dirty 2>/dev/null | grep -qE '^\.(github|specify)/|^specs/'; then
  echo "FAIL: Non-source files in package"
  exit 1
fi
echo "PASS: Package is clean"

echo "=== 3. Tests ==="
cargo test --quiet
echo "PASS: All tests pass"

echo "=== 4. Doc-tests ==="
doc_output=$(cargo test --doc 2>&1)
if echo "$doc_output" | grep -q '0 passed'; then
  echo "FAIL: No doc-tests found"
  exit 1
fi
echo "PASS: Doc-tests pass"

echo "=== All checks passed ==="
```
