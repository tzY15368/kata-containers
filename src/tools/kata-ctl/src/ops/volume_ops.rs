use crate::args::{DirectVolSubcommand, DirectVolumeCommand};

use anyhow::{Ok, anyhow, Result};
use hyper;
use runtimes;
use std::time::Duration;
use url;
use futures::executor;

const TIMEOUT: Duration = Duration::new(5, 0);
const CONTENT_TYPE_JSON :&str = "application/json";

pub fn handle_direct_volume(vol_cmd: DirectVolumeCommand) -> Result<()> {
    let command = vol_cmd.directvol_cmd;
    match command {
        DirectVolSubcommand::Add(args)=>{
            runtimes::direct_volume::add(&args.volume_path, &args.mount_info)?;
        },
        DirectVolSubcommand::Remove(args)=>{
            runtimes::direct_volume::remove(&args.volume_path)?;
        },
        DirectVolSubcommand::Stats(args)=>{
            let result = executor::block_on(stats(&args.volume_path))?;
            println!("{}", &result);
        },
        DirectVolSubcommand::Resize(args)=>{
            executor::block_on(resize(&args.volume_path, args.resize_size))?;
        },
    } 
    Ok(())
}

async fn resize(volume_path: &String, size: u64) -> Result<()> {
    let sandbox_id = runtimes::direct_volume::get_sandbox_id_for_volume(volume_path)?;
    let mount_info = runtimes::direct_volume::get_volumn_mount_info(volume_path)?;
    let resize_req = runtimes::direct_volume::ResizeRequest { 
        size,
        volume_path: mount_info.device,
    };
    let encoded = serde_json::to_string(&resize_req)?;
    let shim_client = runtimes::MgmtClient::new(sandbox_id, Some(TIMEOUT)).unwrap();

    // post_json(url, body)
    let url = runtimes::direct_volume::DIRECT_VOLUME_RESIZE_URL;
    let response = shim_client.post(url, &String::from(CONTENT_TYPE_JSON), &encoded).await?;
    if response.status() != hyper::StatusCode::OK {
        return Err(anyhow!("status {}: failed to resize volume", response.status()));
    }
    return Ok(());
}

async fn stats(volume_path: &String) -> Result<String> {
    let sandbox_id = runtimes::direct_volume::get_sandbox_id_for_volume(volume_path)?;
    let mount_info = runtimes::direct_volume::get_volumn_mount_info(volume_path)?;

    let req_url = url::form_urlencoded::Serializer::new(String::from(runtimes::direct_volume::DIRECT_VOLUME_STAT_URL))
        .append_pair(runtimes::direct_volume::DIRECT_VOLUME_PATH_KEY,&mount_info.device).finish();
        
    // sid: sandbox id
    let shim_client = runtimes::MgmtClient::new(sandbox_id, Some(TIMEOUT)).unwrap();
    shim_client.get(&req_url).await?;
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
