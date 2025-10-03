use regex::Regex;
use lazy_static::lazy_static;
use crate::models::Violation;

// ============================================================================
// TIER 1 COMMENT PATTERNS - High confidence stub indicators
// ============================================================================

lazy_static! {
    static ref TIER1_COMMENT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)IN A REAL").expect("Invalid regex pattern: IN A REAL"),
        Regex::new(r"(?i)IN PRODUCTION").expect("Invalid regex pattern: IN PRODUCTION"),
        Regex::new(r"(?i)IN A PRODUCTION").expect("Invalid regex pattern: IN A PRODUCTION"),
        Regex::new(r"(?i)FOR NOW").expect("Invalid regex pattern: FOR NOW"),
        Regex::new(r"\bTODO\b").expect("Invalid regex pattern: TODO"),
        Regex::new(r"\bFIXME\b").expect("Invalid regex pattern: FIXME"),
        Regex::new(r"\bWIP\b").expect("Invalid regex pattern: WIP"),
        Regex::new(r"(?i)WORK IN PROGRESS").expect("Invalid regex pattern: WORK IN PROGRESS"),
        Regex::new(r"(?i)HACK").expect("Invalid regex pattern: HACK"),
        Regex::new(r"(?i)WOULD REQUIRE").expect("Invalid regex pattern: WOULD REQUIRE"),
        Regex::new(r"(?i)WOULD NEED").expect("Invalid regex pattern: WOULD NEED"),
        Regex::new(r"\bFIX\b").expect("Invalid regex pattern: FIX"),
        Regex::new(r"(?i)IN PRACTICE").expect("Invalid regex pattern: IN PRACTICE"),
        Regex::new(r"(?i)HOPEFUL").expect("Invalid regex pattern: HOPEFUL"),
    ];

    // ========================================================================
    // TIER 2 COMMENT PATTERNS - Possible stub indicators
    // ========================================================================

    static ref TIER2_COMMENT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)DUMMY").expect("Invalid regex pattern: DUMMY"),
        Regex::new(r"(?i)MOCK").expect("Invalid regex pattern: MOCK"),
        Regex::new(r"(?i)PLACEHOLDER").expect("Invalid regex pattern: PLACEHOLDER"),
    ];

    // ========================================================================
    // TIER 3 COMMENT PATTERNS - Lower confidence
    // ========================================================================

    static ref TIER3_COMMENT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\bblock_on\b").expect("Invalid regex pattern: block_on"),
        Regex::new(r"\bspawn_blocking\b").expect("Invalid regex pattern: spawn_blocking"),
        Regex::new(r"(?i)actual").expect("Invalid regex pattern: actual"),
        Regex::new(r"(?i)legacy").expect("Invalid regex pattern: legacy"),
        Regex::new(r"(?i)backward compatibility").expect("Invalid regex pattern: backward compatibility"),
        Regex::new(r"(?i)shim").expect("Invalid regex pattern: shim"),
        Regex::new(r"(?i)fallback").expect("Invalid regex pattern: fallback"),
        Regex::new(r"(?i)fall back").expect("Invalid regex pattern: fall back"),
    ];
}

/// Find all comment violations for a specific tier
///
/// Returns violations with:
/// - line_number: 1-indexed line where match found
/// - search_term: The exact text that matched (e.g., "TODO", "FIXME")
/// - method_name: Empty string (not applicable for comment patterns)
/// - context: 2 lines before + match line + 2 lines after
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
        Regex::new(r"\w+_stub\(").expect("Invalid regex pattern: *_stub("),
        Regex::new(r"stub_\w+\(").expect("Invalid regex pattern: stub_*("),
        Regex::new(r"\w+_temp\(").expect("Invalid regex pattern: *_temp("),
        Regex::new(r"temp_\w+\(").expect("Invalid regex pattern: temp_*("),
        Regex::new(r"\w+_mock\(").expect("Invalid regex pattern: *_mock("),
        Regex::new(r"mock_\w+\(").expect("Invalid regex pattern: mock_*("),
        Regex::new(r"\w+_dummy\(").expect("Invalid regex pattern: *_dummy("),
        Regex::new(r"dummy_\w+\(").expect("Invalid regex pattern: dummy_*("),
        Regex::new(r"\w+_placeholder\(").expect("Invalid regex pattern: *_placeholder("),
        Regex::new(r"placeholder_\w+\(").expect("Invalid regex pattern: placeholder_*("),
        Regex::new(r"\w+_tmp\(").expect("Invalid regex pattern: *_tmp("),
        Regex::new(r"tmp_\w+\(").expect("Invalid regex pattern: tmp_*("),
        Regex::new(r"\w+_hack\(").expect("Invalid regex pattern: *_hack("),
        Regex::new(r"hack_\w+\(").expect("Invalid regex pattern: hack_*("),
    ];

    static ref TIER2_METHOD_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\w+_quick\(").expect("Invalid regex pattern: *_quick("),
        Regex::new(r"quick_\w+\(").expect("Invalid regex pattern: quick_*("),
        Regex::new(r"\w+_workaround\(").expect("Invalid regex pattern: *_workaround("),
        Regex::new(r"workaround_\w+\(").expect("Invalid regex pattern: workaround_*("),
        Regex::new(r"\w+_fake\(").expect("Invalid regex pattern: *_fake("),
        Regex::new(r"fake_\w+\(").expect("Invalid regex pattern: fake_*("),
        Regex::new(r"\w+_unimplemented\(").expect("Invalid regex pattern: *_unimplemented("),
    ];
}

/// Find method naming violations
///
/// Captures the method name (without parenthesis) in the violation
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
        Regex::new(r"\bstub_\w+").expect("Invalid regex pattern: stub_*"),
        Regex::new(r"\b\w+_stub\b").expect("Invalid regex pattern: *_stub"),
        Regex::new(r"\btemp_\w+").expect("Invalid regex pattern: temp_*"),
        Regex::new(r"\b\w+_temp\b").expect("Invalid regex pattern: *_temp"),
        Regex::new(r"\btmp_\w+").expect("Invalid regex pattern: tmp_*"),
        Regex::new(r"\b\w+_tmp\b").expect("Invalid regex pattern: *_tmp"),
        Regex::new(r"\bmock_\w+").expect("Invalid regex pattern: mock_*"),
        Regex::new(r"\b\w+_mock\b").expect("Invalid regex pattern: *_mock"),
        Regex::new(r"\bdummy_\w+").expect("Invalid regex pattern: dummy_*"),
        Regex::new(r"\b\w+_dummy\b").expect("Invalid regex pattern: *_dummy"),
        Regex::new(r"\bfake_\w+").expect("Invalid regex pattern: fake_*"),
        Regex::new(r"\b\w+_fake\b").expect("Invalid regex pattern: *_fake"),
        Regex::new(r"\bhardcoded_\w+").expect("Invalid regex pattern: hardcoded_*"),
        Regex::new(r"\b\w+_hardcoded\b").expect("Invalid regex pattern: *_hardcoded"),
    ];
}

/// Find variable naming violations (Tier 1 only)
///
/// Only checks tier 1 - variable naming is high-confidence stub indicator
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
    static ref HARDCODED_URL: Regex = Regex::new(r"https?://").expect("Invalid regex pattern: URL");
    static ref HARDCODED_IP: Regex = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("Invalid regex pattern: IP address");
    static ref HARDCODED_PORT: Regex = Regex::new(r":\d{4,5}\b").expect("Invalid regex pattern: port number");
}

/// Find hardcoded values (URLs, IPs, ports)
///
/// These are tier 1 violations (should be in config files)
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
