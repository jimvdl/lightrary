// use crate::auth::Authorize;
use crate::error::{AuthFailed, AuthResults, Error, GenKeyResult};
use crate::resources::light::Lights;
use crate::session::Session;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::ops::{Deref, DerefMut};

// TODO: remove, dynamically get this when authenticating bridge
// const USER: &'static str = "JGQOy1ADXKSa3uuJNZDv5xcGrD9t-AHgoEXki-6a";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UnauthBridges(pub(crate) Vec<UnauthBridge>);

impl UnauthBridges {
    pub async fn auth(self) -> AuthResults {
        let mut results = AuthResults {
            success: Bridges(Vec::new()),
            failed: Vec::new(),
        };
        for bridge in self {
            let session = match Session::new() {
                Ok(session) => session,
                Err(e) => {
                    results.failed.push(AuthFailed { bridge, err: e });
                    continue;
                }
            };
            let res = match session
                .get(format!("https://{}/api/0/config", bridge.ip))
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
                app_key: None,
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
    pub(crate) id: Option<String>,
    #[serde(rename = "internalipaddress")]
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
}

impl UnauthBridge {
    pub async fn auth(self) -> Result<Bridge, (UnauthBridge, Error)> {
        let session = match Session::new() {
            Ok(session) => session,
            Err(e) => return Err((self, e)),
        };
        let res = match session
            .get(format!("https://{}/api/0/config", self.ip))
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
            app_key: None,
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

impl Bridges {
    // concept, might change this
    // make sure that the user knows if the list is empty panic and if you have more than one panic
    pub fn into_singular(mut self) -> Bridge {
        self.0.remove(0)
    }
}

impl Deref for Bridges {
    type Target = Vec<Bridge>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bridges {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Bridge {
    pub(crate) id: Option<String>,
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) app_key: Option<String>,
    pub(crate) session: Session,
    pub(crate) config: BridgeConfig,
}

impl Bridge {
    pub fn with_key(mut self, app_key: String) -> Self {
        self.app_key = Some(app_key);
        self
    }

    pub async fn gen_key(
        self,
        app_name: &str,
        instance_name: &str,
    ) -> Result<(Self, String), Error> {
        let app_key = match &self
            .session
            .post(format!("https://{}/api", &self.ip))
            .json(&crate::resources::device::DeviceType {
                app_name,
                instance_name,
            })
            .send()
            .await?
            .json::<Vec<GenKeyResult>>()
            .await?[0]
        {
            GenKeyResult::Success(s) => {
                println!("hallo");
                String::new()
            }
            GenKeyResult::Error(e) => {
                return Err(e.clone().into());
            }
        };

        Ok((
            Self {
                app_key: Some(app_key.clone()),
                ..self
            },
            app_key,
        ))
    }
}

// // TODO: rate limiting on all the requests
// impl Bridge {
//     pub async fn lights<F>(&mut self, f: F) -> Result<(), Error>
//     where
//         F: FnOnce(&mut Lights),
//     {
//         // maybe cache these, lazy loading
//         // they should only need to be reloaded once a new light is added or removed
//         // when that happens, just invalidate this cache and recache/reload all lights
//         // to get an updated list
//         let mut lights = self.session
//             .get(format!("https://{}/api/{}/lights", self.id, self.app_key))
//             .header("hue-application-key", &self.app_key)
//             .send()
//             .await?
//             .json::<Lights>()
//             .await?;

//         f(&mut lights);

//         for (id, light) in lights.0.iter_mut().filter(|(_, l)| l.change.is_some()) {
//             println!("sending {} with state {:?}", id, light.change);
//             self.session
//                 .put(format!(
//                     "https://{}/api/{}/lights/{}/state",
//                     self.id, &self.app_key, id
//                 ))
//                 .json(&light.change)
//                 .send()
//                 .await
//                 .unwrap();
//         }

//         Ok(())
//     }
// }
