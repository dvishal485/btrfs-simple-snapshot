use clap::Parser;
use clap_complete::Shell;
use std::path::PathBuf;

use super::argument::{CleaningArgs, SnapshotArgs, SubvolumeArgs};

#[derive(Parser)]
/// Invoke the cleaning task of given subvolume snapshots
///
/// Atlease one one --keep-count or --keep-since must be provided
pub(crate) struct CleanSubcommand {
    #[clap(flatten)]
    pub(crate) subvol_args: SubvolumeArgs,
    #[clap(flatten)]
    pub(crate) cleaning_args: CleaningArgs,
    /// Limit clean task only to mentioned path (relative to mount point)
    #[clap(long, short = 'p', default_value = ".snapshots")]
    pub(crate) snapshot_path: PathBuf,
}

#[derive(Parser)]
/// Create snapshots of subvolumes and optionally invoke cleaning
pub(crate) struct SnapshotSubcommand {
    #[clap(flatten)]
    pub(crate) subvol_args: SubvolumeArgs,

    #[clap(flatten)]
    pub(crate) snapshot_args: SnapshotArgs,

    #[clap(flatten)]
    pub(crate) cleaning_args: Option<CleaningArgs>,
}

#[derive(Parser)]
/// Generate shell completions file
pub(crate) struct CompletionSubcommand {
    /// Compatible shell for completions file
    pub(crate) shell_completion: Shell,
}
