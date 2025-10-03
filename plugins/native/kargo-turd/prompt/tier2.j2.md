## Tier 2 (possible) Infractions 

{% for violation in tier2_violations %}
- Line {{ violation.line_number }}
  - {{ violation.search_term }}
  - {{ violation.method_name }}

```rust
{{ violation.context }}
```

- is this actually a non-production indicator or a false positive? If false positive, remove it from the task file.
- If IT IS a non-production fake, fabrication, incomplete, dangeours or lacking implementation: add detailed notes explaining the issue and plan out the necessary replacement work in sequential steps. 
- Update this section of the task file with the notes and plan.

{% endfor %}
