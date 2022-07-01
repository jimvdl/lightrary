// use lightrary::discovery::DiscoveryBroker;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // let mut bridge = DiscoveryBroker::new().await?.bridge().auth().await?;

//     // bridge.lights(|lights| {
//     //     lights.name("Hue Go 1").toggle();
//     //     lights.name("Main J").toggle();
//     // }).await?;

//     let broker = DiscoveryBroker::discovery_endpoint();
//     println!("1");
//     let bridges = broker.discover().await?;
//     println!("2");
//     let (bridges, _failed) = bridges.auth().await.into();
//     println!("3");
//     let bridge = bridges.into_singular();
//     println!("4");
//     let (bridge, app_key) = bridge.gen_key("test_app", "yeey").await?;


//     // bridge.lights(|lights| {

//     // }).await?;

//     Ok(())
// }

use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Error, Record, RecordKind};
use std::{net::IpAddr, time::Duration};


const SERVICE_NAME: &'static str = "_hue._tcp.local";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Iterate through responses from each Cast device, asking for new devices every 15s
    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(15))?.listen();
    pin_mut!(stream);

    while let Some(Ok(response)) = stream.next().await {
        println!("{:?}", response);
        let addr = response.records()
                           .filter_map(self::to_ip_addr)
                           .next();

        if let Some(addr) = addr {
            println!("found cast device at {}", addr);
        } else {
            println!("cast device does not advertise address");
        }
    }

    Ok(())
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
