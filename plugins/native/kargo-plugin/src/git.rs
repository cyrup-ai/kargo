use anyhow::Result;
use std::path::Path;

pub async fn clone_repository(url: &str, destination: &Path) -> Result<()> {
    tokio::task::spawn_blocking({
        let url = url.to_string();
        let destination = destination.to_path_buf();
        move || {
            let url = gix::url::parse(url.as_str().into())?;
            let mut prepare_clone = gix::prepare_clone(url, &destination)?;

            let (mut prepare_checkout, _outcome) = prepare_clone
                .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

            let (_repo, _outcome) = prepare_checkout
                .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

            Ok::<_, anyhow::Error>(())
        }
    })
    .await??;

    Ok(())
}
