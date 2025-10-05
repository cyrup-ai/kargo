use anyhow::Result;
use std::path::PathBuf;

pub enum SourceType {
    GitHub { org: String, repo: String },
    LocalPath(PathBuf),
}

pub fn parse_source(source: &str) -> Result<SourceType> {
    // Check if it's a local path
    let path = PathBuf::from(source);
    if path.exists() {
        return Ok(SourceType::LocalPath(path.canonicalize()?));
    }

    // Handle org/repo shorthand
    if !source.contains("://") && !source.starts_with("git@") && source.contains('/') {
        let parts: Vec<&str> = source.split('/').collect();
        if parts.len() == 2 {
            return Ok(SourceType::GitHub {
                org: parts[0].to_string(),
                repo: parts[1].trim_end_matches(".git").to_string(),
            });
        }
    }

    // Parse full URL using git-url-parse
    let git_url = git_url_parse::GitUrl::parse(source)?;

    // Extract owner and repo from the path
    let path = git_url.path();
    let parts: Vec<&str> = path.trim_start_matches('/').trim_end_matches(".git").split('/').collect();

    if parts.len() >= 2 {
        let org = parts[0].to_string();
        let repo = parts[1].to_string();
        Ok(SourceType::GitHub { org, repo })
    } else {
        anyhow::bail!("Could not extract organization and repository from URL: {source}")
    }
}
