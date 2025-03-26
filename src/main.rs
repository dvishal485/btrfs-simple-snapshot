use chrono::NaiveDateTime;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use derive_builder::Builder;
use log;
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};
use thiserror::Error;

#[derive(Parser)]
struct Args {
    /// Mount point of btrfs filesystem
    #[clap(long, short)]
    mount_point: PathBuf,

    /// Path to subvolume to snapshot (relative to mount point)
    #[clap(long, short = 'p')]
    subvol_path: PathBuf,

    /// Path in which snapshots are stored (relative to mount point)
    #[clap(long, short = 's', default_value = ".snapshots")]
    snapshot_path: PathBuf,

    /// Make snapshot readonly
    #[clap(long, short)]
    readonly: bool,

    /// Specify to limit the number of snapshots to keep
    #[clap(long, short)]
    count: Option<usize>,

    /// Prefix for snapshot name
    #[clap(long)]
    prefix: Option<PathBuf>,

    /// Datetime suffix format for snapshot name
    #[clap(long, short, default_value = "%Y-%m-%d-%H%M%S")]
    datetime_format: String,

    /// Verbose output logging
    #[clap(long)]
    verbose: bool,

    /// Generate shell completions for given shell
    #[clap(long)]
    shell_completion: Option<Shell>,
}

#[derive(Error, Debug)]
enum ApplicationError {
    #[error("The given mount point \"{0}\" is not a directory")]
    MountPointNotDir(PathBuf),
    #[error("Failed to run btrfs command")]
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
}

fn verify_path(args: &Args) -> Result<Subvolume, ApplicationError> {
    log::debug!("Verifying mount point");
    if !args.mount_point.is_dir() {
        return Err(ApplicationError::MountPointNotDir(args.mount_point.clone()));
    }

    log::debug!("Fetching subvolume properties");
    get_subvol(&args.subvol_path)
}

fn get_subvol(subvol_path: &PathBuf) -> Result<Subvolume, ApplicationError> {
    let subvol_show = Command::new("btrfs")
        .arg("subvolume")
        .arg("show")
        .arg(subvol_path)
        .output()
        .map_err(|e| ApplicationError::FailedToSpawnCmd(e))?;

    let stdout = String::from_utf8_lossy(&subvol_show.stdout);
    if !stdout.is_empty() {
        log::info!("{}", stdout.trim());
    };

    if !subvol_show.status.success() {
        let stderr = String::from_utf8_lossy(&subvol_show.stderr);
        if !stderr.is_empty() {
            log::error!("{}", stderr.trim());
        }
        return Err(ApplicationError::SubvolumeError);
    }

    let subvol_info = SubvolumeInfo(stdout);

    let subvol = Subvolume::try_from(subvol_info);
    subvol
}

struct SubvolumeInfo<'a>(std::borrow::Cow<'a, str>);

#[derive(Builder)]
#[builder(pattern = "owned")]
struct Subvolume {
    name: String,
    uuid: String,
    snapshots: Vec<PathBuf>,
    creation_time: chrono::NaiveDateTime,
}

impl<'a> TryFrom<SubvolumeInfo<'a>> for Subvolume {
    type Error = ApplicationError;

    fn try_from(SubvolumeInfo(info): SubvolumeInfo<'a>) -> Result<Self, Self::Error> {
        let mut subvol = SubvolumeBuilder::create_empty();
        let mut snapshots = vec![];
        let mut snapshot_capture = false;
        for data in info.lines() {
            let data = data.trim();
            if let Some((property, value)) = data.split_once(':') {
                match property.trim() {
                    "Name" => subvol = subvol.name(value.trim().to_string()),
                    "UUID" => subvol = subvol.uuid(value.trim().to_string()),
                    "Creation time" => {
                        subvol = subvol.creation_time({
                            let datetime_str = value.trim();
                            let subvol_info_time =
                                NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S %z");
                            if let Ok(time) = subvol_info_time {
                                log::info!("Subvolume creation time: {}", time);
                                time
                            } else {
                                return Err(ApplicationError::CreationTimeParseFailed(
                                    subvol_info_time.unwrap_err(),
                                    datetime_str.to_string(),
                                ));
                            }
                        })
                    }
                    property => {
                        if property.contains("Snapshot") {
                            log::debug!("Snapshot column encountered!");
                            snapshot_capture = true;
                        } else if snapshot_capture {
                            snapshot_capture = false;
                            log::debug!("Snapshot column supposedly ended")
                        }
                    }
                }
            } else {
                if snapshot_capture {
                    let snapshot_name = data.trim();
                    log::info!("Found snapshot {}", snapshot_name);
                    snapshots.push(snapshot_name);
                }
            }
        }

        subvol = subvol.snapshots(
            snapshots
                .into_iter()
                .map(|snap| PathBuf::from(snap))
                .collect(),
        );

        subvol
            .build()
            .map_err(|e| ApplicationError::SubvolumeInfoParseFailed(e))
    }
}

fn infer_prefix(args: &Args) -> Result<PathBuf, ApplicationError> {
    // try to make the subvolume name as snapshot name prefix
    if let Some(f) = args.subvol_path.file_name() {
        Ok(PathBuf::from(f))
    } else {
        return Err(ApplicationError::PrefixInferenceFailed);
    }
}

fn make_path_absolute(mut args: Args) -> Args {
    let subvol_path = args.mount_point.join(&args.subvol_path);
    let snapshot_path = args.mount_point.join(&args.snapshot_path);

    if args.verbose {
        log::info!(
            "Making absolute path with base set as mount point {:?}\nSubvolume path: {:?} -> {:?}\nSnapshot path: {:?} -> {:?}",
            args.mount_point,
            args.subvol_path,
            subvol_path,
            args.snapshot_path,
            snapshot_path
        );
    }

    args.subvol_path = subvol_path;
    args.snapshot_path = snapshot_path;

    args
}

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

fn cleaning_job(
    mut snapshots: Vec<(PathBuf, Subvolume)>,
    limit: usize,
) -> Result<(), ApplicationError> {
    log::debug!("Initiating clean job");

    snapshots.sort_by_key(|(_, s)| std::cmp::Reverse(s.creation_time));
    for (path, s) in snapshots.drain(limit..) {
        log::info!("Removing snapshot {:?}", s.name);
        let output = Command::new("btrfs")
            .arg("subvolume")
            .arg("delete")
            .arg(path)
            .output()
            .map_err(|e| ApplicationError::FailedToSpawnCmd(e))?;

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
            return Err(ApplicationError::SubvolumeDeletionFailed);
        } else {
            log::info!("Successfully removed snapshot {}", s.name);
        }
    }

    Ok(())
}

fn btrfs_snapshot(args: &Args, snapshot_file: PathBuf) -> Result<(), ApplicationError> {
    let mut snapshot_cmd = Command::new("btrfs");
    snapshot_cmd.args(&["subvolume", "snapshot"]);
    if args.readonly {
        snapshot_cmd.arg("-r");
    }
    snapshot_cmd.arg(&args.subvol_path);
    snapshot_cmd.arg(snapshot_file);

    let output = snapshot_cmd
        .output()
        .map_err(|e| ApplicationError::FailedToSpawnCmd(e))?;

    if args.verbose {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            log::info!("{}", stdout.trim());
        };
    };

    if !output.status.success() {
        if args.verbose {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                log::error!("{}", stderr.trim());
            };
        }
        return Err(ApplicationError::SubvolumeError);
    }

    Ok(())
}
