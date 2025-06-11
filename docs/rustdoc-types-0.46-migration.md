# rustdoc-types 0.46 Migration Guide

This guide documents the API changes in rustdoc-types version 0.46 compared to earlier versions.

## Key API Changes

### 1. TypeBindingKind Removal
`TypeBindingKind` has been removed from the API. Type bindings are now handled differently in the type system.

### 2. Visibility Enum
The `Visibility` enum is available and should be imported from the crate root:
```rust
use rustdoc_types::Visibility;
```

Available variants:
- `Visibility::Public`
- `Visibility::Default`
- `Visibility::Crate`
- `Visibility::Restricted { parent: Id, path: String }`

### 3. StructKind Enum
The `StructKind` enum is available and should be imported from the crate root:
```rust
use rustdoc_types::StructKind;
```

Available variants:
- `StructKind::Unit`
- `StructKind::Tuple(Vec<Option<Id>>)`
- `StructKind::Plain { fields: Vec<Id>, has_stripped_fields: bool }`

**Important:** The field `fields_stripped` has been renamed to `has_stripped_fields`.

### 4. Type Enum
The `Type` enum is available and should be imported from the crate root:
```rust
use rustdoc_types::Type;
```

Common variants include:
- `Type::ResolvedPath(Path)`
- `Type::DynTrait(DynTrait)`
- `Type::Generic(String)`
- `Type::Primitive(String)`
- `Type::FunctionPointer(Box<FunctionPointer>)`

### 5. Field Renames

| Old Field Name | New Field Name | Affected Types |
|----------------|----------------|----------------|
| `fields_stripped` | `has_stripped_fields` | `Struct`, `Union`, `Enum` |
| `mutable` | `is_mutable` | `Static` |
| `stripped` | `is_stripped` | `Module` |

### 6. Import Changes
All main types should be imported from the crate root:
```rust
use rustdoc_types::{
    Crate, Item, ItemEnum, Visibility, StructKind, Type,
    Struct, Enum, Union, Module, Static, Trait, Impl,
    Function, Constant, TypeAlias, VariantKind,
    GenericParamDefKind, Generics, Path
};
```

### 7. ItemEnum Pattern Matching
The `ItemEnum::Constant` variant is no longer a tuple variant. It's now a struct variant with a named field:
```rust
// Old (incorrect)
ItemEnum::Constant(constant)

// New (correct)
ItemEnum::Constant { constant, .. }
```

### 8. Path Structure
The `Path` type no longer has a direct `name` field. Access path segments through the appropriate fields.

## Common Migration Patterns

### Updating Field Access
```rust
// Old
if struct_data.fields_stripped { ... }
if static_data.mutable { ... }

// New
if struct_data.has_stripped_fields { ... }
if static_data.is_mutable { ... }
```

### Updating Pattern Matching
```rust
// Old
match item.inner {
    ItemEnum::Constant(c) => { ... }
    _ => {}
}

// New
match item.inner {
    ItemEnum::Constant { constant: c, .. } => { ... }
    _ => {}
}
```

### Importing Types
```rust
// Add at the top of files using rustdoc-types
use rustdoc_types::{Visibility, StructKind, Type, VariantKind, GenericParamDefKind};
```

## References
- [rustdoc-json-types source](https://github.com/rust-lang/rust/blob/master/src/rustdoc-json-types/lib.rs)
- [rustdoc-types on crates.io](https://crates.io/crates/rustdoc-types)
- [rustdoc-types documentation](https://docs.rs/rustdoc-types/0.46.1/rustdoc_types/)