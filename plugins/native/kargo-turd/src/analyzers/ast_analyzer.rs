use syn::visit::{self, Visit};
use syn::{Expr, ExprMethodCall, ItemFn, Attribute};
use std::collections::{HashMap, HashSet};
use quote::ToTokens;
use proc_macro2::Span;
use crate::models::{PanicPattern, TestInSrc, FunctionInfo};

// ============================================================================
// CONTEXT EXTRACTION UTILITY
// ============================================================================

/// Extract context: 2 lines before + violation line + 2 lines after
///
/// Same function from `pattern_matcher` - could be extracted to utils
fn extract_context(content: &str, line_num: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = line_num.saturating_sub(2);
    let end = (line_num + 3).min(lines.len());
    lines[start..end].join("\n")
}

// ============================================================================
// PANIC PATTERN VISITOR - Detects .unwrap() and .expect() calls
// ============================================================================

/// Visitor that finds all .`unwrap()` and .`expect()` method calls
///
/// Different rules for src/ vs tests/:
/// - src/: Both `unwrap()` and `expect()` are violations (can panic in production)
/// - tests/: Only `unwrap()` is violation (should use `expect()` with messages)
pub struct PanicPatternVisitor<'a> {
    pub unwrap_calls: Vec<PanicPattern>,
    pub expect_calls: Vec<PanicPattern>,
    file_content: &'a str,
    is_test_file: bool,
}

impl<'a> PanicPatternVisitor<'a> {
    #[must_use] 
    pub fn new(file_content: &'a str, is_test_file: bool) -> Self {
        Self {
            unwrap_calls: Vec::new(),
            expect_calls: Vec::new(),
            file_content,
            is_test_file,
        }
    }
}

/// Implement Visit trait to walk AST and find method calls
impl<'ast> Visit<'ast> for PanicPatternVisitor<'_> {
    /// Called for every method call in the AST
    ///
    /// Examples: `foo.unwrap()`, bar.expect("msg"), `baz.unwrap_or_default()`
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();

        // Check for .unwrap() (but not .unwrap_or* variants)
        if method == "unwrap" {
            // Get line number from span (1-indexed), convert to 0-indexed for our context extraction
            let line = node.method.span().start().line.saturating_sub(1);

            let issue = if self.is_test_file {
                "Should use .expect() with descriptive message in tests".to_string()
            } else {
                "Can panic in production code".to_string()
            };

            self.unwrap_calls.push(PanicPattern {
                line_number: line + 1,  // Keep as 1-indexed for humans
                pattern: ".unwrap()".to_string(),
                issue,
                context: extract_context(self.file_content, line),
            });
        }
        // Check for .expect() in production code (ok in tests)
        else if method == "expect" && !self.is_test_file {
            let line = node.method.span().start().line.saturating_sub(1);

            self.expect_calls.push(PanicPattern {
                line_number: line + 1,
                pattern: ".expect()".to_string(),
                issue: "Can panic in production code".to_string(),
                context: extract_context(self.file_content, line),
            });
        }

        // CRITICAL: Must call default implementation to continue traversal
        // Without this, we only visit top-level method calls
        visit::visit_expr_method_call(self, node);
    }
}

// ============================================================================
// TEST ATTRIBUTE VISITOR - Finds tests in src/ directory
// ============================================================================

/// Visitor that finds test attributes in source files
///
/// Detects: #[test], #[`tokio::test`], #[rstest], #[cfg(test)]
pub struct TestAttributeVisitor<'a> {
    pub tests_found: Vec<TestInSrc>,
    file_content: &'a str,
    file_path: String,
}

impl<'a> TestAttributeVisitor<'a> {
    #[must_use] 
    pub fn new(file_content: &'a str, file_path: String) -> Self {
        Self {
            tests_found: Vec::new(),
            file_content,
            file_path,
        }
    }

    /// Check if attributes contain test markers
    fn check_for_test_attribute(&mut self, attrs: &[Attribute], ident_span: Span) {
        for attr in attrs {
            // Convert attribute path to string (e.g., "test", "tokio::test")
            let attr_str = attr.path().to_token_stream().to_string();

            // Check for various test attribute patterns
            let is_test_attr = attr_str == "test"
                || attr_str == "tokio :: test"  // Note: spaces added by ToTokens
                || attr_str == "rstest"
                || (attr_str == "cfg" && self.is_cfg_test(attr));

            if is_test_attr {
                // Get line number from span (1-indexed), convert to 0-indexed
                let line = ident_span.start().line.saturating_sub(1);

                self.tests_found.push(TestInSrc {
                    line_number: line + 1,
                    test_attribute: format!("#[{attr_str}]"),
                    file_path: self.file_path.clone(),
                    context: extract_context(self.file_content, line),
                });
            }
        }
    }

    /// Check if #[cfg(...)] contains "test"
    fn is_cfg_test(&self, attr: &Attribute) -> bool {
        // Parse #[cfg(test)] or #[cfg(any(test, feature = "test"))]
        attr.to_token_stream()
            .to_string()
            .contains("test")
    }
}

impl<'ast> Visit<'ast> for TestAttributeVisitor<'_> {
    /// Check function attributes for test markers
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.check_for_test_attribute(&node.attrs, node.sig.ident.span());
        visit::visit_item_fn(self, node);
    }

    /// Check module attributes for #[cfg(test)]
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.check_for_test_attribute(&node.attrs, node.ident.span());
        visit::visit_item_mod(self, node);
    }
}

// ============================================================================
// FUNCTION COLLECTOR - Collects function definitions for orphan detection
// ============================================================================

/// Collects all function definitions with metadata
pub struct FunctionCollector<'a> {
    pub functions: HashMap<String, FunctionInfo>,
    file_content: &'a str,
}

impl<'a> FunctionCollector<'a> {
    #[must_use] 
    pub fn new(file_content: &'a str) -> Self {
        Self {
            functions: HashMap::new(),
            file_content,
        }
    }
}

impl<'ast> Visit<'ast> for FunctionCollector<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let name = node.sig.ident.to_string();

        // Get line number from span (1-indexed), convert to 0-indexed
        let line = node.sig.ident.span().start().line.saturating_sub(1);

        // Determine visibility
        let visibility = match &node.vis {
            syn::Visibility::Public(_) => "pub",
            syn::Visibility::Restricted(r) => {
                // Check if it's pub(crate), pub(super), etc.
                let path_str = r.path.to_token_stream().to_string();
                if path_str == "crate" {
                    "pub(crate)"
                } else {
                    "pub(restricted)"
                }
            }
            syn::Visibility::Inherited => "private",
        };

        // Skip special functions that shouldn't be flagged as orphans
        if Self::should_skip(&name, &node.attrs) {
            visit::visit_item_fn(self, node);
            return;
        }

        self.functions.insert(name.clone(), FunctionInfo {
            name,
            line: line + 1,
            visibility: visibility.to_string(),
            context: extract_context(self.file_content, line),
        });

        visit::visit_item_fn(self, node);
    }
}

impl FunctionCollector<'_> {
    /// Check if function should be skipped from orphan detection
    fn should_skip(name: &str, attrs: &[Attribute]) -> bool {
        // Skip main() function
        if name == "main" {
            return true;
        }

        // Skip well-known plugin factory exported by native plugins
        if name == "kargo_plugin_create" {
            return true;
        }

        // Skip test functions
        if Self::has_test_attr(attrs) {
            return true;
        }

        // Skip FFI functions
        if Self::has_no_mangle(attrs) {
            return true;
        }

        false
    }

    fn has_test_attr(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path = attr.path().to_token_stream().to_string();
            path == "test" || path == "tokio :: test" || path == "rstest"
        })
    }

    fn has_no_mangle(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path().to_token_stream().to_string() == "no_mangle"
        })
    }
}

// ============================================================================
// FUNCTION CALL COLLECTOR - Tracks which functions are called
// ============================================================================

/// Collects all function calls to identify used functions
pub struct FunctionCallCollector {
    pub calls: HashSet<String>,
}

impl FunctionCallCollector {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            calls: HashSet::new(),
        }
    }
}

impl Default for FunctionCallCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ast> Visit<'ast> for FunctionCallCollector {
    /// Called for every function call (not method call)
    ///
    /// Example: `foo()`, `bar::baz()`
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Extract function name from call expression
        if let Expr::Path(ref path_expr) = *node.func {
            // Simple case: foo()
            if let Some(ident) = path_expr.path.get_ident() {
                self.calls.insert(ident.to_string());
            }
            // Path case: module::foo()
            else if let Some(segment) = path_expr.path.segments.last() {
                self.calls.insert(segment.ident.to_string());
            }
        }

        visit::visit_expr_call(self, node);
    }

    /// Also track method calls (might call functions via methods)
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.calls.insert(node.method.to_string());
        visit::visit_expr_method_call(self, node);
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Result of analyzing a single file
pub struct AnalysisResult {
    pub panic_patterns: Vec<PanicPattern>,
    pub tests_in_src: Vec<TestInSrc>,
    pub function_defs: HashMap<String, FunctionInfo>,
    pub function_calls: HashSet<String>,
}

/// Analyze a Rust source file using AST parsing
///
/// Returns violations found via `syn::visit` traversal
///
/// # Errors
/// Returns error if file content is not valid Rust syntax
pub fn analyze_file(
    content: &str,
    file_path: &str,
    is_test_file: bool,
) -> anyhow::Result<AnalysisResult> {
    // Parse file content into AST
    // This is the expensive operation (~10ms for 1000-line file)
    let ast = syn::parse_file(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse {file_path}: {e}"))?;

    // Run panic pattern visitor
    let mut panic_visitor = PanicPatternVisitor::new(content, is_test_file);
    panic_visitor.visit_file(&ast);

    // Combine unwrap and expect violations
    let mut panic_patterns = panic_visitor.unwrap_calls;
    panic_patterns.extend(panic_visitor.expect_calls);

    // Run test attribute visitor (only for src files)
    let tests_in_src = if is_test_file {
        Vec::new()
    } else {
        let mut test_visitor = TestAttributeVisitor::new(content, file_path.to_string());
        test_visitor.visit_file(&ast);
        test_visitor.tests_found
    };

    // Collect function definitions
    let mut func_collector = FunctionCollector::new(content);
    func_collector.visit_file(&ast);

    // Collect function calls
    let mut call_collector = FunctionCallCollector::new();
    call_collector.visit_file(&ast);

    Ok(AnalysisResult {
        panic_patterns,
        tests_in_src,
        function_defs: func_collector.functions,
        function_calls: call_collector.calls,
    })
}
