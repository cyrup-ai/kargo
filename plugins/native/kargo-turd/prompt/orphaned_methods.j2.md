## Orphaned Methods

{% for method in orphaned_methods %}
### `{{ method.name }}()`

- **Location**: `{{ method.file_path }}` (line {{ method.line_number }})
- **Visibility**: {{ method.visibility }}
- **Issue**: Function is defined but never called anywhere in the codebase

```rust
{{ method.context }}
```

### Action Required:

- Evaluate the intended purpose of the orphaned method, assuming it is intended to be used by default.
- If it should be used, update this section with instructions on how to incorporate it into the codebase.
- If it is deprecated, ask for permission to remove it.
- Update this section with your findings and instructions on how to proceed.

{% endfor %}
