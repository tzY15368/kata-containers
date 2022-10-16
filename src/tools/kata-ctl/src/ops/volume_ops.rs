use crate::args::{DirectVolSubcommand, DirectVolumeCommand};

use anyhow::{Ok, anyhow, Result};
use hyper;
use runtimes::MgmtClient;
use super::volume_shared;
use std::time::Duration;
use url;
use futures::executor;

const timeout: Duration = Duration::new(5, 0);

pub fn handle_direct_volume(vol_cmd: DirectVolumeCommand) -> Result<()> {
    let command = vol_cmd.directvol_cmd;
    let volume_path = vol_cmd.volume_path;
    match command {
        DirectVolSubcommand::Add => {}
        DirectVolSubcommand::Remove => {}
        DirectVolSubcommand::Resize => {
            let resize_size = vol_cmd.resize_size.unwrap();
            let resize_result = executor::block_on(resize(&volume_path, resize_size))?;
        }
        DirectVolSubcommand::Stats => {
            let stats = executor::block_on(get_volume_stats(&volume_path))?;
            println!("{}", stats);
        }
    }
    Ok(())
}

async fn resize(volume_path: &String, size: u64) -> Result<()> {
    let sandbox_id = volume_shared::get_sandbox_id_for_volume(volume_path)?;
    let mount_info = volume_shared::get_volumn_mount_info(volume_path)?;
    let resize_req = volume_shared::ResizeRequest {
        size: size,
        volume_path: volume_path.to_string(),
    };
    let encoded = serde_json::to_string(&resize_req)?;
    let shim_client = MgmtClient::new(sandbox_id, Some(timeout)).unwrap();

    // post_json(url, body)
    let url = volume_shared::DIRECT_VOLUME_RESIZE_URL;
    let response = shim_client.post_json(url, encoded).await?;
    if response.status() != hyper::StatusCode::OK {
        return Err(anyhow!("status {}: failed to resize volume", response.status()));
    }
    return Ok(());
}

async fn get_volume_stats(volume_path: &String) -> Result<String> {
    let sandbox_id = volume_shared::get_sandbox_id_for_volume(volume_path)?;
    let mount_info = volume_shared::get_volumn_mount_info(volume_path)?;
    
    let req_url = url::form_urlencoded::Serializer::new(String::from(volume_shared::DIRECT_VOLUME_STAT_URL))
        .append_pair(volume_shared::DIRECT_VOLUME_PATH_KEY,&mount_info.device).finish();
        
    // sid: sandbox id
    let shim_client = MgmtClient::new(sandbox_id, Some(timeout)).unwrap();
    let response = shim_client.get(&req_url).await?;
    let status_code = response.status();
    // turn body into string
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let body = String::from_utf8(body.to_vec()).unwrap();
    if status_code != hyper::StatusCode::OK {
        return Err(anyhow!("status {}: failed to get volume stats: {}", status_code, body));
    }
    return Ok(body);
}
