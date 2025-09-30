# TODO: Fix Markdown Linting Issues in kargo-mddoc

## Overview
The generated markdown documentation from kargo-mddoc has 347 linting errors when checked with kargo-mdlint.
We need to fix the markdown generation to produce lint-compliant output.

## Main Issues Found

### High Priority
- [ ] **MD004**: Unordered list style consistency - change all list items to use '-' instead of '*'
  - Fixed in multipage_markdown.rs (TOC lists)
  - Still need to check rust2md/markdown.rs for other instances
- [ ] **MD022**: Headers should be surrounded by blank lines
  - Need to add blank line before and after each header
- [ ] **MD025**: Multiple top level headers in the same document
  - Ensure only one H1 (#) per document
  - Use H2 (##) for subsequent sections

### Medium Priority  
- [ ] **MD031**: Fenced code blocks should be surrounded by blank lines
- [ ] **MD032**: Lists should be surrounded by blank lines
- [ ] **MD001**: Header levels should only increment by one level at a time
- [ ] **MD013**: Line length (some lines exceed 80 chars)
- [ ] **MD038**: Spaces inside code span elements
- [ ] **MD033**: Inline HTML usage

### Low Priority
- [ ] **MD046**: Code block style consistency
- [ ] **MD047**: Files should end with a single newline character
- [ ] **MD024**: Multiple headers with the same content
- [ ] **MD003**: Header style consistency
- [ ] **MD009**: Trailing spaces
- [ ] **MD010**: Hard tabs
- [ ] **MD012**: Multiple consecutive blank lines

## Progress

### Completed
- [x] Add kargo-sap to workspace members
- [x] Fix list style in multipage_markdown.rs TOC (changed * to -)

### In Progress
- [ ] Searching for all list generation locations
- [ ] Creating comprehensive fixes for header spacing

### Next Steps
1. Find all locations where lists are generated
2. Add blank line logic around headers
3. Fix H1/H2 hierarchy issues
4. Add blank lines around code blocks and lists
5. Test with kargo mdlint after each fix

---

# Workspace Lint/Check TODO (Generated 2025-08-07 18:40:24-07:00)

## kargo-cli/src/plugins/manager.rs

- [ ] Warning: unused variable `name` when scanning native plugins directory
  - Location: `kargo-cli/src/plugins/manager.rs` (around discovery under `plugins/native`)
  - Proposed fix: Log discovered plugin candidate names via `log::info!` to surface what is found. This makes the variable meaningfully used and improves observability.
  - QA Task: Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1 - 10 (complete failure through significant improvement). Provide specific feedback on any issues or truly great work (objectively without bragging).

- [ ] Warning: unused variable `name` when scanning wasm plugins directory
  - Location: `kargo-cli/src/plugins/manager.rs` (around discovery under `plugins/wasm`)
  - Proposed fix: Log discovered WASM plugin candidate names via `log::info!`.
  - QA Task: Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1 - 10 (complete failure through significant improvement). Provide specific feedback on any issues or truly great work (objectively without bragging).

- [ ] Warning: unused variable `name` when iterating loaded plugins at end of discovery
  - Location: `kargo-cli/src/plugins/manager.rs` (end of `discover_and_load_plugins`)
  - Proposed fix: Emit `info!` per loaded plugin (e.g., `info!("Loaded plugin: {}", name);`) to confirm successful load and use the variable meaningfully.
  - QA Task: Act as an Objective Rust Expert and rate the quality of the fix on a scale of 1 - 10 (complete failure through significant improvement). Provide specific feedback on any issues or truly great work (objectively without bragging).