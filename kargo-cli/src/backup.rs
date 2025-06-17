use crate::events::{Event, EventBus};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug)]
pub struct Change {
    path: PathBuf,
    backup_path: PathBuf,
}

pub struct BackupManager {
    backup_dir: TempDir,
    changes: Vec<Change>,
    events: EventBus,
}

impl BackupManager {
    pub fn new(events: EventBus) -> Result<Self> {
        Ok(Self {
            backup_dir: TempDir::new()?,
            changes: Vec::new(),
            events,
        })
    }

    pub fn backup_file(&mut self, path: &Path) -> Result<()> {
        let rel_path = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Path has no file name: {}", path.display()))?;
        let backup_path = self.backup_dir.path().join(rel_path);

        fs::copy(path, &backup_path)?;

        self.changes.push(Change {
            path: path.to_owned(),
            backup_path,
        });

        Ok(())
    }

    pub fn rollback(&self) -> Result<()> {
        self.events.publish(Event::RollbackStarted {
            path: self.backup_dir.path().to_owned(),
        });

        for change in &self.changes {
            fs::copy(&change.backup_path, &change.path)?;
        }

        self.events.publish(Event::RollbackFinished {
            path: self.backup_dir.path().to_owned(),
        });

        Ok(())
    }
}
