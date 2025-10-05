use anyhow::Result;
use std::path::PathBuf;

pub enum SourceType {
    GitHub {
        org: String,
        repo: String,
        plugin: Option<String>,
    },
    LocalPath(PathBuf),
}

pub fn parse_source(source: &str) -> Result<SourceType> {
    let path = PathBuf::from(source);
    if path.exists() {
        return Ok(SourceType::LocalPath(path.canonicalize()?));
    }

    if !source.contains("://") && !source.starts_with("git@") && source.contains('/') {
        let parts: Vec<&str> = source.split('/').collect();

        match parts.len() {
            2 => {
                let org = parts[0];
                let repo = parts[1].trim_end_matches(".git");

                if org.is_empty() {
                    anyhow::bail!("Invalid source: organization cannot be empty");
                }
                if repo.is_empty() {
                    anyhow::bail!("Invalid source: repository cannot be empty");
                }

                Ok(SourceType::GitHub {
                    org: org.to_string(),
                    repo: repo.to_string(),
                    plugin: None,
                })
            }
            3 => {
                let org = parts[0];
                let repo = parts[1].trim_end_matches(".git");
                let plugin = parts[2];

                if org.is_empty() {
                    anyhow::bail!("Invalid source: organization cannot be empty");
                }
                if repo.is_empty() {
                    anyhow::bail!("Invalid source: repository cannot be empty");
                }
                if plugin.is_empty() {
                    anyhow::bail!("Invalid source: plugin name cannot be empty");
                }

                Ok(SourceType::GitHub {
                    org: org.to_string(),
                    repo: repo.to_string(),
                    plugin: Some(plugin.to_string()),
                })
            }
            _ => anyhow::bail!(
                "Invalid source format. Expected: org/repo or org/repo/plugin, got: {}",
                source
            ),
        }
    } else {
        let git_url = git_url_parse::GitUrl::parse(source)?;
        let path = git_url.path();
        let parts: Vec<&str> = path
            .trim_start_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .collect();

        if parts.len() >= 2 {
            Ok(SourceType::GitHub {
                org: parts[0].to_string(),
                repo: parts[1].to_string(),
                plugin: None,
            })
        } else {
            anyhow::bail!("Could not extract organization and repository from URL: {source}")
        }
    }
}
