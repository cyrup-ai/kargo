//! Multi-page markdown generator with proper interlinking and lint-valid output.

use crate::error::Error;
use crate::utils;
use log::{debug, info};
use rustdoc_types::{Crate, Enum, Item, ItemEnum, Module, Struct, Trait};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration for multi-page markdown generation
#[derive(Debug, Clone)]
pub struct MultipageConfig {
    /// Output directory for all markdown files
    pub output_dir: PathBuf,
    /// Base URL for cross-references (e.g., "docs/tokio")
    pub base_url: String,
    /// Whether to generate a table of contents index
    pub generate_index: bool,
    /// Maximum items per page before splitting
    pub max_items_per_page: usize,
}

impl Default for MultipageConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("docs"),
            base_url: String::new(),
            generate_index: true,
            max_items_per_page: 50,
        }
    }
}

/// Multi-page markdown generator
pub struct MultipageGenerator {
    crate_data: Crate,
    config: MultipageConfig,
}

impl MultipageGenerator {
    pub fn new(crate_data: Crate, config: MultipageConfig) -> Self {
        Self { crate_data, config }
    }

    /// Generate all markdown documentation pages
    pub fn generate_all(&mut self) -> Result<Vec<PathBuf>, Error> {
        info!("Generating multi-page markdown documentation");

        // Ensure output directory exists
        std::fs::create_dir_all(&self.config.output_dir).map_err(|e| Error::Io(e))?;

        let mut generated_files = Vec::new();

        // Generate main index page
        if self.config.generate_index {
            let index_path = self.generate_index_page()?;
            generated_files.push(index_path);
        }

        // Generate pages by category
        let module_files = self.generate_modules_page()?;
        generated_files.extend(module_files);

        let struct_files = self.generate_structs_page()?;
        generated_files.extend(struct_files);

        let trait_files = self.generate_traits_page()?;
        generated_files.extend(trait_files);

        let enum_files = self.generate_enums_page()?;
        generated_files.extend(enum_files);

        let function_files = self.generate_functions_page()?;
        generated_files.extend(function_files);

        info!("Generated {} markdown files", generated_files.len());
        Ok(generated_files)
    }

    /// Generate the main index page
    fn generate_index_page(&self) -> Result<PathBuf, Error> {
        debug!("Generating index page");

        let mut content = String::new();

        // Lint-valid markdown header
        let crate_name = self
            .crate_data
            .index
            .get(&self.crate_data.root)
            .and_then(|item| item.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Crate");

        content.push_str(&format!("# {} Documentation\n\n", crate_name));

        if let Some(version) = &self.crate_data.crate_version {
            content.push_str(&format!("**Version:** {}\n\n", version));
        }

        // Add crate-level documentation
        if let Some(root_item) = self.crate_data.index.get(&self.crate_data.root) {
            if let Some(docs) = &root_item.docs {
                content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
            }
        }

        // Generate table of contents
        content.push_str("## Table of Contents\n\n");

        // Count items by type
        let mut counts = HashMap::new();
        for item in self.crate_data.index.values() {
            let item_type = match &item.inner {
                ItemEnum::Module(_) => "Modules",
                ItemEnum::Struct(_) => "Structs",
                ItemEnum::Trait(_) => "Traits",
                ItemEnum::Enum(_) => "Enums",
                ItemEnum::Function(_) => "Functions",
                ItemEnum::Constant { .. } => "Constants",
                _ => continue,
            };
            *counts.entry(item_type).or_insert(0) += 1;
        }

        // Generate TOC with links to category pages
        if counts.get("Modules").unwrap_or(&0) > &0 {
            content.push_str(&format!(
                "- [Modules](modules.md) ({} items)\n",
                counts.get("Modules").unwrap_or(&0)
            ));
        }
        if counts.get("Structs").unwrap_or(&0) > &0 {
            content.push_str(&format!(
                "- [Structs](structs.md) ({} items)\n",
                counts.get("Structs").unwrap_or(&0)
            ));
        }
        if counts.get("Traits").unwrap_or(&0) > &0 {
            content.push_str(&format!(
                "- [Traits](traits.md) ({} items)\n",
                counts.get("Traits").unwrap_or(&0)
            ));
        }
        if counts.get("Enums").unwrap_or(&0) > &0 {
            content.push_str(&format!(
                "- [Enums](enums.md) ({} items)\n",
                counts.get("Enums").unwrap_or(&0)
            ));
        }
        if counts.get("Functions").unwrap_or(&0) > &0 {
            content.push_str(&format!(
                "- [Functions](functions.md) ({} items)\n",
                counts.get("Functions").unwrap_or(&0)
            ));
        }

        content.push('\n');

        let index_path = self.config.output_dir.join("README.md");
        utils::write_file(&index_path, &content)?;

        Ok(index_path)
    }

    /// Generate modules page
    fn generate_modules_page(&self) -> Result<Vec<PathBuf>, Error> {
        let mut content = String::new();
        content.push_str("# Modules\n\n");

        let mut modules = Vec::new();
        for (id, item) in &self.crate_data.index {
            if let ItemEnum::Module(_) = &item.inner {
                if let Some(name) = &item.name {
                    modules.push((id, item, name));
                }
            }
        }

        modules.sort_by(|a, b| a.2.cmp(b.2));

        for (_id, item, name) in modules {
            content.push_str(&format!("## `{}`\n\n", name));

            if let Some(docs) = &item.docs {
                let brief = self.extract_brief_docs(docs);
                content.push_str(&format!("{}\n\n", brief));
            }

            // Generate link to detailed page
            let detailed_link = format!("module_{}.md", self.sanitize_filename(name));
            content.push_str(&format!(
                "[View detailed documentation]({})\n\n",
                detailed_link
            ));

            // Generate detailed page for this module
            if let ItemEnum::Module(module) = &item.inner {
                self.generate_detailed_module_page(module, name)?;
            }
        }

        let modules_path = self.config.output_dir.join("modules.md");
        utils::write_file(&modules_path, &content)?;

        Ok(vec![modules_path])
    }

    /// Generate structs page
    fn generate_structs_page(&self) -> Result<Vec<PathBuf>, Error> {
        let mut content = String::new();
        content.push_str("# Structs\n\n");

        let mut structs = Vec::new();
        for (id, item) in &self.crate_data.index {
            if let ItemEnum::Struct(_) = &item.inner {
                if let Some(name) = &item.name {
                    structs.push((id, item, name));
                }
            }
        }

        structs.sort_by(|a, b| a.2.cmp(b.2));

        for (_id, item, name) in structs {
            content.push_str(&format!("## `{}`\n\n", name));

            if let Some(docs) = &item.docs {
                let brief = self.extract_brief_docs(docs);
                content.push_str(&format!("{}\n\n", brief));
            }

            // Generate link to detailed page
            let detailed_link = format!("struct_{}.md", self.sanitize_filename(name));
            content.push_str(&format!(
                "[View detailed documentation]({})\n\n",
                detailed_link
            ));

            // Generate detailed page for this struct
            if let ItemEnum::Struct(struct_item) = &item.inner {
                self.generate_detailed_struct_page(struct_item, name, item)?;
            }
        }

        let structs_path = self.config.output_dir.join("structs.md");
        utils::write_file(&structs_path, &content)?;

        Ok(vec![structs_path])
    }

    /// Generate traits page
    fn generate_traits_page(&self) -> Result<Vec<PathBuf>, Error> {
        let mut content = String::new();
        content.push_str("# Traits\n\n");

        let mut traits = Vec::new();
        for (id, item) in &self.crate_data.index {
            if let ItemEnum::Trait(_) = &item.inner {
                if let Some(name) = &item.name {
                    traits.push((id, item, name));
                }
            }
        }

        traits.sort_by(|a, b| a.2.cmp(b.2));

        for (_id, item, name) in traits {
            content.push_str(&format!("## `{}`\n\n", name));

            if let Some(docs) = &item.docs {
                let brief = self.extract_brief_docs(docs);
                content.push_str(&format!("{}\n\n", brief));
            }

            // Generate link to detailed page
            let detailed_link = format!("trait_{}.md", self.sanitize_filename(name));
            content.push_str(&format!(
                "[View detailed documentation]({})\n\n",
                detailed_link
            ));

            // Generate detailed page for this trait
            if let ItemEnum::Trait(trait_item) = &item.inner {
                self.generate_detailed_trait_page(trait_item, name, item)?;
            }
        }

        let traits_path = self.config.output_dir.join("traits.md");
        utils::write_file(&traits_path, &content)?;

        Ok(vec![traits_path])
    }

    /// Generate enums page
    fn generate_enums_page(&self) -> Result<Vec<PathBuf>, Error> {
        let mut content = String::new();
        content.push_str("# Enums\n\n");

        let mut enums = Vec::new();
        for (id, item) in &self.crate_data.index {
            if let ItemEnum::Enum(_) = &item.inner {
                if let Some(name) = &item.name {
                    enums.push((id, item, name));
                }
            }
        }

        enums.sort_by(|a, b| a.2.cmp(b.2));

        for (_id, item, name) in enums {
            content.push_str(&format!("## `{}`\n\n", name));

            if let Some(docs) = &item.docs {
                let brief = self.extract_brief_docs(docs);
                content.push_str(&format!("{}\n\n", brief));
            }

            // Generate link to detailed page
            let detailed_link = format!("enum_{}.md", self.sanitize_filename(name));
            content.push_str(&format!(
                "[View detailed documentation]({})\n\n",
                detailed_link
            ));

            // Generate detailed page for this enum
            if let ItemEnum::Enum(enum_item) = &item.inner {
                self.generate_detailed_enum_page(enum_item, name, item)?;
            }
        }

        let enums_path = self.config.output_dir.join("enums.md");
        utils::write_file(&enums_path, &content)?;

        Ok(vec![enums_path])
    }

    /// Generate functions page
    fn generate_functions_page(&self) -> Result<Vec<PathBuf>, Error> {
        let mut content = String::new();
        content.push_str("# Functions\n\n");

        let mut functions = Vec::new();
        for (id, item) in &self.crate_data.index {
            if let ItemEnum::Function(_) = &item.inner {
                if let Some(name) = &item.name {
                    functions.push((id, item, name));
                }
            }
        }

        functions.sort_by(|a, b| a.2.cmp(b.2));

        for (_id, item, name) in functions {
            content.push_str(&format!("## `{}`\n\n", name));

            if let Some(docs) = &item.docs {
                let brief = self.extract_brief_docs(docs);
                content.push_str(&format!("{}\n\n", brief));
            }
        }

        let functions_path = self.config.output_dir.join("functions.md");
        utils::write_file(&functions_path, &content)?;

        Ok(vec![functions_path])
    }

    /// Generate detailed module page
    fn generate_detailed_module_page(&self, module: &Module, name: &str) -> Result<(), Error> {
        let mut content = String::new();
        content.push_str(&format!("# Module `{}`\n\n", name));

        // Find the module item for documentation
        for item in self.crate_data.index.values() {
            if let Some(item_name) = &item.name {
                if item_name == name {
                    if let Some(docs) = &item.docs {
                        content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
                    }
                    break;
                }
            }
        }

        // List module contents
        if !module.items.is_empty() {
            content.push_str("## Contents\n\n");

            for item_id in &module.items {
                if let Some(item) = self.crate_data.index.get(item_id) {
                    if let Some(item_name) = &item.name {
                        let item_type = match &item.inner {
                            ItemEnum::Module(_) => "Module",
                            ItemEnum::Struct(_) => "Struct",
                            ItemEnum::Trait(_) => "Trait",
                            ItemEnum::Enum(_) => "Enum",
                            ItemEnum::Function(_) => "Function",
                            ItemEnum::Constant { .. } => "Constant",
                            _ => "Item",
                        };

                        content.push_str(&format!("* **{}** `{}`", item_type, item_name));

                        if let Some(docs) = &item.docs {
                            let brief = self.extract_brief_docs(docs);
                            if !brief.is_empty() {
                                content.push_str(&format!(" - {}", brief));
                            }
                        }
                        content.push('\n');
                    }
                }
            }
            content.push('\n');
        }

        let file_path = self
            .config
            .output_dir
            .join(format!("module_{}.md", self.sanitize_filename(name)));
        utils::write_file(&file_path, &content)?;

        Ok(())
    }

    /// Generate detailed struct page
    fn generate_detailed_struct_page(
        &self,
        _struct_item: &Struct,
        name: &str,
        item: &Item,
    ) -> Result<(), Error> {
        let mut content = String::new();
        content.push_str(&format!("# Struct `{}`\n\n", name));

        if let Some(docs) = &item.docs {
            content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
        }

        // TODO: Add fields documentation when we have better type handling
        content.push_str("## Fields\n\n");
        content.push_str("Field information will be available in a future version.\n\n");

        let file_path = self
            .config
            .output_dir
            .join(format!("struct_{}.md", self.sanitize_filename(name)));
        utils::write_file(&file_path, &content)?;

        Ok(())
    }

    /// Generate detailed trait page
    fn generate_detailed_trait_page(
        &self,
        trait_item: &Trait,
        name: &str,
        item: &Item,
    ) -> Result<(), Error> {
        let mut content = String::new();
        content.push_str(&format!("# Trait `{}`\n\n", name));

        if let Some(docs) = &item.docs {
            content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
        }

        if !trait_item.items.is_empty() {
            content.push_str("## Associated Items\n\n");

            for item_id in &trait_item.items {
                if let Some(assoc_item) = self.crate_data.index.get(item_id) {
                    if let Some(assoc_name) = &assoc_item.name {
                        let item_type = match &assoc_item.inner {
                            ItemEnum::Function(_) => "Method",
                            ItemEnum::AssocType { .. } => "Associated Type",
                            ItemEnum::AssocConst { .. } => "Associated Constant",
                            _ => "Item",
                        };

                        content.push_str(&format!("### {} `{}`\n\n", item_type, assoc_name));

                        if let Some(docs) = &assoc_item.docs {
                            content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
                        }
                    }
                }
            }
        }

        let file_path = self
            .config
            .output_dir
            .join(format!("trait_{}.md", self.sanitize_filename(name)));
        utils::write_file(&file_path, &content)?;

        Ok(())
    }

    /// Generate detailed enum page
    fn generate_detailed_enum_page(
        &self,
        enum_item: &Enum,
        name: &str,
        item: &Item,
    ) -> Result<(), Error> {
        let mut content = String::new();
        content.push_str(&format!("# Enum `{}`\n\n", name));

        if let Some(docs) = &item.docs {
            content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
        }

        content.push_str("## Variants\n\n");

        for variant_id in &enum_item.variants {
            if let Some(variant) = self.crate_data.index.get(variant_id) {
                if let Some(variant_name) = &variant.name {
                    content.push_str(&format!("### `{}`\n\n", variant_name));

                    if let Some(docs) = &variant.docs {
                        content.push_str(&format!("{}\n\n", self.clean_docs(docs)));
                    }
                }
            }
        }

        let file_path = self
            .config
            .output_dir
            .join(format!("enum_{}.md", self.sanitize_filename(name)));
        utils::write_file(&file_path, &content)?;

        Ok(())
    }

    /// Sanitize filename for filesystem safety
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_lowercase()
    }

    /// Clean documentation text for markdown output
    fn clean_docs(&self, docs: &str) -> String {
        docs.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract brief documentation (first paragraph)
    fn extract_brief_docs(&self, docs: &str) -> String {
        docs.split("\n\n")
            .next()
            .unwrap_or("")
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(200)
            .collect::<String>()
            + if docs.len() > 200 { "..." } else { "" }
    }
}

/// Convert JSON documentation to multi-page markdown
pub fn convert_to_multipage_markdown(
    json_path: &Path,
    config: MultipageConfig,
) -> Result<Vec<PathBuf>, Error> {
    debug!(
        "Converting JSON to multi-page markdown: {}",
        json_path.display()
    );

    // Load the JSON data
    let json_content = utils::read_file(json_path)?;
    let data: Crate = serde_json::from_str(&json_content).map_err(|e| Error::JsonParse(e))?;

    // Generate multi-page markdown
    let mut generator = MultipageGenerator::new(data, config);
    generator.generate_all()
}
