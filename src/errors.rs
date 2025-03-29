use crate::btrfs::SubvolumeBuilderError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ApplicationError {
    #[error("The given mount point \"{0}\" is not a directory")]
    MountPointNotDir(PathBuf),
    #[error("Failed to run btrfs command. {0}")]
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
    #[error("Failed to create snapshot directory\n{0}")]
    SnapshotDirCreateFail(std::io::Error),
    #[error("The specified snapshot path {0} is not a directory!")]
    InvalidSnapshotDir(PathBuf),
    #[error("File with same name {0} already exists")]
    SnapshotAlreadyExists(PathBuf),
    #[error("Atleast one of --keep-count or --keep-since is required for cleaning")]
    NoCleaningArg,
    #[error("Time interval too big to work with")]
    TimeOutOfRange,
}
