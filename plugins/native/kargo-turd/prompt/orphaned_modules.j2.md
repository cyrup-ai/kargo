## Orphaned Modules

{% for module in orphaned_modules %}
### `{{ module.name }}`

- **Declared in**: `{{ module.declared_in }}` (line {{ module.line_number }})
- **Visibility**: {{ module.visibility }}
- **Issue**: Module is declared but never used anywhere in the codebase

```rust
{{ module.context }}
```

## Action Required

- Evaluate the intended purpose of the orphaned module, assuming it is intended to be used by default.
- If it should be used, update this section with instructions on how to incorporate it into the codebase.
- If it is deprecated, ask for permission to remove it.
- Update this section with your findings and instructions on how to proceed.

{% endfor %}
