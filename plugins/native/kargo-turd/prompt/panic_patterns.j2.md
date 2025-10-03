## Panic-Prone Code

{% for panic in panic_patterns %}
### Line {{ panic.line_number }}: `{{ panic.pattern }}`

- **Pattern**: {{ panic.pattern }}
- **Issue**: {{ panic.issue }}

```rust
{{ panic.context }}
```

### Action Required

- unwrap() should never be used in `./src/**/*.rs` or `./tests/**/*.rs` (period). The code should be updated with proper error handling and all match arms addressed.
- unwrap_or_else() is a-ok. 
- expect() should never be used in `./src/**/*.rs` but should ALWAYS BE USED in `./tests/**/*.rs` (rather than unwrap)
- panic can be approved with my written consent for situations that should in practice never happen  
  - ASK FOR WRITTEN PERMISSION
  - If granted, annotate the code with a comment "APPROVED PANIC {{ date }}"

{% endfor %}
