use crate::blake2b_internal::hash_blake2b_single;
use crate::types::Hash256;

// https://github.com/SiaFoundation/core/blob/092850cc52d3d981b19c66cd327b5d945b3c18d3/types/encoding.go#L16
// TODO go implementation limits this to 1024 bytes, should we?
#[derive(Default)]
pub struct Encoder {
    pub buffer: Vec<u8>,
}

impl Encoder {
    pub fn reset(&mut self) { self.buffer.clear(); }

    /// writes a length-prefixed []byte to the underlying stream.
    pub fn write_len_prefixed_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(&(data.len() as u64).to_le_bytes());
        self.buffer.extend_from_slice(data);
    }

    // equivalent of Sia Core's EncodeSlice()
    pub fn write_len_prefixed_vec<T: Encodable>(&mut self, data: &Vec<T>) {
        self.write_u64(data.len() as u64);
        for item in data {
            item.encode(self);
        }
    }

    pub fn write_slice(&mut self, data: &[u8]) { self.buffer.extend_from_slice(data); }

    pub fn write_u8(&mut self, u: u8) { self.buffer.extend_from_slice(&[u]) }

    pub fn write_u64(&mut self, u: u64) { self.buffer.extend_from_slice(&u.to_le_bytes()); }

    pub fn write_u128(&mut self, u: u128) { self.buffer.extend_from_slice(&u.to_le_bytes()); }

    pub fn write_string(&mut self, p: &str) { self.write_len_prefixed_bytes(p.to_string().as_bytes()); }

    pub fn write_distinguisher(&mut self, p: &str) { self.buffer.extend_from_slice(format!("sia/{}|", p).as_bytes()); }

    pub fn write_bool(&mut self, b: bool) { self.buffer.push(b as u8) }

    pub fn hash(&self) -> Hash256 { hash_blake2b_single(&self.buffer) }

    // Utility method to create, encode, and hash
    pub fn encode_and_hash<T: Encodable>(item: &T) -> Hash256 {
        let mut encoder = Encoder::default();
        item.encode(&mut encoder);
        encoder.hash()
    }
}

pub trait Encodable {
    fn encode(&self, encoder: &mut Encoder);
}

impl Encodable for Hash256 {
    fn encode(&self, encoder: &mut Encoder) { encoder.write_slice(&self.0); }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    cross_target_tests! {
        fn test_encoder_default_hash() {
            assert_eq!(
                Encoder::default().hash(),
                Hash256::from_str("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8").unwrap()
            )
        }

        fn test_encoder_write_bytes() {
            let mut encoder = Encoder::default();
            encoder.write_len_prefixed_bytes(&[1, 2, 3, 4]);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("d4a72b52e2e1f40e20ee40ea6d5080a1b1f76164786defbb7691a4427f3388f5").unwrap()
            );
        }

        fn test_encoder_write_u8() {
            let mut encoder = Encoder::default();
            encoder.write_u8(1);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("ee155ace9c40292074cb6aff8c9ccdd273c81648ff1149ef36bcea6ebb8a3e25").unwrap()
            );
        }

        fn test_encoder_write_u64() {
            let mut encoder = Encoder::default();
            encoder.write_u64(1);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("1dbd7d0b561a41d23c2a469ad42fbd70d5438bae826f6fd607413190c37c363b").unwrap()
            );
        }

        fn test_encoder_write_distiguisher() {
            let mut encoder = Encoder::default();
            encoder.write_distinguisher("test");
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("25fb524721bf98a9a1233a53c40e7e198971b003bf23c24f59d547a1bb837f9c").unwrap()
            );
        }

        fn test_encoder_write_bool() {
            let mut encoder = Encoder::default();
            encoder.write_bool(true);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("ee155ace9c40292074cb6aff8c9ccdd273c81648ff1149ef36bcea6ebb8a3e25").unwrap()
            );
        }

        fn test_encoder_reset() {
            let mut encoder = Encoder::default();
            encoder.write_bool(true);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("ee155ace9c40292074cb6aff8c9ccdd273c81648ff1149ef36bcea6ebb8a3e25").unwrap()
            );

            encoder.reset();
            encoder.write_bool(false);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314").unwrap()
            );
        }

        fn test_encoder_complex() {
            let mut encoder = Encoder::default();
            encoder.write_distinguisher("test");
            encoder.write_bool(true);
            encoder.write_u8(1);
            encoder.write_len_prefixed_bytes(&[1, 2, 3, 4]);
            assert_eq!(
                encoder.hash(),
                Hash256::from_str("b66d7a9bef9fb303fe0e41f6b5c5af410303e428c4ff9231f6eb381248693221").unwrap()
            );
        }
    }
}
