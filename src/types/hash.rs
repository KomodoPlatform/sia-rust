use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseHashError {
    #[error("invalid prefix: expected 32 byte hex string prefixed with 'h:', found {0}")]
    InvalidPrefix(String),
    #[error("invalid hex: expected 32 byte hex string prefixed with 'h:', found {0}")]
    InvalidHex(String),
    #[error("invalid length: expected 32 byte hex string prefixed with 'h:', found {0}")]
    InvalidLength(String),
    #[error("invalid slice length: expected 32 byte slice, found {0} byte slice")]
    InvalidSliceLength(usize),
}
#[derive(Clone, Eq, PartialEq)]
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

    fn try_from(value: &str) -> Result<Self, Self::Error> { Hash256::from_str(value) }
}

impl TryFrom<&[u8]> for Hash256 {
    type Error = ParseHashError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let slice_len = slice.len();
        if slice_len == 32 {
            let mut array = [0u8; 32];
            array.copy_from_slice(slice);
            Ok(Hash256(array))
        } else {
            Err(ParseHashError::InvalidSliceLength(slice_len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    cross_target_tests! {
        fn test_default() {
            let hash = Hash256::try_from("h:0000000000000000000000000000000000000000000000000000000000000000").unwrap();
            assert_eq!(hash, Hash256::default());
        }

        fn test_valid() {
            let hash = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").unwrap();
            assert_eq!(hash.to_string(), "h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        }

        fn test_display() {
            let hash = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").unwrap();
            assert_eq!(hash.to_string(), "h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        }

        fn test_debug() {
            let hash = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").unwrap();
            assert_eq!(format!("{:?}", hash), "h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        }

        fn test_serialize() {
            let hash = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").unwrap();
            let serialized = serde_json::to_string(&hash).unwrap();
            assert_eq!(&serialized, r#""h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee""#);
        }

        fn test_deserialize() {
            let hash = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").unwrap();
            let deserialized: Hash256 = serde_json::from_str(r#""h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee""#).unwrap();
            assert_eq!(deserialized, hash);
        }

        fn test_deserialize_missing_prefix() {
            let err  = serde_json::from_str::<Hash256>(r#""c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee""#).expect_err("no prefix");
            assert!(format!("{:?}", err).contains("expected a string prefixed with 'h:' and followed by a 32 byte hex string"));
        }

        fn test_missing_prefix() {
            let test_case = "c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            let err = Hash256::try_from(test_case).expect_err("no prefix");
            match err {
                ParseHashError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_corrupt_prefix() {
            let test_case = ":c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            let err = Hash256::try_from(test_case).expect_err("no prefix");
            match err {
                ParseHashError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_wrong_prefix() {
            let test_case = "i:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            let err = Hash256::try_from(test_case).expect_err("wrong prefix");
            match err {
                ParseHashError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_invalid_hex() {
            let err = Hash256::try_from("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeg").expect_err("no prefix");
            let expected = "c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeg";
            match err {
                ParseHashError::InvalidHex(ref e) if expected == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_invalid_length() {
            let err = Hash256::try_from("h:badc0de").expect_err("invalid length");
            let expected = "badc0de";
            match err {
                ParseHashError::InvalidLength(ref e) if expected == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_valid() {
            let hash = Hash256::from_str_no_prefix("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
            assert_eq!(hash, Hash256::default())
        }

        fn test_from_str_no_prefix_invalid_length() {
            let err = Hash256::from_str_no_prefix("badc0de").expect_err("invalid length");
            let expected = "badc0de";
            match err {
                ParseHashError::InvalidLength(ref e) if expected == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_invalid_hex() {
            let err = Hash256::from_str_no_prefix("g00000000000000000000000000000000000000000000000000000000000000e").expect_err("invalid hex");
            let expected = "g00000000000000000000000000000000000000000000000000000000000000e";
            match err {
                ParseHashError::InvalidHex(ref e) if expected == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_invalid_has_prefix() {
            let err = Hash256::from_str_no_prefix("h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee").expect_err("invalid hex");
            let expected = "h:c0ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
            match err {
                ParseHashError::InvalidLength(ref e) if expected == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }
    }
}
