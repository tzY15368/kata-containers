use anyhow::{anyhow, Result, Ok};
use std::{collections::HashMap, path::PathBuf};
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

fn join_path(volume_path:&String)->PathBuf{
    // TODO: check if we should use escape or encode
    let urlencoded_vol_path = base64_url::escape(volume_path).to_string();
    let mount_info_file_path = Path::new(KATA_DIRECT_VOLUME_ROOT_PATH).join(urlencoded_vol_path);
    return mount_info_file_path;
}


// add writes the mount info (json string) of a direct volume into a filesystem path known to Kata Container.
pub fn add(volume_path:&String, mount_info:&String)->Result<()>{
    let mount_info_dir_path = join_path(volume_path);
    // check if path exists and is dir, if not, create this dir
    if !mount_info_dir_path.exists(){
        fs::create_dir_all(&mount_info_dir_path)?;
    } else if !mount_info_dir_path.is_dir(){
        return Err(anyhow!("{} is not a directory", mount_info_dir_path.to_str().unwrap()));
    }


    // This behavior of deserializing and serializing comes from `src/runtime/pkg/direct-volume/utils.go/add`
    // Assuming that this is for the purpose of validating the json schema.
    let unserialized_mount_info:VolumeMountInfo = serde_json::from_str(mount_info)?;

    let mount_info_file_path = mount_info_dir_path.join(MOUNT_INFO_FILE_NAME);
    let serialized_mount_info = serde_json::to_string(&unserialized_mount_info)?;
    fs::write(mount_info_file_path, serialized_mount_info)?;
    Ok(())
}

// remove deletes the direct volume path including all the files inside it.
pub fn remove(volume_path:&String)->Result<()>{
    let path = join_path(volume_path);
    // removes path and any children it contains.
    fs::remove_dir_all(path)?;
    Ok(())
}

pub fn get_volumn_mount_info(volume_path:&String)->Result<VolumeMountInfo>{
    let mount_info_file_path = join_path(volume_path);
    let mount_info_file = fs::read_to_string(mount_info_file_path)?;
    let mount_info: VolumeMountInfo = serde_json::from_str(&mount_info_file)?;
    return Ok(mount_info);
}

// get_sandbox_id_for_volume finds the id of the first sandbox found in the dir.
// We expect a direct-assigned volume is associated with only a sandbox at a time.
pub fn get_sandbox_id_for_volume(volume_path:&String)->Result<String> {
    let dir_path = join_path(volume_path);
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