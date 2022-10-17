use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use base64_url;
use serde::{Deserialize,Serialize};

const KATA_DIRECT_VOLUME_ROOT_PATH: &str = "/run/kata-containers/shared/direct-volumes";
const MOUNT_INFO_FILE_NAME: &str = "mountInfo.json";

pub const DIRECT_VOLUME_STAT_URL: &str   = "/direct-volume/stats";
pub const DIRECT_VOLUME_RESIZE_URL: &str = "/direct-volume/resize";
pub const DIRECT_VOLUME_PATH_KEY:&str   = "path";

#[derive(Serialize, Deserialize)]
pub struct VolumeMountInfo {
    pub volume_type: String,
    pub device: String,
    pub fs_type: String,
    pub metadata: HashMap<String, String>,
    pub options: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct ResizeRequest {
    pub size: u64,
    pub volume_path: String,
}

pub fn get_volumn_mount_info(volume_path:&String)->Result<VolumeMountInfo>{
    // TODO: check if we should use escape or encode
    let urlencoded_vol_path = base64_url::escape(volume_path).to_string();
    let mount_info_file_path = Path::new(KATA_DIRECT_VOLUME_ROOT_PATH).join(urlencoded_vol_path);
    let mount_info_file = fs::read_to_string(mount_info_file_path)?;
    let mount_info: VolumeMountInfo = serde_json::from_str(&mount_info_file)?;
    return Ok(mount_info);
}

pub fn get_sandbox_id_for_volume(volume_path:&String)->Result<String> {
    // TODO: SAME ABOVE
    let urlencoded_vol_path = base64_url::escape(volume_path).to_string();
    let dir_path = Path::new(KATA_DIRECT_VOLUME_ROOT_PATH).join(urlencoded_vol_path);
    let paths = fs::read_dir(dir_path)?;
    for path in paths {
        let path = path?;
        // compare with MOUNT_INFO_FILE_NAME
        if path.file_name() == MOUNT_INFO_FILE_NAME {
            return Ok(String::from(path.file_name().to_str().unwrap()));
        }
    }
    return Err(anyhow!("no sandbox found for {}".to_string()))?;
}