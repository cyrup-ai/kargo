use regex::Regex;
use std::collections::HashMap;

/// Processor for Cargo command output to make it more LLM-friendly
#[derive(Clone)]
pub struct OutputProcessor {
    /// Regex patterns to identify and transform specific output patterns
    patterns: HashMap<String, Regex>,
    /// Pre-compiled transformations for common patterns
    transformations: HashMap<String, String>,
}

impl OutputProcessor {
    pub fn new() -> anyhow::Result<Self> {
        let mut patterns = HashMap::new();
        let mut transformations = HashMap::new();

        // Add regex patterns for identifying cargo output sections
        patterns.insert(
            "error".to_string(),
            Regex::new(r"(?m)^error(\[E\d+\])?: .*$")
                .map_err(|e| anyhow::anyhow!("Invalid error regex pattern: {}", e))?,
        );

        patterns.insert(
            "warning".to_string(),
            Regex::new(r"(?m)^warning: .*$")
                .map_err(|e| anyhow::anyhow!("Invalid warning regex pattern: {}", e))?,
        );

        patterns.insert(
            "compiler_artifact".to_string(),
            Regex::new(r"(?m)^\s*Compiling .*$")
                .map_err(|e| anyhow::anyhow!("Invalid compiler artifact regex pattern: {}", e))?,
        );

        // Add pattern for test results
        patterns.insert(
            "test_result".to_string(),
            Regex::new(r"(?m)^\s*test .* ... (?:ok|FAILED)$")
                .map_err(|e| anyhow::anyhow!("Invalid test result regex pattern: {}", e))?,
        );

        // Configure transformations for specific output types
        transformations.insert("json_summary".to_string(), "SUMMARY".to_string());

        transformations.insert("compiler_artifact".to_string(), "COMPILING".to_string());

        transformations.insert("test_result".to_string(), "TEST".to_string());

        Ok(Self {
            patterns,
            transformations,
        })
    }

    /// Add a new transformation
    pub fn add_transformation(&mut self, pattern_name: &str, transformation: &str) {
        self.transformations
            .insert(pattern_name.to_string(), transformation.to_string());
    }

    /// Add a new pattern
    pub fn add_pattern(&mut self, pattern_name: &str, pattern: &str) -> anyhow::Result<()> {
        match Regex::new(pattern) {
            Ok(regex) => {
                self.patterns.insert(pattern_name.to_string(), regex);
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Invalid regex pattern: {}", e)),
        }
    }

    /// Process a single line of output
    pub fn process_line(&self, line: &str) -> String {
        // Check if line matches any of our patterns and apply transformations
        for (pattern_name, regex) in &self.patterns {
            if regex.is_match(line) {
                // For errors and warnings, we want to highlight them
                if pattern_name == "error" {
                    return format!("ERROR: {}", line);
                } else if pattern_name == "warning" {
                    return format!("WARNING: {}", line);
                }

                // Apply custom transformations if available for this pattern
                if let Some(transform) = self.transformations.get(pattern_name) {
                    return format!("{}: {}", transform, line);
                }
            }
        }

        // Return the original line if no transformations apply
        line.to_string()
    }

    /// Process the entire command output
    pub fn process_output(&self, output: &str) -> String {
        // Split output into lines and process each line
        let processed_lines: Vec<String> =
            output.lines().map(|line| self.process_line(line)).collect();

        // Join the processed lines back together
        let processed_output = processed_lines.join("\n");

        // Add a summary at the end if output is large
        if processed_output.lines().count() > 20 {
            let mut summary = String::new();
            let errors = self.count_pattern_matches(&processed_output, "error");
            let warnings = self.count_pattern_matches(&processed_output, "warning");

            if errors > 0 {
                summary.push_str(&format!("\n{} error(s) found\n", errors));
            }

            if warnings > 0 {
                summary.push_str(&format!("\n{} warning(s) found\n", warnings));
            }

            // Apply any custom summary transformations
            if let Some(summary_transform) = self.transformations.get("json_summary") {
                if !summary.is_empty() {
                    summary = format!("\n{}: {}", summary_transform, summary.trim());
                }
            }

            if !summary.is_empty() {
                return format!("{}{}", processed_output, summary);
            }
        }

        processed_output
    }

    /// Count matches for a specific pattern
    fn count_pattern_matches(&self, text: &str, pattern_name: &str) -> usize {
        if let Some(regex) = self.patterns.get(pattern_name) {
            regex.find_iter(text).count()
        } else {
            0
        }
    }
}
