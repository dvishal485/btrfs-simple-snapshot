use clap::{
    Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};

use super::subcommand::{CleanSubcommand, CompletionSubcommand, SnapshotSubcommand};

fn make_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Blue.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
}

#[derive(Parser)]
#[command(version, about, styles=make_style())]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Action,

    /// Verbose output logging
    #[clap(long, global = true)]
    pub(crate) verbose: bool,
}

#[derive(Subcommand)]
pub(crate) enum Action {
    Completions(CompletionSubcommand),
    Snapshot(SnapshotSubcommand),
    Clean(CleanSubcommand),
}
