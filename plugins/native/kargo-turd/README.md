# kargo-turd

This is a `kargo` plugin to find stubbed, incomplete, and non-production quality code in Rust projects.

using `jwalk` with `rayon` for parallel traversal of the directory tree.

`kargo turd` will run the analysis and output task files into ./task/_<project_name>/<tier/<file_name_no_extension>_<file_hash>.md

ONLY and EXACTLY one task file should be created for each file in the project. The task file will contain ALL VIOLATIONS found in the file.

TASK FILES generated will be post processed by the `claude` cli tool for LLM analysis and remediation. The task files are effectively prompts for an LLM to analyze the infractions and take appropriate remdiation actions if confirmed to be problematic.

## Step 1: Identify Rust Projects

- from the directory where the binary is executed, find all directories containing a `Cargo.toml` file
  - ./src (if `./src` exists) 
  - ./packages/**/Cargo.toml (if `./packages` exists) 

## Step 2: Analyze the Textual Content of all Rust Files 

- IF the project with Cargo.toml has a `./src` directory, then analyze all files in `./src/**/*.rs` loading them into a prioritized queue for processing
- ALSO analyze files in `./tests/**/*.rs` (if `./tests` exists) with different rules for panic patterns
- This should also be parallelized but with a max number of workers equal to the number of CPU cores
- Prioritity in the queue should be given to the largest files which are likely to take the longest to analyze and most prone to "stubbery" as complexity 

### Analyzing Code Comments

Matches are categorized into tiers with tier 1 being (almost certainly) stubbed code and tier 3 being (least likely) stubbed code and often times not stubbed at all but fully appropriate.

#### Tier 1 Matches

- "IN A REAL" 
- "IN PRODUCTION"
- "IN A PRODUCTION"
- "FOR NOW"
- "TODO"
- "FIXME"
- "WIP"
- "WORK IN PROGRESS"
- "HACK"
- "WOULD REQUIRE"
- "WOULD NEED"
- "FIX"
- "IN PRACTICE"
- "HOPEFUL"
- "WOULD NEED"
- "WOULD REQUIRE"

#### Tier 2 Matches

"DUMMY"
"MOCK"
"PLACEHOLDER"

#### Tier 3 Matches

"block_on"
"spawn_blocking"
"actual"
"legacy"
"backward compatibility"
"shim"
"fallback"
"fall back"

### Analyzing Method Naming

Detect methods with names that suggest temporary or stubbed implementations:

#### Tier 1 Method Name Patterns
- `*_stub` / `stub_*` - e.g., `stub_authenticate()`, `get_user_stub()`
- `*_temp` / `temp_*` - e.g., `temp_handler()`, `create_temp()`
- `*_mock` / `mock_*` - e.g., `mock_api_call()`, `get_mock_data()`
- `*_dummy` / `dummy_*` - e.g., `dummy_response()`, `create_dummy()`
- `*_placeholder` / `placeholder_*` - e.g., `placeholder_validation()`
- `*_tmp` / `tmp_*` - e.g., `tmp_process()`, `handle_tmp()`
- `*_test` / `test_*` (outside of test modules) - production code with test-like names
- `*_hack` / `hack_*` - e.g., `hack_around_issue()`

#### Tier 2 Method Name Patterns  
- `*_quick` / `quick_*` - e.g., `quick_fix()`, `quick_and_dirty()`
- `*_workaround` / `workaround_*` - e.g., `workaround_for_bug()`
- `*_fake` / `fake_*` - e.g., `fake_authentication()`
- `*_unimplemented` - e.g., `get_user_unimplemented()`

### Analyzing Variable Naming

Detect variables with names suggesting temporary or incomplete implementations:

#### Tier 1 Variable Name Patterns
- `*_stub` / `stub_*` - e.g., `stub_config`, `user_stub`
- `*_temp` / `temp_*` / `tmp_*` / `*_tmp` - e.g., `temp_data`, `tmp_result`
- `*_mock` / `mock_*` - e.g., `mock_user`, `response_mock`
- `*_dummy` / `dummy_*` - e.g., `dummy_value`, `config_dummy`
- `*_placeholder` / `placeholder_*` - e.g., `placeholder_id`
- `*_fake` / `fake_*` - e.g., `fake_token`, `user_fake`
- `*_hardcoded` / `hardcoded_*` - e.g., `hardcoded_secret`

#### Tier 2 Variable Name Patterns
- `*_workaround` / `workaround_*`
- `*_hack` / `hack_*`
- Single letter variables outside of iterators/common patterns (e.g., `x`, `y`, `z` used for business logic)

#### Tier 3 Variable Name Patterns
- `unused_*` prefix (might be legitimately documented as unused)
- Variables with names like `data`, `value`, `result` with no additional context

### Detecting Hardcoded Values

Scan for hardcoded values that should be configurable:

#### Tier 1 Hardcoded Values
- Hardcoded URLs (http://, https://) outside of test code
- Hardcoded IP addresses (regex: `\b(?:\d{1,3}\.){3}\d{1,3}\b`)
- Hardcoded ports in production code (e.g., `:8080`, `:3000`)
- Hardcoded credentials patterns (even if fake): `password`, `secret`, `api_key`, `token` as string literals
- Hardcoded file paths that aren't platform-agnostic

#### Tier 2 Hardcoded Values
- Magic numbers without const declarations (numbers > 1 that aren't 2, 10, 100, 1000)
- Hardcoded database connection strings
- Hardcoded service names

### Detecting Panic-Prone Code

Scan for code that can panic in production or lacks proper error messages:

#### Tier 1 Panic Patterns (in `./src` only)
- `.unwrap()` - Direct unwrap calls that can panic
  - **Exception**: `.unwrap_or*` variants are acceptable (`unwrap_or`, `unwrap_or_else`, `unwrap_or_default`)
  - **Detection**: Match `.unwrap()` but NOT `.unwrap_or` (check for word boundary after `unwrap`)
- `.expect("...")` - Expect calls that can panic in production
  - **Exception**: Allowed in `./tests` directory
  - **Detection**: Match `.expect(` in files under `./src`

#### Tier 2 Panic Patterns (in `./tests` only)
- `.unwrap()` - Tests should use `.expect()` with descriptive messages
  - **Detection**: Match `.unwrap()` in files under `./tests`
  - **Rationale**: Test failures should have clear messages explaining what went wrong

#### Tier 2 Test Organization
- `#[test]`, `#[tokio::test]`, `#[rstest]`, or `#[cfg(test)]` in `./src` files
  - **Requirement**: All tests must be in `./tests` directory (sister to `./src`)
  - **Detection**: Search for test attributes/macros in `./src/**/*.rs`
  - **Exception**: `#[cfg(test)] mod tests` if you absolutely must have integration tests near code (discouraged)
  - **Rationale**: Clean separation of concerns, tests don't bloat production binary

## Finding Orphaned Modules

Detect Rust modules that are declared but never used:

### Detection Logic
1. **Parse `mod` declarations**: Find all `mod module_name;` and `mod module_name { }` declarations
2. **Track module usage**: Search for `use path::to::module` or `module::item` patterns
3. **Check visibility**: 
   - Private modules (`mod`) only need to be used within their parent module
   - Public modules (`pub mod`) should be used somewhere in the crate or re-exported
4. **Flag as orphaned if**:
   - Module is declared but never imported via `use`
   - No items from the module are referenced anywhere
   - Module is not re-exported in a public API surface
   - Exception: `mod tests` (test modules are expected to be unused in production)

### Output Format
```
ORPHANED MODULE: src/utils/unused_helper.rs
  - Declared in: src/utils/mod.rs:15
  - Visibility: private
  - No references found in codebase
```

## Finding Orphaned Methods

Detect methods/functions that are defined but never called:

### Detection Logic
1. **Parse function definitions**: Find all `fn function_name` declarations with their visibility
2. **Track function calls**: Search for invocations via AST parsing or text pattern matching
3. **Consider visibility**:
   - Private functions (`fn`) must be called within their module
   - Public functions (`pub fn`) might be part of public API - only flag if the module itself is private
   - `#[allow(dead_code)]` or `#[cfg(test)]` should be excluded
4. **Special cases to exclude**:
   - `main()` function
   - Test functions (`#[test]`, `#[tokio::test]`, `#[rstest]`)
   - Trait implementations (required by trait, may not be called directly)
   - Functions with `#[no_mangle]` (FFI)
   - Entry points for external systems

### Tier Classification
- **Tier 1**: Private functions with no references and stubby names (e.g., `temp_*`, `stub_*`)
- **Tier 2**: Private functions with no references and normal names
- **Tier 3**: Public functions in private modules with no internal references

## Finding Unused Dependencies

Detect dependencies in `Cargo.toml` that are not imported or used:

### Detection Logic
1. **Parse `Cargo.toml`**: Extract all dependencies from `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`
2. **Search for imports**: Look for `use dependency_name::` or `extern crate dependency_name` in all `.rs` files
3. **Handle naming conventions**:
   - Convert kebab-case to snake_case (e.g., `tokio-util` → `tokio_util`)
   - Check for glob imports that might hide usage
4. **Flag as unused if**:
   - No `use` statements reference the dependency
   - No `extern crate` declarations
   - Not referenced in any macro invocations

### Special Considerations
- `[dev-dependencies]` only need to be used in tests, benchmarks, examples
- `[build-dependencies]` only need to be used in `build.rs`
- Some dependencies are used implicitly (e.g., `tokio` macros might not show explicit imports)

### Output Format
```
UNUSED DEPENDENCY: serde_yaml
  - Declared in: Cargo.toml:[dependencies]
  - No imports found in any .rs files
  - Suggestion: Remove from Cargo.toml or add usage
```

## Files > 300 Lines of Code

Large files should be decomposed into smaller, focused modules:

### Detection and Reporting
- **Threshold**: Files with > 300 lines of code (excluding blank lines and comments)
- **Priority**: This violation should appear FIRST in the task file
- **Action Required**: Decomposition must be completed before addressing other stubs/violations

### Task File Content for Large Files
```markdown
# PRIORITY: File Decomposition Required

**File**: `src/service/user_handler.rs`  
**Lines of Code**: 487 (excluding comments/blanks)  
**Threshold Exceeded**: 300 lines

## Required Action
This file must be decomposed into logical submodules before any stub remediation.

### Suggested Decomposition Strategy
1. Analyze the file structure and identify logical groupings:
   - User authentication logic → `user_handler/auth.rs`
   - User CRUD operations → `user_handler/crud.rs`
   - User validation → `user_handler/validation.rs`
   - User queries → `user_handler/queries.rs`

2. Create a new directory: `src/service/user_handler/`
3. Split the file into focused modules (each < 300 lines)
4. Update `src/service/mod.rs` to re-export the public API
5. Ensure all tests still pass after decomposition

### Separation of Concerns Checklist
- [ ] Each module has a single, clear responsibility
- [ ] No circular dependencies between new modules
- [ ] Public API surface is preserved
- [ ] Internal implementation details are properly encapsulated
- [ ] Related functionality is grouped together

---
*Note: Additional stub violations found in this file will be addressed AFTER decomposition.*
```

## Task File Output Format

### Directory Structure
```
./task/_<project_name>/
  ├── tier1/
  │   ├── <file_name_no_ext>_<hash>.md
  │   └── ...
  ├── tier2/
  │   ├── <file_name_no_ext>_<hash>.md
  │   └── ...
  └── tier3/
      ├── <file_name_no_ext>_<hash>.md
      └── ...
```

### MiniJinja Template System

Task files are generated using **MiniJinja** templates located in `./prompt/`. The system uses a master template that includes child templates for each violation type.

#### Template Files

**Master Template** (`master.j2.md`):
- Main orchestrator that combines all sections
- Conditionally includes child templates based on findings
- Provides file metadata and summary

**Child Templates**:
- `decompose.j2.md` - File decomposition requirements (>300 LOC)
- `tier1.j2.md` - Tier 1 violations (loop over `tier1_violations`)
- `tier2.j2.md` - Tier 2 violations (loop over `tier2_violations`)
- `tier3.j2.md` - Tier 3 violations (loop over `tier3_violations`)
- `panic_patterns.j2.md` - Panic-prone code (`.unwrap()`, `.expect()`)
- `tests_in_src.j2.md` - Tests found in `./src` directory
- `orphaned_modules.j2.md` - Unused module declarations
- `orphaned_methods.j2.md` - Unused functions/methods
- `unused_dependencies.j2.md` - Dependencies not imported

#### Template Processing Flow

```
1. Analyze file and collect all violations
   ↓
2. Group violations by type into context object:
   {
     file_path: "src/main.rs",
     project_name: "my-crate",
     file_hash: "a3f2b1...",
     lines_of_code: 450,
     needs_decomposition: true,
     tier1_violations: [...],
     tier2_violations: [...],
     tier3_violations: [...],
     panic_patterns: [...],
     tests_in_src: [...],
     orphaned_modules: [...],
     orphaned_methods: [...],
     unused_dependencies: [...],
     total_violations: 15,
     tier1_count: 5,
     ...
   }
   ↓
3. Load master.j2.md template
   ↓
4. Master template includes child templates conditionally:
   - {% if needs_decomposition %} → include decompose.j2.md
   - {% if tier1_violations %} → include tier1.j2.md
   - Each child template loops over its violation array
   ↓
5. Render complete task file markdown
   ↓
6. Write to ./task/_<project>/tier<N>/<file>_<hash>.md
```

#### Template Variables - Complete Reference

**Root-Level Variables** (available in `master.j2.md`):
```rust
{
  // File identification
  project_relative_path: String,  // e.g., "src/service/handler.rs"
  absolute_path: String,           // e.g., "/Users/dev/project/src/service/handler.rs"
  project_name: String,            // From Cargo.toml package name
  file_hash: String,               // SHA256 hash for unique file naming
  timestamp: String,               // ISO 8601 timestamp
  lines_of_code: u32,             // Excluding blanks and comments
  version: String,                 // kargo-turd version
  
  // Control flags (booleans)
  needs_decomposition: bool,       // true if lines_of_code > 300
  
  // Violation arrays (empty if none found)
  tier1_violations: Vec<Violation>,
  tier2_violations: Vec<Violation>,
  tier3_violations: Vec<Violation>,
  panic_patterns: Vec<PanicPattern>,
  tests_in_src: Vec<TestInSrc>,
  orphaned_modules: Vec<OrphanedModule>,
  orphaned_methods: Vec<OrphanedMethod>,
  unused_dependencies: Vec<UnusedDependency>,
}
```

**Violation Object** (used in `tier1_violations`, `tier2_violations`, `tier3_violations`):
```rust
{
  line_number: u32,
  search_term: String,    // Pattern matched (e.g., "TODO", "FIXME", "stub_")
  method_name: String,    // Method/function name if applicable, "" otherwise
  context: String,        // Code snippet: 2 lines before + violation line + 2 lines after
}
```

**PanicPattern Object** (used in `panic_patterns`):
```rust
{
  line_number: u32,
  pattern: String,        // e.g., ".unwrap()", ".expect()"
  issue: String,          // Description: "Can panic in production" / "Should use expect in tests"
  context: String,        // Code snippet
}
```

**TestInSrc Object** (used in `tests_in_src`):
```rust
{
  line_number: u32,
  test_attribute: String, // e.g., "#[test]", "#[tokio::test]", "#[cfg(test)]"
  file_path: String,      // Relative path to file
  context: String,        // Code snippet
}
```

**OrphanedModule Object** (used in `orphaned_modules`):
```rust
{
  name: String,           // Module name
  declared_in: String,    // File where module is declared
  line_number: u32,
  visibility: String,     // "pub" or "private"
  context: String,        // Code snippet
}
```

**OrphanedMethod Object** (used in `orphaned_methods`):
```rust
{
  name: String,           // Function name (without parens)
  file_path: String,      // File containing the function
  line_number: u32,
  visibility: String,     // "pub", "pub(crate)", or "private"
  context: String,        // Code snippet
}
```

**UnusedDependency Object** (used in `unused_dependencies`):
```rust
{
  name: String,           // Crate name
  cargo_toml: String,     // Path to Cargo.toml
  section: String,        // "[dependencies]", "[dev-dependencies]", or "[build-dependencies]"
}
```

**Additional Variables in Child Templates**:

In `decompose.j2.md`:
- `file_path` - Same as `project_relative_path` from master
- `lines_of_code` - Same as root level

**Complete variable reference**: See [`./prompt/VARIABLES.md`](./prompt/VARIABLES.md) for detailed documentation, example JSON context, and Rust type definitions.

#### Customizing Prompts

To modify the prompts given to LLMs:

1. **Edit child templates** in `./prompt/` directory
2. **Keep loop syntax intact**: `{% for violation in tier1_violations %} ... {% endfor %}`
3. **Use template variables**: `{{ violation.line_number }}`, `{{ violation.context }}`
4. **Modify prompt language** between the loops
5. Templates marked with `TODO` are placeholders for you to fill in

The template system ensures:
- **One task file per source file** with ALL violations included
- **Consistent formatting** across all generated files
- **Easy customization** of prompt language without touching Rust code
- **Efficient rendering** with MiniJinja's performance

**See `./prompt/README.md` for detailed template customization guide.**

## Command Line Options

Simple and opinionated - minimal configuration needed:

```bash
kargo turd                     # Analyze all Rust projects in current directory
kargo turd --watch .           # Run analysis, then watch for changes
kargo turd --exclude "pattern" # Exclude files matching glob (can be repeated)
```

### Flags
- `--watch <path>` - Watch mode: run initial analysis, then monitor for file changes and re-analyze (requires `watchexec`)
- `--exclude <pattern>` - Exclude files matching glob pattern (can be repeated, e.g., `--exclude "**/generated/**"`)

### Opinionated Defaults
- **Output**: Always `./task/_<project_name>/tier<N>/`
- **All tiers reported**: 1, 2, and 3
- **All analyses run**: Comments, naming, orphans, dependencies, file size
- **Workers**: Auto-detected based on CPU cores
- **File threshold**: 300 lines (not configurable)
- **Test files**: Excluded by default

## Exit Codes

- `0` - Success, no violations found
- `1` - Violations found, task files generated
- `2` - File system error or missing `watchexec` in watch mode
- `3` - Parse error (invalid Rust syntax encountered)

## Performance Considerations

1. **Parallel Processing**: Use `rayon` thread pool with worker count = CPU cores
2. **File Prioritization**: Queue largest files first to maximize parallelism efficiency
3. **Incremental Analysis**: Use file hashes to skip unchanged files (future enhancement)
4. **Memory Management**: Stream file content rather than loading all into memory
5. **Early Exit**: Skip analysis of files matching exclude patterns before reading content

## Implementation Architecture

### Core Components

#### 1. Project Discovery (`project_scanner.rs`)
- Walks directory tree using `jwalk`
- Identifies all `Cargo.toml` files
- Builds project structure map
- Validates project paths

#### 2. File Queue Manager (`file_queue.rs`)
- Collects all `.rs` files from discovered projects
- Calculates file sizes and line counts
- Sorts by size (largest first) for optimal parallelism
- Manages priority queue for processing

#### 3. Pattern Matcher (`pattern_matcher.rs`)
- Comment pattern matching (tier 1, 2, 3)
- Method name pattern matching
- Variable name pattern matching
- Hardcoded value detection
- Panic-prone code detection (`.unwrap()`, `.expect()`)
- Configurable regex patterns

#### 4. AST Analyzer (`ast_analyzer.rs`)
- Uses `syn` crate for AST parsing
- Parse Rust syntax tree for:
  - Function definitions and calls
  - Module declarations and usage
  - Import statements
  - Variable declarations
  - Method calls (`.unwrap()`, `.expect()`)
  - Test attributes (`#[test]`, `#[cfg(test)]`)
- More accurate than text-based matching for complex patterns

#### 5. Orphan Detector (`orphan_detector.rs`)
- Module orphan detection
- Method/function orphan detection
- Cross-reference builder for usage tracking
- Call graph construction

#### 6. Dependency Analyzer (`dependency_analyzer.rs`)
- Parse `Cargo.toml` files
- Extract all dependency declarations
- Scan source files for import statements
- Build dependency usage map
- Report unused dependencies

#### 7. Task File Generator (`task_generator.rs`)
- Aggregates violations per file
- Determines tier classification (highest tier wins)
- Generates markdown task files
- Creates directory structure
- Computes file hashes for uniqueness

#### 8. Parallel Executor (`executor.rs`)
- Coordinates parallel processing with `rayon`
- Manages worker pool
- Collects results from all workers
- Handles errors and logging

#### 9. Watch Mode Manager (`watch_manager.rs`)
- Integrates with `watchexec` (latest version) for file monitoring
- Runs initial full analysis on startup
- Monitors glob patterns: `**/*.rs`, `**/Cargo.toml`
- Debounces file change events to avoid excessive re-analysis
- Re-runs full pipeline for changed/new files only
- Handles multiple simultaneous file changes efficiently
- Provides real-time feedback during watch mode

### Data Flow

#### Standard Mode
```
1. CLI Entry Point
   ↓
2. Load Configuration (.kargo-turd.toml + CLI flags)
   ↓
3. Project Discovery (find all Cargo.toml)
   ↓
4. File Collection (gather all .rs files)
   ↓
5. Priority Queue (sort by size)
   ↓
6. Parallel Analysis (rayon workers)
   ├─→ Pattern Matching
   ├─→ AST Analysis (optional)
   ├─→ Orphan Detection
   └─→ Hardcode Detection
   ↓
7. Dependency Analysis (per project)
   ↓
8. Aggregate Results (per file)
   ↓
9. Task File Generation
   ↓
10. Summary Report
```

#### Watch Mode (`--watch <path>`)
```
1. CLI Entry Point (--watch flag detected)
   ↓
2. Load Configuration
   ↓
3. Check for watchexec installation
   ↓
4. Run Initial Full Analysis (steps 3-10 from standard mode)
   ↓
5. Initialize watchexec with glob patterns
   │  - Monitor: **/*.rs, **/Cargo.toml
   │  - Exclude: **/target/**, **/task/**, configured exclusions
   │  - Debounce: 500ms (configurable)
   ↓
6. Watch Loop ───────────────┐
   │                         │
   ├─ File Change Detected   │
   │     ↓                   │
   ├─ Debounce Wait          │
   │     ↓                   │
   ├─ Identify Changed Files │
   │     ↓                   │
   ├─ IF .rs file changed:   │
   │     ├─ Re-analyze that file only
   │     └─ Update/regenerate task file
   │     ↓                   │
   ├─ IF Cargo.toml changed: │
   │     ├─ Re-run dependency analysis
   │     └─ Re-analyze all project files
   │     ↓                   │
   ├─ Display Change Summary │
   │     ↓                   │
   └───────────────────────────┘
   
   Press Ctrl+C to exit
```

## Testing Strategy

### Unit Tests
- **Pattern Matching**: Test each tier pattern against positive and negative examples
- **File Hash Generation**: Ensure consistent hashing across runs
- **Path Normalization**: Test platform-specific path handling
- **Configuration Parsing**: Validate TOML parsing and CLI flag parsing

### Integration Tests
- **End-to-End**: Run against known test projects with expected violations
- **Multi-Project**: Test workspace with multiple Cargo.toml files
- **Edge Cases**: Empty projects, single-file projects, deeply nested structures

### Test Projects
Create fixture projects in `tests/fixtures/`:

```
tests/fixtures/
├── clean_project/        # No violations, should generate no task files
├── tier1_violations/     # Only tier 1 violations
├── tier2_violations/     # Mix of tier 1 and 2
├── tier3_violations/     # All tiers
├── large_file/           # File > 300 lines
├── orphaned_code/        # Unused modules and methods
├── unused_deps/          # Dependencies not imported
└── complex_workspace/    # Multi-crate workspace
```

### Performance Tests
- **Benchmark**: Track analysis time for projects of varying sizes
- **Memory Usage**: Profile memory consumption during parallel processing
- **Scalability**: Test with 1, 10, 100, 1000+ files

## Future Enhancements

### Phase 2 Features
1. **Incremental Analysis**: Cache results and only re-analyze changed files
2. **IDE Integration**: LSP server for real-time violation highlighting
3. **Auto-fix Mode**: Automatically apply safe remediations (remove unused imports, etc.)
4. **Custom Rules**: User-defined patterns and violation types
5. **CI/CD Integration**: GitHub Actions, GitLab CI templates
6. **HTML Reports**: Interactive web-based violation browser
7. **Metrics Dashboard**: Track violation trends over time

### Phase 3 Features
1. **Machine Learning**: Learn project-specific patterns for false positive reduction
2. **Team Metrics**: Aggregate stats across team/organization
3. **Remediation Templates**: Suggested fixes based on common patterns
4. **Integration with Issue Trackers**: Auto-create Jira/GitHub issues for violations

## Development Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use `clippy` for linting (`cargo clippy`)
- Maintain > 80% test coverage
- Document all public APIs

### Error Handling
- Use `anyhow` for application errors
- Use `thiserror` for library errors
- Provide helpful error messages with context
- Never panic in production code paths

### Logging
- Use `tracing` for structured logging
- Log levels:
  - `ERROR`: Failed to process file, invalid config
  - `WARN`: Skipped file, pattern match edge case
  - `INFO`: Project discovered, analysis complete
  - `DEBUG`: File processed, violation found
  - `TRACE`: Pattern match attempts, AST traversal

### Dependencies
Core dependencies:
- `jwalk` - Fast directory traversal
- `rayon` - Parallel processing
- `regex` - Pattern matching
- `syn` - AST parsing for method/orphan analysis
- `toml` - Parse Cargo.toml files
- `clap` - CLI argument parsing
- `anyhow` / `thiserror` - Error handling
- `tracing` - Logging
- `sha2` - File hashing for task file names
- `minijinja` - Template rendering for task file generation

External requirements:
- `watchexec` (latest) - Required for `--watch` mode
  - Install: `cargo install watchexec-cli`
  - Runtime check: Tool verifies availability before watch mode

## Example Usage

```bash
# Analyze current directory - simple!
kargo turd

# Exclude generated files
kargo turd --exclude "**/generated/**" --exclude "**/*_pb.rs"

# Watch mode - monitor for changes and re-analyze
kargo turd --watch .

# Watch with exclusions
kargo turd --watch . --exclude "**/target/**"

# Verbose logging (optional)
RUST_LOG=debug kargo turd
```

## Integration with Claude CLI

Generated task files are designed to be consumed by the `claude` CLI tool:

```bash
# Process all tier 1 task files
for f in ./task/_*/tier1/*.md; do
  claude --file "$f" --prompt "Analyze this file and implement the recommended fixes"
done

# Batch process with confirmation
find ./task/_*/tier1 -name "*.md" | xargs -I {} claude --file {} --interactive
```

Each task file provides sufficient context for an LLM to:
1. Understand the violation
2. Analyze if it's truly problematic
3. Suggest or implement fixes
4. Verify the solution 