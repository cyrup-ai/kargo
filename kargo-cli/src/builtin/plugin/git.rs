use anyhow::{Context, Result};
use std::path::Path;

pub async fn clone_repository(url: &str, destination: &Path, branch: Option<&str>) -> Result<()> {
    tokio::task::spawn_blocking({
        let url = url.to_string();
        let destination = destination.to_path_buf();
        let branch = branch.map(|s| s.to_string());
        move || {
            let parsed_url = gix::url::parse(url.as_str().into())
                .context("Failed to parse git URL")?;

            let mut prepare_clone = gix::prepare_clone(parsed_url, &destination)
                .context("Failed to prepare clone")?;

            // Configure branch if specified
            if let Some(branch_name) = branch {
                let refspec = format!("+refs/heads/{branch_name}:refs/heads/{branch_name}");
                prepare_clone = prepare_clone.configure_remote(move |remote| {
                    remote.with_refspecs(
                        [refspec.as_str()],
                        gix::remote::Direction::Fetch,
                    ).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                });
            }

            let (mut prepare_checkout, _outcome) = prepare_clone
                .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
                    std::num::NonZeroU32::new(1)
                        .ok_or_else(|| anyhow::anyhow!("Invalid shallow depth"))?,
                ))
                .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                .context("Failed to fetch repository")?;

            let (_repo, _outcome) = prepare_checkout
                .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                .context("Failed to checkout worktree")?;

            Ok::<_, anyhow::Error>(())
        }
    })
    .await
    .context("Clone task panicked")??;

    Ok(())
}
