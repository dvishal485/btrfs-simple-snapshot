use std::{path::PathBuf, process::Command};

use crate::{args::SnapshotArgs, errors::ApplicationError};

pub(crate) fn btrfs_snapshot(args: &SnapshotArgs, snapshot_file: PathBuf) -> Result<(), ApplicationError> {
    let mut snapshot_cmd = Command::new("btrfs");
    snapshot_cmd.args(["subvolume", "snapshot"]);
    if args.readonly {
        snapshot_cmd.arg("-r");
    }
    snapshot_cmd.arg(&args.subvol_path);
    snapshot_cmd.arg(snapshot_file);

    let output = snapshot_cmd
        .output()
        .map_err(ApplicationError::FailedToSpawnCmd)?;

    if args.verbose {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            log::info!("{}", stdout.trim());
        };
    };

    if !output.status.success() {
        if args.verbose {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                log::error!("{}", stderr.trim());
            };
        }
        return Err(ApplicationError::SubvolumeError);
    }

    Ok(())
}
