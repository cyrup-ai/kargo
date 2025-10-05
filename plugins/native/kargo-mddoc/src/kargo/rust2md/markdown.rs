//! A module for converting rustdoc JSON into human-friendly Markdown documentation.

use anyhow::{Context, Result};
use rustdoc_types::{Attribute, Crate, Id, Item, ItemEnum, ReprKind, StructKind, VariantKind, Visibility};
use rustdoc_types::{Enum, Impl, Module, Struct, Trait, Type, Union};
use rustdoc_types::{GenericParamDefKind, Generics};
use rustdoc_types::{GenericArg, GenericArgs};
use rustdoc_types::{AssocItemConstraintKind, Term};
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
            output.push_str(&format!("**Version:** {version}\n\n"));
        }

        output.push_str(&format!(
            "**Format Version:** {}\n\n",
            self.crate_data.format_version
        ));

        // Process the root module to start
        let root_id = self.crate_data.root;
        if let Some(root_item) = self.crate_data.index.get(&root_id) {
            if let ItemEnum::Module(module) = &root_item.inner {
                if let Some(name) = &root_item.name {
                    output.push_str(&format!("# Module `{name}`\n\n"));
                } else if module.is_crate {
                    output.push_str("# Crate Root\n\n");
                }

                // Add root documentation if available
                if let Some(docs) = &root_item.docs {
                    output.push_str(&format!("{docs}\n\n"));
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

/// Format an `AttributeRepr` into clean repr syntax
fn format_repr(repr: &rustdoc_types::AttributeRepr) -> String {
    let mut parts = Vec::new();

    // Add the kind (C, transparent, etc.)
    match repr.kind {
        ReprKind::Rust => parts.push("Rust".to_string()),
        ReprKind::C => parts.push("C".to_string()),
        ReprKind::Transparent => parts.push("transparent".to_string()),
        ReprKind::Simd => parts.push("simd".to_string()),
    }

    // Add align if present
    if let Some(align) = repr.align {
        parts.push(format!("align({align})"));
    }

    // Add packed if present
    if let Some(packed) = repr.packed {
        if packed == 1 {
            parts.push("packed".to_string());
        } else {
            parts.push(format!("packed({packed})"));
        }
    }

    // Add int discriminant if present (for enums)
    if let Some(int) = &repr.int {
        parts.push(int.clone());
    }

    parts.join(", ")
}

/// Format an attribute for documentation
fn format_attribute(attr: &Attribute) -> String {
    match attr {
        Attribute::NonExhaustive => "#[non_exhaustive]".to_string(),
        Attribute::MustUse { reason } => {
            if let Some(r) = reason {
                format!("#[must_use = \"{r}\"]")
            } else {
                "#[must_use]".to_string()
            }
        }
        Attribute::MacroExport => "#[macro_export]".to_string(),
        Attribute::ExportName(name) => format!("#[export_name = \"{name}\"]"),
        Attribute::LinkSection(section) => format!("#[link_section = \"{section}\"]"),
        Attribute::AutomaticallyDerived => "#[automatically_derived]".to_string(),
        Attribute::Repr(repr) => format!("#[repr({})]", format_repr(repr)),
        Attribute::NoMangle => "#[no_mangle]".to_string(),
        Attribute::TargetFeature { enable } => {
            let features = enable.iter()
                .map(|f| format!("enable = \"{f}\""))
                .collect::<Vec<_>>()
                .join(", ");
            format!("#[target_feature({features})]")
        }
        Attribute::Other(s) => format!("#[{s}]"),
    }
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
                ItemEnum::Module(_) => grouped.modules.push(*id),
                ItemEnum::Struct(_)
                | ItemEnum::Enum(_)
                | ItemEnum::Union(_)
                | ItemEnum::TypeAlias(_) => grouped.types.push(*id),
                ItemEnum::Trait(_) | ItemEnum::TraitAlias(_) => grouped.traits.push(*id),
                ItemEnum::Function(_) => grouped.functions.push(*id),
                ItemEnum::Constant { .. } | ItemEnum::Static(_) => {
                    grouped.constants.push(*id);
                }
                ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => grouped.macros.push(*id),
                ItemEnum::Use(_) => grouped.reexports.push(*id),
                _ => grouped.other_items.push(*id),
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
            if name == source_name {
                output.push_str(&format!("{heading} Re-export `{name}`\n\n"));
            } else {
                output.push_str(&format!(
                    "{heading} Re-export `{source_name}` as `{name}`\n\n"
                ));
            }
        } else {
            output.push_str(&format!("{heading} Re-export `{source_name}`\n\n"));
        }
    } else {
        // Handle named items (mod, struct, enum, trait, etc.)
        if let Some(name) = &item.name {
            match &item.inner {
                ItemEnum::Module(_) => {
                    // For modules, use "##" to make them more prominent
                    output.push_str(&format!("## Module `{name}`\n\n"));
                }
                ItemEnum::Struct(_) => {
                    output.push_str(&format!("{heading} Struct `{name}`\n\n"));
                }
                ItemEnum::Enum(_) => output.push_str(&format!("{heading} Enum `{name}`\n\n")),
                ItemEnum::Union(_) => output.push_str(&format!("{heading} Union `{name}`\n\n")),
                ItemEnum::Trait(_) => output.push_str(&format!("{heading} Trait `{name}`\n\n")),
                ItemEnum::TraitAlias(_) => {
                    output.push_str(&format!("{heading} Trait Alias `{name}`\n\n"));
                }
                ItemEnum::Function(_) => {
                    output.push_str(&format!("{heading} Function `{name}`\n\n"));
                }
                ItemEnum::TypeAlias(_) => {
                    output.push_str(&format!("{heading} Type Alias `{name}`\n\n"));
                }
                ItemEnum::Constant { .. } => {
                    output.push_str(&format!("{heading} Constant `{name}`\n\n"));
                }
                ItemEnum::Static(_) => {
                    output.push_str(&format!("{heading} Static `{name}`\n\n"));
                }
                ItemEnum::Macro(_) => output.push_str(&format!("{heading} Macro `{name}`\n\n")),
                ItemEnum::ProcMacro(_) => {
                    output.push_str(&format!("{heading} Procedural Macro `{name}`\n\n"));
                }
                ItemEnum::ExternCrate {
                    name: crate_name, ..
                } => output.push_str(&format!("{heading} Extern Crate `{crate_name}`\n\n")),
                // For everything else with a name
                _ => output.push_str(&format!("{heading} `{name}`\n\n")),
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
                _ => output.push_str(&format!("{heading} Unnamed Item\n\n")),
            }
        }
    }

    // Add item attributes if present
    if !item.attrs.is_empty() {
        output.push_str("**Attributes:**\n\n");
        for attr in &item.attrs {
            output.push_str(&format!("- `{}`\n", format_attribute(attr)));
        }
        output.push('\n');
    }

    // Add deprecation info if present
    if let Some(deprecation) = &item.deprecation {
        output.push_str("**⚠️ Deprecated");
        if let Some(since) = &deprecation.since {
            output.push_str(&format!(" since {since}"));
        }
        output.push_str("**");
        if let Some(note) = &deprecation.note {
            output.push_str(&format!(": {note}"));
        }
        output.push_str("\n\n");
    }

    // Add documentation if available
    if let Some(docs) = &item.docs {
        output.push_str(&format!("{docs}\n\n"));
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
        Visibility::Restricted { path, .. } => output.push_str(&format!("pub(in {path}) ")),
        Visibility::Default => {}
    }

    match &item.inner {
        // For modules
        ItemEnum::Module(_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("mod {name} {{ /* ... */ }}"));
            }
        }
        // For structs
        ItemEnum::Struct(struct_) => {
            if let Some(name) = &item.name {
                output.push_str(&format!("struct {name}"));
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
                                                output.push_str(&format!("pub(in {path}) "));
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
                                                output.push_str(&format!("    pub(in {path}) "));
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

                output.push_str(&format!("fn {name}"));
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

/// Format trait bounds for generic parameters
fn format_trait_bounds(output: &mut String, bounds: &[rustdoc_types::GenericBound], data: &Crate) {
    for (i, bound) in bounds.iter().enumerate() {
        match bound {
            rustdoc_types::GenericBound::TraitBound {
                trait_,
                generic_params,
                modifier,
            } => {
                match modifier {
                    rustdoc_types::TraitBoundModifier::None => {}
                    rustdoc_types::TraitBoundModifier::Maybe => output.push('?'),
                    rustdoc_types::TraitBoundModifier::MaybeConst => output.push_str("~const "),
                }

                if !generic_params.is_empty() {
                    output.push_str("for<");
                    for (j, param) in generic_params.iter().enumerate() {
                        match &param.kind {
                            rustdoc_types::GenericParamDefKind::Lifetime { .. } => {
                                output.push_str(&format!("'{}", param.name));
                            }
                            _ => output.push_str(&param.name),
                        }

                        if j < generic_params.len() - 1 {
                            output.push_str(", ");
                        }
                    }
                    output.push_str("> ");
                }

                output.push_str(&trait_.path);
                if let Some(args) = &trait_.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, args, data);
                    output.push_str(&args_str);
                }
            }
            rustdoc_types::GenericBound::Outlives(lifetime) => {
                output.push_str(&format!("'{lifetime}"));
            }
            // Handle other bound types if needed
            _ => output.push_str("/* unsupported bound */"),
        }

        if i < bounds.len() - 1 {
            output.push_str(" + ");
        }
    }
}

/// Format ABI for functions
fn format_abi(output: &mut String, abi: &rustdoc_types::Abi) {
    match abi {
        rustdoc_types::Abi::Rust => {}
        rustdoc_types::Abi::C { unwind } => {
            if *unwind {
                output.push_str("extern \"C-unwind\" ");
            } else {
                output.push_str("extern \"C\" ");
            }
        }
        rustdoc_types::Abi::Cdecl { unwind } => {
            if *unwind {
                output.push_str("extern \"cdecl-unwind\" ");
            } else {
                output.push_str("extern \"cdecl\" ");
            }
        }
        rustdoc_types::Abi::Stdcall { unwind } => {
            if *unwind {
                output.push_str("extern \"stdcall-unwind\" ");
            } else {
                output.push_str("extern \"stdcall\" ");
            }
        }
        rustdoc_types::Abi::Fastcall { unwind } => {
            if *unwind {
                output.push_str("extern \"fastcall-unwind\" ");
            } else {
                output.push_str("extern \"fastcall\" ");
            }
        }
        rustdoc_types::Abi::Aapcs { unwind } => {
            if *unwind {
                output.push_str("extern \"aapcs-unwind\" ");
            } else {
                output.push_str("extern \"aapcs\" ");
            }
        }
        rustdoc_types::Abi::Win64 { unwind } => {
            if *unwind {
                output.push_str("extern \"win64-unwind\" ");
            } else {
                output.push_str("extern \"win64\" ");
            }
        }
        rustdoc_types::Abi::SysV64 { unwind } => {
            if *unwind {
                output.push_str("extern \"sysv64-unwind\" ");
            } else {
                output.push_str("extern \"sysv64\" ");
            }
        }
        rustdoc_types::Abi::System { unwind } => {
            if *unwind {
                output.push_str("extern \"system-unwind\" ");
            } else {
                output.push_str("extern \"system\" ");
            }
        }
        rustdoc_types::Abi::Other(abi) => {
            output.push_str(&format!("extern \"{abi}\" "));
        }
    }
}

/// Format generic arguments for a type
fn format_generic_args(output: &mut String, args: &GenericArgs, data: &Crate) {
    match args {
        GenericArgs::AngleBracketed { args, constraints } => {
            if args.is_empty() && constraints.is_empty() {
                return;
            }

            output.push('<');

            // Format args
            for (i, arg) in args.iter().enumerate() {
                match arg {
                    GenericArg::Lifetime(lifetime) => output.push_str(&format!("'{lifetime}")),
                    GenericArg::Type(type_) => output.push_str(&format_type(type_, data)),
                    GenericArg::Const(constant) => output.push_str(&constant.expr),
                    GenericArg::Infer => output.push('_'),
                }

                if i < args.len() - 1 || !constraints.is_empty() {
                    output.push_str(", ");
                }
            }

            // Format constraints (previously called bindings)
            for (i, constraint) in constraints.iter().enumerate() {
                output.push_str(&constraint.name);

                // Format constraint args if present
                if let Some(args) = &constraint.args {
                    let mut args_str = String::new();
                    format_generic_args(&mut args_str, args, data);
                    if !args_str.is_empty() && args_str != "<>" {
                        output.push_str(&args_str);
                    }
                }

                // In newer rustdoc-types, AssocItemConstraint has name, args, and binding
                // The binding is now an enum with Equality and Constraint variants
                match &constraint.binding {
                    AssocItemConstraintKind::Equality(term) => {
                        output.push_str(" = ");
                        match term {
                            Term::Type(type_) => output.push_str(&format_type(type_, data)),
                            Term::Constant(constant) => output.push_str(&constant.expr),
                        }
                    }
                    AssocItemConstraintKind::Constraint(bounds) => {
                        output.push_str(": ");
                        format_trait_bounds(output, bounds, data);
                    }
                }

                if i < constraints.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push('>');
        }
        GenericArgs::Parenthesized {
            inputs,
            output: output_type,
        } => {
            output.push('(');

            for (i, input) in inputs.iter().enumerate() {
                output.push_str(&format_type(input, data));
                if i < inputs.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push(')');

            if let Some(output_ty) = output_type {
                output.push_str(&format!(" -> {}", format_type(output_ty, data)));
            }
        }
        _ => {
            output.push_str("/* unsupported generic args */");
        }
    }
}

/// Format type for display
fn format_type(ty: &Type, data: &Crate) -> String {
    match ty {
        Type::ResolvedPath(path) => {
            let mut result = path.path.clone();
            if let Some(args) = &path.args {
                format_generic_args(&mut result, args, data);
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
                result.push_str(&format!("'{lt} "));
            }
            if *is_mutable {
                result.push_str("mut ");
            }
            result.push_str(&format_type(type_, data));
            result
        }
        Type::DynTrait(dyn_trait) => {
            let mut result = String::from("dyn ");
            
            for (i, trait_) in dyn_trait.traits.iter().enumerate() {
                // Higher-rank bounds if necessary
                if !trait_.generic_params.is_empty() {
                    result.push_str("for<");
                    for (j, param) in trait_.generic_params.iter().enumerate() {
                        match &param.kind {
                            rustdoc_types::GenericParamDefKind::Lifetime { .. } => {
                                result.push_str(&format!("'{}", param.name));
                            }
                            _ => result.push_str(&param.name),
                        }
                        
                        if j < trait_.generic_params.len() - 1 {
                            result.push_str(", ");
                        }
                    }
                    result.push_str("> ");
                }
                
                result.push_str(&trait_.trait_.path);
                if let Some(args) = &trait_.trait_.args {
                    format_generic_args(&mut result, args, data);
                }
                
                if i < dyn_trait.traits.len() - 1 {
                    result.push_str(" + ");
                }
            }
            
            // Lifetime bound if present
            if let Some(lifetime) = &dyn_trait.lifetime {
                result.push_str(&format!(" + '{lifetime}"));
            }
            
            result
        }
        Type::FunctionPointer(fn_ptr) => {
            let mut result = String::new();
            
            // For clarity about the parameters
            if !fn_ptr.generic_params.is_empty() {
                result.push_str("for<");
                for (j, param) in fn_ptr.generic_params.iter().enumerate() {
                    match &param.kind {
                        rustdoc_types::GenericParamDefKind::Lifetime { .. } => {
                            result.push_str(&format!("'{}", param.name));
                        }
                        _ => result.push_str(&param.name),
                    }
                    
                    if j < fn_ptr.generic_params.len() - 1 {
                        result.push_str(", ");
                    }
                }
                result.push_str("> ");
            }
            
            // Function header (const, unsafe, extern, etc.)
            if fn_ptr.header.is_const {
                result.push_str("const ");
            }
            if fn_ptr.header.is_unsafe {
                result.push_str("unsafe ");
            }
            
            // ABI
            format_abi(&mut result, &fn_ptr.header.abi);
            
            result.push_str("fn(");
            
            // Parameters
            for (i, (_, param_type)) in fn_ptr.sig.inputs.iter().enumerate() {
                result.push_str(&format_type(param_type, data));
                if i < fn_ptr.sig.inputs.len() - 1 || fn_ptr.sig.is_c_variadic {
                    result.push_str(", ");
                }
            }
            
            // Variadic
            if fn_ptr.sig.is_c_variadic {
                result.push_str("...");
            }
            
            result.push(')');
            
            // Return type
            if let Some(return_type) = &fn_ptr.sig.output {
                result.push_str(&format!(" -> {}", format_type(return_type, data)));
            }
            
            result
        }
        Type::ImplTrait(bounds) => {
            let mut result = String::from("impl ");
            format_trait_bounds(&mut result, bounds, data);
            result
        }
        Type::Infer => "_".to_string(),
        Type::RawPointer { is_mutable, type_ } => {
            let mut result = if *is_mutable {
                String::from("*mut ")
            } else {
                String::from("*const ")
            };
            result.push_str(&format_type(type_, data));
            result
        }
        Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => {
            let mut result = String::from("<");
            result.push_str(&format_type(self_type, data));
            
            if let Some(trait_path) = trait_ {
                result.push_str(&format!(" as {}", trait_path.path));
                if let Some(trait_args) = &trait_path.args {
                    format_generic_args(&mut result, trait_args, data);
                }
            }
            
            result.push_str(&format!(">::{name}"));
            
            if let Some(args) = args {
                let mut args_str = String::new();
                format_generic_args(&mut args_str, args, data);
                if !args_str.is_empty() && args_str != "<>" {
                    result.push_str(&args_str);
                }
            }
            
            result
        }
        _ => {
            // Unknown or unsupported type variant
            "/* unsupported type */".to_string()
        }
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
                    output.push_str(&format!("| {i} | `private` | *Private field* |\n"));
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
                    output.push_str(&format!("{docs}\n\n"));
                }

                if let ItemEnum::Variant(variant) = &variant_item.inner {
                    if variant.kind == VariantKind::Plain {
                        if let Some(discriminant) = &variant.discriminant {
                            output.push_str(&format!(
                                "Discriminant: `{}`\n\n",
                                discriminant.expr
                            ));
                        }
                    }
                    // For tuple and struct variants, we could add tables similar to struct fields
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
                            output.push_str(&format!("- `{name}`"));
                            if let Some(docs) = &item.docs {
                                if let Some(first_line) = docs.lines().next() {
                                    if !first_line.trim().is_empty() {
                                        output.push_str(&format!(": {first_line}"));
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
                        output.push_str(&format!("- `{name}`"));
                        if let Some(docs) = &item.docs {
                            if let Some(first_line) = docs.lines().next() {
                                if !first_line.trim().is_empty() {
                                    output.push_str(&format!(": {first_line}"));
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
