use crate::resources::{Bridge, Bridges, UnauthBridge};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Mdns(#[from] mdns::Error),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    GenKey(#[from] GenKeyError),
}

#[derive(Debug)]
pub struct AuthResult {
    pub success: Bridge,
    pub failed: AuthFailed,
}

#[derive(Debug)]
pub struct AuthResults {
    pub success: Bridges,
    pub failed: Vec<AuthFailed>,
}

impl From<AuthResults> for (Bridges, Vec<AuthFailed>) {
    fn from(result: AuthResults) -> Self {
        (result.success, result.failed)
    }
}

#[derive(Debug)]
pub struct AuthFailed {
    pub bridge: UnauthBridge,
    pub err: Error,
}

#[derive(Debug, Deserialize)]
pub enum GenKeyResult {
    #[serde(rename = "success")]
    Success(GenKeySuccess),
    #[serde(rename = "error")]
    Error(GenKeyError),
}

#[derive(Debug, Deserialize)]
pub struct GenKeySuccess {}

#[derive(Error, Debug, Clone, Deserialize)]
#[error("link button not pressed")]
pub struct GenKeyError {
    #[serde(rename = "type")]
    pub t: i32,
    pub address: String,
    pub description: String,
}
