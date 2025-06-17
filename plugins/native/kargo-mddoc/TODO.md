# TODO: Kargo Project

## Project Understanding
- [ ] Analyze Cargo.toml files in each subproject
- [ ] Understand dependencies and their versions
- [ ] Investigate the cyrup-ai/async_task crate (critical per conventions)
- [ ] Map relationships between kargo subprojects
- [ ] Understand the plugin architecture

## Research Required
- [ ] Find documentation for SurrealDB 2.2.1+ (preferred database)
- [ ] Study async patterns without async_trait or Box<dyn Future> returns
- [ ] Review examples of plugin architecture implementations in Rust

## Implementation Strategy
- [ ] Follow project conventions strictly
  - [ ] No async_trait or async fn in public API
  - [ ] No Box<dyn Future> returns
  - [ ] Use cyrup-ai/async_task crate for async operations
  - [ ] Keep files under 300 lines
  - [ ] Use snake_case for variables/functions
  - [ ] Place tests in tests/ directory
- [ ] Run "cargo fmt && cargo check --message-format short --quiet" after every change

## Plugin Architecture Implementation (CRITICAL - READ CAREFULLY)

### Core Plugin Principles
- **PLUGINS ARE UNKNOWN TO THE CORE**: kargo-cli must have ZERO compile-time knowledge of any plugin implementations
- **DYNAMIC DISCOVERY ONLY**: Plugins are discovered at runtime by scanning directories for .so/.dylib/.dll/.wasm files
- **NO STATIC REGISTRATION**: Never use static registries, inventory crate, or any compile-time plugin registration
- **NO FEATURE FLAGS FOR PLUGINS**: kargo-cli cannot have feature flags for plugins because it doesn't know they exist

### Native Plugin Implementation Tasks
- [ ] Create kargo-plugin-native crate that provides utilities for BUILDING plugins, NOT for registering them
  - This crate helps plugin authors create compatible .so/.dylib/.dll files
  - It does NOT create any registry or collection of plugins
  - It exports helper macros like `#[kargo_plugin]` that generate the required `extern "C" fn kargo_plugin_create()` function
  - Plugins depend on this crate; kargo-cli does NOT

### WASM Plugin Implementation Tasks  
- [ ] Create kargo-plugin-wasm crate that provides WASM plugin runtime support
  - This is used BY kargo-cli to load and execute WASM plugins at runtime
  - It wraps extism and provides the WasmPlugin adapter
  - It does NOT know about specific WASM plugins at compile time

### Plugin Loading Architecture
- [ ] kargo-cli uses dynamic loading (libloading) to load native plugins at runtime
- [ ] kargo-cli scans plugin directories and loads .so/.dylib/.dll files it finds
- [ ] Each plugin exports `kargo_plugin_create() -> Box<dyn PluginCommand>` 
- [ ] kargo-cli has NO compile-time knowledge of what plugins exist
- [ ] Plugins are separate crates that are built independently and dropped into plugin directories

### Converting Existing Plugins
- [ ] Update kargo-mddoc to export `kargo_plugin_create` function instead of any static registration
- [ ] Update kargo-sap to export `kargo_plugin_create` function
- [ ] Ensure plugins are built as cdylib targets that produce .so/.dylib/.dll files
- [ ] Plugins should NOT be dependencies of kargo-cli in Cargo.toml

### What NOT to do (CRITICAL)
- [ ] DO NOT make kargo-cli depend on plugin crates
- [ ] DO NOT use static plugin registries
- [ ] DO NOT use the inventory crate for plugin collection  
- [ ] DO NOT add feature flags to kargo-cli for plugins
- [ ] DO NOT compile plugins into the kargo-cli binary

## Next Steps
- [ ] Determine specific task/objective from user
- [ ] Create detailed implementation plan based on objective
- [ ] Implement solution with production-quality code following conventions

## Documentation
- [x] Create rustdoc-types API changes mapping in docs/rustdoc-types-api-changes.md
- [ ] Create/update README.md with project overview
- [ ] Document key libraries and patterns in docs/

## Rustdoc-types API Migration (0.46 compatibility)

### Field Name Changes
- [x] Fix Path.name → Path.path field access (completed)
- [x] Fix Static mutable → is_mutable field rename (completed)
- [x] Remove rustdoc_md references (completed)
- [x] Add missing rustdoc_types imports (completed)
- [x] Fix FunctionHeader field names: `const_` → `is_const`, `unsafe_` → `is_unsafe`, `async_` → `is_async` (lines 860, 863, 1048, 1051, 1054 in markdown.rs)
- [x] Fix FunctionSignature field access: `decl.` → `sig.` and `c_variadic` → `is_c_variadic` (lines 873, 875, 881, 888, 1070, 1076, 1082, 1089 in markdown.rs)
- [x] Fix struct field names: `fields_stripped` → `has_stripped_fields` in StructKind::Plain (lines 400, 1336 in markdown.rs; lines 393, 579 in rust2md/markdown.rs)
- [x] Fix variant field names: `fields_stripped` → `has_stripped_fields` in VariantKind::Struct (lines 1160, 1538 in markdown.rs)
- [x] Fix Union field names: `fields_stripped` → `has_stripped_fields` (lines 466, 1687 in markdown.rs; line 646 in rust2md/markdown.rs)
- [x] Fix Enum field names: `variants_stripped` → `has_stripped_variants` (lines 1177, 1554 in markdown.rs)
- [x] Fix GenericParamDefKind field names: `synthetic` → `is_synthetic` (line 542 in markdown.rs)
- [x] Fix Type field names: `mutable` → `is_mutable` in BorrowedRef and RawPointer (lines 922, 932, 476 in markdown.rs)
- [x] Fix Trait field names: `is_object_safe` → `is_dyn_compatible` (line 1765 in markdown.rs)

### Enum Variant Changes  
- [x] Fix WherePredicate variants: `RegionPredicate` → `LifetimePredicate`, `bounds` → `outlives` (line 614 in markdown.rs)

### Method vs Field Access
- [x] Fix FunctionHeader method calls to field access in rust2md/markdown.rs (lines 406-407, 423)

### ItemEnum Pattern Updates
- [x] Standardize Constant handling: Update `ItemEnum::Constant(_)` patterns to struct variant `ItemEnum::Constant { type_, const_ }` (lines 99, 411-418, 508 in markdown.rs)

### Additional API Fixes Found
- [x] Fix Impl field names: `negative` → `is_negative`, `synthetic` → `is_synthetic`
- [x] Fix AssocConst pattern: `default` → `value` field
- [x] Fix LifetimePredicate outlives type: Vec<GenericBound> → Vec<String>

## kargo-mddoc Status
- [x] All rustdoc-types 0.46 API compatibility issues fixed
- [x] Successfully compiles with only unused import warnings