use crate::error::Error;
use crate::resources::UnauthBridge;
use reqwest::Client;

#[derive(Debug, Default)]
pub(crate) struct Session {
    inner: Client,
}

impl Session {
    pub fn new(bridge: &UnauthBridge) -> Result<Self, Error> {
        let client = Client::builder()
            .https_only(true)
            .resolve(&bridge.id, format!("{}:443", bridge.ip).parse()?)
            .build()?;

        Ok(Self { inner: client })
    }
}

impl std::ops::Deref for Session {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
