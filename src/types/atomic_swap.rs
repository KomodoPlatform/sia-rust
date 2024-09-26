use crate::types::{Address, SatisfiedPolicy, SpendPolicy};
use thiserror::Error;

/*
The full representation of the atomic swap contract is as follows:
    SpendPolicy::Threshold { 
        n: 1,
        of: vec![ 
            SpendPolicy::Threshold { 
                n: 2, 
                of: vec![
                    SpendPolicy::Hash(<sha256(secret)>),
                    SpendPolicy::PublicKey(<Alice pubkey>)
                ] 
            },
            SpendPolicy::Threshold { 
                n: 2, 
                of: vec![
                    SpendPolicy::After(<timestamp>),
                    SpendPolicy::PublicKey(<Bob pubkey>)
                ] 
            },
        ]
    }

In English, the above specifies that:
    - Alice can spend the UTXO if:
        - Alice provides the preimage of the SHA256 hash specified in the UTXO (the secret)
        - Alice provides a valid signature
    - Bob can spend the UTXO if:
        - the current time is greater than the specified timestamp
        - Bob provides a valid signature

To lock funds in such a contract, we generate the address(see SpendPolicy::address) of the above SpendPolicy and use this Address in a transaction output.

The resulting UTXO can then be spent by either Alice or Bob by meeting the conditions specified above.

It is only neccesary to reveal the path that will be satisfied. The other path will be opacified(see SpendPolicy::opacify) and replaced with SpendPolicy::Opaque(<hash of unused path>).

Alice can spend the UTXO by providing a signature, the secret and revealing the relevant path within the full SpendPolicy.

Alice can construct the following SatisfiedPolicy to spend the UTXO:

SatisfiedPolicy {
    policy: SpendPolicy::Threshold { 
                n: 1,
                of: vec![ 
                    SpendPolicy::Threshold { 
                        n: 2, 
                        of: vec![
                            SpendPolicy::Hash(<sha256(secret)>),
                            SpendPolicy::PublicKey(<Alice pubkey>)
                        ] 
                    },
                    SpendPolicy::Opaque(<hash of unused path>),
                ]
            },
    signatures: vec![<Alice signature>],
    preimages: vec![<secret>]
}

Similarly, Bob can spend the UTXO with the following SatisfiedPolicy assuming he waits until the timestamp has passed:

SatisfiedPolicy {
    policy: SpendPolicy::Threshold { 
                n: 1,
                of: vec![ 
                    SpendPolicy::Threshold { 
                        n: 2, 
                        of: vec![
                            SpendPolicy::After(<timestamp>),
                            SpendPolicy::PublicKey(<Bob pubkey>)
                        ] 
                    },
                    SpendPolicy::Opaque(<hash of unused path>),
                ]
            },
    signatures: vec![<Bob signature>],
    preimages: vec![<secret>]
}

*/

/// Represents a validated SpendPolicy. Each unique structure of SpendPolicy should implement this trait.
/// The stored SpendPolicy must not be exposed via pub
/// otherwise a consumer can initialize with an invalid SpendPolicy. ie, AtomicSwap(invalid_policy)
trait IsValidatedSpendPolicy {
    type Error;

    // allow reference to inner policy because it is not public
    fn policy(&self) -> &SpendPolicy;

    fn is_valid(policy: &SpendPolicy) -> Result<(), Self::Error>;
}

#[derive(Debug, Error)]
pub enum AtomicSwapError {
    #[error("invalid atomic swap, invalid hash component: {}", .0)]
    InvalidHashComponent(ComponentError),
    #[error("invalid atomic swap, invalid time component: {}", .0)]
    InvalidTimeComponent(ComponentError),
    #[error("invalid atomic swap, wrong n:{} policy: {:?}", n, policy)]
    InvalidN{ policy: SpendPolicy, n : u8 },
    #[error("invalid atomic swap, wrong m:{} policy: {:?}", m, policy)]
    InvalidM{ policy: SpendPolicy, m : usize },
    #[error("invalid atomic swap, not a threshold: {:?}", .0)]
    InvalidSpendPolicyVariant(SpendPolicy),
}

/// Represents an atomic swap contract.
/// SpendPolicy:address is used to generate the address of the contract.
/// Funds can then be locked in the contract by creating a transaction output with this address.
/// This is used only to create outputs, never inputs./// The order of the SpendPolicys within a SpendPolicy::Threshold have no meaningful impact on logic, but we enforce a strict structure for simplicity.
#[derive(Debug)]
pub struct AtomicSwap(SpendPolicy);

impl AtomicSwap {
    pub fn new(policy: SpendPolicy) -> Result<Self, AtomicSwapError> {
        Self::is_valid(&policy).map(|_| Self(policy))
    }

    pub fn address(&self) -> Address {
        self.policy().address()
    }

    pub fn opacify(&self) -> SpendPolicy {
        self.policy().opacify()
    }
}

impl IsValidatedSpendPolicy for AtomicSwap {
    type Error = AtomicSwapError;

    fn policy(&self) -> &SpendPolicy {
        &self.0
    }

    fn is_valid(policy: &SpendPolicy) -> Result<(), Self::Error> {
        match policy {
            SpendPolicy::Threshold { 
                n: 1,
                of
            } if of.len() == 2 => {
                HashLockPath::is_valid(&of[0]).map_err(AtomicSwapError::InvalidHashComponent)?;
                TimeLockPath::is_valid(&of[1]).map_err(AtomicSwapError::InvalidTimeComponent)?;
                Ok(())
            },
            SpendPolicy::Threshold { n: 1, of } => Err(AtomicSwapError::InvalidM{ policy: policy.clone(), m: of.len() }),
            SpendPolicy::Threshold { n, of: _ } => Err(AtomicSwapError::InvalidN{ policy: policy.clone(), n: n.clone() }),
            _ => Err(AtomicSwapError::InvalidSpendPolicyVariant(policy.clone())),
        }
    }
}

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("invalid hash lock component, hash lock path: {:?}", .0)]
    HashLockInvalidThresholdStructure(SpendPolicy),
    #[error("invalid hash lock component, not a threshold: {:?}", .0)]
    HashLockInvalidSpendPolicyVariant(SpendPolicy),
    #[error("invalid hash lock component, wrong n:{} policy: {:?}", n, policy)]
    HashLockInvalidN{ policy: SpendPolicy, n : u8 },
    #[error("invalid hash lock component, wrong m:{} policy: {:?}", m, policy)]
    HashLockInvalidM{ policy: SpendPolicy, m : usize },
    #[error("invalid time lock component, time lock path: {:?}", .0)]
    TimeLockInvalidThresholdStructure(SpendPolicy),
    #[error("invalid time lock component, not a threshold: {:?}", .0)]
    TimeLockInvalidSpendPolicyVariant(SpendPolicy),
    #[error("invalid time lock component, wrong n:{} policy: {:?}", n, policy)]
    TimeLockInvalidN{ policy: SpendPolicy, n : u8 },
    #[error("invalid time lock component, wrong m:{} policy: {:?}", m, policy)]
    TimeLockInvalidM{ policy: SpendPolicy, m : usize },
}

/// The hash locked threshold path of the atomic swap contract.
/// 2 of 2 threshold of:
///     SpendPolicy::Hash(<secret_hash>) and SpendPolicy::PublicKey(<participant's public key>)
/// where:
///     secret_hash == sha256(secret)
/// fulfillment conditions:
///     - signature from participant's public key
///     - sha256(secret) == hash
///     - length(secret) == 32
#[derive(Debug)]
pub struct HashLockPath(SpendPolicy);

impl HashLockPath {
    pub fn new(policy: SpendPolicy) -> Result<Self, ComponentError> {
        Self::is_valid(&policy).map(|_| Self(policy))
    }

}

impl IsValidatedSpendPolicy for HashLockPath {
    type Error = ComponentError;

    fn policy(&self) -> &SpendPolicy {
        &self.0
    }

    fn is_valid(policy: &SpendPolicy) -> Result<(), Self::Error> {
        match policy {
            SpendPolicy::Threshold{ n: 2, of } if of.len() == 2 => {
                match of.as_slice() {
                    [SpendPolicy::Hash(_), SpendPolicy::PublicKey(_)] => Ok(()),
                    _ => Err(ComponentError::HashLockInvalidThresholdStructure(policy.clone())),
                }
            },
            SpendPolicy::Threshold{ n: 2, of } => Err(ComponentError::HashLockInvalidM{ policy: policy.clone(), m: of.len() }),
            SpendPolicy::Threshold{ n, of: _ } => Err(ComponentError::HashLockInvalidN{ policy: policy.clone(), n: n.clone() }),
            _ => Err(ComponentError::HashLockInvalidSpendPolicyVariant(policy.clone())),
        }
    }
}

/// The time based threshold path of the atomic swap contract.
/// 2 of 2 threshold of SpendPolicy::After(timestamp) and SpendPolicy::PublicKey(participant's public key)
/// fulfillment conditions:
/// - signature from participant's public key
/// - timestamp has passed
#[derive(Debug)]
pub struct TimeLockPath(SpendPolicy);

impl TimeLockPath {
    pub fn new(policy: SpendPolicy) -> Result<Self, ComponentError> {
        Self::is_valid(&policy).map(|_| Self(policy))
    }
}

impl IsValidatedSpendPolicy for TimeLockPath {
    type Error = ComponentError;

    fn policy(&self) -> &SpendPolicy {
        &self.0
    }

    fn is_valid(policy: &SpendPolicy) -> Result<(), Self::Error> {
        match policy {
            SpendPolicy::Threshold{ n: 2, of } if of.len() == 2 => {
                match of.as_slice() {
                    [SpendPolicy::After(_), SpendPolicy::PublicKey(_)] => Ok(()),
                    _ => Err(ComponentError::TimeLockInvalidThresholdStructure(policy.clone())),
                }
            },
            SpendPolicy::Threshold{ n: 2, of } => Err(ComponentError::TimeLockInvalidM{ policy: policy.clone(), m: of.len() }),
            SpendPolicy::Threshold{ n, of: _ } => Err(ComponentError::TimeLockInvalidN{ policy: policy.clone(), n: n.clone() }),
            _ => Err(ComponentError::TimeLockInvalidSpendPolicyVariant(policy.clone())),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Hash256, PublicKey};

    fn pubkey0() -> PublicKey {
        PublicKey::from_bytes(
            &hex::decode("0102030000000000000000000000000000000000000000000000000000000000").unwrap(),
        ).unwrap()
    }

    fn pubkey1() -> PublicKey {
        PublicKey::from_bytes(
            &hex::decode("06C87838297B7BB16AB23946C99DFDF77FF834E35DB07D71E9B1D2B01A11E96D").unwrap(),
        )
        .unwrap()
    }

    fn valid_atomic_swap_spend_policy() -> SpendPolicy {
        SpendPolicy::Threshold {
            n: 1,
            of: vec![
                SpendPolicy::Threshold {
                    n: 2,
                    of: vec![
                        SpendPolicy::Hash(Hash256::default()),
                        SpendPolicy::PublicKey(pubkey0()),
                    ],
                },
                SpendPolicy::Threshold {
                    n: 2,
                    of: vec![
                        SpendPolicy::After(0),
                        SpendPolicy::PublicKey(pubkey1()),
                    ],
                },
            ],
        }
    }
    
    fn valid_component_hash_lock() -> SpendPolicy {
        SpendPolicy::Threshold {
            n: 2,
            of: vec![
                SpendPolicy::Hash(Hash256::default()),
                SpendPolicy::PublicKey(pubkey0()),
            ],
        }
    }

    fn valid_component_time_lock() -> SpendPolicy {
        SpendPolicy::Threshold {
            n: 2,
            of: vec![
                SpendPolicy::After(0),
                SpendPolicy::PublicKey(pubkey1()),
            ],
        }
    }

    #[test]
    fn test_atomic_swap_contract_valid() {
        AtomicSwap::new(valid_atomic_swap_spend_policy()).unwrap();
    }

    #[test]
    fn test_atomic_swap_contract_invalid_hash_lock_path() {
        let policy = SpendPolicy::Threshold {
            n: 1,
            of: vec![SpendPolicy::PublicKey(pubkey0()), valid_component_time_lock()],
        };

        match AtomicSwap::new(policy.clone()).unwrap_err() {
            AtomicSwapError::InvalidHashComponent(ComponentError::HashLockInvalidSpendPolicyVariant(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_time_lock_path() {
        let policy = SpendPolicy::Threshold {
            n: 1,
            of: vec![valid_component_hash_lock(), SpendPolicy::PublicKey(pubkey0())],
        };

        match AtomicSwap::new(policy.clone()).unwrap_err() {
            AtomicSwapError::InvalidTimeComponent(ComponentError::TimeLockInvalidSpendPolicyVariant(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_components_wrong_order() {
        let policy = SpendPolicy::Threshold {
            n: 1,
            of: vec![valid_component_time_lock(), valid_component_hash_lock()],
        };

        match AtomicSwap::new(policy.clone()).unwrap_err() {
            AtomicSwapError::InvalidHashComponent(ComponentError::HashLockInvalidThresholdStructure(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_components_too_many() {
        let mut policy = valid_atomic_swap_spend_policy();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                of.push(SpendPolicy::PublicKey(pubkey0()));
            },
            _ => unreachable!(),
        }

        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidM { policy: _, m }) => {
                assert_eq!(m, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_components_missing_time_lock_path() {
        let mut policy = valid_atomic_swap_spend_policy();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                let _ = of.pop().unwrap();
            },
            _ => unreachable!(),
        }
        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidM { policy: _, m }) => {
                assert_eq!(m, 1);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_components_missing_hash_lock_path() {
        let mut policy = valid_atomic_swap_spend_policy();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                let _ = of.remove(0);
            },
            _ => unreachable!(),
        }
        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidM { policy: _, m }) => {
                assert_eq!(m, 1);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_components_missing_both_paths() {
        let mut policy = valid_atomic_swap_spend_policy();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![];
            },
            _ => unreachable!(),
        }
        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidM { policy: _, m }) => {
                assert_eq!(m, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_n() {
        let mut policy = valid_atomic_swap_spend_policy();
        match &mut policy {
            SpendPolicy::Threshold { n, .. } => *n = 10,
            _ => unreachable!(),
        }

        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidN { policy: _, n }) => {
                assert_eq!(n, 10);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_contract_invalid_policy_variant() {
        let policy = SpendPolicy::PublicKey(pubkey0());

        match AtomicSwap::new(policy.clone()) {
            Err(AtomicSwapError::InvalidSpendPolicyVariant { .. }) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_valid() {
        HashLockPath::new(valid_component_hash_lock()).unwrap();
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_threshold_structure() {
        let policy = SpendPolicy::Threshold {
            n: 2,
            of: vec![SpendPolicy::PublicKey(pubkey0()) , SpendPolicy::PublicKey(pubkey0())],
        };

        match HashLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::HashLockInvalidThresholdStructure(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_wrong_order() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                of.reverse();
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::HashLockInvalidThresholdStructure(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_too_many() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                of.push(SpendPolicy::PublicKey(pubkey0()));
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy).unwrap_err() {
            ComponentError::HashLockInvalidM{ policy: _, m } => assert_eq!(m, 3),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_missing_public_key() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![SpendPolicy::Hash(Hash256::default())]
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy).unwrap_err() {
            ComponentError::HashLockInvalidM{ policy: _, m } => assert_eq!(m, 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_missing_hash() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![SpendPolicy::PublicKey(pubkey0())]
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy).unwrap_err() {
            ComponentError::HashLockInvalidM{ policy: _, m } => assert_eq!(m, 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_empty_threshold() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![]
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy).unwrap_err() {
            ComponentError::HashLockInvalidM{ policy: _, m } => assert_eq!(m, 0),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_n() {
        let mut policy = valid_component_hash_lock();
        match &mut policy {
            SpendPolicy::Threshold { n, of:_ } => {
                *n = 10;
            },
            _ => unreachable!(),
        }

        match HashLockPath::new(policy).unwrap_err() {
            ComponentError::HashLockInvalidN{ policy: _, n } => assert_eq!(n, 10),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_hash_lock_invalid_policy_variant() {
        let policy = SpendPolicy::PublicKey(pubkey0());

        match HashLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::HashLockInvalidSpendPolicyVariant(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_valid() {
        TimeLockPath::new(valid_component_time_lock()).unwrap();
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_threshold_structure() {
        let policy = SpendPolicy::Threshold {
            n: 2,
            of: vec![SpendPolicy::PublicKey(pubkey0()) , SpendPolicy::PublicKey(pubkey0())],
        };

        match TimeLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::TimeLockInvalidThresholdStructure(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_wrong_order() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                of.reverse();
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::TimeLockInvalidThresholdStructure(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_too_many() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                of.push(SpendPolicy::PublicKey(pubkey0()));
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy).unwrap_err() {
            ComponentError::TimeLockInvalidM{ policy: _, m } => assert_eq!(m, 3),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_missing_public_key() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![SpendPolicy::After(0)]
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy).unwrap_err() {
            ComponentError::TimeLockInvalidM{ policy: _, m } => assert_eq!(m, 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_missing_time() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![SpendPolicy::PublicKey(pubkey1())]
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy).unwrap_err() {
            ComponentError::TimeLockInvalidM{ policy: _, m } => assert_eq!(m, 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_empty_threshold() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n:_, of } => {
                *of = vec![]
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy).unwrap_err() {
            ComponentError::TimeLockInvalidM{ policy: _, m } => assert_eq!(m, 0),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_n() {
        let mut policy = valid_component_time_lock();
        match &mut policy {
            SpendPolicy::Threshold { n, of:_ } => {
                *n = 10;
            },
            _ => unreachable!(),
        }

        match TimeLockPath::new(policy).unwrap_err() {
            ComponentError::TimeLockInvalidN{ policy: _, n } => assert_eq!(n, 10),
            _ => panic!(),
        }
    }

    #[test]
    fn test_atomic_swap_component_time_lock_invalid_policy_variant() {
        let policy = SpendPolicy::PublicKey(pubkey0());

        match TimeLockPath::new(policy.clone()).unwrap_err() {
            ComponentError::TimeLockInvalidSpendPolicyVariant(p) => assert_eq!(p, policy),
            _ => panic!(),
        }
    }
}
