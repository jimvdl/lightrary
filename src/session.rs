use crate::error::Error;
use reqwest::Client;

// keeping this type private for the time being, until I figure out its place in the public
// api and what to do with the `danger_accept_invalid_certs`
#[derive(Debug)]
pub(crate) struct Session {
    inner: Client,
}

impl Session {
    pub fn new() -> Result<Self, Error> {
        let client = Client::builder()
            .https_only(true)
            // the resolver is pretty useless because when you connect through 
            // any other means than the discovery endpoint you won't have the bridge
            // id and I don't know any way to obtain one; perhaps file an issue with philips
            // .resolve(&bridge.id, format!("{}:443", bridge.ip).parse()?)

            // FIXME: there should be a better way of getting the bridge certs to work
            // without it blocking every single request we send its way
            // simply adding the root cert via pem file doesn't work here either
            .danger_accept_invalid_certs(true)
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
