use super::{NetworkModel, NetworkModelType};
use crate::network::NetworkPair;
use anyhow::{Context, Result};
use async_trait::async_trait;

#[derive(Debug)]
pub(crate) struct MacVTapModel {}

impl MacVTapModel {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl NetworkModel for MacVTapModel {
    fn model_type(&self) -> super::NetworkModelType {
        NetworkModelType::MacVTap
    }

    async fn add(&self, pair: &NetworkPair) -> Result<()> {
        Ok(())
    }

    async fn del(&self, pair: &NetworkPair) -> Result<()> {
        Ok(())
    }
}
