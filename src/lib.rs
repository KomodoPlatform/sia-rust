use derive_more::Display;
use std::ops::Deref;
use std::fmt;
use serde::{Deserialize, Serialize};
pub use ed25519_dalek::{SecretKey, Signature};
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey, SignatureError};

pub mod blake2b_internal;
pub mod encoding;
pub mod hash;
pub mod http_client;
pub mod http_endpoints;
pub mod specifier;
pub mod spend_policy;
pub mod transaction;
pub mod types;

#[derive(Debug, Display)]
pub enum KeypairError {
    InvalidSecretKey(SignatureError),
}

#[cfg(test)] mod tests;
#[cfg(test)]
#[macro_use]
extern crate serde_json;

pub struct Keypair(pub Ed25519Keypair);

impl Keypair {
    pub fn from_private_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        let secret = SecretKey::from_bytes(&bytes).map_err(|e|KeypairError::InvalidSecretKey(e))?;
        let public = Ed25519PublicKey::from(&secret);
        Ok(Keypair( Ed25519Keypair{secret, public}))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PublicKey(pub Ed25519PublicKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        let public_key = Ed25519PublicKey::from_bytes(bytes)?;
        Ok(PublicKey(public_key))
    }
}

impl From<Ed25519PublicKey> for PublicKey {
    fn from(public_key: Ed25519PublicKey) -> Self { PublicKey(public_key) }
}

impl Deref for PublicKey {
    type Target = Ed25519PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_bytes()))
    }
}

impl Deref for Keypair {
    type Target = Ed25519Keypair;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Keypair {
    pub fn public(&self) -> PublicKey {
        PublicKey(self.0.public.clone())
    }
}