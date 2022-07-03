use crate::error::Error;
use crate::resources::{UnauthBridge, UnauthBridges};
use async_trait::async_trait;
use std::net::Ipv4Addr;

#[async_trait]
pub trait Discoverer {
    type Device;

    async fn discover(&self) -> Result<Self::Device, Error>;
}

// TODO: mDNS

#[derive(Debug)]
pub struct DiscoveryEndpoint;

#[async_trait]
impl Discoverer for DiscoveryEndpoint {
    type Device = UnauthBridges;

    async fn discover(&self) -> Result<Self::Device, Error> {
        reqwest::get("https://discovery.meethue.com")
            .await?
            .json::<UnauthBridges>()
            .await
            .map_err(|e| e.into())
    }
}

// maybe add a Ipv4Addr to manual conversion method
#[derive(Debug)]
pub struct Manual(Ipv4Addr);

#[async_trait]
impl Discoverer for Manual {
    type Device = UnauthBridge;

    async fn discover(&self) -> Result<Self::Device, Error> {
        Ok(UnauthBridge {
            id: None,
            ip: self.0,
            port: 443,
        })
    }
}

#[derive(Debug)]
pub struct DiscoveryBroker<D>
where
    D: Discoverer,
{
    discoverer: D,
}

impl<D> DiscoveryBroker<D>
where
    D: Discoverer,
{
    pub async fn discover(&self) -> Result<D::Device, Error> {
        self.discoverer.discover().await
    }
}

impl DiscoveryBroker<DiscoveryEndpoint> {
    pub fn discovery_endpoint() -> Self {
        let discoverer = DiscoveryEndpoint;

        Self { discoverer }
    }
}

impl DiscoveryBroker<Manual> {
    pub fn manual(ip: Ipv4Addr) -> Self {
        let discoverer = Manual(ip);

        Self { discoverer }
    }
}
