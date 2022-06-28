use crate::resources::{Bridges, UnauthBridge};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
}

#[derive(Debug)]
pub struct AuthResult {
    pub success: Bridges,
    pub failed: Vec<AuthFailed>,
}

impl From<AuthResult> for (Bridges, Vec<AuthFailed>) {
    fn from(result: AuthResult) -> Self {
        (result.success, result.failed)
    }
}

#[derive(Debug)]
pub struct AuthFailed {
    pub bridge: UnauthBridge,
    pub err: Error,
}
