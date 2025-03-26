use log;
use std::path::PathBuf;

use crate::errors::ApplicationError;
use crate::{
    args::Args,
    btrfs::{Subvolume, get_subvol},
};

pub(crate) fn verify_path(args: &Args) -> Result<Subvolume, ApplicationError> {
    log::debug!("Verifying mount point");
    if !args.mount_point.is_dir() {
        return Err(ApplicationError::MountPointNotDir(args.mount_point.clone()));
    }

    log::debug!("Fetching subvolume properties");
    get_subvol(&args.subvol_path)
}

pub(crate) fn infer_prefix(args: &Args) -> Result<PathBuf, ApplicationError> {
    // try to make the subvolume name as snapshot name prefix
    if let Some(f) = args.subvol_path.file_name() {
        Ok(PathBuf::from(f))
    } else {
        return Err(ApplicationError::PrefixInferenceFailed);
    }
}

pub(crate) fn make_path_absolute(mut args: Args) -> Args {
    let subvol_path = args.mount_point.join(&args.subvol_path);
    let snapshot_path = args.mount_point.join(&args.snapshot_path);

    if args.verbose {
        log::info!(
            "Making absolute path with base set as mount point {:?}\nSubvolume path: {:?} -> {:?}\nSnapshot path: {:?} -> {:?}",
            args.mount_point,
            args.subvol_path,
            subvol_path,
            args.snapshot_path,
            snapshot_path
        );
    }

    args.subvol_path = subvol_path;
    args.snapshot_path = snapshot_path;

    args
}
