# MiniJinja Template Variables - Master Reference

Complete list of all variables available in kargo-turd templates.

## Root-Level Variables

Available in `master.j2.md` and passed to all included templates:

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `project_relative_path` | String | Path relative to project root | `"src/service/handler.rs"` |
| `absolute_path` | String | Full filesystem path | `"/Users/dev/project/src/service/handler.rs"` |
| `project_name` | String | Cargo.toml package name | `"kargo-turd"` |
| `file_hash` | String | SHA256 hash (first 8 chars) | `"a3f2b1c4"` |
| `timestamp` | String | ISO 8601 timestamp | `"2025-10-02T20:22:00-07:00"` |
| `lines_of_code` | u32 | LOC excluding blanks/comments | `487` |
| `version` | String | kargo-turd version | `"0.1.0"` |
| `needs_decomposition` | bool | File exceeds 300 LOC | `true` |

## Violation Arrays

### `tier1_violations` / `tier2_violations` / `tier3_violations`

Array of violation objects, each containing:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `line_number` | u32 | Line number in file | `42` |
| `search_term` | String | Pattern that matched | `"TODO"`, `"FIXME"`, `"stub_"` |
| `method_name` | String | Method/function name (if applicable) | `"authenticate"`, `""` |
| `context` | String | Code snippet (2 before + line + 2 after) | `"fn stub_auth() {\n  // TODO\n}"` |

**Usage in templates:**
```jinja
{% for violation in tier1_violations %}
  Line {{ violation.line_number }}: {{ violation.search_term }}
{% endfor %}
```

---

### `panic_patterns`

Array of panic pattern objects:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `line_number` | u32 | Line number | `57` |
| `pattern` | String | Panic pattern matched | `".unwrap()"`, `".expect()"` |
| `issue` | String | Description of issue | `"Can panic in production code"` |
| `context` | String | Code snippet | `"let user = db.get_user().unwrap();"` |

**Usage:**
```jinja
{% for panic in panic_patterns %}
  {{ panic.pattern }} at line {{ panic.line_number }}
{% endfor %}
```

---

### `tests_in_src`

Array of test-in-src violations:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `line_number` | u32 | Line number | `123` |
| `test_attribute` | String | Test attribute/macro | `"#[test]"`, `"#[tokio::test]"`, `"#[cfg(test)]"` |
| `file_path` | String | File path | `"src/handler.rs"` |
| `context` | String | Code snippet | `"#[test]\nfn test_auth() { ... }"` |

**Usage:**
```jinja
{% for test in tests_in_src %}
  Test at {{ test.file_path }}:{{ test.line_number }}
{% endfor %}
```

---

### `orphaned_modules`

Array of unused module declarations:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `name` | String | Module name | `"unused_helper"` |
| `declared_in` | String | File declaring the module | `"src/utils/mod.rs"` |
| `line_number` | u32 | Declaration line | `15` |
| `visibility` | String | Visibility modifier | `"pub"`, `"private"` |
| `context` | String | Code snippet | `"mod unused_helper;"` |

**Usage:**
```jinja
{% for module in orphaned_modules %}
  {{ module.visibility }} mod {{ module.name }}
{% endfor %}
```

---

### `orphaned_methods`

Array of unused function definitions:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `name` | String | Function name (no parens) | `"calculate_score"` |
| `file_path` | String | File containing function | `"src/scoring.rs"` |
| `line_number` | u32 | Function definition line | `78` |
| `visibility` | String | Visibility | `"pub"`, `"pub(crate)"`, `"private"` |
| `context` | String | Code snippet | `"fn calculate_score() -> i32 { ... }"` |

**Usage:**
```jinja
{% for method in orphaned_methods %}
  {{ method.name }}() is never called
{% endfor %}
```

---

### `unused_dependencies`

Array of unused Cargo dependencies:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `name` | String | Crate name | `"serde_yaml"` |
| `cargo_toml` | String | Path to Cargo.toml | `"./Cargo.toml"` |
| `section` | String | Cargo.toml section | `"[dependencies]"`, `"[dev-dependencies]"` |

**Usage:**
```jinja
{% for dep in unused_dependencies %}
  {{ dep.name }} in {{ dep.section }}
{% endfor %}
```

---

## Template-Specific Variables

### In `decompose.j2.md`

These are aliases/duplicates of root-level variables for consistency:

| Variable | Same As | Description |
|----------|---------|-------------|
| `file_path` | `project_relative_path` | Path to file needing decomposition |
| `lines_of_code` | `lines_of_code` | LOC count |

---

## Data Types Reference

```rust
// Rust type definitions for reference

struct TemplateContext {
    // Root level
    project_relative_path: String,
    absolute_path: String,
    project_name: String,
    file_hash: String,
    timestamp: String,
    lines_of_code: u32,
    version: String,
    needs_decomposition: bool,
    
    // Arrays
    tier1_violations: Vec<Violation>,
    tier2_violations: Vec<Violation>,
    tier3_violations: Vec<Violation>,
    panic_patterns: Vec<PanicPattern>,
    tests_in_src: Vec<TestInSrc>,
    orphaned_modules: Vec<OrphanedModule>,
    orphaned_methods: Vec<OrphanedMethod>,
    unused_dependencies: Vec<UnusedDependency>,
}

struct Violation {
    line_number: u32,
    search_term: String,
    method_name: String,
    context: String,
}

struct PanicPattern {
    line_number: u32,
    pattern: String,
    issue: String,
    context: String,
}

struct TestInSrc {
    line_number: u32,
    test_attribute: String,
    file_path: String,
    context: String,
}

struct OrphanedModule {
    name: String,
    declared_in: String,
    line_number: u32,
    visibility: String,
    context: String,
}

struct OrphanedMethod {
    name: String,
    file_path: String,
    line_number: u32,
    visibility: String,
    context: String,
}

struct UnusedDependency {
    name: String,
    cargo_toml: String,
    section: String,
}
```

---

## Example Complete Context Object

```json
{
  "project_relative_path": "src/service/auth.rs",
  "absolute_path": "/Users/dev/myapp/src/service/auth.rs",
  "project_name": "myapp",
  "file_hash": "a3f2b1c4",
  "timestamp": "2025-10-02T20:22:00-07:00",
  "lines_of_code": 487,
  "version": "0.1.0",
  "needs_decomposition": true,
  
  "tier1_violations": [
    {
      "line_number": 42,
      "search_term": "TODO",
      "method_name": "authenticate",
      "context": "fn authenticate() {\n    // TODO: implement real auth\n    Ok(())\n}"
    }
  ],
  
  "tier2_violations": [],
  "tier3_violations": [],
  
  "panic_patterns": [
    {
      "line_number": 57,
      "pattern": ".unwrap()",
      "issue": "Can panic in production code",
      "context": "let user = db.get_user(id).unwrap();"
    }
  ],
  
  "tests_in_src": [
    {
      "line_number": 123,
      "test_attribute": "#[test]",
      "file_path": "src/service/auth.rs",
      "context": "#[test]\nfn test_auth() { ... }"
    }
  ],
  
  "orphaned_modules": [],
  "orphaned_methods": [
    {
      "name": "old_validate",
      "file_path": "src/service/auth.rs",
      "line_number": 89,
      "visibility": "private",
      "context": "fn old_validate() -> bool { false }"
    }
  ],
  
  "unused_dependencies": []
}
```
