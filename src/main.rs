use clap::{CommandFactory, Parser};
use errors::ApplicationError;
use std::process::ExitCode;

mod args;
pub(crate) mod btrfs;
pub(crate) mod errors;
mod utils;
use args::{Action, Cli, SnapshotArgs};
use btrfs::{btrfs_snapshot, cleaning_job, get_subvol};
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
        Action::Completion(args) => {
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
    }
}

fn handle_snapshot(mut args: SnapshotArgs) -> Result<(), ApplicationError> {
    make_path_absolute(&mut args);

    let prefix = {
        if let Some(prefix) = args.prefix.take() {
            prefix
        } else {
            let prefix = infer_prefix(&args)?;
            log::info!("Snapshot prefix inferred: {:?}", prefix);
            prefix
        }
    };

    verify_path(&args)?;

    log::debug!("Fetching subvolume properties");
    let subvol = get_subvol(&args.subvol_path)?;

    log::debug!(
        "The specified subvolume {} with UUID {} created on {} has {} snapshots",
        subvol.name,
        subvol.uuid,
        subvol.creation_time,
        subvol.snapshots.len()
    );

    if !subvol.snapshots.is_empty() {
        log::info!("Subvolume snapshots: {:?}", subvol.snapshots);
    }

    let curr_time = chrono::Local::now();
    let suffix = curr_time
        .format(&args.suffix_format)
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
    let snapshot_file = args.snapshot_path.join(&filename);

    log::info!("Snapshot file: {:?}\nPath: {:?}", filename, snapshot_file);

    if snapshot_file.exists() {
        return Err(ApplicationError::SnapshotAlreadyExists(filename));
    }

    btrfs_snapshot(&args, snapshot_file)?;

    log::debug!("Initiating removal of old snapshots");

    let snapshots: Vec<_> = subvol
        .snapshots
        .iter()
        .map(|s| args.mount_point.join(s))
        .filter(|s| s.starts_with(&args.snapshot_path))
        .filter_map(|s| get_subvol(&s).ok().map(|subvol| (s, subvol)))
        .collect();

    if let Some(keep) = args.cleaning_args.keep_count {
        if snapshots.len() > keep {
            cleaning_job(snapshots, keep)?;
        } else {
            log::info!("No snapshots to remove, count is less than total snapshots");
        }
    }

    log::info!("Program finished successfully");
    Ok(())
}
