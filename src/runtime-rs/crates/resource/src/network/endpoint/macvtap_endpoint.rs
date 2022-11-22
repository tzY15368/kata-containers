use async_trait::async_trait;

use super::endpoint_persist::{EndpointState, MacVTapEndpointState};
use super::Endpoint;
use crate::network::network_model::MACVTAP_NET_MODEL_STR;
use crate::network::{utils, NetworkPair};
use anyhow::{Context, Ok, Result};
use hypervisor::{device::NetworkConfig, Device, Hypervisor};
use std::fs::{self, File};
use std::io::{self, Error};
use std::os::unix::prelude::OpenOptionsExt;

const DEFAULT_FILE_PERMS: u32 = 0o660;

// no telemetry ?

#[derive(Debug)]
pub struct MacVTapEndpoint {
    net_pair: NetworkPair,
    pub(crate) vm_fds: Vec<File>,
    pub(crate) vhost_fds: Vec<File>,
    // pci path??
}

impl MacVTapEndpoint {
    pub async fn new(
        handle: &rtnetlink::Handle,
        name: &str,
        idx: u32,
        queues: usize,
    ) -> Result<Self> {
        let net_pair = NetworkPair::new(handle, idx, name, MACVTAP_NET_MODEL_STR, queues)
            .await
            .context("create network pair for macvtap")?;
        Ok(MacVTapEndpoint {
            net_pair,
            vm_fds: vec![],
            vhost_fds: vec![],
        })
    }

    // duplicate code everywhere!
    fn get_network_config(&self) -> Result<NetworkConfig> {
        let iface = &self.net_pair.tap.tap_iface;
        let guest_mac = utils::parse_mac(&iface.hard_addr).ok_or_else(|| {
            Error::new(
                io::ErrorKind::InvalidData,
                format!("hard_addr {}", &iface.hard_addr),
            )
        })?;
        Ok(NetworkConfig {
            id: self.net_pair.virt_iface.name.clone(),
            host_dev_name: iface.name.clone(),
            guest_mac: Some(guest_mac),
        })
    }
}

async fn create_mac_vtap_fds(link_index: u64, queues: i32) -> Result<Vec<File>> {
    let tap_device = format!("/dev/tap{}", link_index);
    create_fds(&tap_device, queues).await
}

async fn create_vhost_fds(num_fds: i32) -> Result<Vec<File>> {
    let vhost_device = "/dev/vhost-net";
    create_fds(&vhost_device, num_fds).await
}

async fn create_fds(device: &str, num_fds: i32) -> Result<Vec<File>> {
    // is this even right?

    // create an array of fds
    let mut fds = Vec::new();
    for _ in 0..num_fds {
        // open file with permission 0600
        let fd = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .mode(DEFAULT_FILE_PERMS)
            .open(device)?;
        fds.push(fd);
    }
    Ok(fds)
}

#[async_trait]
impl Endpoint for MacVTapEndpoint {
    async fn name(&self) -> String {
        // netlink interface.name
    }

    async fn hardware_addr(&self) -> String {
        // netlink interface.hardware addr
    }

    async fn attach(&self, hypervisor: &dyn Hypervisor) -> Result<()> {
        let h_config = hypervisor.hypervisor_config().await;
        let num_vcpus = h_config.cpu_info.default_vcpus;
        //TODO: FIXME
        let idx: i32 = 1;
        //TODO: FIXME
        let iface_name: &str = &self.net_pair.tap.name;
        self.vm_fds = create_mac_vtap_fds(idx, num_vcpus)
            .await
            .context(format!("setup macvtap fds {}", iface_name))?;
        if !h_config.network_info.disable_vhost_net {
            // why don't we rollback the vm_fds creation?
            self.vhost_fds = create_vhost_fds(num_vcpus)
                .await
                .context(format!("setup vhost fds {}", iface_name))?;
        }
        let config = self
            .get_network_config()
            .context("error getting network config")?;
        hypervisor
            .add_device(Device::Network(config))
            .await
            .context("add device to hypervisor")
    }

    async fn detach(&self, hypervisor: &dyn Hypervisor) -> Result<()> {
        // macvttap_endpoint did nothing here??
        self.net_pair.del_network_model().await?;
        let config = self
            .get_network_config()
            .context("error getting network config")?;
        hypervisor
            .remove_device(Device::Network(config))
            .await
            .context("remove device from hypervisor")
    }

    async fn save(&self) -> Option<EndpointState> {
        Some(EndpointState {
            macvtap_endpoint: Some(MacVTapEndpointState{
                //TODO: FIXME
            }),
            ..Default::default()
        })
    }
}
