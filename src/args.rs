use clap::Parser;
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// Mount point of btrfs filesystem
    #[clap(long, short)]
    pub mount_point: PathBuf,

    /// Path to subvolume to snapshot (relative to mount point)
    #[clap(long, short = 'p')]
    pub subvol_path: PathBuf,

    /// Path in which snapshots are stored (relative to mount point)
    #[clap(long, short = 's', default_value = ".snapshots")]
    pub snapshot_path: PathBuf,

    /// Make snapshot readonly
    #[clap(long, short)]
    pub readonly: bool,

    /// Specify to limit the number of snapshots to keep
    #[clap(long, short)]
    pub count: Option<usize>,

    /// Prefix for snapshot name
    #[clap(long)]
    pub prefix: Option<PathBuf>,

    /// Datetime suffix format for snapshot name
    #[clap(long, short, default_value = "%Y-%m-%d-%H%M%S")]
    pub datetime_format: String,

    /// Verbose output logging
    #[clap(long)]
    pub verbose: bool,

    /// Generate shell completions for given shell
    #[clap(long)]
    pub shell_completion: Option<Shell>,
}
