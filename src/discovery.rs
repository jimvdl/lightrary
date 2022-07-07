use crate::error::Error;
use crate::resources::{UnauthBridge, UnauthBridges};
use async_trait::async_trait;
use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Record, RecordKind};
use std::net::Ipv4Addr;
use std::{net::IpAddr, time::Duration};

/// Interchangeable discovery protocol for the `DiscoveryBroker`.
#[async_trait]
pub trait Discoverer {
    type Device;

    async fn discover(&self) -> Result<Self::Device, Error>;
}

/// Discovers all bridges present on your local network through Multicast DNS or 
/// [mDNS](https://en.wikipedia.org/wiki/Multicast_DNS) for short.
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

/// Discovery protocol that uses Philips' discovery endpoint: <https://discovery.meethue.com> 
/// 
/// TODO: document request limit and actually implement it
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

/// Manually discover your bridge on a local network. 
/// 
/// You can find your bridge's IP on your router. If not found check the connectivity
/// of the bridge and see if the second LED is on. (which signifies the network connection state)
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

/// Broker which discovers your bridge(s) on your local network.
/// 
/// | Protocol           | Associated function                                                          |
/// |--------------------|------------------------------------------------------------------------------|
/// | mDNS               | [`DiscoveryBroker::mdns`](DiscoveryBroker::mdns)                             |
/// | Discovery Endpoint | [`DiscoveryBroker::discovery_endpoint`](DiscoveryBroker::discovery_endpoint) |
/// | Manual             | [`DiscoveryBroker::manual`](DiscoveryBroker::manual)                         |
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
    /// Use the currently selected discovery method; mDNS, discovery-endpoint or manual,
    /// to discover your bridge(s) on your local network.
    /// 
    /// Either yields the found device(s) or network connectivity errors or mDNS errors.
    pub async fn discover(&self) -> Result<D::Device, Error> {
        self.discoverer.discover().await
    }
}

impl DiscoveryBroker<Mdns> {
    /// Creates a discovery broker with the mDNS protocol.
    pub fn mdns() -> Self {
        let discoverer = Mdns(UnauthBridges::default());

        Self { discoverer }
    }
}

impl DiscoveryBroker<DiscoveryEndpoint> {
    /// Creates a discovery broker with the discovery-endpoint access.
    /// 
    /// Note: TODO: throughput rates.
    pub fn discovery_endpoint() -> Self {
        let discoverer = DiscoveryEndpoint;

        Self { discoverer }
    }
}

impl DiscoveryBroker<Manual> {
    /// Creates a discovery broker with the manually entered IP connecting
    /// straight to your bridge.
    pub fn manual(ip: Ipv4Addr) -> Self {
        let discoverer = Manual(ip);

        Self { discoverer }
    }
}
