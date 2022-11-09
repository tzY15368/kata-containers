use async_trait::async_trait;

use super::Endpoint;
use kata_types::config::hypervisor::NetworkInfo;

#[derive(Debug)]
pub struct MACVTAPEndpoint {
    pub(crate) endpoint_properties: NetworkInfo,
    endpoint_type: &str,
    VM_fds,//list of fd
    VHost_fds,// list of fd
    pci_path// struct of pciPath
    // ratelimiter?
}

impl MACVTAPEndpoint {
    pub async fn new(
        //...
    )->Result<Self>{
        Ok(...)
    }
}

#[async_trait]
impl Endpoint for MACVTAPEndpoint {
    async fn name(&self) -> String {}

    async fn hardware_addr(&self) -> String {}

    async fn attach(&self, hypervisor: &dyn Hypervisor) -> Result<()> {}

    async fn detach(&self, _hypervisor: &dyn Hypervisor) -> Result<()> {}

    async fn save(&self) -> Option<EndpointState> {}
}
