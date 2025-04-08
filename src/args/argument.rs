use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub(crate) struct SubvolumeArgs {
    /// Mount point of btrfs filesystem
    pub(crate) mount_point: PathBuf,

    /// Path to subvolume to snapshot (relative to mount point)
    pub(crate) subvol_path: PathBuf,
}

#[derive(Parser)]
pub(crate) struct SnapshotArgs {
    /// Path in which snapshots are stored (relative to mount point)
    #[clap(long, short = 'p', default_value = ".snapshots")]
    pub(crate) snapshot_path: PathBuf,

    /// Make snapshot readonly
    #[clap(long, short = 'r')]
    pub(crate) readonly: bool,

    /// Prefix for snapshot name (defaults to subvolume name)
    #[clap(long)]
    pub(crate) prefix: Option<PathBuf>,

    /// Datetime suffix format for snapshot name
    #[clap(long, short = 'f', default_value = "%Y-%m-%d-%H.%M.%S")]
    pub(crate) suffix_format: String,
}

#[derive(Parser)]
pub(crate) struct CleaningArgs {
    /// Minimum number of snapshots to preserve
    #[clap(long, short = 'c')]
    pub(crate) keep_count: Option<usize>,

    /// Minimum age of snapshots to preserve
    /// (does not clean snapshots younger than given duration)
    ///
    /// Example: 5d (5 days), 6h 30m (6 hours 30 minutes), 1y (1 year), 5M 1w (5 months 1 week)
    ///
    /// This takes precedence over "--keep-count", i.e. - Even if number of younger
    /// snapshots is greater than keep_count, they are not removed.
    ///
    /// Only the older snapshots are considered for removal.
    #[clap(long, short = 's')]
    pub(crate) keep_since: Option<humantime::Duration>,
}
