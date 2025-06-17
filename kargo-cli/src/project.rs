use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use toml_edit::{DocumentMut, Item};

/// Enhanced project type recognition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Standard binary crate
    Binary(BinaryConfig),
    /// Standard library crate
    Library(LibraryConfig),
    /// Hybrid crate (both bin and lib)
    Hybrid(HybridConfig),
    /// Workspace root
    Workspace(WorkspaceConfig),
    /// Workspace member
    WorkspaceMember(WorkspaceMemberConfig),
    /// Rust script with embedded cargo
    RustScript(RustScriptConfig),
    /// Proc macro crate
    ProcMacro(ProcMacroConfig),
    /// Unknown project type
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryConfig {
    pub name: String,
    pub path: PathBuf,
    pub bin_path: Option<PathBuf>,
    pub has_build_script: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryConfig {
    pub name: String,
    pub path: PathBuf,
    pub lib_path: Option<PathBuf>,
    pub has_build_script: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HybridConfig {
    pub name: String,
    pub path: PathBuf,
    pub bin_path: Option<PathBuf>,
    pub lib_path: Option<PathBuf>,
    pub has_build_script: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub path: PathBuf,
    pub members: Vec<PathBuf>,
    pub default_members: Option<Vec<PathBuf>>,
    pub exclude: Option<Vec<PathBuf>>,
    pub is_virtual: bool,
    pub package_inheritance: HashMap<String, bool>,
    pub dependency_inheritance: HashMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMemberConfig {
    pub name: String,
    pub path: PathBuf,
    pub workspace_root: PathBuf,
    pub inherited_fields: HashMap<String, bool>,
    pub workspace_dependencies: Vec<String>,
    pub project_type: Box<ProjectType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RustScriptConfig {
    pub path: PathBuf,
    pub dependencies: HashMap<String, String>,
    pub cargo_sections: Vec<CargoSection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcMacroConfig {
    pub name: String,
    pub path: PathBuf,
    pub has_build_script: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoSection {
    pub start: usize,
    pub end: usize,
    pub content: String,
}

pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    /// Create a new project analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze a project at the given path
    pub async fn analyze(&self, path: impl AsRef<Path>) -> Result<ProjectType> {
        let path = path.as_ref();

        // First, check if this is a rust script
        if path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(true) = self.is_rust_script(path).await {
                return self.analyze_rust_script(path).await;
            }
        }

        // Check if this is a Cargo.toml file
        let cargo_path = if path.file_name().map_or(false, |name| name == "Cargo.toml") {
            path.to_path_buf()
        } else {
            path.join("Cargo.toml")
        };

        if cargo_path.exists() {
            return self.analyze_cargo_toml(&cargo_path).await;
        }

        Err(anyhow!("No Rust project found at {}", path.display()))
    }

    /// Check if a file is a rust-script with cargo dependencies
    async fn is_rust_script(&self, path: &Path) -> Result<bool> {
        if !path.is_file() {
            return Ok(false);
        }

        // Check file extension
        if path.extension().map_or(true, |ext| ext != "rs") {
            return Ok(false);
        }

        let content = fs::read_to_string(path).await?;

        // Look for cargo section in various formats
        let has_cargo_section = content.contains("```cargo")
            || content.contains("//! ```cargo")
            || content.contains("// ```cargo");

        Ok(has_cargo_section)
    }

    /// Analyze a rust-script file
    async fn analyze_rust_script(&self, path: &Path) -> Result<ProjectType> {
        let content = fs::read_to_string(path).await?;
        let mut dependencies = HashMap::new();
        let mut cargo_sections = Vec::new();

        // Find cargo sections with regex patterns for different formats
        let regex_patterns = [
            r"```cargo\s*\n([\s\S]*?)```",       // Standard format
            r"//!\s*```cargo\s*\n([\s\S]*?)```", // Doc comment format
            r"//\s*```cargo\s*\n([\s\S]*?)```",  // Line comment format
        ];

        for pattern in regex_patterns {
            let regex = regex::Regex::new(pattern)?;

            for captures in regex.captures_iter(&content) {
                if let Some(cargo_match) = captures.get(1) {
                    let cargo_content = cargo_match.as_str();
                    let range = cargo_match.range();

                    // Add to cargo sections
                    cargo_sections.push(CargoSection {
                        start: range.start,
                        end: range.end,
                        content: cargo_content.to_string(),
                    });

                    // Try to parse as TOML to extract dependencies
                    if let Ok(doc) = cargo_content.parse::<DocumentMut>() {
                        if let Some(deps) = doc.get("dependencies") {
                            if let Some(deps_table) = deps.as_table() {
                                for (key, value) in deps_table.iter() {
                                    let version = extract_version_from_toml(value);
                                    if let Some(version) = version {
                                        dependencies.insert(key.to_string(), version);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ProjectType::RustScript(RustScriptConfig {
            path: path.to_path_buf(),
            dependencies,
            cargo_sections,
        }))
    }

    /// Analyze a Cargo.toml file
    async fn analyze_cargo_toml(&self, path: &Path) -> Result<ProjectType> {
        let content = fs::read_to_string(path).await?;
        let document = content.parse::<DocumentMut>()?;

        // Check if this is a workspace
        if document.get("workspace").is_some() {
            return self.analyze_workspace(path, document).await;
        }

        // Determine the project type and configuration
        let is_binary = path
            .parent()
            .map_or(false, |parent| parent.join("src/main.rs").exists());
        let is_library = path
            .parent()
            .map_or(false, |parent| parent.join("src/lib.rs").exists());
        let is_proc_macro = document
            .get("lib")
            .and_then(|lib| lib.get("proc-macro"))
            .and_then(|proc_macro| proc_macro.as_bool())
            == Some(true);

        let name = document
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(|name| name.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing package name in Cargo.toml"))?
            .to_string();

        let has_build_script = path
            .parent()
            .map_or(false, |parent| parent.join("build.rs").exists());

        // Check if this is a workspace member
        let workspace_info = self.extract_workspace_info(path, &document).await;

        // Handle workspace member
        if let Some((workspace_root, inherited_fields, workspace_deps)) = workspace_info {
            // Determine the inner project type
            let inner_type: ProjectType = if is_proc_macro {
                ProjectType::ProcMacro(ProcMacroConfig {
                    name: name.clone(),
                    path: path.to_path_buf(),
                    has_build_script,
                })
            } else if is_binary && is_library {
                ProjectType::Hybrid(HybridConfig {
                    name: name.clone(),
                    path: path.to_path_buf(),
                    bin_path: path.parent().map(|p| p.join("src/main.rs")),
                    lib_path: path.parent().map(|p| p.join("src/lib.rs")),
                    has_build_script,
                })
            } else if is_binary {
                ProjectType::Binary(BinaryConfig {
                    name: name.clone(),
                    path: path.to_path_buf(),
                    bin_path: path.parent().map(|p| p.join("src/main.rs")),
                    has_build_script,
                })
            } else if is_library {
                ProjectType::Library(LibraryConfig {
                    name: name.clone(),
                    path: path.to_path_buf(),
                    lib_path: path.parent().map(|p| p.join("src/lib.rs")),
                    has_build_script,
                })
            } else {
                ProjectType::Unknown
            };

            return Ok(ProjectType::WorkspaceMember(WorkspaceMemberConfig {
                name,
                path: path.to_path_buf(),
                workspace_root,
                inherited_fields,
                workspace_dependencies: workspace_deps,
                project_type: Box::new(inner_type),
            }));
        }

        // Handle standalone project types
        if is_proc_macro {
            Ok(ProjectType::ProcMacro(ProcMacroConfig {
                name,
                path: path.to_path_buf(),
                has_build_script,
            }))
        } else if is_binary && is_library {
            Ok(ProjectType::Hybrid(HybridConfig {
                name,
                path: path.to_path_buf(),
                bin_path: path.parent().map(|p| p.join("src/main.rs")),
                lib_path: path.parent().map(|p| p.join("src/lib.rs")),
                has_build_script,
            }))
        } else if is_binary {
            Ok(ProjectType::Binary(BinaryConfig {
                name,
                path: path.to_path_buf(),
                bin_path: path.parent().map(|p| p.join("src/main.rs")),
                has_build_script,
            }))
        } else if is_library {
            Ok(ProjectType::Library(LibraryConfig {
                name,
                path: path.to_path_buf(),
                lib_path: path.parent().map(|p| p.join("src/lib.rs")),
                has_build_script,
            }))
        } else {
            Ok(ProjectType::Unknown)
        }
    }

    /// Analyze a workspace Cargo.toml
    async fn analyze_workspace(&self, path: &Path, document: DocumentMut) -> Result<ProjectType> {
        let workspace = document
            .get("workspace")
            .ok_or_else(|| anyhow::anyhow!("No [workspace] section found in Cargo.toml"))?;

        // Extract workspace members
        let parent_dir = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cargo.toml has no parent directory"))?;

        let members = match workspace
            .get("members")
            .and_then(|members| members.as_array())
        {
            Some(members) => members
                .iter()
                .filter_map(|m| m.as_str())
                .map(|m| {
                    if m.starts_with("/") {
                        PathBuf::from(m)
                    } else {
                        parent_dir.join(m)
                    }
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        let default_members = match workspace
            .get("default-members")
            .and_then(|members| members.as_array())
        {
            Some(members) => Some(
                members
                    .iter()
                    .filter_map(|m| m.as_str())
                    .map(|m| {
                        if m.starts_with("/") {
                            PathBuf::from(m)
                        } else {
                            parent_dir.join(m)
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            None => None,
        };

        let exclude = match workspace
            .get("exclude")
            .and_then(|members| members.as_array())
        {
            Some(members) => Some(
                members
                    .iter()
                    .filter_map(|m| m.as_str())
                    .map(|m| {
                        if m.starts_with("/") {
                            PathBuf::from(m)
                        } else {
                            parent_dir.join(m)
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            None => None,
        };

        // Check for package section to determine if it's a virtual workspace
        let is_virtual = document.get("package").is_none();

        // Check which fields can be inherited from workspace
        let mut package_inheritance = HashMap::new();
        let workspace_package = workspace.get("package");

        if let Some(workspace_package) = workspace_package {
            // Check common inheritable fields
            let inheritable_fields = [
                "version",
                "authors",
                "description",
                "documentation",
                "readme",
                "homepage",
                "repository",
                "license",
                "edition",
                "rust-version",
            ];

            for field in inheritable_fields {
                package_inheritance
                    .insert(field.to_string(), workspace_package.get(field).is_some());
            }
        }

        // Check for workspace dependencies
        let mut dependency_inheritance = HashMap::new();
        let workspace_dependencies = workspace.get("dependencies");

        if let Some(deps) = workspace_dependencies {
            if let Some(deps_table) = deps.as_table() {
                for (key, _) in deps_table.iter() {
                    dependency_inheritance.insert(key.to_string(), true);
                }
            }
        }

        Ok(ProjectType::Workspace(WorkspaceConfig {
            path: path.to_path_buf(),
            members,
            default_members,
            exclude,
            is_virtual,
            package_inheritance,
            dependency_inheritance,
        }))
    }

    /// Extract workspace information for a member crate
    async fn extract_workspace_info(
        &self,
        path: &Path,
        document: &DocumentMut,
    ) -> Option<(PathBuf, HashMap<String, bool>, Vec<String>)> {
        // Check if this is explicitly a workspace member
        let workspace_path = document
            .get("package")
            .and_then(|package| package.get("workspace"))
            .and_then(|workspace| workspace.as_str());

        let parent_dir = path.parent()?;

        let workspace_root = if let Some(workspace_path) = workspace_path {
            // Explicit workspace path
            if workspace_path.starts_with("/") {
                PathBuf::from(workspace_path)
            } else {
                parent_dir.join(workspace_path)
            }
        } else {
            // Look for Cargo.toml in parent directories
            let mut current = parent_dir;
            loop {
                let potential_workspace = current.join("Cargo.toml");

                // Check if this Cargo.toml exists and has a workspace section
                if potential_workspace.exists() {
                    if let Ok(content) = std::fs::read_to_string(&potential_workspace) {
                        if let Ok(doc) = content.parse::<DocumentMut>() {
                            if doc.get("workspace").is_some() {
                                return Some((potential_workspace, HashMap::new(), Vec::new()));
                            }
                        }
                    }
                }

                // Move to parent directory
                match current.parent() {
                    Some(parent) => current = parent,
                    None => break,
                }
            }

            // No workspace found
            return None;
        };

        // Collect inherited fields
        let mut inherited_fields = HashMap::new();

        // Check for fields using workspace inheritance
        for (key, value) in document.as_table().iter() {
            // Check for fields like version.workspace = true
            if key.contains(".workspace") {
                inherited_fields.insert(key.replace(".workspace", ""), true);
                continue;
            }

            // Check for table entries with workspace = true
            if let Some(table) = value.as_table() {
                if table.get("workspace").and_then(|w| w.as_bool()) == Some(true) {
                    inherited_fields.insert(key.to_string(), true);
                }
            }
        }

        // Workspace dependencies
        let mut workspace_deps = Vec::new();

        // Check dependencies
        if let Some(deps) = document.get("dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (key, value) in deps_table.iter() {
                    if let Some(table) = value.as_table() {
                        if table.get("workspace").is_some() {
                            workspace_deps.push(key.to_string());
                        }
                    }
                }
            }
        }

        // Check dev-dependencies
        if let Some(deps) = document.get("dev-dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (key, value) in deps_table.iter() {
                    if let Some(table) = value.as_table() {
                        if table.get("workspace").is_some() {
                            workspace_deps.push(key.to_string());
                        }
                    }
                }
            }
        }

        // Check build-dependencies
        if let Some(deps) = document.get("build-dependencies") {
            if let Some(deps_table) = deps.as_table() {
                for (key, value) in deps_table.iter() {
                    if let Some(table) = value.as_table() {
                        if table.get("workspace").is_some() {
                            workspace_deps.push(key.to_string());
                        }
                    }
                }
            }
        }

        Some((workspace_root, inherited_fields, workspace_deps))
    }
}

/// Extract version from a TOML value
fn extract_version_from_toml(value: &Item) -> Option<String> {
    match value {
        Item::Value(value) => {
            if let Some(version) = value.as_str() {
                Some(version.to_string())
            } else {
                None
            }
        }
        Item::Table(table) => {
            if let Some(version) = table.get("version") {
                if let Some(version_str) = version.as_str() {
                    Some(version_str.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
