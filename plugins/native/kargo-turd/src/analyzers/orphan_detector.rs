use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::ItemMod;
use crate::models::*;
use super::ast_analyzer::AnalysisResult;

// ============================================================================
// CONTEXT EXTRACTION UTILITY
// ============================================================================

/// Extract context: 2 lines before + violation line + 2 lines after
fn extract_context(content: &str, line_num: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = line_num.saturating_sub(2);
    let end = (line_num + 3).min(lines.len());
    lines[start..end].join("\n")
}

// ============================================================================
// MODULE DECLARATION COLLECTOR
// ============================================================================

/// Visitor that collects all `mod` declarations
pub struct ModuleCollector<'a> {
    pub modules: Vec<OrphanedModule>,
    file_content: &'a str,
    file_path: String,
}

impl<'a> ModuleCollector<'a> {
    pub fn new(file_content: &'a str, file_path: String) -> Self {
        Self {
            modules: Vec::new(),
            file_content,
            file_path,
        }
    }
}

impl<'ast, 'a> Visit<'ast> for ModuleCollector<'a> {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let name = node.ident.to_string();

        // Skip test modules (they're expected to be unused in production)
        if name == "tests" {
            visit::visit_item_mod(self, node);
            return;
        }

        // Get line number from span (1-indexed), convert to 0-indexed for context extraction
        let line = node.ident.span().start().line.saturating_sub(1);

        let visibility = match &node.vis {
            syn::Visibility::Public(_) => "pub",
            syn::Visibility::Inherited => "private",
            syn::Visibility::Restricted(_) => "pub(crate)",
        };

        self.modules.push(OrphanedModule {
            name,
            declared_in: self.file_path.clone(),
            line_number: line + 1,  // Convert to 1-indexed
            visibility: visibility.to_string(),
            context: extract_context(self.file_content, line),
        });

        visit::visit_item_mod(self, node);
    }
}

/// Collect all module declarations from a file
pub fn collect_module_declarations(
    content: &str,
    file_path: &str,
) -> anyhow::Result<Vec<OrphanedModule>> {
    let ast = syn::parse_file(content)?;
    let mut collector = ModuleCollector::new(content, file_path.to_string());
    collector.visit_file(&ast);
    Ok(collector.modules)
}

// ============================================================================
// MODULE USAGE COLLECTOR
// ============================================================================

/// Collect all module names used in import statements
///
/// Extracts modules from: use foo::bar; use baz;
pub fn collect_module_uses(content: &str) -> anyhow::Result<HashSet<String>> {
    let ast = syn::parse_file(content)?;
    let mut uses = HashSet::new();

    for item in ast.items {
        if let syn::Item::Use(use_item) = item {
            extract_used_modules(&use_item.tree, &mut uses);
        }
    }

    Ok(uses)
}

/// Recursively extract module names from use tree
///
/// Examples:
/// - use foo::bar::baz; → extracts: foo, bar, baz
/// - use std::{fs, io}; → extracts: std, fs, io
/// - use crate::models::*; → extracts: crate, models
fn extract_used_modules(tree: &syn::UseTree, uses: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            // use foo::bar → extract "foo"
            uses.insert(path.ident.to_string());
            extract_used_modules(&path.tree, uses);
        }
        syn::UseTree::Name(name) => {
            // use foo → extract "foo"
            uses.insert(name.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            // use foo::{bar, baz} → recurse into group
            for item in &group.items {
                extract_used_modules(item, uses);
            }
        }
        syn::UseTree::Glob(_) => {
            // use foo::* → nothing to extract (already got "foo" from Path)
        }
        syn::UseTree::Rename(rename) => {
            // use foo as bar → extract "foo"
            uses.insert(rename.ident.to_string());
        }
    }
}

// ============================================================================
// ORPHAN DETECTOR - Project-level analysis
// ============================================================================

/// Accumulates function definitions and calls across entire project
///
/// Two-phase usage:
/// 1. Call add_file_analysis() for each file
/// 2. Call find_orphaned_methods() to get orphans
pub struct OrphanDetector {
    /// Maps function name to all locations where it's defined
    /// Vec because same name can exist in different modules
    all_function_defs: HashMap<String, Vec<(String, FunctionInfo)>>,

    /// Set of all function names that are called somewhere
    all_function_calls: HashSet<String>,

    /// All module declarations found
    all_module_decls: Vec<OrphanedModule>,

    /// Set of all module names that are used/imported
    all_module_uses: HashSet<String>,
}

impl OrphanDetector {
    pub fn new() -> Self {
        Self {
            all_function_defs: HashMap::new(),
            all_function_calls: HashSet::new(),
            all_module_decls: Vec::new(),
            all_module_uses: HashSet::new(),
        }
    }

    /// Add analysis results from one file
    ///
    /// Called by executor for each file during parallel processing
    pub fn add_file_analysis(&mut self, file_path: &str, result: &AnalysisResult) {
        // Collect function definitions
        for (name, info) in &result.function_defs {
            self.all_function_defs
                .entry(name.clone())
                .or_default()
                .push((file_path.to_string(), info.clone()));
        }

        // Collect function calls
        for call in &result.function_calls {
            self.all_function_calls.insert(call.clone());
        }
    }

    /// Add module information from one file
    ///
    /// Call this alongside add_file_analysis() during file processing
    pub fn add_module_info(
        &mut self,
        decls: Vec<OrphanedModule>,
        uses: HashSet<String>,
    ) {
        self.all_module_decls.extend(decls);
        for use_name in uses {
            self.all_module_uses.insert(use_name);
        }
    }

    /// Find all orphaned methods after all files analyzed
    ///
    /// Returns orphans grouped by source file for task file generation
    pub fn find_orphaned_methods(&self) -> HashMap<String, Vec<OrphanedMethod>> {
        let mut orphans_by_file: HashMap<String, Vec<OrphanedMethod>> = HashMap::new();

        for (func_name, definitions) in &self.all_function_defs {
            // Check if this function is ever called
            if !self.all_function_calls.contains(func_name) {
                // Never called = orphan!
                for (file_path, info) in definitions {
                    orphans_by_file
                        .entry(file_path.clone())
                        .or_default()
                        .push(OrphanedMethod {
                            name: info.name.clone(),
                            file_path: file_path.clone(),
                            line_number: info.line,
                            visibility: info.visibility.clone(),
                            context: info.context.clone(),
                        });
                }
            }
        }

        orphans_by_file
    }

    /// Find all orphaned modules after all files analyzed
    ///
    /// Returns modules that are declared but never imported
    pub fn find_orphaned_modules(&self) -> Vec<OrphanedModule> {
        self.all_module_decls
            .iter()
            .filter(|module| !self.all_module_uses.contains(&module.name))
            .cloned()
            .collect()
    }
}

impl Default for OrphanDetector {
    fn default() -> Self {
        Self::new()
    }
}
