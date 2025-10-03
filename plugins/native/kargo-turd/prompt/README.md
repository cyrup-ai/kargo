# Task File Templates

This directory contains **MiniJinja** templates used to generate task files for the Claude CLI to process.

## Template Structure

### Master Template
- `master.j2.md` - Main orchestrator that combines all violation sections

### Violation Type Templates
- `tier1.j2.md` - Tier 1 violations (high confidence stub indicators)
- `tier2.j2.md` - Tier 2 violations (possible stub indicators)
- `tier3.j2.md` - Tier 3 violations (low confidence, needs evaluation)
- `panic_patterns.j2.md` - Panic-prone code (`.unwrap()`, `.expect()`)
- `tests_in_src.j2.md` - Tests found in `./src` instead of `./tests`
- `orphaned_modules.j2.md` - Module declarations with no usage
- `orphaned_methods.j2.md` - Function definitions with no calls
- `unused_dependencies.j2.md` - Cargo.toml dependencies not imported
- `decompose.j2.md` - Files exceeding 300 lines requiring decomposition

## How Templates Work

### 1. Data Collection
The analyzer collects violations and groups them by type into a context object.

### 2. Template Rendering
The master template (`master.j2.md`) conditionally includes child templates:

```jinja
{% if tier1_violations %}
{% include "tier1.j2.md" %}
{% endif %}
```

### 3. Looping Over Violations
Each child template loops over its violation array:

```jinja
{% for violation in tier1_violations %}
- Line {{ violation.line_number }}
  - {{ violation.search_term }}
  
```rust
{{ violation.context }}
```
{% endfor %}
```

### 4. Output
A single markdown file is generated per source file containing ALL violations found.

## Template Variables

**See [`VARIABLES.md`](./VARIABLES.md) for the complete reference.**

### Quick Reference

**Root-level variables:**
- `project_relative_path`, `absolute_path`, `project_name`
- `file_hash`, `timestamp`, `lines_of_code`, `version`
- `needs_decomposition` (bool)

**Violation arrays (each with `line_number`, `search_term`, `method_name`, `context`):**
- `tier1_violations`, `tier2_violations`, `tier3_violations`

**Specialized arrays:**
- `panic_patterns` - `.unwrap()` / `.expect()` issues
- `tests_in_src` - Tests in wrong directory
- `orphaned_modules` - Unused module declarations
- `orphaned_methods` - Unused function definitions
- `unused_dependencies` - Cargo.toml deps not imported

## Customizing Prompts

### Keep the Structure
- **DO NOT** change loop syntax: `{% for ... %} ... {% endfor %}`
- **DO NOT** remove template variable references: `{{ violation.line_number }}`
- **DO NOT** change conditional includes in `master.j2.md`

### Modify the Content
- **DO** edit the prompt language between loops
- **DO** add/remove checklist items
- **DO** adjust the questions posed to the LLM
- **DO** change formatting and emphasis

### Example Customization

**Before:**
```markdown
- is this actually a non-production indicator or a false positive?
```

**After:**
```markdown
- **CRITICAL**: Is this truly stubbed code that will fail in production?
- **IMPACT**: What breaks if this remains unfixed?
- **PRIORITY**: High/Medium/Low based on business risk
```

## Templates Marked TODO

Some templates contain `TODO` placeholders:
- `orphaned_modules.j2.md`
- `orphaned_methods.j2.md`
- `unused_dependencies.j2.md`
- `panic_patterns.j2.md`
- `tests_in_src.j2.md`

These are ready for you to add the specific prompt language for each finding type.

## Testing Templates

To test template rendering locally:

```bash
# Install minijinja CLI
cargo install minijinja-cli

# Render with test data
minijinja-cli master.j2.md --env test_data.json
```

Create `test_data.json` with sample violation data to validate your template changes.
