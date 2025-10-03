## Tests in Source Directory

{% for test in tests_in_src %}
### Line {{ test.line_number }}: `{{ test.test_attribute }}`

- **Location**: `{{ test.file_path }}` (line {{ test.line_number }})
- **Issue**: Tests must be in `./tests` directory, not in `./src`

```rust
{{ test.context }}
```

### Action Required

- Extract tests into `./tests` directory
  - `tests/` should mirror the file structure of the `src/` with file names prepended with `test_`
  - Update this section with specific remediation instructions
  

{% endfor %}
