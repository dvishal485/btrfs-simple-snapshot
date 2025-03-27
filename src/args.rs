use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Action,

    /// Verbose output logging
    #[clap(long, global = true)]
    pub(crate) verbose: bool,
}

#[derive(Subcommand)]
pub(crate) enum Action {
    Completion(CompletionSubcommand),
    Snapshot(SnapshotSubcommand),
    Clean(CleanSubcommand),
}

#[derive(Parser)]
pub(crate) struct SubvolumeArgs {
    /// Mount point of btrfs filesystem
    pub(crate) mount_point: PathBuf,

    /// Path to subvolume to snapshot (relative to mount point)
    pub(crate) subvol_path: PathBuf,
}

#[derive(Parser)]
pub(crate) struct CleanSubcommand {
    #[clap(flatten)]
    pub(crate) subvol_args: SubvolumeArgs,
    #[clap(flatten)]
    pub(crate) cleaning_args: CleaningArgs,
}

#[derive(Parser)]
pub(crate) struct SnapshotSubcommand {
    #[clap(flatten)]
    pub(crate) subvol_args: SubvolumeArgs,

    #[clap(flatten)]
    pub(crate) snapshot_args: SnapshotArgs,

    #[clap(flatten)]
    pub(crate) cleaning_args: CleaningArgs,
}

#[derive(Parser)]
/// Generate shell completions file
pub(crate) struct CompletionSubcommand {
    /// Compatible shell for completions file
    pub(crate) shell_completion: Shell,
}

#[derive(Parser)]
/// With btrfs-auto-snapshot paired with a cron job or timer, you can easily create snapshots of btrfs subvolumes and maintain a particular number of snapshots at disposal for simpler backup solution.
pub(crate) struct SnapshotArgs {
    /// Path in which snapshots are stored (relative to mount point)
    #[clap(long, short, default_value = ".snapshots")]
    pub(crate) snapshot_path: PathBuf,

    /// Make snapshot readonly
    #[clap(long, short)]
    pub(crate) readonly: bool,

    /// Prefix for snapshot name (defaults to subvolume name)
    #[clap(long)]
    pub(crate) prefix: Option<PathBuf>,

    /// Datetime suffix format for snapshot name
    #[clap(long, short = 'f', default_value = "%Y-%m-%d-%H%M%S")]
    pub(crate) suffix_format: String,
}

#[derive(Parser)]
pub(crate) struct CleaningArgs {
    /// Specify to limit the number of snapshots to keep
    #[clap(long, short)]
    pub(crate) keep_count: Option<usize>,


}
