use derive_more::Display;
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey, SecretKey,
                    SignatureError as Ed25519SignatureError, Signer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

use crate::types::{Signature, SignatureError}; // FIXME remove this when we move Keypair

#[derive(Debug, Display)]
pub enum KeypairError {
    InvalidSecretKey(Ed25519SignatureError),
}

pub struct Keypair(pub Ed25519Keypair);

impl Keypair {
    pub fn from_private_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        let secret = SecretKey::from_bytes(bytes).map_err(KeypairError::InvalidSecretKey)?;
        let public = Ed25519PublicKey::from(&secret);
        Ok(Keypair(Ed25519Keypair { secret, public }))
    }

    pub fn sign(&self, message: &[u8]) -> Signature { Signature::new(self.0.sign(message)) }
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
