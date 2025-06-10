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

## Next Steps
- [ ] Determine specific task/objective from user
- [ ] Create detailed implementation plan based on objective
- [ ] Implement solution with production-quality code following conventions

## Documentation
- [ ] Create/update README.md with project overview
- [ ] Document key libraries and patterns in docs/