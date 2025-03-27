use std::path::PathBuf;

use crate::args::{SnapshotArgs, SubvolumeArgs};
use crate::btrfs::Subvolume;
use crate::errors::ApplicationError;
use crate::{SnapshotSubcommand, get_subvol};

#[inline]
pub(crate) fn verify_mount_path(args: &SubvolumeArgs) -> Result<(), ApplicationError> {
    log::debug!("Verifying mount point");
    if !args.mount_point.is_dir() {
        Err(ApplicationError::MountPointNotDir(
            args.mount_point.to_owned(),
        ))
    } else {
        Ok(())
    }
}

#[inline]
pub(crate) fn verify_snapshot_path(args: &SnapshotArgs) -> Result<(), ApplicationError> {
    log::debug!("Verifying snapshot path");
    if !args.snapshot_path.exists() {
        log::warn!("Snapshot directory does not exists, creating it");
        std::fs::create_dir_all(&args.snapshot_path)
            .map_err(ApplicationError::SnapshotDirCreateFail)
    } else if !args.snapshot_path.is_dir() {
        Err(ApplicationError::InvalidSnapshotDir(
            args.snapshot_path.to_owned(),
        ))
    } else {
        log::info!("Snapshot directory already exists");
        Ok(())
    }
}

#[inline]
pub(crate) fn infer_prefix(args: &SnapshotSubcommand) -> Result<PathBuf, ApplicationError> {
    // try to make the subvolume name as snapshot name prefix
    args.subvol_args
        .subvol_path
        .file_name()
        .map(|f| PathBuf::from(f))
        .ok_or(ApplicationError::PrefixInferenceFailed)
}

#[inline]
pub(crate) fn make_path_absolute(args: &mut SnapshotSubcommand) {
    let subvol_path = args
        .subvol_args
        .mount_point
        .join(&args.subvol_args.subvol_path);
    let snapshot_path = args
        .subvol_args
        .mount_point
        .join(&args.snapshot_args.snapshot_path);

    log::info!(
        "Making absolute path with base set as mount point {:?}\nSubvolume path: {:?} -> {:?}\nSnapshot path: {:?} -> {:?}",
        args.subvol_args.mount_point,
        args.subvol_args.subvol_path,
        subvol_path,
        args.snapshot_args.snapshot_path,
        snapshot_path
    );

    args.subvol_args.subvol_path = subvol_path;
    args.snapshot_args.snapshot_path = snapshot_path;
}

pub(crate) fn get_subvol_wrapped(path: &PathBuf) -> Result<Subvolume, ApplicationError> {
    log::debug!("Fetching subvolume properties");
    let subvol = get_subvol(path)?;

    log::debug!(
        "The specified subvolume {} with UUID {} created on {} has {} snapshots",
        subvol.name,
        subvol.uuid,
        subvol.creation_time,
        subvol.snapshots.len()
    );

    if !subvol.snapshots.is_empty() {
        log::info!("Subvolume snapshots: {:?}", subvol.snapshots);
    }

    Ok(subvol)
}
