use crate::resources::UnauthBridges;
use async_trait::async_trait;

#[async_trait]
pub trait Discoverer {
    async fn discover(&self) -> Result<UnauthBridges, Box<dyn std::error::Error>>;
}

// TODO: mDNS
// TODO: manual (although i don't know what route to use)

#[derive(Debug)]
pub struct DiscoveryEndpoint;

#[async_trait]
impl Discoverer for DiscoveryEndpoint {
    async fn discover(&self) -> Result<UnauthBridges, Box<dyn std::error::Error>> {
        reqwest::get("https://discovery.meethue.com")
            .await?
            .json::<UnauthBridges>()
            .await
            .map_err(|e| e.into())
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
    pub async fn discover(&self) -> Result<UnauthBridges, Box<dyn std::error::Error>> {
        self.discoverer.discover().await
    }
}

impl DiscoveryBroker<DiscoveryEndpoint> {
    pub fn discovery_endpoint() -> Self {
        let discoverer = DiscoveryEndpoint;

        Self { discoverer }
    }
}
