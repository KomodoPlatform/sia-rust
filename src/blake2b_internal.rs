use crate::specifier::Specifier;
use crate::spend_policy::UnlockKey;
use crate::types::{Hash256, PublicKey};
use blake2b_simd::Params;
use std::default::Default;

#[cfg(test)] use hex;
#[cfg(test)] use std::convert::{TryFrom, TryInto};

const LEAF_HASH_PREFIX: [u8; 1] = [0u8];
const NODE_HASH_PREFIX: [u8; 1] = [1u8];

// Precomputed hash values used for all standard v1 addresses
// a standard address has 1 ed25519 public key, requires 1 signature and has a timelock of 0
// https://github.com/SiaFoundation/core/blob/b5b08cde6b7d0f1b3a6f09b8aa9d0b817e769efb/types/hash.go#L94
const STANDARD_TIMELOCK_BLAKE2B_HASH: [u8; 32] = [
    0x51, 0x87, 0xb7, 0xa8, 0x02, 0x1b, 0xf4, 0xf2, 0xc0, 0x04, 0xea, 0x3a, 0x54, 0xcf, 0xec, 0xe1, 0x75, 0x4f, 0x11,
    0xc7, 0x62, 0x4d, 0x23, 0x63, 0xc7, 0xf4, 0xcf, 0x4f, 0xdd, 0xd1, 0x44, 0x1e,
];

const STANDARD_SIGS_REQUIRED_BLAKE2B_HASH: [u8; 32] = [
    0xb3, 0x60, 0x10, 0xeb, 0x28, 0x5c, 0x15, 0x4a, 0x8c, 0xd6, 0x30, 0x84, 0xac, 0xbe, 0x7e, 0xac, 0x0c, 0x4d, 0x62,
    0x5a, 0xb4, 0xe1, 0xa7, 0x6e, 0x62, 0x4a, 0x87, 0x98, 0xcb, 0x63, 0x49, 0x7b,
];

// FIXME remove direct indexing of arrays or add sanity checks to prevent out of bound access

/// Directly ported from Sia core
/// https://github.com/SiaFoundation/core/blob/0f61e58ab7ea932da7e9f710c592d595406356c6/internal/blake2b/blake2b.go#L66
#[derive(Debug, PartialEq)]
pub struct Accumulator {
    trees: [Hash256; 64],
    num_leaves: u64,
}

impl Default for Accumulator {
    fn default() -> Self {
        // Initialize all bytes to zero
        Accumulator {
            trees: std::array::from_fn(|_| Hash256::default()),
            num_leaves: 0,
        }
    }
}

impl Accumulator {
    // Check if there is a tree at the given height
    fn has_tree_at_height(&self, height: u64) -> bool { self.num_leaves & (1 << height) != 0 }

    // Add a leaf to the accumulator
    pub fn add_leaf(&mut self, h: Hash256) {
        let mut i = 0;
        let mut new_hash = h;
        while self.has_tree_at_height(i) {
            new_hash = hash_blake2b_pair(&NODE_HASH_PREFIX, &self.trees[i as usize].0, &new_hash.0);
            i += 1;
        }
        self.trees[i as usize] = new_hash;
        self.num_leaves += 1;
    }

    // Calulate the root hash of the Merkle tree
    pub fn root(&self) -> Hash256 {
        // trailing_zeros determines the height Merkle tree accumulator where the current lowest single leaf is located
        let i = self.num_leaves.trailing_zeros() as u64;
        if i == 64 {
            return Hash256::default(); // Return all zeros if no leaves
        }
        let mut root = self.trees[i as usize].clone();
        for j in i + 1..64 {
            if self.has_tree_at_height(j) {
                root = hash_blake2b_pair(&NODE_HASH_PREFIX, &self.trees[j as usize].0, &root.0);
            }
        }
        root
    }
}

pub fn sigs_required_leaf(sigs_required: u64) -> Hash256 {
    let sigs_required_array: [u8; 8] = sigs_required.to_le_bytes();
    let mut combined = Vec::new();
    combined.extend_from_slice(&LEAF_HASH_PREFIX);
    combined.extend_from_slice(&sigs_required_array);

    hash_blake2b_single(&combined)
}

// public key leaf is
// blake2b(leafHashPrefix + 16_byte_ascii_algorithm_identifier + public_key_length_u64 + public_key)
pub fn public_key_leaf(unlock_key: &UnlockKey) -> Hash256 {
    let mut combined = Vec::new();
    combined.extend_from_slice(&LEAF_HASH_PREFIX);
    match unlock_key {
        UnlockKey::Ed25519(pubkey) => {
            combined.extend_from_slice(Specifier::Ed25519.as_bytes());
            combined.extend_from_slice(&32u64.to_le_bytes());
            combined.extend_from_slice(pubkey.as_bytes());
        },
        UnlockKey::NonStandard { algorithm, public_key } => {
            combined.extend_from_slice(algorithm.as_bytes());
            combined.extend_from_slice(&(public_key.len() as u64).to_le_bytes());
            combined.extend_from_slice(public_key);
        },
    }
    hash_blake2b_single(&combined)
}

pub fn timelock_leaf(timelock: u64) -> Hash256 {
    let timelock: [u8; 8] = timelock.to_le_bytes();
    let mut combined = Vec::new();
    combined.extend_from_slice(&LEAF_HASH_PREFIX);
    combined.extend_from_slice(&timelock);

    hash_blake2b_single(&combined)
}

// https://github.com/SiaFoundation/core/blob/b5b08cde6b7d0f1b3a6f09b8aa9d0b817e769efb/types/hash.go#L96
// An UnlockHash is the Merkle root of UnlockConditions. Since the standard
// UnlockConditions use a single public key, the Merkle tree is:
//
//           ┌─────────┴──────────┐
//     ┌─────┴─────┐              │
//  timelock     pubkey     sigsrequired
pub fn standard_unlock_hash(pubkey: &PublicKey) -> Hash256 {
    let pubkey_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey.clone()));
    let timelock_pubkey_node = hash_blake2b_pair(&NODE_HASH_PREFIX, &STANDARD_TIMELOCK_BLAKE2B_HASH, &pubkey_leaf.0);
    hash_blake2b_pair(
        &NODE_HASH_PREFIX,
        &timelock_pubkey_node.0,
        &STANDARD_SIGS_REQUIRED_BLAKE2B_HASH,
    )
}

pub fn hash_blake2b_single(preimage: &[u8]) -> Hash256 {
    let hash = Params::new().hash_length(32).to_state().update(preimage).finalize();
    let mut array = [0u8; 32];
    debug_assert!(hash.as_bytes().len() == 32);
    array.copy_from_slice(hash.as_bytes());
    Hash256(array)
}

fn hash_blake2b_pair(prefix: &[u8], leaf1: &[u8], leaf2: &[u8]) -> Hash256 {
    let hash = Params::new()
        .hash_length(32)
        .to_state()
        .update(prefix)
        .update(leaf1)
        .update(leaf2)
        .finalize();
    let mut array = [0u8; 32];
    debug_assert!(hash.as_bytes().len() == 32);
    array.copy_from_slice(hash.as_bytes());
    Hash256(array)
}

#[test]
fn test_accumulator_new() {
    let default_accumulator = Accumulator::default();

    let expected = Accumulator {
        trees: std::array::from_fn(|_| Hash256::default()),
        num_leaves: 0,
    };
    assert_eq!(default_accumulator, expected)
}

#[test]
fn test_accumulator_root_default() { assert_eq!(Accumulator::default().root(), Hash256::default()) }

#[test]
fn test_accumulator_root() {
    let mut accumulator = Accumulator::default();

    let timelock_leaf = timelock_leaf(0u64);
    accumulator.add_leaf(timelock_leaf);

    let pubkey = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();
    let pubkey_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey));
    accumulator.add_leaf(pubkey_leaf);

    let sigs_required_leaf = sigs_required_leaf(1u64);
    accumulator.add_leaf(sigs_required_leaf);

    let expected = Hash256::try_from("h:72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515d").unwrap();
    assert_eq!(accumulator.root(), expected);
}

#[test]
fn test_accumulator_add_leaf_standard_unlock_hash() {
    let mut accumulator = Accumulator::default();

    let pubkey = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();

    let pubkey_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey));
    let timelock_leaf = timelock_leaf(0u64);
    let sigs_required_leaf = sigs_required_leaf(1u64);

    accumulator.add_leaf(timelock_leaf);
    accumulator.add_leaf(pubkey_leaf);
    accumulator.add_leaf(sigs_required_leaf);

    let root = accumulator.root();
    let expected = Hash256::try_from("h:72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515d").unwrap();
    assert_eq!(root, expected)
}

#[test]
fn test_accumulator_add_leaf_2of2_multisig_unlock_hash() {
    let mut accumulator = Accumulator::default();

    let pubkey1 = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();
    let pubkey2 = PublicKey::from_bytes(
        &hex::decode("0101010000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();

    let pubkey1_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey1));
    let pubkey2_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey2));

    let timelock_leaf = timelock_leaf(0u64);
    let sigs_required_leaf = sigs_required_leaf(2u64);

    accumulator.add_leaf(timelock_leaf);
    accumulator.add_leaf(pubkey1_leaf);
    accumulator.add_leaf(pubkey2_leaf);
    accumulator.add_leaf(sigs_required_leaf);

    let root = accumulator.root();
    let expected = Hash256::try_from("h:1e94357817d236167e54970a8c08bbd41b37bfceeeb52f6c1ce6dd01d50ea1e7").unwrap();
    assert_eq!(root, expected)
}

#[test]
fn test_accumulator_add_leaf_1of2_multisig_unlock_hash() {
    let mut accumulator = Accumulator::default();

    let pubkey1 = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();
    let pubkey2 = PublicKey::from_bytes(
        &hex::decode("0101010000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();

    let pubkey1_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey1));
    let pubkey2_leaf = public_key_leaf(&UnlockKey::Ed25519(pubkey2));

    let timelock_leaf = timelock_leaf(0u64);
    let sigs_required_leaf = sigs_required_leaf(1u64);

    accumulator.add_leaf(timelock_leaf);
    accumulator.add_leaf(pubkey1_leaf);
    accumulator.add_leaf(pubkey2_leaf);
    accumulator.add_leaf(sigs_required_leaf);

    let root = accumulator.root();
    let expected = Hash256::try_from("h:d7f84e3423da09d111a17f64290c8d05e1cbe4cab2b6bed49e3a4d2f659f0585").unwrap();
    assert_eq!(root, expected)
}

#[test]
fn test_standard_unlock_hash() {
    let pubkey = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();

    let hash = standard_unlock_hash(&pubkey);
    let expected = Hash256::try_from("h:72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515d").unwrap();
    assert_eq!(hash, expected)
}

#[test]
fn test_hash_blake2b_pair() {
    let left: [u8; 32] = hex::decode("cdcce3978a58ceb6c8480d218646db4eae85eb9ea9c2f5138fbacb4ce2c701e3")
        .unwrap()
        .try_into()
        .unwrap();
    let right: [u8; 32] = hex::decode("b36010eb285c154a8cd63084acbe7eac0c4d625ab4e1a76e624a8798cb63497b")
        .unwrap()
        .try_into()
        .unwrap();

    let hash = hash_blake2b_pair(&NODE_HASH_PREFIX, &left, &right);
    let expected = Hash256::try_from("h:72b0762b382d4c251af5ae25b6777d908726d75962e5224f98d7f619bb39515d").unwrap();
    assert_eq!(hash, expected)
}

#[test]
fn test_timelock_leaf() {
    let hash = timelock_leaf(0);
    let expected = Hash256(STANDARD_TIMELOCK_BLAKE2B_HASH);
    assert_eq!(hash, expected)
}

#[test]
fn test_sigs_required_leaf() {
    let hash = sigs_required_leaf(1u64);
    let expected = Hash256(STANDARD_SIGS_REQUIRED_BLAKE2B_HASH);
    assert_eq!(hash, expected)
}

#[test]
fn test_hash_blake2b_single() {
    let hash = hash_blake2b_single(&hex::decode("006564323535313900000000000000000020000000000000000102030000000000000000000000000000000000000000000000000000000000").unwrap());
    let expected = Hash256::try_from("h:21ce940603a2ee3a283685f6bfb4b122254894fd1ed3eb59434aadbf00c75d5b").unwrap();
    assert_eq!(hash, expected)
}

#[test]
fn test_public_key_leaf() {
    let pubkey = PublicKey::from_bytes(
        &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
    )
    .unwrap();

    let hash = public_key_leaf(&UnlockKey::Ed25519(pubkey));
    let expected = Hash256::try_from("h:21ce940603a2ee3a283685f6bfb4b122254894fd1ed3eb59434aadbf00c75d5b").unwrap();
    assert_eq!(hash, expected)
}
