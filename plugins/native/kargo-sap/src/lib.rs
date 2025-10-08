pub mod plugin;

use anyhow::Result;
use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Anthropic API structures
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize, Default)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct FileRelevance {
    files: HashMap<String, f64>,
}

pub struct SapCommand;

impl Default for SapCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl SapCommand {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    async fn smart_list(
        &self,
        path: &Path,
        objective: Option<&String>,
        context: Option<&String>,
        show_all: bool,
    ) -> Result<()> {

        // Print header with context if provided
        if objective.is_some() || context.is_some() {
            println!("🤖 Smart Agent Protocol - Focused Directory Listing");
            if let Some(obj) = objective {
                println!("📎 Objective: {obj}");
            }
            if let Some(ctx) = context {
                println!("📝 Context: {ctx}");
            }
            println!();
        }

        let entries = self.collect_entries(path, show_all)?;
        let filtered = self.filter_entries(entries, objective, context).await;

        // Display results
        self.display_entries(&filtered);

        Ok(())
    }

    fn collect_entries(&self, path: &Path, show_all: bool) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .skip(1)
        // Skip the directory itself
        {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            // Skip hidden files unless --all is specified
            if !show_all && name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata()?;
            entries.push(FileEntry {
                name: name.to_string(),
                path: path.clone(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }

        // Sort directories first, then by name
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(entries)
    }

    async fn analyze_relevance_with_llm(
        &self,
        files: &[FileEntry],
        objective: &str,
        context: Option<&String>,
    ) -> Result<HashMap<String, f64>> {
        // Get API key from environment
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("CLAUDE_API_KEY"))
            .map_err(|_| anyhow::anyhow!("No API key found"))?;

        // Build file list for analysis
        let file_list: Vec<String> = files
            .iter()
            .map(|f| {
                if f.is_dir {
                    format!("{}/", f.name)
                } else {
                    f.name.clone()
                }
            })
            .collect();

        // Construct prompt
        let system_prompt = "You are analyzing file relevance for an AI coding agent. \
            Return ONLY valid JSON with relevance scores (0.0-1.0) for each file. \
            Higher scores mean more relevant to the objective. \
            Format: {\"files\": {\"filename1\": 0.95, \"filename2\": 0.3}}";

        let user_prompt = format!(
            "Objective: {}\n\n{}\n\nFiles:\n{}\n\nReturn relevance scores as JSON.",
            objective,
            context.map(|c| format!("Context: {c}")).unwrap_or_default(),
            file_list.join("\n")
        );

        // Make API request
        let client = reqwest::Client::new();
        let request = AnthropicRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 2048,
            system: Some(system_prompt.to_string()),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_prompt,
            }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("API request failed ({status}): {text}");
        }

        let api_response: AnthropicResponse = response.json().await?;

        // Extract JSON from response text
        let response_text = api_response.content
            .into_iter()
            .find(|c| c.content_type == "text")
            .ok_or_else(|| anyhow::anyhow!("No text content in response"))?
            .text;

        // Parse the JSON response
        let relevance: FileRelevance = serde_json::from_str(&response_text)
            .or_else(|_| {
                // Try to extract JSON from markdown code block
                let json_start = response_text.find('{').unwrap_or(0);
                let json_end = response_text.rfind('}').unwrap_or(response_text.len());
                serde_json::from_str(&response_text[json_start..=json_end])
            })?;

        log::debug!("LLM API usage: {} input, {} output tokens",
            api_response.usage.input_tokens,
            api_response.usage.output_tokens);

        Ok(relevance.files)
    }

    fn apply_basic_filters(&self, entries: Vec<FileEntry>) -> Vec<FileEntry> {
        entries
            .into_iter()
            .filter(|entry| {
                let name_lower = entry.name.to_lowercase();

                // Filter out common build/cache directories
                if entry.is_dir
                    && matches!(
                        name_lower.as_str(),
                        "target" | "node_modules" | ".git" | ".cache" | "dist" | "build"
                    )
                {
                    return false;
                }

                // Filter out OS-specific files
                if matches!(name_lower.as_str(), ".ds_store" | "thumbs.db") {
                    return false;
                }

                // For files, show source code and configs
                if !entry.is_dir {
                    let is_source = name_lower.ends_with(".rs")
                        || name_lower.ends_with(".toml")
                        || name_lower.ends_with(".md")
                        || name_lower.ends_with(".js")
                        || name_lower.ends_with(".ts")
                        || name_lower.ends_with(".py")
                        || name_lower.ends_with(".go")
                        || name_lower.ends_with(".c")
                        || name_lower.ends_with(".h")
                        || name_lower.ends_with(".cpp");

                    let is_config = matches!(
                        name_lower.as_str(),
                        "cargo.toml" | "package.json" | "go.mod" | "makefile" | ".env"
                    );

                    return is_source || is_config;
                }

                true
            })
            .collect()
    }

    async fn filter_entries(
        &self,
        entries: Vec<FileEntry>,
        objective: Option<&String>,
        context: Option<&String>,
    ) -> Vec<FileEntry> {
        // If no objective provided, return all entries (basic filtering only)
        let objective = match objective {
            Some(obj) => obj,
            None => return self.apply_basic_filters(entries),
        };

        // Try LLM analysis first
        match self.analyze_relevance_with_llm(&entries, objective, context).await {
            Ok(scores) => {
                log::info!("Using LLM for relevance analysis");

                // Filter files based on LLM scores (threshold: 0.5)
                let filtered: Vec<FileEntry> = entries
                    .iter()
                    .filter(|entry| {
                        let score = scores.get(&entry.name)
                            .or_else(|| scores.get(&format!("{}/", entry.name)))
                            .copied()
                            .unwrap_or(0.0);

                        score >= 0.5
                    })
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    log::warn!("LLM filtered all files, falling back to basic filtering");
                    return self.apply_basic_filters(entries);
                }

                filtered
            }
            Err(e) => {
                log::warn!("LLM analysis failed ({e}), using basic filtering");
                self.apply_basic_filters(entries)
            }
        }
    }

    fn display_entries(&self, entries: &[FileEntry]) {
        if entries.is_empty() {
            println!("No relevant files found for the given objective.");
            return;
        }

        println!("📁 Relevant files and directories:");
        println!();

        for entry in entries {
            let icon = if entry.is_dir { "📂" } else { "📄" };
            let size_str = if entry.is_dir {
                String::new()
            } else {
                format!(" ({})", format_size(entry.size))
            };

            println!("{} {}{}", icon, entry.name, size_str);
        }

        println!();
        println!("Total: {} items", entries.len());
    }
}

#[derive(Clone)]
struct FileEntry {
    name: String,
    #[allow(dead_code)]
    path: std::path::PathBuf,
    is_dir: bool,
    size: u64,
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub async fn list_directory(
    path: &Path,
    objective: Option<&String>,
    context: Option<&String>,
    show_all: bool,
) -> Result<()> {
    let cmd = SapCommand::new();
    cmd.smart_list(path, objective, context, show_all).await
}
