# Unwrap/Expect Cleanup Map

## Summary
- **expect()**: 5 occurrences (2 crates)
- **unwrap()**: 165 occurrences (7 crates)  
- **unwrap_or()**: 17 occurrences (3 crates)

## Occurrences by Crate

### kargo-plugin-builder
- **expect()**: 3 occurrences
  - src/lib.rs:70, 117, 122

### kargo-cli
- **expect()**: 1 occurrence
  - src/cli.rs:49
- **unwrap()**: 20 occurrences
  - src/backup.rs:29
  - src/rustscript.rs:128, 132, 133, 194, 200, 271
  - src/project.rs:252, 253, 260, 267, 295, 296, 303, 310, 320, 334, 353, 371
  - src/plugins/wasm_adapter.rs:41, 89
  - src/plugins/host/tasks.rs:68, 80, 96
  - src/plugins/host_functions.rs:29, 39
  - tests/test_command_deserialization.rs:26
- **unwrap_or()**: 4 occurrences
  - src/project.rs:223, 229, 494
  - src/plugins/wasm_adapter.rs:59
  - src/plugins/manager.rs:103, 167

### kargo-kurate
- **unwrap()**: 4 occurrences
  - src/processor.rs:21, 26, 31, 37

### kargo-walk
- **unwrap()**: 7 occurrences
  - src/main.rs:121, 131 (x2), 217, 223, 235, 237
- **unwrap_or()**: 1 occurrence
  - src/main.rs:171

### kargo-mddoc
- **unwrap()**: 46 occurrences (including tests)
  - src/plugin.rs:92, 93
  - src/kargo/package.rs:65, 117, 124, 138, 141, 147, 151
  - src/kargo/rust2md/mod.rs:49
  - src/kargo/rust2md/markdown.rs:80, 87, 94, 101, 108, 115, 122, 129
  - test/test_doc_generation.rs:23, 106
  - test/test_documentation_generator.rs:20, 31, 32, 35, 38
- **unwrap_or()**: 7 occurrences
  - src/kargo/markdown.rs:199, 1286, 1318, 1486, 1520, 1669, 1733
  - src/kargo/rust2md/markdown.rs:188, 547, 568, 634

### kargo-upgrade
- **unwrap()**: 88 occurrences
  - src/updaters.rs:171, 183, 186, 189, 191, 192, 214, 216, 217, 254, 261, 276, 286, 288, 289, 331, 343, 345, 349, 370, 391
  - src/parsers/rust_script_parser.rs:10-18, 53, 57, 58, 71, 72, 101, 108, 122, 129, 143, 150, 164
  - test/test_dependency_updater.rs:58, 62, 66
  - test/test_dependency_parser.rs:53, 100, 107
  - test/test_dependency_writer.rs:117, 118, 190, 191

## Priorities
1. Production code in src/ directories (non-test)
2. Library crates before binary crates
3. Public API methods before internal methods