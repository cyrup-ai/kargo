# TODO: Fix Vendor and Forks Exclusion Architecture

## Architecture Issue

The current implementation has duplicate exclusion logic:
- `config.rs` defines `exclude_patterns` with glob patterns (e.g., `**/vendor/**`) but they are NEVER used
- `project_scanner.rs` has hardcoded `PRUNE_DIRS` arrays (duplicated twice in the file) that do simple string matching
- This creates architectural inconsistency and prevents proper exclusion of vendor/forks directories

## Solution Architecture

Refactor the scanner to use `config.exclude_patterns` as the single source of truth:
- Pass `Config` through the entire scanning chain
- Use `globset` crate for efficient pattern matching
- Remove hardcoded `PRUNE_DIRS` arrays
- Ensure paths are matched against full absolute paths to catch `/vendor/` anywhere in the tree

---

## Implementation Tasks

### Task 1: Add globset dependency to kargo-turd
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/Cargo.toml`  
**Line:** ~34 (in dependencies section)  
**Action:** Add `globset = { workspace = true }` to dependencies

**Implementation Notes:**
- globset is already in workspace dependencies at version 0.4.16
- Use workspace = true for version consistency
- Add after the "Pattern matching" section with regex and lazy_static

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 2: QA - Verify globset dependency
Act as an Objective QA Rust developer. Verify:
- globset was added to Cargo.toml dependencies section
- Uses `{ workspace = true }` syntax
- No version number specified (uses workspace version)
- No other dependencies were added or modified
- cargo check passes with the new dependency

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 3: Update find_projects_with_progress function signature
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Line:** 10  
**Action:** Change signature from `pub fn find_projects_with_progress(root_path: &Path)` to `pub fn find_projects_with_progress(root_path: &Path, config: &Config)`

**Implementation Notes:**
- Add `use crate::Config;` at top of file
- Pass config through to find_cargo_toml_files()
- Keep all existing logic, only add parameter

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 4: QA - Verify function signature update
Act as an Objective QA Rust developer. Verify:
- Function signature has config: &Config parameter
- Import statement for Config exists
- Function body was not modified beyond adding parameter
- Documentation comment updated if it exists
- No unwrap() or expect() added

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 5: Refactor find_cargo_toml_files to use config.exclude_patterns
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Lines:** 39-71  
**Action:** Replace PRUNE_DIRS hardcoded logic with globset pattern matching

**Implementation Notes:**
- Add `config: &Config` parameter to function signature (line 39)
- Import globset: `use globset::{Glob, GlobSetBuilder};`
- Build GlobSet from config.exclude_patterns at start of function
- Handle GlobSetBuilder::build() errors with proper Result propagation (no unwrap/expect)
- Replace the process_read_dir closure logic (lines 52-60) to use globset.is_match() on full path
- Remove the PRUNE_DIRS const array entirely (lines 43-46)
- Keep jwalk parallelism and other settings unchanged

**Specific Logic:**
```rust
// Build glob matcher from config
let mut builder = GlobSetBuilder::new();
for pattern in &config.exclude_patterns {
    let glob = Glob::new(pattern).map_err(|e| anyhow::anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
    builder.add(glob);
}
let globset = builder.build().map_err(|e| anyhow::anyhow!("Failed to build globset: {}", e))?;

// In process_read_dir closure, replace PRUNE_DIRS check with:
if let Ok(entry) = res && entry.file_type().is_dir() {
    let path = entry.path();
    return !globset.is_match(&path);
}
```

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 6: QA - Verify find_cargo_toml_files refactoring
Act as an Objective QA Rust developer. Verify:
- Function has config: &Config parameter
- globset imports added (Glob, GlobSetBuilder)
- GlobSet built from config.exclude_patterns with proper error handling
- No unwrap() or expect() used (must use ?)
- PRUNE_DIRS const removed completely
- process_read_dir uses globset.is_match() on full path
- All error paths return Result with descriptive messages
- No other logic was modified
- jwalk configuration unchanged

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 7: Refactor collect_rs_files to use config.exclude_patterns
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Lines:** 74-126  
**Action:** Add config parameter and replace second PRUNE_DIRS with globset

**Implementation Notes:**
- Add `config: &Config` parameter to function signature (line 74)
- Build GlobSet from config.exclude_patterns (same as Task 5)
- Replace process_read_dir closure logic (lines 106-114) with globset.is_match()
- Remove the second PRUNE_DIRS const array (lines 98-101)
- Pass config to collect_rs_files calls from collect_rust_files (lines 81, 87)

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 8: QA - Verify collect_rs_files refactoring
Act as an Objective QA Rust developer. Verify:
- Function has config: &Config parameter
- GlobSet built from config.exclude_patterns with proper error handling
- No unwrap() or expect() used
- Second PRUNE_DIRS const removed completely
- process_read_dir uses globset.is_match()
- All collect_rs_files calls pass config parameter
- No other logic modified

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 9: Update collect_rust_files to pass config through
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Lines:** 74-91  
**Action:** Add config parameter and pass to collect_rs_files calls

**Implementation Notes:**
- Add `config: &Config` to function signature (line 74)
- Line 81: Change `collect_rs_files(&src_dir)?` to `collect_rs_files(&src_dir, config)?`
- Line 87: Change `collect_rs_files(&tests_dir)?` to `collect_rs_files(&tests_dir, config)?`
- Keep all other logic unchanged

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 10: QA - Verify collect_rust_files update
Act as an Objective QA Rust developer. Verify:
- Function has config: &Config parameter
- Both collect_rs_files calls pass config
- No other changes made to function
- Function still returns Result<()>

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 11: Update find_projects_with_progress to pass config through chain
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Line:** 21  
**Action:** Pass config to find_cargo_toml_files call

**Implementation Notes:**
- Line 21: Change `let cargo_toml_paths = find_cargo_toml_files(root_path)?;` to `let cargo_toml_paths = find_cargo_toml_files(root_path, config)?;`
- Line 29: Change `collect_rust_files(&cargo_path, &mut project)?;` to `collect_rust_files(&cargo_path, &mut project, config)?;`

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 12: QA - Verify function call chain
Act as an Objective QA Rust developer. Verify:
- find_cargo_toml_files call passes config
- collect_rust_files call passes config
- No other modifications made
- Error handling unchanged

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 13: Update lib.rs to pass config to scanner
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/lib.rs`  
**Line:** 52  
**Action:** Pass config to find_projects_with_progress

**Implementation Notes:**
- Line 52: Change `let projects = find_projects_with_progress(&current_dir)?;` to `let projects = find_projects_with_progress(&current_dir, config)?;`
- This is the only call site in the codebase

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 14: QA - Verify lib.rs update
Act as an Objective QA Rust developer. Verify:
- find_projects_with_progress call passes config reference
- config is &Config type (not owned)
- No other changes in lib.rs
- Search codebase for any other calls to find_projects_with_progress

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 15: Remove duplicate entries from project_scanner.rs
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/src/project_scanner.rs`  
**Action:** Remove "forks" and "vendor" from any remaining PRUNE_DIRS if they still exist after refactoring

**Implementation Notes:**
- After Tasks 5 and 7, PRUNE_DIRS should be completely removed
- This is a verification step to ensure no remnants remain
- If found, remove them - config.rs is the single source of truth

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 16: QA - Verify no duplication remains
Act as an Objective QA Rust developer. Verify:
- No PRUNE_DIRS constants exist in project_scanner.rs
- No hardcoded "vendor" or "forks" strings in exclusion logic
- config.exclude_patterns is the only exclusion definition
- grep for "vendor" and "forks" in project_scanner.rs shows no matches

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 17: Build and verify compilation
**Action:** Run cargo check and cargo build in kargo-turd directory

**Implementation Notes:**
- Change to `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd`
- Run `cargo check` to verify no compilation errors
- Run `cargo build --release` for full build
- Verify no warnings introduced by changes
- No unwrap() or expect() should exist in src files

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

### Task 18: QA - Verify build success
Act as an Objective QA Rust developer. Verify:
- cargo check passes with 0 errors
- cargo build --release succeeds
- No new warnings introduced
- Grep src/ for unwrap() returns no matches
- Grep src/ for expect() returns no matches

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 19: Create integration test for vendor exclusion
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/tests/exclusion_tests.rs` (new file)  
**Action:** Create test that verifies vendor directories are excluded

**Implementation Notes:**
- Use tempfile crate to create temporary test directory
- Create structure: `temp/vendor/test-crate/Cargo.toml` and `temp/normal-crate/Cargo.toml`
- Call find_projects_with_progress with test config
- Assert that normal-crate is found but vendor/test-crate is not
- Test can use expect() since it's in tests/
- Clean test structure and focused assertions

**Specific Test Structure:**
```rust
#[test]
fn test_vendor_directory_excluded() {
    // Setup temp dir with vendor and non-vendor projects
    // Create config with default exclude patterns
    // Call find_projects_with_progress
    // Assert vendor project NOT in results
    // Assert normal project IS in results
}
```

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 20: QA - Verify vendor exclusion test
Act as an Objective QA Rust developer. Verify:
- Test file created at correct path
- Uses tempfile for directory creation
- Creates vendor and non-vendor test cases
- Calls find_projects_with_progress with proper config
- Assertions check exclusion works correctly
- Test can use expect() (tests/ directory)
- Test cleans up (tempfile handles this)
- Run `cargo test test_vendor_directory_excluded` and verify it passes

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 21: Create integration test for forks exclusion
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/tests/exclusion_tests.rs`  
**Action:** Add test that verifies forks directories are excluded

**Implementation Notes:**
- Same pattern as Task 19
- Create structure: `temp/forks/test-crate/Cargo.toml` and `temp/normal-crate/Cargo.toml`
- Verify forks/test-crate excluded, normal-crate included

**Specific Test Structure:**
```rust
#[test]
fn test_forks_directory_excluded() {
    // Setup temp dir with forks and non-forks projects
    // Create config with default exclude patterns
    // Call find_projects_with_progress
    // Assert forks project NOT in results
    // Assert normal project IS in results
}
```

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 22: QA - Verify forks exclusion test
Act as an Objective QA Rust developer. Verify:
- Test added to exclusion_tests.rs
- Creates forks directory test case
- Proper assertions for exclusion
- Run `cargo test test_forks_directory_excluded` and verify it passes
- Both exclusion tests can run together successfully

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 23: Create integration test for nested paths
**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-turd/tests/exclusion_tests.rs`  
**Action:** Add test for vendor/forks at various depths

**Implementation Notes:**
- Test paths like `temp/deep/nested/vendor/crate/Cargo.toml`
- Test paths like `temp/project/forks/crate/Cargo.toml`
- Verify exclusion works at any depth, not just top level
- This validates the `**/vendor/**` glob pattern works correctly

**Specific Test Structure:**
```rust
#[test]
fn test_exclusion_at_various_depths() {
    // Create temp/a/b/c/vendor/crate/Cargo.toml
    // Create temp/x/forks/y/crate/Cargo.toml
    // Create temp/normal/crate/Cargo.toml
    // Assert excluded paths not found
    // Assert normal path found
}
```

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Task 24: QA - Verify nested path exclusion test
Act as an Objective QA Rust developer. Verify:
- Test added to exclusion_tests.rs
- Tests multiple nesting levels
- Verifies glob pattern ** works correctly
- Run `cargo test test_exclusion_at_various_depths` passes
- All three tests pass: `cargo test exclusion_tests`

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 25: Run full test suite
**Action:** Execute all tests to verify no regressions

**Implementation Notes:**
- Run `cargo test` from kargo-turd directory
- Verify all existing tests still pass
- Verify new exclusion tests pass
- Check for any test failures or warnings

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

### Task 26: QA - Verify full test suite
Act as an Objective QA Rust developer. Verify:
- All tests pass: `cargo test`
- No test failures introduced
- New exclusion tests included in run
- Test coverage for vendor and forks exclusion is adequate
- No flaky tests observed

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

### Task 27: Manual verification with real directories
**Action:** Create manual test case with actual vendor and forks directories

**Implementation Notes:**
- In a test project, create `vendor/` and `forks/` directories with Cargo.toml files
- Run kargo-turd on the project
- Verify excluded directories don't appear in analysis output
- Verify non-excluded directories do appear
- This is real-world validation beyond unit tests

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required.

### Task 28: QA - Verify manual testing results
Act as an Objective QA Rust developer. Verify:
- Created test project with vendor and forks subdirectories
- Each contains valid Cargo.toml
- Ran kargo-turd plugin on test project
- Output shows projects were excluded (not in discovered list)
- Other non-excluded projects were found
- Document test results showing exclusion works in practice

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA.

---

## Final Verification Checklist

- [ ] No PRUNE_DIRS constants remain in project_scanner.rs
- [ ] config.exclude_patterns is the single source of truth for exclusions
- [ ] No "vendor" or "forks" hardcoded strings in scanner logic
- [ ] All functions properly pass config through the chain
- [ ] globset dependency added to Cargo.toml
- [ ] No unwrap() in src/ files
- [ ] No expect() in src/ files
- [ ] Integration tests verify vendor exclusion
- [ ] Integration tests verify forks exclusion
- [ ] Integration tests verify nested path exclusion
- [ ] Manual testing confirms real-world usage works
- [ ] cargo check passes
- [ ] cargo build --release succeeds
- [ ] cargo test passes all tests
- [ ] No compilation warnings introduced

## Success Criteria

The refactoring is complete and successful when:
1. Any directory named "vendor" at any depth is excluded from project scanning
2. Any directory named "forks" at any depth is excluded from project scanning
3. config.exclude_patterns is the single source of truth (no duplication)
4. All tests pass including new exclusion tests
5. Manual verification shows exclusions work in practice
6. No unwrap() or expect() exist in src/ files
7. Code compiles without errors or new warnings
