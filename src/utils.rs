use std::path::PathBuf;

use crate::args::SnapshotArgs;
use crate::errors::ApplicationError;

pub(crate) fn verify_path(args: &SnapshotArgs) -> Result<(), ApplicationError> {
    log::debug!("Verifying mount point");
    if !args.mount_point.is_dir() {
        return Err(ApplicationError::MountPointNotDir(
            args.mount_point.to_owned(),
        ));
    }

    log::debug!("Verifying snapshot path");
    if !args.snapshot_path.exists() {
        log::warn!("Snapshot directory does not exists, creating it");
        std::fs::create_dir_all(&args.snapshot_path)
            .map_err(ApplicationError::SnapshotDirCreateFail)?;
    } else if !args.snapshot_path.is_dir() {
        return Err(ApplicationError::InvalidSnapshotDir(
            args.snapshot_path.to_owned(),
        ));
    } else {
        log::info!("Snapshot directory already exists");
    }

    Ok(())
}

#[inline]
pub(crate) fn infer_prefix(args: &SnapshotArgs) -> Result<PathBuf, ApplicationError> {
    // try to make the subvolume name as snapshot name prefix
    args.subvol_path
        .file_name()
        .map(|f| PathBuf::from(f))
        .ok_or(ApplicationError::PrefixInferenceFailed)
}

#[inline]
pub(crate) fn make_path_absolute(args: &mut SnapshotArgs) {
    let subvol_path = args.mount_point.join(&args.subvol_path);
    let snapshot_path = args.mount_point.join(&args.snapshot_path);

    log::info!(
        "Making absolute path with base set as mount point {:?}\nSubvolume path: {:?} -> {:?}\nSnapshot path: {:?} -> {:?}",
        args.mount_point,
        args.subvol_path,
        subvol_path,
        args.snapshot_path,
        snapshot_path
    );

    args.subvol_path = subvol_path;
    args.snapshot_path = snapshot_path;
}
