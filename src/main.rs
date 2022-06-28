use lightrary::discovery::DiscoveryBroker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut bridge = DiscoveryBroker::new().await?.bridge().auth().await?;

    // bridge.lights(|lights| {
    //     lights.name("Hue Go 1").toggle();
    //     lights.name("Main J").toggle();
    // }).await?;

    let broker = DiscoveryBroker::discovery_endpoint();
    let bridges = broker.discover().await?;
    let (bridges, _failed) = bridges.auth().await.into();
    println!("{:?}", bridges);

    Ok(())
}
