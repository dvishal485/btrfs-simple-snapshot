use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use log;
use std::{
    io,
    path::PathBuf,
    process::{Command, ExitCode},
};
use thiserror::Error;

#[derive(Parser, Debug)]
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
enum ApplicationError<'a> {
    #[error("The given mount point \"{0}\" is not a directory")]
    MountPointNotDir(&'a PathBuf),
    #[error("Failed to run btrfs command")]
    FailedToSpawnCmd(io::Error),
    #[error("Failed to query the given subvolume")]
    SubvolumeError,
    #[error("Could not infer snapshot prefix, specify it with --prefix <PREFIX>")]
    PrefixInferenceFailed,
}

fn verify_path(args: &Args) -> Result<(), ApplicationError> {
    if !args.mount_point.is_dir() {
        return Err(ApplicationError::MountPointNotDir(&args.mount_point));
    }
    verify_subvol(args)?;
    Ok(())
}

fn btrfs_show(path: &PathBuf) -> Result<std::process::Output, ApplicationError> {
    std::process::Command::new("btrfs")
        .arg("subvolume")
        .arg("show")
        .arg(path)
        .output()
        .map_err(|e| ApplicationError::FailedToSpawnCmd(e))
}

fn verify_subvol(args: &Args) -> Result<(), ApplicationError> {
    log::debug!("Verifying subvolume properties");

    let subvol_show = btrfs_show(&args.subvol_path)?;

    if args.verbose {
        let stdout = String::from_utf8_lossy(&subvol_show.stdout);
        if !stdout.is_empty() {
            log::info!("{}", stdout.trim());
        };
    };

    if !subvol_show.status.success() {
        if args.verbose {
            let stderr = String::from_utf8_lossy(&subvol_show.stderr);
            if !stderr.is_empty() {
                log::error!("{}", stderr.trim());
            };
        }
        return Err(ApplicationError::SubvolumeError);
    }

    Ok(())
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
        clap_complete::generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
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

    if let Err(e) = verify_path(&args) {
        log::error!("{}", e);
        return ExitCode::FAILURE;
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

    ExitCode::SUCCESS
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
