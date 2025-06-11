# Rustdoc-Types API Changes (v0.41+)

This document maps the API changes in rustdoc-types that affect kargo-mddoc.

## Field Renames

### Static
- `mutable` → `is_mutable`

### Struct/Union/Enum  
- `fields_stripped` → `has_stripped_fields`

### Module
- `stripped` → `is_stripped` (if applicable)

### Type Variants
- `RawPointer { mutable, type_ }` → `RawPointer { is_mutable, type_ }`
- `BorrowedRef { mutable, ... }` → `BorrowedRef { is_mutable, ... }`

### Path
- No longer has a `name` field
- Has `path: String` - the actual path string
- Has `id: Id` - the ID reference
- Has `args: Option<Box<GenericArgs>>`

### FunctionHeader
- `const_` → `is_const` (likely)

## Structural Changes

### ItemEnum::Constant
Changed from tuple variant to struct variant:
```rust
// Old
Constant(Constant)

// New  
Constant {
    #[serde(rename = "type")]
    type_: Type,
    #[serde(rename = "const")]
    const_: Constant,
}
```

### GenericArgs
- `bindings` → `constraints` (Vec<AssocItemConstraint>)
- Added `ReturnTypeNotation` variant

### Type Bindings
- `TypeBinding` → `AssocItemConstraint`
- `TypeBindingKind` → `AssocItemConstraintKind`

### GenericParamDefKind
- Need to find the new structure/location

### WherePredicate
- `RegionPredicate` variant may have been removed or renamed

## Import Changes
- `Typedef` → `TypeAlias`

## Usage Patterns

### Accessing Path names
```rust
// Old
path.name

// New
path.path
```

### Checking mutability
```rust
// Old  
if static_.mutable { ... }

// New
if static_.is_mutable { ... }
```

### Pattern matching Constant
```rust
// Old
ItemEnum::Constant(constant) => { ... }

// New
ItemEnum::Constant { type_, const_ } => { ... }
```

### Working with constraints
```rust
// Old
match binding.binding {
    TypeBindingKind::Equality(term) => ...
}

// New
match constraint.binding {
    AssocItemConstraintKind::Equality(term) => ...
}
```