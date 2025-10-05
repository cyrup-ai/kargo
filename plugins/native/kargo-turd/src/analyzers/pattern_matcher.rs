use regex::Regex;
use lazy_static::lazy_static;
use crate::models::Violation;

// Helper function to compile regex patterns with proper error handling
fn compile_regex(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            eprintln!("FATAL: Failed to compile regex pattern '{pattern}': {e}");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// TIER 1 COMMENT PATTERNS - High confidence stub indicators
// ============================================================================

lazy_static! {
    static ref TIER1_COMMENT_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"(?i)IN A REAL"),
        compile_regex(r"(?i)IN PRODUCTION"),
        compile_regex(r"(?i)IN A PRODUCTION"),
        compile_regex(r"(?i)FOR NOW"),
        compile_regex(r"\bTODO\b"),
        compile_regex(r"\bFIXME\b"),
        compile_regex(r"\bWIP\b"),
        compile_regex(r"(?i)WORK IN PROGRESS"),
        compile_regex(r"(?i)HACK"),
        compile_regex(r"(?i)WOULD REQUIRE"),
        compile_regex(r"(?i)WOULD NEED"),
        compile_regex(r"\bFIX\b"),
        compile_regex(r"(?i)IN PRACTICE"),
        compile_regex(r"(?i)HOPEFUL"),
    ];

    // ========================================================================
    // TIER 2 COMMENT PATTERNS - Possible stub indicators
    // ========================================================================

    static ref TIER2_COMMENT_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"(?i)DUMMY"),
        compile_regex(r"(?i)MOCK"),
        compile_regex(r"(?i)PLACEHOLDER"),
    ];

    // ========================================================================
    // TIER 3 COMMENT PATTERNS - Lower confidence
    // ========================================================================

    static ref TIER3_COMMENT_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"\bblock_on\b"),
        compile_regex(r"\bspawn_blocking\b"),
        compile_regex(r"(?i)actual"),
        compile_regex(r"(?i)legacy"),
        compile_regex(r"(?i)backward compatibility"),
        compile_regex(r"(?i)shim"),
        compile_regex(r"(?i)fallback"),
        compile_regex(r"(?i)fall back"),
    ];
}

/// Find all comment violations for a specific tier
///
/// Returns violations with:
/// - `line_number`: 1-indexed line where match found
/// - `search_term`: The exact text that matched (e.g., "TODO", "FIXME")
/// - `method_name`: Empty string (not applicable for comment patterns)
/// - context: 2 lines before + match line + 2 lines after
#[must_use] 
pub fn find_comment_violations(content: &str, tier: u8) -> Vec<Violation> {
    let patterns = match tier {
        1 => &*TIER1_COMMENT_PATTERNS,
        2 => &*TIER2_COMMENT_PATTERNS,
        3 => &*TIER3_COMMENT_PATTERNS,
        _ => return vec![],
    };

    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in patterns {
            // Find first match on this line (don't report duplicates)
            if let Some(m) = pattern.find(line) {
                violations.push(Violation {
                    line_number: line_num + 1,  // 1-indexed for humans
                    search_term: m.as_str().to_string(),
                    method_name: String::new(),  // Not applicable for comments
                    context: extract_context(content, line_num),
                });
                // Note: If multiple patterns match same line, we'll get multiple violations
                // This is intentional - e.g., "TODO: FIX this hack" has 3 violations
            }
        }
    }

    violations
}

/// Extract context: 2 lines before + violation line + 2 lines after
///
/// Handles edge cases:
/// - Near start of file: fewer than 2 lines before
/// - Near end of file: fewer than 2 lines after
/// - Single-line files: returns just that line
fn extract_context(content: &str, line_num: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // saturating_sub: won't go below 0
    let start = line_num.saturating_sub(2);

    // min: won't exceed file length
    let end = (line_num + 3).min(lines.len());

    lines[start..end].join("\n")
}

// ============================================================================
// METHOD NAMING PATTERNS
// ============================================================================

lazy_static! {
    static ref TIER1_METHOD_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"\w+_stub\("),
        compile_regex(r"stub_\w+\("),
        compile_regex(r"\w+_temp\("),
        compile_regex(r"temp_\w+\("),
        compile_regex(r"\w+_mock\("),
        compile_regex(r"mock_\w+\("),
        compile_regex(r"\w+_dummy\("),
        compile_regex(r"dummy_\w+\("),
        compile_regex(r"\w+_placeholder\("),
        compile_regex(r"placeholder_\w+\("),
        compile_regex(r"\w+_tmp\("),
        compile_regex(r"tmp_\w+\("),
        compile_regex(r"\w+_hack\("),
        compile_regex(r"hack_\w+\("),
    ];

    static ref TIER2_METHOD_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"\w+_quick\("),
        compile_regex(r"quick_\w+\("),
        compile_regex(r"\w+_workaround\("),
        compile_regex(r"workaround_\w+\("),
        compile_regex(r"\w+_fake\("),
        compile_regex(r"fake_\w+\("),
        compile_regex(r"\w+_unimplemented\("),
    ];
}

/// Find method naming violations
///
/// Captures the method name (without parenthesis) in the violation
#[must_use] 
pub fn find_method_naming_violations(content: &str, tier: u8) -> Vec<Violation> {
    let patterns = match tier {
        1 => &*TIER1_METHOD_PATTERNS,
        2 => &*TIER2_METHOD_PATTERNS,
        _ => return vec![],
    };

    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in patterns {
            if let Some(m) = pattern.find(line) {
                // Strip the trailing '(' to get just the method name
                let method_name = m.as_str().trim_end_matches('(').to_string();

                violations.push(Violation {
                    line_number: line_num + 1,
                    search_term: "stubby method name".to_string(),
                    method_name,
                    context: extract_context(content, line_num),
                });
            }
        }
    }

    violations
}

// ============================================================================
// VARIABLE NAMING PATTERNS (Tier 1 only)
// ============================================================================

lazy_static! {
    static ref TIER1_VAR_PATTERNS: Vec<Regex> = vec![
        compile_regex(r"\bstub_\w+"),
        compile_regex(r"\b\w+_stub\b"),
        compile_regex(r"\btemp_\w+"),
        compile_regex(r"\b\w+_temp\b"),
        compile_regex(r"\btmp_\w+"),
        compile_regex(r"\b\w+_tmp\b"),
        compile_regex(r"\bmock_\w+"),
        compile_regex(r"\b\w+_mock\b"),
        compile_regex(r"\bdummy_\w+"),
        compile_regex(r"\b\w+_dummy\b"),
        compile_regex(r"\bfake_\w+"),
        compile_regex(r"\b\w+_fake\b"),
        compile_regex(r"\bhardcoded_\w+"),
        compile_regex(r"\b\w+_hardcoded\b"),
    ];
}

/// Find variable naming violations (Tier 1 only)
///
/// Only checks tier 1 - variable naming is high-confidence stub indicator
#[must_use] 
pub fn find_variable_naming_violations(content: &str, tier: u8) -> Vec<Violation> {
    if tier != 1 {
        return vec![];  // Only tier 1 has variable patterns
    }

    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for pattern in &*TIER1_VAR_PATTERNS {
            if let Some(m) = pattern.find(line) {
                violations.push(Violation {
                    line_number: line_num + 1,
                    search_term: "stubby variable name".to_string(),
                    method_name: m.as_str().to_string(),  // The variable name
                    context: extract_context(content, line_num),
                });
            }
        }
    }

    violations
}

// ============================================================================
// HARDCODED VALUE PATTERNS
// ============================================================================

lazy_static! {
    static ref HARDCODED_URL: Regex = compile_regex(r"https?://");
    static ref HARDCODED_IP: Regex = compile_regex(r"\b(?:\d{1,3}\.){3}\d{1,3}\b");
    static ref HARDCODED_PORT: Regex = compile_regex(r":\d{4,5}\b");
}

/// Find hardcoded values (URLs, IPs, ports)
///
/// These are tier 1 violations (should be in config files)
#[must_use] 
pub fn find_hardcoded_values(content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Check for URLs
        if HARDCODED_URL.is_match(line) {
            // Skip if this is a comment (common in docs)
            if !line.trim_start().starts_with("//") && !line.trim_start().starts_with("///") {
                violations.push(Violation {
                    line_number: line_num + 1,
                    search_term: "hardcoded URL".to_string(),
                    method_name: String::new(),
                    context: extract_context(content, line_num),
                });
            }
        }

        // Check for IP addresses
        if HARDCODED_IP.is_match(line) {
            // Skip common non-problematic IPs
            if !line.contains("127.0.0.1") && !line.contains("0.0.0.0") {
                violations.push(Violation {
                    line_number: line_num + 1,
                    search_term: "hardcoded IP address".to_string(),
                    method_name: String::new(),
                    context: extract_context(content, line_num),
                });
            }
        }
    }

    violations
}
