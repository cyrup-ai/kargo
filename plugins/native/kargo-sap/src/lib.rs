use anyhow::Result;
use clap::{Arg, Command};
use kargo_plugin_api::{BoxFuture, ExecutionContext, PluginCommand};
use jwalk::WalkDir;
use std::path::Path;

pub struct SapCommand;

impl SapCommand {
    pub fn new() -> Self {
        Self
    }
}

impl PluginCommand for SapCommand {
    fn clap(&self) -> Command {
        Command::new("sap")
            .about("Smart Agent Protocol - AI-enhanced directory listing for LLM agents")
            .arg(
                Arg::new("path")
                    .help("Path to list (defaults to current directory)")
                    .value_name("PATH")
                    .index(1)
            )
            .arg(
                Arg::new("objective")
                    .long("objective")
                    .short('o')
                    .help("The objective or task the agent is trying to accomplish")
                    .value_name("TEXT")
            )
            .arg(
                Arg::new("context")
                    .long("context")
                    .short('c')
                    .help("Additional context about the current work")
                    .value_name("TEXT")
            )
            .arg(
                Arg::new("all")
                    .long("all")
                    .short('a')
                    .help("Show all files (including hidden)")
                    .action(clap::ArgAction::SetTrue)
            )
    }

    fn run(&self, ctx: ExecutionContext) -> BoxFuture {
        Box::pin(async move {
            let cmd = SapCommand::new();
            cmd.run_async(ctx).await
        })
    }
}

impl SapCommand {
    async fn run_async(&self, ctx: ExecutionContext) -> Result<()> {
        // Parse arguments from the execution context
        let args: Vec<&str> = ctx.matched_args.iter().map(|s| s.as_str()).collect();
        let matches = self.clap().try_get_matches_from(args)?;
        
        let path = matches.get_one::<String>("path")
            .map(|s| s.as_str())
            .unwrap_or(".");
            
        let objective = matches.get_one::<String>("objective");
        let context = matches.get_one::<String>("context");
        let show_all = matches.get_flag("all");
        
        // Run the smart listing
        self.smart_list(path, objective, context, show_all)?;
        
        Ok(())
    }
    
    fn smart_list(
        &self,
        path: &str,
        objective: Option<&String>,
        context: Option<&String>,
        show_all: bool,
    ) -> Result<()> {
        let path = Path::new(path);
        
        // Print header with context if provided
        if objective.is_some() || context.is_some() {
            println!("🤖 Smart Agent Protocol - Focused Directory Listing");
            if let Some(obj) = objective {
                println!("📎 Objective: {}", obj);
            }
            if let Some(ctx) = context {
                println!("📝 Context: {}", ctx);
            }
            println!();
        }
        
        // For now, implement a basic smart filtering
        // In a full implementation, this would use an LLM to analyze relevance
        let entries = self.collect_entries(path, show_all)?;
        let filtered = self.filter_entries(entries, objective, context);
        
        // Display results
        self.display_entries(&filtered);
        
        Ok(())
    }
    
    fn collect_entries(&self, path: &Path, show_all: bool) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        
        for entry in WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .skip(1) // Skip the directory itself
        {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
                
            // Skip hidden files unless --all is specified
            if !show_all && name.starts_with('.') {
                continue;
            }
            
            let metadata = entry.metadata()?;
            entries.push(FileEntry {
                name: name.to_string(),
                path: path.to_path_buf(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }
        
        // Sort directories first, then by name
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        
        Ok(entries)
    }
    
    fn filter_entries(
        &self,
        entries: Vec<FileEntry>,
        objective: Option<&String>,
        _context: Option<&String>,
    ) -> Vec<FileEntry> {
        // Basic smart filtering without LLM
        // In production, this would call an LLM API to analyze relevance
        
        if objective.is_none() {
            return entries;
        }
        
        let _objective = objective.unwrap().to_lowercase();
        
        entries.into_iter().filter(|entry| {
            let name_lower = entry.name.to_lowercase();
            
            // Filter out common build/cache directories
            if entry.is_dir && matches!(name_lower.as_str(), "target" | "node_modules" | ".git" | ".cache") {
                return false;
            }
            
            // Filter out OS-specific files
            if matches!(name_lower.as_str(), ".ds_store" | "thumbs.db") {
                return false;
            }
            
            // Basic relevance check - in real implementation, LLM would determine this
            // For now, show source files and important configs
            if !entry.is_dir {
                let is_source = name_lower.ends_with(".rs") || 
                               name_lower.ends_with(".toml") ||
                               name_lower.ends_with(".md");
                               
                let is_config = matches!(name_lower.as_str(), "cargo.toml" | "config.toml" | ".env");
                
                return is_source || is_config;
            }
            
            true
        }).collect()
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

// Plugin registration
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
#[allow(unsafe_code)]
pub extern "C" fn kargo_plugin_create() -> Box<dyn PluginCommand> {
    Box::new(SapCommand::new())
}