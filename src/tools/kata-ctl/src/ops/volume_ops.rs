use crate::args::{DirectVolSubcommand, DirectVolumeCommand};

use anyhow::{Ok, Result};
use hyper;
use runtimes::MgmtClient;
use super::volume_shared;
use std::time::Duration;
use url;
use futures::executor;


pub fn handle_direct_volume(vol_cmd: DirectVolumeCommand) -> Result<()> {
    let command = vol_cmd.directvol_cmd;
    let volume_path = vol_cmd.volume_path;
    match command {
        DirectVolSubcommand::Add => {}
        DirectVolSubcommand::Remove => {}
        DirectVolSubcommand::Resize => {}
        DirectVolSubcommand::Stats => {
            let stats = executor::block_on(get_volume_stats(&volume_path))?;
            println!("{}", stats);
        }
    }
    Ok(())
}

async fn get_volume_stats(volume_path: &String) -> Result<String> {
    let timeout = Duration::new(5, 0);
    let sandbox_id = volume_shared::get_sandbox_id_for_volume(volume_path)?;

    let mount_info = volume_shared::get_volumn_mount_info(volume_path)?;

    // 	fmt.Sprintf("%s?%s=%s", containerdshim.DirectVolumeStatUrl, containerdshim.DirectVolumePathKey, urlSafeDevicePath))
    let req_url = url::form_urlencoded::Serializer::new(String::from(volume_shared::DIRECT_VOLUME_STAT_URL))
        .append_pair(volume_shared::DIRECT_VOLUME_PATH_KEY,&mount_info.device).finish();
        
    // sid: sandbox id
    let shim_client = MgmtClient::new(sandbox_id, Some(timeout)).unwrap();
    let response = shim_client.get(&req_url).await?;
    // turn body into string
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let body = String::from_utf8(body.to_vec()).unwrap();
    return Ok(body);
}
