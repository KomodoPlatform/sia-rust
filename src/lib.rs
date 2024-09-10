use derive_more::Display;
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey, SecretKey,
                    Signature as Ed25519Signature, SignatureError as Ed25519SignatureError, Signer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

pub mod blake2b_internal;
pub mod encoding;
pub mod hash;
pub mod http;
pub mod specifier;
pub mod spend_policy;
pub mod transaction;
pub mod types;

#[derive(Debug, Display)]
pub enum KeypairError {
    InvalidSecretKey(Ed25519SignatureError),
}

#[cfg(test)] mod tests;
#[cfg(test)]
#[macro_use]
extern crate serde_json;

pub struct Keypair(pub Ed25519Keypair);

impl Keypair {
    pub fn from_private_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        let secret = SecretKey::from_bytes(bytes).map_err(KeypairError::InvalidSecretKey)?;
        let public = Ed25519PublicKey::from(&secret);
        Ok(Keypair(Ed25519Keypair { secret, public }))
    }

    pub fn sign(&self, message: &[u8]) -> Signature { self.0.sign(message).into() }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Signature(pub Ed25519Signature);

impl Signature {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        let signature = Ed25519Signature::from_bytes(bytes).map_err(SignatureError::ParseError)?;
        Ok(Signature(signature))
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(signature: Ed25519Signature) -> Self { Signature(signature) }
}

impl Deref for Signature {
    type Target = Ed25519Signature;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Display)]
pub enum SignatureError {
    ParseError(ed25519_dalek::ed25519::Error),
    InvalidSignature(Ed25519SignatureError),
}

impl From<ed25519_dalek::ed25519::Error> for SignatureError {
    fn from(e: Ed25519SignatureError) -> Self { SignatureError::InvalidSignature(e) }
}

impl fmt::LowerHex for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", hex::encode(self.0.to_bytes())) }
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ed25519Signature::from_str(s)
            .map(Signature)
            .map_err(SignatureError::InvalidSignature)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
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

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", hex::encode(self.as_bytes())) }
}

impl Deref for Keypair {
    type Target = Ed25519Keypair;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Keypair {
    pub fn public(&self) -> PublicKey { PublicKey(self.0.public) }
}
