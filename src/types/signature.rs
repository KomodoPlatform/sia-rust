use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::fmt;
use thiserror::Error;
use ed25519_dalek::{Signature as Ed25519Signature, SIGNATURE_LENGTH};
use curve25519_dalek::edwards::CompressedEdwardsY;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("parsing error: expected 64 byte hex string ed25519 signature prefixed with 'sig:', found {0}")]
    Parse(#[from] ed25519_dalek::ed25519::Error),
    #[error("invalid prefix: expected 64 byte hex string ed25519 signature prefixed with 'sig:', found {0}")]
    InvalidPrefix(String),
    #[error("corrupt R point: expected 64 byte hex string ed25519 signature prefixed with 'sig:', found {0}")]
    CorruptRPoint(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Ed25519Signature);

impl Signature {
    pub fn new(signature: Ed25519Signature) -> Self { Signature(signature) }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        let signature = Ed25519Signature::from_bytes(bytes)
            .map(Signature)
            .map_err(SignatureError::Parse)?;

        match signature.validate_r_point() {
            true => Ok(signature),
            false => Err(SignatureError::CorruptRPoint(hex::encode(bytes))),
        }
    }

    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] { self.0.to_bytes() }

    // Method for parsing a hex string without the "sig:" prefix
    pub fn from_str_no_prefix(hex_str: &str) -> Result<Self, SignatureError> {
        let signature = Ed25519Signature::from_str(hex_str)
            .map(Signature)
            .map_err(SignatureError::Parse)?;

        match signature.validate_r_point() {
            true => Ok(signature),
            false => Err(SignatureError::CorruptRPoint(hex_str.to_string())),
        }
    }

    /// Check if R value is a valid point on the Ed25519 curve
    pub fn validate_r_point(&self) -> bool {
        let r_bytes = &self.0.to_bytes()[0..SIGNATURE_LENGTH/2];

        println!("r_bytes len: {}", r_bytes.len());
        // Create a CompressedEdwardsY point from the first 32 bytes
        CompressedEdwardsY::from_slice(r_bytes).decompress().is_some()
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

