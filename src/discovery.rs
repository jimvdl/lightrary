//! Discover one or multiple bridges on your local network with a variety of protocols.
//!
//! # Discovery protocols and their usage:
//!
//! When in doubt which protocol to use always prioritize mDNS followed by the
//! discovery-endpoint and finally manual discovery.
//!
//! mDNS is truely local without any outside connectivity required and has no request limit,
//! whereas the hue endpoint does require a outside connection due to a cloud dependency
//! and should limit requests to 1 request every 15 minutes. Manual should only be considered
//! if neither mDNS nor the endpoint show your bridge(s), manual also doesn't need a request limit.
//!
//! [tower-limit](https://crates.io/crates/tower-limit) is a great request limiter for the `DiscoveryBroker<DiscoveryEndpoint>`.
//!
//! | Protocol | Characteristics | Priority |
//! |----------|--------------|----------|
//! | mDNS     | Truely local: no outside connection (wanted or available) without a request limit.  | mDNS over discovery-endpoint |
//! | Discovery-endpoint | Cloud dependant, should limit requests to 1 request every 15 minutes. | Discovery-endpoint over manual |
//! | Manual | As a last resort when all others fail, no request limit needed. | Fallback |
//!
//! # Examples
//!
//! Example using the mDNS protocol:
//! ```no_run
//! use lightrary::discovery::DiscoveryBroker;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let broker = DiscoveryBroker::mdns();
//! let bridges = broker.discover().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Example using the discovery endpoint:
//! ```no_run
//! use lightrary::discovery::DiscoveryBroker;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let broker = DiscoveryBroker::discovery_endpoint();
//! // note: you should request limit this to once every 15 minutes.
//! let bridges = broker.discover().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Example using manual connection:
//! ```no_run
//! use lightrary::discovery::DiscoveryBroker;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let broker = DiscoveryBroker::manual("192.168.50.173".parse()?);
//! // note: manual only supports one bridge at a time, possible list
//! // might be supported in the future.
//! let bridge = broker.discover().await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::resources::{UnauthBridge, UnauthBridges};
use async_trait::async_trait;
use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Record, RecordKind};
use std::net::Ipv4Addr;
use std::str::FromStr;
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
    // TODO: find a way to support a longer search duration without early return
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
/// Note: should be limited to 1 request every 15 minutes.
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
///
/// An [`Ipv4Addr`](std::net::Ipv4Addr) can also be converted to a `DiscoveryBroker<Manual>`
/// using the [`From`](std::convert::From) trait.
///
/// ```no_run
/// use lightrary::discovery::DiscoveryBroker;
/// use std::net::Ipv4Addr;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manual_broker = DiscoveryBroker::from(Ipv4Addr::new(192, 168, 50, 173));
/// # Ok(())
/// # }
/// ```
///
/// You can also directly convert a ip str into a `DiscoveryBroker<Manual>` using
/// [`FromStr`](std::str::FromStr):
/// ```no_run
/// use lightrary::discovery::{DiscoveryBroker, Manual};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manual_broker: DiscoveryBroker<Manual> = "192.168.50.173".parse()?;
/// # Ok(())
/// # }
/// ```
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
    /// Note: should be limited to 1 request every 15 minutes.
    pub fn discovery_endpoint() -> Self {
        let discoverer = DiscoveryEndpoint;

        Self { discoverer }
    }
}

impl DiscoveryBroker<Manual> {
    /// Creates a discovery broker with the manually entered IP connecting
    /// straight to your bridge.
    ///
    /// Note: only use this if no other protocol works.
    pub fn manual(ip: Ipv4Addr) -> Self {
        let discoverer = Manual(ip);

        Self { discoverer }
    }
}

impl From<Ipv4Addr> for DiscoveryBroker<Manual> {
    fn from(ip: Ipv4Addr) -> Self {
        Self {
            discoverer: Manual(ip),
        }
    }
}

impl FromStr for DiscoveryBroker<Manual> {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            discoverer: Manual(s.parse()?),
        })
    }
}
