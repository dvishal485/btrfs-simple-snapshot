use crate::btrfs::SubvolumeBuilderError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("The given mount point \"{0}\" is not a directory")]
    MountPointNotDir(PathBuf),
    #[error("Failed to run btrfs command")]
    FailedToSpawnCmd(std::io::Error),
    #[error("Failed to query the given subvolume")]
    SubvolumeError,
    #[error("Could not infer snapshot prefix, specify it with --prefix <PREFIX>")]
    PrefixInferenceFailed,
    #[error("Failed to parse subvolume information\n{0}")]
    SubvolumeInfoParseFailed(SubvolumeBuilderError),
    #[error("Failed to delete older subvolume")]
    SubvolumeDeletionFailed,
    #[error("Failed to parse subvolume creation time \"{1}\" : {0}")]
    CreationTimeParseFailed(chrono::ParseError, String),
}
