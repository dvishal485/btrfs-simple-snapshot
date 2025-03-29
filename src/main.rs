use clap::{CommandFactory, Parser};
use errors::ApplicationError;
use std::process::ExitCode;

mod args;
pub(crate) mod btrfs;
pub(crate) mod errors;
mod utils;
use args::{Action, Cli, SnapshotSubcommand};
use btrfs::{btrfs_snapshot, cleaning_job, get_subvol, handle_clean};
use utils::*;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut clog = colog::default_builder();
    clog.filter(
        None,
        if cli.verbose {
            log::LevelFilter::max()
        } else {
            log::LevelFilter::Error
        },
    );
    clog.init();

    match cli.command {
        Action::Completions(args) => {
            let cmd = &mut Cli::command();
            clap_complete::generate(
                args.shell_completion,
                cmd,
                cmd.get_name().to_string(),
                &mut std::io::stdout(),
            );
            ExitCode::SUCCESS
        }
        Action::Snapshot(args) => {
            if let Err(e) = handle_snapshot(args) {
                log::error!("{}", e);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Action::Clean(args) => {
            if let Err(e) = handle_clean(args) {
                log::error!("{}", e);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}

fn handle_snapshot(mut args: SnapshotSubcommand) -> Result<(), ApplicationError> {
    make_path_absolute(&mut args);

    let prefix = {
        if let Some(prefix) = args.snapshot_args.prefix.take() {
            prefix
        } else {
            let prefix = infer_prefix(&args)?;
            log::info!("Snapshot prefix inferred: {:?}", prefix);
            prefix
        }
    };

    verify_mount_path(&args.subvol_args)?;
    verify_snapshot_path(&args.snapshot_args)?;

    let subvol = get_subvol_wrapped(&args.subvol_args.subvol_path)?;

    let curr_time = chrono::Local::now();
    let suffix = curr_time
        .format(&args.snapshot_args.suffix_format)
        .to_string()
        .replace('/', "-");

    log::info!(
        "Current datetime: {curr_time}\nSnapshot suffix: {:?}",
        suffix
    );

    let mut filename = prefix.clone();
    if !suffix.is_empty() {
        filename.as_mut_os_string().push("-");
        filename.as_mut_os_string().push(suffix);
    }
    let snapshot_file = args.snapshot_args.snapshot_path.join(&filename);

    log::info!("Snapshot file: {:?}\nPath: {:?}", filename, snapshot_file);

    if snapshot_file.exists() {
        return Err(ApplicationError::SnapshotAlreadyExists(filename));
    }

    btrfs_snapshot(&args, snapshot_file)?;

    log::debug!("Initiating removal of old snapshots");

    let snapshots: Vec<_> = subvol
        .snapshots
        .iter()
        .map(|s| args.subvol_args.mount_point.join(s))
        .filter(|s| s.starts_with(&args.snapshot_args.snapshot_path))
        .filter_map(|s| get_subvol(&s).ok().map(|subvol| (s, subvol)))
        .collect();

    if let Some(cleaning_args) = args.cleaning_args {
        cleaning_job(snapshots, cleaning_args, curr_time.naive_local())?
    }

    log::info!("Program finished successfully");
    Ok(())
}
