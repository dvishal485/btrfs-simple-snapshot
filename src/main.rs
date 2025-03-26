use clap::{CommandFactory, Parser};
use log;
use std::process::ExitCode;

mod args;
pub mod btrfs;
pub mod errors;
mod utils;
use args::Args;
use btrfs::{btrfs_snapshot, cleaning_job, get_subvol};
use utils::*;

fn main() -> ExitCode {
    let args = Args::parse();
    if let Some(shell) = args.shell_completion {
        let cmd = &mut Args::command();
        clap_complete::generate(
            shell,
            cmd,
            cmd.get_name().to_string(),
            &mut std::io::stdout(),
        );
        return ExitCode::SUCCESS;
    }

    let mut clog = colog::default_builder();

    clog.filter(
        None,
        if args.verbose {
            log::LevelFilter::max()
        } else {
            log::LevelFilter::Error
        },
    );
    clog.init();

    let mut args = make_path_absolute(args);

    if !args.snapshot_path.exists() {
        log::warn!("Snapshot directory does not exists, creating it");
        if let Err(e) = std::fs::create_dir_all(&args.snapshot_path) {
            log::error!("Failed to create snapshot directory\n{}", e);
            return ExitCode::FAILURE;
        }
    } else if !args.snapshot_path.is_dir() {
        log::error!(
            "The specified snapshot path {:?} is not a directory!",
            args.snapshot_path
        );
        return ExitCode::FAILURE;
    } else {
        log::info!("Snapshot directory already exists");
    }

    let prefix = {
        if let Some(prefix) = args.prefix.take() {
            prefix
        } else {
            let prefix = infer_prefix(&args);
            if let Err(e) = prefix {
                log::error!("{}", e);
                return ExitCode::FAILURE;
            }
            let prefix = prefix.unwrap();
            log::info!("Snapshot prefix inferred: {:?}", prefix);
            prefix
        }
    };

    let subvol = verify_path(&args);
    if let Err(e) = subvol {
        log::error!("{}", e);
        return ExitCode::FAILURE;
    }
    let subvol = subvol.unwrap();

    log::debug!(
        "The specified subvolume {} with UUID {} created on {} has {} snapshots",
        subvol.name,
        subvol.uuid,
        subvol.creation_time,
        subvol.snapshots.len()
    );

    if subvol.snapshots.len() > 0 {
        log::info!("Subvolume snapshots: {:?}", subvol.snapshots);
    }

    let curr_time = chrono::Local::now();
    let suffix = curr_time
        .format(&args.datetime_format)
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
        log::error!("File with same name {:?} already exists", filename);
        return ExitCode::FAILURE;
    }

    if let Err(e) = btrfs_snapshot(&args, snapshot_file) {
        log::error!("{}", e);
        return ExitCode::FAILURE;
    };

    log::debug!("Initiating removal of old snapshots");

    let snapshots: Vec<_> = subvol
        .snapshots
        .iter()
        .map(|s| args.mount_point.join(s))
        .filter(|s| s.starts_with(&args.snapshot_path))
        .filter_map(|s| get_subvol(&s).ok().map(|subvol| (s, subvol)))
        .collect();

    if let Some(keep) = args.count {
        if snapshots.len() > keep {
            if let Err(e) = cleaning_job(snapshots, keep) {
                log::error!("{}", e);
                return ExitCode::FAILURE;
            }
        } else {
            log::info!("No snapshots to remove, count is less than total snapshots");
        }
    }

    ExitCode::SUCCESS
}
