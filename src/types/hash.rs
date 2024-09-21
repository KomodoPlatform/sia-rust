use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Error)]
pub enum ParseHashError {
    #[error("expected hex string prefixed with 'h:', found {0}")]
    InvalidPrefix(String),
    #[error("expected hex string, found {0}")]
    InvalidHex(String),
    #[error("expected 32 byte hex string, found {0}")]
    InvalidLength(String),
}
#[derive(Clone, Copy, PartialEq)]
pub struct Hash256(pub [u8; 32]);

impl Hash256 {
    const fn const_default() -> Hash256 { Hash256([0; 32]) }

    // Method for parsing a hex string without the "h:" prefix
    pub fn from_str_no_prefix(hex_str: &str) -> Result<Self, ParseHashError> {
        if hex_str.len() != 64 {
            return Err(ParseHashError::InvalidLength(hex_str.to_string()));
        }

        let mut bytes = [0u8; 32];
        match hex::decode_to_slice(hex_str, &mut bytes) {
            Ok(_) => Ok(Hash256(bytes)),
            Err(_) => Err(ParseHashError::InvalidHex(hex_str.to_string())),
        }
    }
}

impl Default for Hash256 {
    fn default() -> Self { Hash256::const_default() }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { write!(f, "h:{:02x}", self) }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { fmt::Display::fmt(self, f) }
}

impl fmt::LowerHex for Hash256 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Serialize for Hash256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hash256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct H256Visitor;

        impl<'de> serde::de::Visitor<'de> for H256Visitor {
            type Value = Hash256;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string prefixed with 'h:' and followed by a 32 byte hex string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Hash256::from_str(value).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &self))
            }
        }

        deserializer.deserialize_str(H256Visitor)
    }
}

impl FromStr for Hash256 {
    type Err = ParseHashError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(hex_str) = value.strip_prefix("h:") {
            Hash256::from_str_no_prefix(hex_str)
        } else {
            Err(ParseHashError::InvalidPrefix(value.to_string()))
        }
    }
}

impl TryFrom<&str> for Hash256 {
    type Error = ParseHashError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Hash256::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    cross_target_test! {
        fn test_placeholder_fixme() {
            let str_reversed = "h:00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
            match Hash256::from_str(str_reversed) {
                Ok(reversed_hash) => assert_eq!(format!("{:?}", reversed_hash), str_reversed),
                _ => panic!("unexpected"),
            }

            let str_reversed = "XXXYYY";
            if Hash256::from_str(str_reversed).is_ok() {
                panic!("unexpected");
            }
        }
    }
}
