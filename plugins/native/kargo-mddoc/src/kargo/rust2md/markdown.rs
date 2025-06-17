//! A module for converting rustdoc JSON into human-friendly Markdown documentation.

use anyhow::{Context, Result};
use rustdoc_types::{Crate, Id, Item, ItemEnum, StructKind, VariantKind, Visibility};
use rustdoc_types::{Enum, Impl, Module, Struct, Trait, Type, Union};
use rustdoc_types::{GenericParamDefKind, Generics};
use std::path::Path;
use tokio::fs;

/// Generates markdown documentation from rustdoc JSON output
pub struct MarkdownGenerator {
    crate_data: Crate,
}

impl MarkdownGenerator {
    pub fn new(crate_data: Crate) -> Self {
        Self { crate_data }
    }

    /// Load rustdoc JSON from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .await
            .context("Failed to read rustdoc JSON file")?;
        let crate_data: Crate =
            serde_json::from_str(&content).context("Failed to parse rustdoc JSON")?;
        Ok(Self::new(crate_data))
    }

    /// Generates markdown documentation for the entire crate
    pub fn generate_markdown(&self) -> String {
        let mut output = String::new();

        // Add crate header and basic info
        output.push_str("# Crate Documentation\n\n");

        if let Some(version) = &self.crate_data.crate_version {
            output.push_str(&format!("**Version:** {}\n\n", version));
        }

        output.push_str(&format!(
            "**Format Version:** {}\n\n",
            self.crate_data.format_version
        ));

        // Process the root module to start
        let root_id = self.crate_data.root.clone();
        if let Some(root_item) = self.crate_data.index.get(&root_id) {
            if let ItemEnum::Module(module) = &root_item.inner {
                if let Some(name) = &root_item.name {
                    output.push_str(&format!("# Module `{}`\n\n", name));
                } else if module.is_crate {
                    output.push_str("# Crate Root\n\n");
                }

                // Add root documentation if available
                if let Some(docs) = &root_item.docs {
                    output.push_str(&format!("{}\n\n", docs));
                }

                // Process items in the root module at heading level 2
                process_items(&mut output, &module.items, &self.crate_data, 2);
            }
        }

        output
    }
}

/// Process items from a module by grouping them into user-friendly sections.
///
/// Each section (modules, types, traits, etc.) is printed with a consistent heading level.
fn process_items(output: &mut String, item_ids: &[Id], data: &Crate, level: usize) {
    let heading_level = std::cmp::min(level, 6);

    // Group item IDs by category
    let grouped = group_module_items(item_ids, data);

    // Process each category in an order that matches typical Rust docs
    if !grouped.modules.is_empty() {
        output.push_str(&format!("{} Modules\n\n", "#".repeat(heading_level)));
        for id in &grouped.modules {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.types.is_empty() {
        output.push_str(&format!("{} Types\n\n", "#".repeat(heading_level)));
        for id in &grouped.types {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.traits.is_empty() {
        output.push_str(&format!("{} Traits\n\n", "#".repeat(heading_level)));
        for id in &grouped.traits {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.functions.is_empty() {
        output.push_str(&format!("{} Functions\n\n", "#".repeat(heading_level)));
        for id in &grouped.functions {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.constants.is_empty() {
        output.push_str(&format!(
            "{} Constants and Statics\n\n",
            "#".repeat(heading_level)
        ));
        for id in &grouped.constants {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.macros.is_empty() {
        output.push_str(&format!("{} Macros\n\n", "#".repeat(heading_level)));
        for id in &grouped.macros {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.reexports.is_empty() {
        output.push_str(&format!("{} Re-exports\n\n", "#".repeat(heading_level)));
        for id in &grouped.reexports {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }

    if !grouped.other_items.is_empty() {
        output.push_str(&format!("{} Other Items\n\n", "#".repeat(heading_level)));
        for id in &grouped.other_items {
            if let Some(item) = data.index.get(id) {
                process_item(output, item, data, level + 1);
            }
        }
    }
}

/// Helper struct to hold grouped item IDs for a module.
struct GroupedItems {
    modules: Vec<Id>,
    types: Vec<Id>,
    traits: Vec<Id>,
    functions: Vec<Id>,
    constants: Vec<Id>,
    macros: Vec<Id>,
    reexports: Vec<Id>,
    other_items: Vec<Id>,
}

/// Group the items in a module by their "kind": modules, types, traits, functions, etc.
fn group_module_items(item_ids: &[Id], data: &Crate) -> GroupedItems {
    let mut grouped = GroupedItems {
        modules: Vec::new(),
        types: Vec::new(),
        traits: Vec::new(),
        functions: Vec::new(),
        constants: Vec::new(),
        macros: Vec::new(),
        reexports: Vec::new(),
        other_items: Vec::new(),
    };

    for id in item_ids {
        if let Some(item) = data.index.get(id) {
            match &item.inner {
                ItemEnum::Module(_) => grouped.modules.push(id.clone()),
                ItemEnum::Struct(_)
                | ItemEnum::Enum(_)
                | ItemEnum::Union(_)
                | ItemEnum::TypeAlias(_) => grouped.types.push(id.clone()),
                ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => grouped.traits.push(id.clone()),
                ItemEnum::Function(_) => grouped.functions.push(id.clone()),
                ItemEnum::Constant { .. } | ItemEnum::Static(_) => {
                    grouped.constants.push(id.clone())
                }
                ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => grouped.macros.push(id.clone()),
                ItemEnum::Use(_) => grouped.reexports.push(id.clone()),
                _ => grouped.other_items.push(id.clone()),
            }
        }
    }

    grouped
}

/// Process a single item (struct, enum, trait, function, etc.) and render it as Markdown.
fn process_item(output: &mut String, item: &Item, data: &Crate, level: usize) {
    let heading_level = std::cmp::min(level, 6);
    let heading = "#".repeat(heading_level);

    // Handle re-exports specially first
    if let ItemEnum::Use(use_item) = &item.inner {
        // This is a re-export
        let source_name = match use_item.source.split("::").last() {
            Some(name) => name,
            None => &use_item.source,
        };
        if use_item.is_glob {
            output.push_str(&format!(
                "{} Re-export `{}::*`\n\n",
                heading, use_item.source
            ));
        } else if let Some(name) = &item.name {
            if name != source_name {
                output.push_str(&format!(
                    "{} Re-export `{}` as `{}`\n\n",
                    heading, source_name, name
                ));
            } else {
                output.push_str(&format!("{} Re-export `{}`\n\n", heading, name));
            }
        } else {
            output.push_str(&format!("{} Re-export `{}`\n\n", heading, source_name));
        }
    } else {
        // Handle named items (mod, struct, enum, trait, etc.)
        if let Some(name) = &item.name {
            match &item.inner {
                ItemEnum::Module(_) => {
                    // For modules, use "##" to make them more prominent
                    output.push_str(&format!("## Module `{}`\n\n", name))
                }
                ItemEnum::Struct(_) => {
                    output.push_str(&format!("{} Struct `{}`\n\n", heading, name))
                }
                ItemEnum::Enum(_) => output.push_str(&format!("{} Enum `{}`\n\n", heading, name)),
                ItemEnum::Union(_) => output.push_str(&format!("{} Union `{}`\n\n", heading, name)),
                ItemEnum::Trait(_) => output.push_str(&format!("{} Trait `{}`\n\n", heading, name)),
                ItemEnum::TraitAlias(_) => {
                    output.push_str(&format!("{} Trait Alias `{}`\n\n", heading, name))
                }
                ItemEnum::Function(_) => {
                    output.push_str(&format!("{} Function `{}`\n\n", heading, name))
                }
                ItemEnum::TypeAlias(_) => {
                    output.push_str(&format!("{} Type Alias `{}`\n\n", heading, name))
                }
                ItemEnum::Constant { .. } => {
                    output.push_str(&format!("{} Constant `{}`\n\n", heading, name))
                }
                ItemEnum::Static(_) => {
                    output.push_str(&format!("{} Static `{}`\n\n", heading, name))
                }
                ItemEnum::Macro(_) => output.push_str(&format!("{} Macro `{}`\n\n", heading, name)),
                ItemEnum::ProcMacro(_) => {
                    output.push_str(&format!("{} Procedural Macro `{}`\n\n", heading, name))
                }
                ItemEnum::ExternCrate {
                    name: crate_name, ..
                } => output.push_str(&format!("{} Extern Crate `{}`\n\n", heading, crate_name)),
                // For everything else with a name
                _ => output.push_str(&format!("{} `{}`\n\n", heading, name)),
            }
        } else {
            // Handle items that don't have a name (e.g. impl blocks)
            match &item.inner {
                ItemEnum::Impl(impl_) => {
                    if let Some(trait_) = &impl_.trait_ {
                        // For trait impls
                        output.push_str(&format!(
                            "{} Implementation of `{}` for `{}`\n\n",
                            heading,
                            trait_.path,
                            format_type(&impl_.for_, data)
                        ));
                    } else {
                        // For inherent impls
                        output.push_str(&format!(
                            "{} Implementation for `{}`\n\n",
                            heading,
                            format_type(&impl_.for_, data)
                        ));
                    }
                }
                // Fallback for anything else unnamed
                _ => output.push_str(&format!("{} Unnamed Item\n\n", heading)),
            }
        }
    }

    // Add item attributes if present
    if !item.attrs.is_empty() {
        output.push_str("**Attributes:**\n\n");
        for attr in &item.attrs {
            output.push_str(&format!("- `{}`\n", attr));
        }
        output.push('\n');
    }

    // Add deprecation info if present
    if let Some(deprecation) = &item.deprecation {
        output.push_str("**⚠️ Deprecated");
        if let Some(since) = &deprecation.since {
            output.push_str(&format!(" since {}", since));
        }
        output.push_str("**");
        if let Some(note) = &deprecation.note {
            output.push_str(&format!(": {}", note));
        }
        output.push_str("\n\n");
    }

    // Add documentation if available
    if let Some(docs) = &item.docs {
        output.push_str(&format!("{}\n\n", docs));
    }

    // Add code block with item signature
    output.push_str("```rust\n");
    format_item_signature(output, item, data);
    output.push_str("\n```\n\n");

    // Process additional details based on item kind
    match &item.inner {
        ItemEnum::Module(module) => process_module_details(output, module, data, level + 1),
        ItemEnum::Struct(s) => process_struct_details(output, s, data, level + 1),
        ItemEnum::Enum(e) => process_enum_details(output, e, data, level + 1),
        ItemEnum::Union(u) => process_union_details(output, u, data, level + 1),
        ItemEnum::Trait(t) => process_trait_details(output, t, data, level + 1),
        ItemEnum::Impl(i) => process_impl_details(output, i, data, level + 1),
        _ => {}
    }
}

/// Create a Rust-style signature for an item (e.g., `fn`, `struct`, etc.) and append it to `output`.
fn format_item_signature(output: &mut String, item: &Item, data: &Crate) {
    // Format visibility
    match &item.visibility {
        Visibility::Public => output.push_str("pub "),
        Visibility::Crate => output.push_str("pub(crate) "),
        Visibility::Restricted { path, .. } => output.push_str(&format!("pub(in {}) ", path)),
        Visibility::Default => {}
    }

    match &item.inner {
        // For modules
        ItemEnum::Module(_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("mod {} {{ /* ... */ }}", name));
            }
        }
        // For structs
        ItemEnum::Struct(struct_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("struct {}", name));
                format_generics(output, &struct_.generics);

                match &struct_.kind {
                    StructKind::Unit => output.push(';'),
                    StructKind::Tuple(fields) => {
                        output.push('(');
                        for (i, field_opt) in fields.iter().enumerate() {
                            if let Some(field_id) = field_opt {
                                if let Some(field_item) = data.index.get(field_id) {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        match &field_item.visibility {
                                            Visibility::Public => output.push_str("pub "),
                                            Visibility::Crate => output.push_str("pub(crate) "),
                                            Visibility::Restricted { path, .. } => {
                                                output.push_str(&format!("pub(in {}) ", path))
                                            }
                                            Visibility::Default => {}
                                        }
                                        output.push_str(&format_type(field_type, data));
                                    }
                                }
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            } else {
                                output.push_str("/* private field */");
                                if i < fields.len() - 1 {
                                    output.push_str(", ");
                                }
                            }
                        }
                        output.push_str(");");
                    }
                    StructKind::Plain {
                        fields,
                        has_stripped_fields,
                    } => {
                        output.push_str(" {\n");
                        for field_id in fields {
                            if let Some(field_item) = data.index.get(field_id) {
                                if let Some(field_name) = &field_item.name {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        match &field_item.visibility {
                                            Visibility::Public => output.push_str("    pub "),
                                            Visibility::Crate => output.push_str("    pub(crate) "),
                                            Visibility::Restricted { path, .. } => {
                                                output.push_str(&format!("    pub(in {}) ", path))
                                            }
                                            Visibility::Default => output.push_str("    "),
                                        }
                                        output.push_str(&format!(
                                            "{}: {},\n",
                                            field_name,
                                            format_type(field_type, data)
                                        ));
                                    }
                                }
                            }
                        }
                        if *has_stripped_fields {
                            output.push_str("    // Some fields omitted\n");
                        }
                        output.push('}');
                    }
                }
            }
        }
        // For other item types, add basic signature formatting...
        // For enums, traits, functions, etc. would follow similar patterns,
        // but for brevity we'll just implement a subset here
        ItemEnum::Function(function) => {
            if let Some(name) = &item.name {
                if function.header.is_const {
                    output.push_str("const ");
                }
                if function.header.is_unsafe {
                    output.push_str("unsafe ");
                }
                if function.header.is_async {
                    output.push_str("async ");
                }

                // Could add ABI handling here...

                output.push_str(&format!("fn {}", name));
                format_generics(output, &function.generics);

                // Params
                output.push('(');
                for (i, (param_name, param_type)) in function.sig.inputs.iter().enumerate() {
                    output.push_str(&format!(
                        "{}: {}",
                        param_name,
                        format_type(param_type, data)
                    ));
                    if i < function.sig.inputs.len() - 1 || function.sig.is_c_variadic {
                        output.push_str(", ");
                    }
                }

                if function.sig.is_c_variadic {
                    output.push_str("...");
                }
                output.push(')');

                // Return
                if let Some(return_type) = &function.sig.output {
                    output.push_str(&format!(" -> {}", format_type(return_type, data)));
                }

                if function.has_body {
                    output.push_str(" { /* ... */ }");
                } else {
                    output.push(';');
                }
            }
        }
        // For other types, we would implement similar formatting
        _ => output.push_str("/* Signature not implemented for this item type */"),
    }
}

/// Format type for display
fn format_type(ty: &Type, data: &Crate) -> String {
    match ty {
        Type::ResolvedPath(path) => {
            let mut result = path.path.clone();
            if let Some(_args) = &path.args {
                // We would format generic args here for full implementation
                result.push_str("<...>");
            }
            result
        }
        Type::Generic(name) => name.clone(),
        Type::Primitive(name) => name.clone(),
        Type::Tuple(ts) => {
            if ts.is_empty() {
                "()".to_string()
            } else {
                let types: Vec<String> = ts.iter().map(|t| format_type(t, data)).collect();
                format!("({})", types.join(", "))
            }
        }
        Type::Slice(elem_ty) => format!("[{}]", format_type(elem_ty, data)),
        Type::Array { type_, len } => format!("[{}; {}]", format_type(type_, data), len),
        Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => {
            let mut result = String::from("&");
            if let Some(lt) = lifetime {
                result.push_str(&format!("'{} ", lt));
            }
            if *is_mutable {
                result.push_str("mut ");
            }
            result.push_str(&format_type(type_, data));
            result
        }
        // For other type variants, we would implement similar formatting
        _ => "/* Type formatting not fully implemented */".to_string(),
    }
}

/// Format generics for display
fn format_generics(output: &mut String, generics: &Generics) {
    if generics.params.is_empty() {
        return;
    }

    output.push('<');
    for (i, param) in generics.params.iter().enumerate() {
        match &param.kind {
            GenericParamDefKind::Lifetime { .. } => {
                output.push_str(&format!("'{}", param.name));
            }
            GenericParamDefKind::Type { .. } => {
                output.push_str(&param.name);
            }
            GenericParamDefKind::Const { .. } => {
                output.push_str(&format!("const {}: /* type */", param.name));
            }
        }

        if i < generics.params.len() - 1 {
            output.push_str(", ");
        }
    }
    output.push('>');
}

/// Process module details
fn process_module_details(output: &mut String, module: &Module, data: &Crate, level: usize) {
    if module.is_stripped {
        output.push_str("> **Note:** This module is stripped. Some items may be omitted.\n\n");
    }
    // Reset level to avoid going too deep
    process_items(output, &module.items, data, level);
}

/// Process struct details
fn process_struct_details(output: &mut String, struct_: &Struct, data: &Crate, level: usize) {
    // Process struct fields and implementations
    let heading_level = std::cmp::min(level, 6);

    // Detail fields
    match &struct_.kind {
        StructKind::Unit => {}
        StructKind::Tuple(fields) => {
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Index | Type | Documentation |\n");
            output.push_str("|-------|------|---------------|\n");
            for (i, field_opt) in fields.iter().enumerate() {
                if let Some(field_id) = field_opt {
                    if let Some(field_item) = data.index.get(field_id) {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs = match field_item.docs.as_deref() {
                                Some(d) => d.replace('\n', "<br>"),
                                None => String::new(),
                            };
                            output.push_str(&format!(
                                "| {} | `{}` | {} |\n",
                                i,
                                format_type(field_type, data),
                                docs
                            ));
                        }
                    }
                } else {
                    output.push_str(&format!("| {} | `private` | *Private field* |\n", i));
                }
            }
            output.push('\n');
        }
        StructKind::Plain {
            fields,
            has_stripped_fields,
        } => {
            output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
            output.push_str("| Name | Type | Documentation |\n");
            output.push_str("|------|------|---------------|\n");
            for field_id in fields {
                if let Some(field_item) = data.index.get(field_id) {
                    if let Some(field_name) = &field_item.name {
                        if let ItemEnum::StructField(field_type) = &field_item.inner {
                            let docs = match field_item.docs.as_deref() {
                                Some(d) => d.replace('\n', "<br>"),
                                None => String::new(),
                            };
                            output.push_str(&format!(
                                "| `{}` | `{}` | {} |\n",
                                field_name,
                                format_type(field_type, data),
                                docs
                            ));
                        }
                    }
                }
            }
            if *has_stripped_fields {
                output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
            }
            output.push('\n');
        }
    }
}

/// Process enum details
fn process_enum_details(output: &mut String, enum_: &Enum, data: &Crate, level: usize) {
    // Process enum variants and implementations
    let heading_level = std::cmp::min(level, 6);

    // Detail variants
    output.push_str(&format!("{} Variants\n\n", "#".repeat(heading_level)));
    for variant_id in &enum_.variants {
        if let Some(variant_item) = data.index.get(variant_id) {
            if let Some(variant_name) = &variant_item.name {
                let variant_heading_level = std::cmp::min(heading_level + 1, 6);
                output.push_str(&format!(
                    "{} `{}`\n\n",
                    "#".repeat(variant_heading_level),
                    variant_name
                ));

                if let Some(docs) = &variant_item.docs {
                    output.push_str(&format!("{}\n\n", docs));
                }

                if let ItemEnum::Variant(variant) = &variant_item.inner {
                    match &variant.kind {
                        VariantKind::Plain => {
                            if let Some(discriminant) = &variant.discriminant {
                                output.push_str(&format!(
                                    "Discriminant: `{}`\n\n",
                                    discriminant.expr
                                ));
                            }
                        }
                        // For tuple and struct variants, we could add tables similar to struct fields
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Process union details
fn process_union_details(output: &mut String, union_: &Union, data: &Crate, level: usize) {
    // Similar to struct details
    let heading_level = std::cmp::min(level, 6);

    // Detail fields
    output.push_str(&format!("{} Fields\n\n", "#".repeat(heading_level)));
    output.push_str("| Name | Type | Documentation |\n");
    output.push_str("|------|------|---------------|\n");

    for field_id in &union_.fields {
        if let Some(field_item) = data.index.get(field_id) {
            if let Some(field_name) = &field_item.name {
                if let ItemEnum::StructField(field_type) = &field_item.inner {
                    let docs = match field_item.docs.as_deref() {
                        Some(d) => d.replace('\n', "<br>"),
                        None => String::new(),
                    };
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        field_name,
                        format_type(field_type, data),
                        docs
                    ));
                }
            }
        }
    }

    if union_.has_stripped_fields {
        output.push_str("| *private fields* | ... | *Some fields have been omitted* |\n");
    }

    output.push('\n');
}

/// Process trait details
fn process_trait_details(output: &mut String, trait_: &Trait, data: &Crate, level: usize) {
    let heading_level = std::cmp::min(level, 6);

    if trait_.is_auto {
        output.push_str("> This is an auto trait.\n\n");
    }
    if trait_.is_unsafe {
        output.push_str("> This trait is unsafe to implement.\n\n");
    }

    // Process trait items, bounds, and implementations
    if !trait_.items.is_empty() {
        output.push_str(&format!(
            "{} Required Methods\n\n",
            "#".repeat(heading_level)
        ));

        for item_id in &trait_.items {
            if let Some(item) = data.index.get(item_id) {
                if let Some(name) = &item.name {
                    match &item.inner {
                        ItemEnum::Function(func) if !func.has_body => {
                            output.push_str(&format!("- `{}`", name));
                            if let Some(docs) = &item.docs {
                                if let Some(first_line) = docs.lines().next() {
                                    if !first_line.trim().is_empty() {
                                        output.push_str(&format!(": {}", first_line));
                                    }
                                }
                            }
                            output.push('\n');
                        }
                        _ => {}
                    }
                }
            }
        }

        output.push('\n');
    }
}

/// Process impl details
fn process_impl_details(output: &mut String, impl_: &Impl, data: &Crate, level: usize) {
    let heading_level = std::cmp::min(level, 6);

    // List items in the impl
    if !impl_.items.is_empty() {
        output.push_str(&format!("{} Methods\n\n", "#".repeat(heading_level)));

        for item_id in &impl_.items {
            if let Some(item) = data.index.get(item_id) {
                if let ItemEnum::Function(_) = &item.inner {
                    if let Some(name) = &item.name {
                        output.push_str(&format!("- `{}`", name));
                        if let Some(docs) = &item.docs {
                            if let Some(first_line) = docs.lines().next() {
                                if !first_line.trim().is_empty() {
                                    output.push_str(&format!(": {}", first_line));
                                }
                            }
                        }
                        output.push('\n');
                    }
                }
            }
        }

        output.push('\n');
    }
}

/// Loads rustdoc JSON from a file and converts it to markdown
pub async fn rustdoc_json_to_markdown<P: AsRef<Path>>(json_path: P) -> Result<String> {
    let generator = MarkdownGenerator::from_file(json_path).await?;
    Ok(generator.generate_markdown())
}
