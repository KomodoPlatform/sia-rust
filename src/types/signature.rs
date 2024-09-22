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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    const VALID_STR: &str = "sig:f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
    const VALID_JSON_STR: &str = r#""sig:f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902""#;

    fn valid_signature() -> Signature {
        Signature::from_str(VALID_STR).unwrap()
    }

    cross_target_tests! {
        fn test_display() {
            assert_eq!(valid_signature().to_string(), VALID_STR);
        }

        fn test_debug() {
            assert_eq!(format!("{:?}", valid_signature()), "Signature(ed25519::Signature(F43380794A6384E3D24D9908143C05DD37AAAC8959EFB65D986FEB70FE289A5E26B84E0AC712AF01A2F85F8727DA18AAE13A599A51FB066D098591E40CB26902))");
        }

        fn test_serialize() {
            assert_eq!(&serde_json::to_string(&valid_signature()).unwrap(), VALID_JSON_STR);
        }

        fn test_deserialize() {
            assert_eq!(serde_json::from_str::<Signature>(VALID_JSON_STR).unwrap(), valid_signature());
        }

        fn test_deserialize_missing_prefix() {
            let err  = serde_json::from_str::<Signature>(r#""f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902""#).expect_err("no prefix");
            let mystr = format!("{:?}", err);
            assert!(mystr.contains("expected a 64 byte hex string representing a ed25519 signature prefixed with 'sig:'"));
        }

        fn test_missing_prefix() {
            let test_case = "f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_corrupt_prefix() {
            let test_case = ":f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_wrong_prefix() {
            let test_case = "dig:f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::InvalidPrefix(ref e) if test_case == e => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_invalid_hex() {
            let test_case = "sig:g43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::Parse(_) => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_invalid_r_signature() {
            let test_case = "sig:00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::CorruptRPoint(_) => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_invalid_length() {
            let test_case = "sig:badc0de";
            let err = Signature::from_str(test_case).expect_err("no prefix");
            match err {
                SignatureError::Parse(e) => println!("{:?}", e),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_valid() {
            let sig = Signature::from_str_no_prefix("f43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902").unwrap();
            assert_eq!(sig, valid_signature())
        }

        fn test_from_str_no_prefix_invalid_length() {
            let test_case = "badc0de";
            let err = Signature::from_str_no_prefix(test_case).expect_err("invalid length");
            match err {
                SignatureError::Parse(_) => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_invalid_hex() {
            let test_case = "g43380794a6384e3d24d9908143c05dd37aaac8959efb65d986feb70fe289a5e26b84e0ac712af01a2f85f8727da18aae13a599a51fb066d098591e40cb26902";
            let err = Signature::from_str_no_prefix(test_case).expect_err("invalid hex");
            match err {
                SignatureError::Parse(_) => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }

        fn test_from_str_no_prefix_invalid_has_prefix() {
            let err = Signature::from_str_no_prefix(VALID_STR).expect_err("invalid hex");
            match err {
                SignatureError::Parse(_) => (),
                _ => panic!("unexpected error: {:?}", err),
            }
        }
    }
}
