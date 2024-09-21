use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::fmt;
use thiserror::Error;
use ed25519_dalek::Signature as Ed25519Signature;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("expected 64 byte hex string prefixed with 'sig:', found {0}")]
    Parse(#[from] ed25519_dalek::ed25519::Error),
    #[error("expected 64 byte hex string prefixed with 'sig:',, found {0}")]
    InvalidPrefix(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Ed25519Signature);

impl Signature {
    pub fn new(signature: Ed25519Signature) -> Self { Signature(signature) }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        Ed25519Signature::from_bytes(bytes)
            .map(Signature)
            .map_err(SignatureError::Parse)
    }

    pub fn to_bytes(&self) -> [u8; 64] { self.0.to_bytes() }

    // Method for parsing a hex string without the "sig:" prefix
    pub fn from_str_no_prefix(hex_str: &str) -> Result<Self, SignatureError> {
        Ed25519Signature::from_str(hex_str)
            .map(Signature)
            .map_err(SignatureError::Parse)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SignatureVisitor;

        impl<'de> serde::de::Visitor<'de> for SignatureVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 64 byte hex string representing a ed25519 signature prefixed with 'sig:'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Signature::from_str(value).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
            }
        }

        deserializer.deserialize_str(SignatureVisitor)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "sig:{:x}", self.0) }
}

impl fmt::LowerHex for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Delegate to the fmt::LowerHex implementation of the inner Ed25519Signature
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(hex_str) = value.strip_prefix("sig:") {
            Signature::from_str_no_prefix(hex_str)
        } else {
            Err(SignatureError::InvalidPrefix(value.to_string()))
        }
    }
}

