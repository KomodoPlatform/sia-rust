use crate::types::{PublicKey, SpendPolicy};

use std::marker::PhantomData;
use thiserror::Error;

/*
SpendPolicy::Threshold { 
	n: 1,
	of: vec![ 
		SpendPolicy::Threshold { 
			n: 2, 
			of: vec![
				SpendPolicy::After(<SOME DYNAMIC VALUE>),
				SpendPolicy::PublicKey(<SOME DYNAMIC VALUE>)
			] 
		},
		SpendPolicy::Threshold { 
			n: 2, 
			of: vec![
				SpendPolicy::Above(<SOME DYNAMIC VALUE>),
				SpendPolicy::PublicKey(<SOME DYNAMIC VALUE>)
			] 
		},
	]
}
*/

pub enum AtomicSwapError {
    #[error("invalid time threshold: {:?}", 0)]
    InvalidTimeThreshold(SpendPolicy),
    #[error("invalid hash threshold: {:?}", 0)]
    InvalidHashThreshold(SpendPolicy),
    #[error("invalid refund path: {:?}", 0)]
    InvalidHashThreshold(SpendPolicy),
}

/// Represents an atomic swap contract.
/// PhantomData is used to enforce type safety on the structure of the SpendPolicy.
pub struct AtomicSwap<T> {
    policy: SpendPolicy,
    _marker: PhantomData<T>,
}

/// Represents one of the Threshold components of an atomic swap contract.
/// PhantomData is used to enforce type safety on the structure of the SpendPolicy.
pub struct AtomicSwapComponent<T> {
    policy: SpendPolicy,
    _marker: PhantomData<T>,
}


/// The full atomic swap contract.
/// This is opacified(hashed) and used as the SpendPolicy for the locked transaction output.
/// This is used only to create outputs, never inputs.
struct Full;

/// The success branch of the atomic swap contract.
/// The refund path is opacified and resulting SpendPolicy is used in transaction input's SatifiedPolicy
/// This is used only to create inputs, never outputs.
struct SuccessBranch;

/// The refund branch of the atomic swap contract.
/// The success path is opacified and resulting SpendPolicy is used in transaction input's SatisfiedPolicy
/// This is used only to create inputs, never outputs.
struct RefundBranch;

/// 2 of 2 threshold of SpendPolicy::Hash and SpendPolicy::PublicKey
struct HashPublicKey2of2;

impl AtomicSwapComponent<HashPublicKey2of2> {
    pub fn new(policy: SpendPolicy) -> Result<Self, AtomicSwapError> {
        if Self::is_valid_hash_threshold(&policy) {
            Ok(Self {
                policy,
                _marker: PhantomData,
            })
        } else {
            Err(AtomicSwapError::InvalidHashThreshold(policy))
        }
    }

    fn is_valid_hash_threshold(policy: &SpendPolicy) -> bool {
        match policy {
            SpendPolicy::Threshold{ n: 2, of: [SpendPolicy::Hash(_), SpendPolicy::PublicKey(_)] } => true,
            _ => false,
        }
    }
}

/// The time threshold branch of the atomic swap contract.
/// 2 of 2 threshold of SpendPolicy::After and SpendPolicy::PublicKey
struct TimeThreshold;

impl AtomicSwapComponent<TimeThreshold> {
    pub fn new(policy: SpendPolicy) -> Result<Self, AtomicSwapError> {
        if Self::is_valid_time_threshold(&policy) {
            Ok(Self {
                policy,
                _marker: PhantomData,
            })
        } else {
            Err(AtomicSwapError::InvalidTimeThreshold(policy))
        }
    }

    fn is_valid_time_threshold(policy: &SpendPolicy) -> bool {
        match policy {
            SpendPolicy::Threshold{ n: 2, of: [SpendPolicy::After(_), SpendPolicy::PublicKey(_)] } => true,
            _ => false,
        }
    }
}




