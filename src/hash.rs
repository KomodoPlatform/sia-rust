//! Fixed-size hashes
use rustc_hex::{FromHex, FromHexError, ToHex};
use std::hash::{Hash, Hasher};
use std::{cmp, fmt, ops, str};
use serde;
use serde::de::Unexpected;
use std::cmp::Ordering;
use std::str::FromStr;

macro_rules! impl_global_hash {
    ($name: ident, $size: expr) => {
        #[derive(Copy)]
        #[repr(C)]
        pub struct $name([u8; $size]);

        impl Default for $name {
            fn default() -> Self { $name([0u8; $size]) }
        }

        impl AsRef<$name> for $name {
            fn as_ref(&self) -> &$name { self }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] { &self.0 }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                let mut result = Self::default();
                result.copy_from_slice(&self.0);
                result
            }
        }

        impl From<[u8; $size]> for $name {
            fn from(h: [u8; $size]) -> Self { $name(h) }
        }

        impl From<$name> for [u8; $size] {
            fn from(h: $name) -> Self { h.0 }
        }

        impl<'a> From<&'a [u8]> for $name {
            fn from(slc: &[u8]) -> Self {
                let mut inner = [0u8; $size];
                inner[..].clone_from_slice(&slc[0..$size]);
                $name(inner)
            }
        }

        impl From<&'static str> for $name {
            fn from(s: &'static str) -> Self { s.parse().unwrap() }
        }

        impl From<u8> for $name {
            fn from(v: u8) -> Self {
                let mut result = Self::default();
                result.0[0] = v;
                result
            }
        }

        impl str::FromStr for $name {
            type Err = FromHexError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let vec: Vec<u8> = s.from_hex()?;
                match vec.len() {
                    $size => {
                        let mut result = [0u8; $size];
                        result.copy_from_slice(&vec);
                        Ok($name(result))
                    },
                    _ => Err(FromHexError::InvalidHexLength),
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str(&self.0.to_hex::<String>()) }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str(&self.0.to_hex::<String>()) }
        }

        impl ops::Deref for $name {
            type Target = [u8; $size];

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }

        impl cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref == other_ref
            }
        }

        impl cmp::PartialEq<&$name> for $name {
            fn eq(&self, other: &&Self) -> bool {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref == other_ref
            }
        }

        impl cmp::PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref.partial_cmp(other_ref)
            }
        }

        impl Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: Hasher,
            {
                state.write(&self.0);
                state.finish();
            }
        }

        impl Eq for $name {}

        impl $name {
            pub fn take(self) -> [u8; $size] { self.0 }

            pub fn as_slice(&self) -> &[u8] { &self.0 }

            pub fn reversed(&self) -> Self {
                let mut result = self.clone();
                result.reverse();
                result
            }

            pub fn size() -> usize { $size }

            pub fn is_zero(&self) -> bool { self.0.iter().all(|b| *b == 0) }
        }
    };
}

macro_rules! impl_hash {
    ($name: ident, $other: ident, $size: expr) => {
        /// Hash serialization
        #[derive(Clone, Copy)]
        pub struct $name(pub [u8; $size]);

        impl $name {
            pub const fn const_default() -> $name { $name([0; $size]) }
        }

        impl Default for $name {
            fn default() -> Self { $name::const_default() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { write!(f, "{:02x}", self) }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> { write!(f, "{:02x}", self) }
        }

        impl<T> From<T> for $name
        where
            $other: From<T>,
        {
            fn from(o: T) -> Self { $name($other::from(o).take()) }
        }

        impl FromStr for $name {
            type Err = <$other as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let other = $other::from_str(s)?;
                Ok($name(other.take()))
            }
        }

        #[allow(clippy::from_over_into)]
        impl Into<$other> for $name {
            fn into(self) -> $other { $other::from(self.0) }
        }

        #[allow(clippy::from_over_into)]
        impl Into<Vec<u8>> for $name {
            fn into(self) -> Vec<u8> { self.0.to_vec() }
        }

        impl Eq for $name {}

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref.cmp(other_ref)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref == other_ref
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                let self_ref: &[u8] = &self.0;
                let other_ref: &[u8] = &other.0;
                self_ref.partial_cmp(other_ref)
            }
        }

        impl Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: Hasher,
            {
                $other::from(self.0.clone()).hash(state)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut hex = String::new();
                hex.push_str(&$other::from(self.0.clone()).to_hex::<String>());
                serializer.serialize_str(&hex)
            }
        }

        impl<'a> serde::Deserialize<'a> for $name {
            fn deserialize<D>(deserializer: D) -> Result<$name, D::Error>
            where
                D: serde::Deserializer<'a>,
            {
                struct HashVisitor;

                impl<'b> serde::de::Visitor<'b> for HashVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a hash string")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if value.len() != $size * 2 {
                            return Err(E::invalid_value(Unexpected::Str(value), &self));
                        }

                        match value[..].from_hex::<Vec<u8>>() {
                            Ok(ref v) => {
                                let mut result = [0u8; $size];
                                result.copy_from_slice(v);
                                Ok($name($other::from(result).take()))
                            },
                            _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
                        }
                    }

                    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        self.visit_str(value.as_ref())
                    }
                }

                deserializer.deserialize_identifier(HashVisitor)
            }
        }

        impl ::core::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                for i in &self.0[..] {
                    write!(f, "{:02x}", i)?;
                }
                Ok(())
            }
        }
    };
}

impl H256 {
    #[inline]
    pub fn reversed(&self) -> Self {
        let mut result = *self;
        result.0.reverse();
        result
    }
}

impl GlobalH256 {
    #[inline]
    pub fn from_reversed_str(s: &'static str) -> Self { GlobalH256::from(s).reversed() }

    #[inline]
    pub fn to_reversed_str(self) -> String { self.reversed().to_string() }
}

impl_global_hash!(GlobalH256, 32);
impl_hash!(H256, GlobalH256, 32);

#[cfg(test)]
mod tests {
    use super::{GlobalH256, H256};
    use std::str::FromStr;

    #[test]
    fn hash_debug() {
        let str_reversed = "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
        let reversed_hash = H256::from(str_reversed);
        let debug_result = format!("{:?}", reversed_hash);
        assert_eq!(debug_result, str_reversed);
    }

    #[test]
    fn hash_from_str() {
        let str_reversed = "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
        match H256::from_str(str_reversed) {
            Ok(reversed_hash) => assert_eq!(format!("{:?}", reversed_hash), str_reversed),
            _ => panic!("unexpected"),
        }

        let str_reversed = "XXXYYY";
        if H256::from_str(str_reversed).is_ok() {
            panic!("unexpected");
        }
    }

    #[test]
    fn hash_to_global_hash() {
        let str_reversed = "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
        let reversed_hash = H256::from(str_reversed);
        let global_hash = GlobalH256::from(str_reversed);
        let global_converted: GlobalH256 = reversed_hash.into();
        assert_eq!(global_converted, global_hash);
    }
}