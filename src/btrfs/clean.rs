use std::{path::PathBuf, process::Command};

use super::Subvolume;
use crate::errors::ApplicationError;

pub fn cleaning_job(
    mut snapshots: Vec<(PathBuf, Subvolume)>,
    limit: usize,
) -> Result<(), ApplicationError> {
    log::debug!("Initiating clean job");

    snapshots.sort_by_key(|(_, s)| std::cmp::Reverse(s.creation_time));
    for (path, s) in snapshots.drain(limit..) {
        log::info!("Removing snapshot {:?}", s.name);
        let output = Command::new("btrfs")
            .arg("subvolume")
            .arg("delete")
            .arg(path)
            .output()
            .map_err(|e| ApplicationError::FailedToSpawnCmd(e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            log::info!("{}", stdout.trim());
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                log::error!("{}", stderr.trim());
            };
            log::error!("Clean job failed for snapshot {}", s.name);
            return Err(ApplicationError::SubvolumeDeletionFailed);
        } else {
            log::info!("Successfully removed snapshot {}", s.name);
        }
    }

    Ok(())
}
