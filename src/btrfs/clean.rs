use chrono::{Local, NaiveDateTime, TimeDelta};
use std::{path::PathBuf, process::Command};

use super::Subvolume;
use crate::{args::CleaningArgs, subcommand::CleanSubcommand};
use crate::{errors::ApplicationError, get_subvol, get_subvol_wrapped, verify_mount_path};

pub(crate) fn handle_clean(mut args: CleanSubcommand) -> Result<(), ApplicationError> {
    verify_mount_path(&args.subvol_args)?;

    let curr_time = Local::now().naive_local();

    // make path absolute
    args.subvol_args.subvol_path = args
        .subvol_args
        .mount_point
        .join(args.subvol_args.subvol_path);
    args.snapshot_path = args.subvol_args.mount_point.join(args.snapshot_path);

    let subvol = get_subvol_wrapped(&args.subvol_args.subvol_path)?;

    let snapshots: Vec<_> = subvol
        .snapshots
        .iter()
        .map(|s| args.subvol_args.mount_point.join(s))
        .filter(|s| s.starts_with(&args.snapshot_path))
        .filter_map(|s| get_subvol(&s).ok().map(|subvol| (s, subvol)))
        .collect();

    cleaning_job(snapshots, args.cleaning_args, curr_time)
}

pub(crate) fn cleaning_job(
    mut snapshots: Vec<(PathBuf, Subvolume)>,
    cleaning_args: CleaningArgs,
    curr_time: NaiveDateTime,
) -> Result<(), ApplicationError> {
    log::debug!("Initiating clean job");

    snapshots.sort_by_key(|(_, s)| std::cmp::Reverse(s.creation_time));

    let deletion_idx = match (cleaning_args.keep_count, cleaning_args.keep_since) {
        (None, None) => Err(ApplicationError::NoCleaningArg),
        (Some(limit), None) => Ok(limit),
        (keep_count, Some(duration)) => {
            let limit;
            if let Some(keep_count) = keep_count {
                limit = keep_count;
                if snapshots.len() <= keep_count {
                    log::info!("Number of snapshots is within given limit, no cleaning required");
                    return Ok(());
                }
            } else {
                limit = 0;
            }

            let delta =
                TimeDelta::from_std(*duration).map_err(|_| ApplicationError::TimeOutOfRange)?;

            let delete_before = curr_time
                .checked_sub_signed(delta)
                .ok_or(ApplicationError::TimeOutOfRange)?;

            Ok(snapshots
                .partition_point(|(_, s)| s.creation_time >= delete_before)
                .max(limit))
        }
    }?;

    if snapshots.len() < deletion_idx {
        log::info!("Number of snapshots is within given limit, no cleaning required");
        return Ok(());
    }

    for (path, s) in snapshots.drain(deletion_idx..) {
        remove_snapshot(s, path)?
    }
    Ok(())
}

fn remove_snapshot(s: Subvolume, path: PathBuf) -> Result<(), ApplicationError> {
    log::info!("Removing snapshot {:?}", s.name);
    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("delete")
        .arg(path)
        .output()
        .map_err(ApplicationError::FailedToSpawnCmd)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        log::info!("{}", stdout.trim());
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            log::error!("{}", stderr.trim());
        };
        log::error!("Clean job failed for snapshot {}", s.name);
        Err(ApplicationError::SubvolumeDeletionFailed)
    } else {
        log::info!("Successfully removed snapshot {}", s.name);
        Ok(())
    }
}
