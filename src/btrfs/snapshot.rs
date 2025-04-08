use std::{path::PathBuf, process::Command};

use crate::{errors::ApplicationError, subcommand::SnapshotSubcommand};

pub(crate) fn btrfs_snapshot(
    args: &SnapshotSubcommand,
    snapshot_file: PathBuf,
) -> Result<(), ApplicationError> {
    let snap = &args.snapshot_args;
    let mut snapshot_cmd = Command::new("btrfs");
    snapshot_cmd.args(["subvolume", "snapshot"]);
    if snap.readonly {
        snapshot_cmd.arg("-r");
    }
    snapshot_cmd.arg(&args.subvol_args.subvol_path);
    snapshot_cmd.arg(snapshot_file);

    let output = snapshot_cmd
        .output()
        .map_err(ApplicationError::FailedToSpawnCmd)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        log::info!("{}", stdout.trim());
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            log::error!("{}", stderr.trim());
        };
        return Err(ApplicationError::SubvolumeError);
    }

    Ok(())
}
