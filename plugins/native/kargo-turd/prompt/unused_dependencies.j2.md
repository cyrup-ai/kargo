## Unused Dependencies

{% for dep in unused_dependencies %}
### `{{ dep.name }}`

- **Declared in**: `{{ dep.cargo_toml }}` ({{ dep.section }})
- **Issue**: Dependency is declared but no imports found in any `.rs` files

**Action Required**:
- Unused dependencies should be removed from the `Cargo.toml` file
- Update this section with the specific remediation instructions

{% endfor %}
