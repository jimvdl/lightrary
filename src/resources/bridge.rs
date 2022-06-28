// use crate::auth::Authorize;
use crate::error::{AuthFailed, AuthResult, Error};
use crate::resources::light::Lights;
use crate::session::Session;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

// TODO: remove, dynamically get this when authenticating bridge
// const USER: &'static str = "JGQOy1ADXKSa3uuJNZDv5xcGrD9t-AHgoEXki-6a";

#[derive(Debug, Serialize, Deserialize)]
pub struct UnauthBridges(pub(crate) Vec<UnauthBridge>);

impl UnauthBridges {
    pub async fn auth(self) -> AuthResult {
        let mut results = AuthResult {
            success: Bridges(Vec::new()),
            failed: Vec::new(),
        };
        for bridge in self {
            let session = match Session::new(&bridge) {
                Ok(session) => session,
                Err(e) => {
                    results.failed.push(AuthFailed { bridge, err: e });
                    continue;
                }
            };
            let res = match session
                .get(format!("https://{}/api/0/config", bridge.id))
                .send()
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    results.failed.push(AuthFailed {
                        bridge,
                        err: e.into(),
                    });
                    continue;
                }
            };
            let config = match res.json::<BridgeConfig>().await {
                Ok(config) => config,
                Err(e) => {
                    results.failed.push(AuthFailed {
                        bridge,
                        err: e.into(),
                    });
                    continue;
                }
            };
            results.success.0.push(Bridge {
                id: bridge.id,
                ip: bridge.ip,
                port: bridge.port,
                session,
                config,
            })
        }
        results
    }
}

impl IntoIterator for UnauthBridges {
    type Item = UnauthBridge;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnauthBridge {
    pub(crate) id: String,
    #[serde(rename = "internalipaddress")]
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
}

impl UnauthBridge {
    pub async fn auth(self) -> Result<Bridge, (UnauthBridge, Error)> {
        let session = match Session::new(&self) {
            Ok(session) => session,
            Err(e) => return Err((self, e)),
        };
        let res = match session
            .get(format!("https://{}/api/0/config", self.id))
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => return Err((self, e.into())),
        };
        let config = match res.json::<BridgeConfig>().await {
            Ok(config) => config,
            Err(e) => return Err((self, e.into())),
        };
        Ok(Bridge {
            id: self.id,
            ip: self.ip,
            port: self.port,
            session,
            config,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct BridgeConfig {
    name: String,
    datastoreversion: String,
    swversion: String,
    apiversion: String,
    mac: String,
    pub(crate) bridgeid: String,
    factorynew: bool,
    replacebridgeid: Option<String>,
    modelid: String,
    starterkitid: Option<String>,
}

#[derive(Debug)]
pub struct Bridges(pub(crate) Vec<Bridge>);

#[derive(Debug)]
pub struct Bridge {
    pub(crate) id: String,
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) session: Session,
    pub(crate) config: BridgeConfig,
}

// // TODO: rate limiting on all the requests
// impl Bridge {
//     pub async fn lights<F>(&mut self, f: F) -> Result<(), Box<dyn std::error::Error>>
//     where
//         F: FnOnce(&mut Lights),
//     {
//         // maybe cache these, lazy loading
//         // they should only need to be reloaded once a new light is added or removed
//         // when that happens, just invalidate this cache and recache/reload all lights
//         // to get an updated list
//         let client = reqwest::Client::builder()
//             .https_only(true)
//             .resolve(&self.id, format!("{}:443", self.ip).parse()?)
//             .build()?;
//         let mut lights = client
//             .get(format!("https://{}/api/{}/lights", self.id, USER,))
//             .header("hue-application-key", USER)
//             .send()
//             .await?
//             .json::<Lights>()
//             .await?;

//         f(&mut lights);

//         // let mut buf = Vec::new();
//         // std::fs::File::open("C:/Users/Jim/Desktop/hue_bridge.pem")?.read_to_end(&mut buf)?;
//         // let client = reqwest::Client::builder()
//         //     .add_root_certificate(reqwest::tls::Certificate::from_pem(&buf)?)
//         //     .build()?;
//         for (id, light) in lights.0.iter_mut().filter(|(_, l)| l.change.is_some()) {
//             println!("sending {} with state {:?}", id, light.change);
//             client
//                 .put(format!(
//                     "https://{}/api/{}/lights/{}/state",
//                     self.id, USER, id
//                 ))
//                 .json(&light.change)
//                 .send()
//                 .await
//                 .unwrap();
//         }

//         Ok(())
//     }
// }
