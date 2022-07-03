use crate::error::Error;
use crate::resources::{UnauthBridge, UnauthBridges};
use async_trait::async_trait;
use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Record, RecordKind};
use std::net::Ipv4Addr;
use std::{net::IpAddr, time::Duration};

#[async_trait]
pub trait Discoverer {
    type Device;

    async fn discover(&self) -> Result<Self::Device, Error>;
}

#[derive(Debug)]
pub struct Mdns(UnauthBridges);

#[async_trait]
impl Discoverer for Mdns {
    type Device = UnauthBridges;

    // might possible miss available bridges on the network due to early return during polling
    // can't really test because I only have one bridge.
    async fn discover(&self) -> Result<Self::Device, Error> {
        let stream = mdns::discover::all("_hue._tcp.local", Duration::from_millis(150))?.listen();
        let mut bridges = UnauthBridges::default();
        pin_mut!(stream);

        match stream.next().await {
            Some(Ok(response)) => {
                let addr = response.records().filter_map(self::to_ip_addr).next();

                if let Some(addr) = addr {
                    match addr {
                        IpAddr::V4(ip) => bridges.0.push(UnauthBridge {
                            id: None,
                            ip,
                            port: 443,
                        }),
                        _ => unreachable!("ipv6 not yet supported."),
                    }
                }
            }
            Some(Err(e)) => return Err(e.into()),
            None => return Ok(bridges),
        }

        Ok(bridges)
    }
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

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

impl DiscoveryBroker<Mdns> {
    pub fn mdns() -> Self {
        let discoverer = Mdns(UnauthBridges::default());

        Self { discoverer }
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
