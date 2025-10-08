## Unused Dependencies

{%- for dep in unused_dependencies %}
### `{{ dep.name }}`

- **Declared in**: `{{ dep.cargo_toml }}` ({{ dep.section }})
- **Issue**: Dependency is declared but no imports found in any `.rs` files

- **Cargo.toml snippet**:
```toml
{{ dep.toml_snippet -}}
```

- **Suggested removal (unified diff)**:
```diff
--- a{{ dep.cargo_toml }}
+++ b{{ dep.cargo_toml }}
{{ dep.toml_diff -}}
```

- **Optional: cargo-edit command**:
```bash
# If you have cargo-edit installed: cargo install cargo-edit
cargo remove {{ dep.name }} --manifest-path {{ dep.cargo_toml }}
```

**Action Required**:
- Unused dependencies should be removed from the `Cargo.toml` file
- Update this section with the specific remediation instructions

{%- endfor %}
