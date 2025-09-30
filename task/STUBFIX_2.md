# STUBFIX_2: Fix Cargo Dependency Parser Test Workaround

## OBJECTIVE

Fix the `CargoParser` implementation to correctly parse all dependencies (including `tokio` with features), allowing the test workaround to be removed and full test coverage restored.

## CONTEXT

**File:** `/Volumes/samsung_t9/kargo/plugins/native/kargo-upgrade/test/test_dependency_parser.rs:61`

**Current State:** The test `test_cargo_parser()` contains a workaround that only validates `anyhow` dependency because the parser fails to correctly parse `tokio` with its feature specification.

**Stub Comment:**
```rust
// We don't necessarily get tokio in the result set anymore,
// so we'll just check anyhow for now until we can fix the parsing
```

**Test Input:**
```toml
[dependencies]
anyhow = "1.0.0"
tokio = { version = "1.0.0", features = ["full"] }
```

## SUBTASKS

### SUBTASK1: Diagnose CargoParser parsing failure

- Review `CargoParser` implementation in `plugins/native/kargo-upgrade/src/parsers/cargo_parser.rs`
- Identify why dependencies with inline table syntax (features, optional flags) are being skipped or parsed incorrectly
- Determine if the issue is in TOML parsing logic or dependency extraction logic
- Document root cause in implementation comments

### SUBTASK2: Fix CargoParser to handle all dependency formats

- Modify `CargoParser::parse()` to correctly handle:
  - Simple string version: `anyhow = "1.0.0"`
  - Inline table with version: `tokio = { version = "1.0.0", features = ["full"] }`
  - Other common formats (path, git, workspace dependencies if relevant)
- Ensure version extraction works for all formats
- Maintain existing `DependencyLocation::CargoTomlDirect` classification
- Preserve all existing functionality for simple dependencies

### SUBTASK3: Remove test workaround and restore full validation

- Update `test_cargo_parser()` in `test_dependency_parser.rs` (lines 60-62)
- Remove the workaround comment
- Restore validation for `tokio` dependency:
  - Assert it exists in parsed results
  - Verify version is "1.0.0"
  - Confirm location is `DependencyLocation::CargoTomlDirect`
- Update assertion: `assert_eq!(dependencies.len(), 2);` should now pass with both deps

## DEFINITION OF DONE

- [ ] No stub comments or workarounds remain in test file
- [ ] `CargoParser` correctly parses dependencies with inline table syntax
- [ ] Test validates both `anyhow` and `tokio` dependencies
- [ ] Test assertion for length (2 dependencies) passes
- [ ] All existing CargoParser functionality remains intact
- [ ] Code compiles without warnings

## CONSTRAINTS

- **NO TESTS:** Do not create new test files (only modify existing test in `test_dependency_parser.rs`)
- **NO BENCHMARKS:** Do not write benchmark code
- **FOCUS:** Only modify:
  - `/Volumes/samsung_t9/kargo/plugins/native/kargo-upgrade/src/parsers/cargo_parser.rs`
  - `/Volumes/samsung_t9/kargo/plugins/native/kargo-upgrade/test/test_dependency_parser.rs`

## RESEARCH NOTES

- Parser likely uses `toml_edit` or `toml` crate for parsing
- TOML inline tables: `key = { field1 = "value1", field2 = "value2" }`
- Need to distinguish between string values and table values
- Check if parser is filtering out dependencies with features
- Reference: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html

## DEPENDENCIES

- Existing `toml_edit` dependency (check workspace `Cargo.toml`)
- Models defined in `krater::up2date::models`
- `DependencyLocation`, `DependencyParser`, `DependencySource` types
