use chrono::NaiveDateTime;
use derive_builder::Builder;
use log;
use std::{path::PathBuf, process::Command};

use crate::errors::ApplicationError;

pub(crate) struct SubvolumeInfo<'a>(std::borrow::Cow<'a, str>);

#[derive(Builder)]
#[builder(pattern = "owned")]
pub(crate) struct Subvolume {
    pub(crate) name: String,
    pub(crate) uuid: String,
    pub(crate) snapshots: Vec<PathBuf>,
    pub(crate) creation_time: chrono::NaiveDateTime,
}

pub(crate) fn get_subvol(subvol_path: &PathBuf) -> Result<Subvolume, ApplicationError> {
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
