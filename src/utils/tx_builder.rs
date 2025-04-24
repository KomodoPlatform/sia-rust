use crate::encoding::{Encodable, Encoder};
use crate::transport::client::{error::ClientError, helpers::generic_errors::FundTxSingleSourceErrorGeneric,
                               ApiClientHelpers, Client};
use crate::types::{Address, ArbitraryData, Attestation, ChainIndex, Currency, CurrencyVersion, FileContractRevisionV2,
                   Hash256, Keypair, Preimage, PublicKey, SatisfiedPolicy, SiacoinElement, SiacoinInputV2,
                   SiacoinOutput, SiacoinOutputVersion, SiafundInputV2, SiafundOutput, SiafundOutputVersion,
                   SpendPolicy, UnlockKey, UtxoWithBasis, V2FileContract, V2FileContractResolution, V2Transaction,
                   V2_REPLAY_PREFIX};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct V2TransactionBuilder {
    pub siacoin_inputs: Vec<SiacoinInputV2>,
    pub siacoin_outputs: Vec<SiacoinOutput>,
    pub siafund_inputs: Vec<SiafundInputV2>,
    pub siafund_outputs: Vec<SiafundOutput>,
    pub file_contracts: Vec<V2FileContract>,
    pub file_contract_revisions: Vec<FileContractRevisionV2>,
    pub file_contract_resolutions: Vec<V2FileContractResolution>,
    pub attestations: Vec<Attestation>,
    pub arbitrary_data: ArbitraryData,
    pub new_foundation_address: Option<Address>,
    pub miner_fee: Currency,
    // fee_policy is not part Sia consensus and it not encoded into any resulting transaction.
    // fee_policy has no effect unless a helper like `ApiClientHelpers::fund_tx_single_source` utilizes it.
    pub fee_policy: Option<FeePolicy>,
    // basis is not part Sia consensus and it not encoded into any resulting transaction.
    // It is the ChainIndex required to broadcast the transaction. This is provided by the
    // /api/addresses/:addr/siacoin/outputs Walletd API endpoint.
    pub basis: Option<ChainIndex>,
}

impl Encodable for V2TransactionBuilder {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_u64(self.siacoin_inputs.len() as u64);
        for si in &self.siacoin_inputs {
            si.parent.id.encode(encoder);
        }

        encoder.write_u64(self.siacoin_outputs.len() as u64);
        for so in &self.siacoin_outputs {
            SiacoinOutputVersion::V2(so).encode(encoder);
        }

        encoder.write_u64(self.siafund_inputs.len() as u64);
        for si in &self.siafund_inputs {
            si.parent.id.encode(encoder);
        }

        encoder.write_u64(self.siafund_outputs.len() as u64);
        for so in &self.siafund_outputs {
            SiafundOutputVersion::V2(so).encode(encoder);
        }

        encoder.write_u64(self.file_contracts.len() as u64);
        for fc in &self.file_contracts {
            fc.with_nil_sigs().encode(encoder);
        }

        encoder.write_u64(self.file_contract_revisions.len() as u64);
        for fcr in &self.file_contract_revisions {
            fcr.parent.id.encode(encoder);
            fcr.revision.with_nil_sigs().encode(encoder);
        }

        encoder.write_u64(self.file_contract_resolutions.len() as u64);
        for fcr in &self.file_contract_resolutions {
            fcr.parent.id.encode(encoder);
            fcr.with_nil_sigs().encode(encoder);
            // FIXME .encode() leads to unimplemented!()
        }

        encoder.write_u64(self.attestations.len() as u64);
        for att in &self.attestations {
            att.encode(encoder);
        }

        self.arbitrary_data.encode(encoder);

        encoder.write_bool(self.new_foundation_address.is_some());
        match &self.new_foundation_address {
            Some(addr) => addr.encode(encoder),
            None => (),
        }
        CurrencyVersion::V2(&self.miner_fee).encode(encoder);
    }
}

#[derive(Debug, Error)]
pub enum V2TransactionBuilderError {
    #[error("V2TransactionBuilder::satisfy_atomic_swap_success: provided index: {index} is out of bounds for inputs of length: {len}")]
    SatisfySuccessIndexOutOfBounds { len: usize, index: u32 },
    #[error("V2TransactionBuilder::satisfy_atomic_swap_refund: provided index: {index} is out of bounds for inputs of length: {len}")]
    SatisfyRefundIndexOutOfBounds { len: usize, index: u32 },
    #[error("V2TransactionBuilder::fund_tx_single_source: ApiClientHelpers methods failed: {0}")]
    FundTxSingleSource(#[from] FundTxSingleSourceErrorGeneric<ClientError>),
}

impl V2TransactionBuilder {
    pub fn new() -> Self {
        Self {
            siacoin_inputs: Vec::new(),
            siacoin_outputs: Vec::new(),
            siafund_inputs: Vec::new(),
            siafund_outputs: Vec::new(),
            file_contracts: Vec::new(),
            file_contract_revisions: Vec::new(),
            file_contract_resolutions: Vec::new(),
            attestations: Vec::new(),
            arbitrary_data: ArbitraryData::default(),
            new_foundation_address: None,
            miner_fee: Currency::ZERO,
            fee_policy: None,
            basis: None,
        }
    }

    pub fn siacoin_inputs(&mut self, inputs: Vec<SiacoinInputV2>) -> &mut Self {
        self.siacoin_inputs = inputs;
        self
    }

    pub fn siacoin_outputs(&mut self, outputs: Vec<SiacoinOutput>) -> &mut Self {
        self.siacoin_outputs = outputs;
        self
    }

    pub fn siafund_inputs(&mut self, inputs: Vec<SiafundInputV2>) -> &mut Self {
        self.siafund_inputs = inputs;
        self
    }

    pub fn siafund_outputs(&mut self, outputs: Vec<SiafundOutput>) -> &mut Self {
        self.siafund_outputs = outputs;
        self
    }

    pub fn file_contracts(&mut self, contracts: Vec<V2FileContract>) -> &mut Self {
        self.file_contracts = contracts;
        self
    }

    pub fn file_contract_revisions(&mut self, revisions: Vec<FileContractRevisionV2>) -> &mut Self {
        self.file_contract_revisions = revisions;
        self
    }

    pub fn file_contract_resolutions(&mut self, resolutions: Vec<V2FileContractResolution>) -> &mut Self {
        self.file_contract_resolutions = resolutions;
        self
    }

    pub fn attestations(&mut self, attestations: Vec<Attestation>) -> &mut Self {
        self.attestations = attestations;
        self
    }

    pub fn arbitrary_data(&mut self, data: ArbitraryData) -> &mut Self {
        self.arbitrary_data = data;
        self
    }

    pub fn new_foundation_address(&mut self, address: Address) -> &mut Self {
        self.new_foundation_address = Some(address);
        self
    }

    pub fn miner_fee(&mut self, fee: Currency) -> &mut Self {
        self.miner_fee = fee;
        self
    }

    /**
     * "weight" is the size of the transaction in bytes. This can be used to estimate miner fees.
     * The recommended method for calculating a suitable fee is to multiply the response of
     * `/txpool/fee` API endpoint and the weight to get the fee in hastings.
     */
    pub fn weight(&self) -> u64 {
        let mut encoder = Encoder::default();
        self.encode(&mut encoder);
        encoder.buffer.len() as u64
    }

    /* Input is a special case becuase we cannot generate signatures until after fully constructing
    the transaction. Only the parent field is utilized while encoding the transaction to
    calculate the signature hash.
    Policy is included here to give any signing function or method a schema for producing a
    signature for the input. Do not use this method if you are manually creating SatisfiedPolicys.
    Use siacoin_inputs() to add fully formed inputs instead. */
    pub fn add_siacoin_input(&mut self, parent: SiacoinElement, policy: SpendPolicy) -> &mut Self {
        self.siacoin_inputs.push(SiacoinInputV2 {
            parent,
            satisfied_policy: SatisfiedPolicy {
                policy,
                signatures: Vec::new(),
                preimages: Vec::new(),
            },
        });
        self
    }

    /// Update the basis of the transaction. The basis is the ChainIndex required to broadcast the
    /// transaction.
    pub fn update_basis(&mut self, basis: ChainIndex) -> &mut Self {
        // Only update the basis if the new basis is higher than the existing basis.
        match &self.basis {
            Some(existing_basis) if existing_basis.height >= basis.height => {},
            _ => self.basis = Some(basis),
        }
        self
    }

    pub fn add_siacoin_input_with_basis(&mut self, parent: UtxoWithBasis, policy: SpendPolicy) -> &mut Self {
        self.add_siacoin_input(parent.output, policy);
        self.update_basis(parent.basis);
        self
    }

    pub fn add_siacoin_output(&mut self, output: SiacoinOutput) -> &mut Self {
        self.siacoin_outputs.push(output);
        self
    }

    pub fn input_sig_hash(&self) -> Hash256 {
        let mut encoder = Encoder::default();
        encoder.write_distinguisher("sig/input");
        encoder.write_u8(V2_REPLAY_PREFIX);
        self.encode(&mut encoder);
        encoder.hash()
    }

    // Sign all PublicKey or UnlockConditions policies with the provided keypairs
    // Incapable of handling threshold policies
    pub fn sign_simple(&mut self, keypairs: Vec<&Keypair>) -> &mut Self {
        let sig_hash = self.input_sig_hash();
        for keypair in keypairs {
            let sig = keypair.sign(&sig_hash.0);
            for si in &mut self.siacoin_inputs {
                match &si.satisfied_policy.policy {
                    SpendPolicy::PublicKey(pk) if pk == &keypair.public() => {
                        si.satisfied_policy.signatures.push(sig.clone())
                    },
                    SpendPolicy::UnlockConditions(uc) => {
                        for p in &uc.unlock_keys {
                            match p {
                                UnlockKey::Ed25519(pk) if pk == &keypair.public() => {
                                    si.satisfied_policy.signatures.push(sig.clone())
                                },
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
        self
    }

    pub async fn fund_tx_single_source<T: ApiClientHelpers>(
        &mut self,
        client: &Client,
        source_public_key: &PublicKey,
    ) -> Result<&mut Self, V2TransactionBuilderError> {
        client.fund_tx_single_source(self, &source_public_key).await?;
        Ok(self)
    }

    pub fn satisfy_atomic_swap_success(
        &mut self,
        keypair: &Keypair,
        secret: Preimage,
        input_index: u32,
    ) -> Result<&mut Self, V2TransactionBuilderError> {
        let sig_hash = self.input_sig_hash();
        let sig = keypair.sign(&sig_hash.0);

        // check input_index exists prior to indexing into the vector
        if self.siacoin_inputs.len() <= (input_index as usize) {
            return Err(V2TransactionBuilderError::SatisfySuccessIndexOutOfBounds {
                len: self.siacoin_inputs.len(),
                index: input_index,
            });
        }

        let htlc_input = &mut self.siacoin_inputs[input_index as usize];
        htlc_input.satisfied_policy.signatures.push(sig);
        htlc_input.satisfied_policy.preimages.push(secret);
        Ok(self)
    }

    pub fn satisfy_atomic_swap_refund(
        &mut self,
        keypair: &Keypair,
        input_index: u32,
    ) -> Result<&mut Self, V2TransactionBuilderError> {
        let sig_hash = self.input_sig_hash();
        let sig = keypair.sign(&sig_hash.0);

        // check input_index exists prior to indexing into the vector
        if self.siacoin_inputs.len() <= (input_index as usize) {
            return Err(V2TransactionBuilderError::SatisfyRefundIndexOutOfBounds {
                len: self.siacoin_inputs.len(),
                index: input_index,
            });
        }

        let htlc_input = &mut self.siacoin_inputs[input_index as usize];
        htlc_input.satisfied_policy.signatures.push(sig);
        Ok(self)
    }

    pub fn build(&mut self) -> V2Transaction {
        let cloned = self.clone();
        V2Transaction {
            siacoin_inputs: cloned.siacoin_inputs,
            siacoin_outputs: cloned.siacoin_outputs,
            siafund_inputs: cloned.siafund_inputs,
            siafund_outputs: cloned.siafund_outputs,
            file_contracts: cloned.file_contracts,
            file_contract_revisions: cloned.file_contract_revisions,
            file_contract_resolutions: cloned.file_contract_resolutions,
            attestations: cloned.attestations,
            arbitrary_data: cloned.arbitrary_data,
            new_foundation_address: cloned.new_foundation_address,
            miner_fee: cloned.miner_fee,
            basis: cloned.basis,
        }
    }
}

impl Default for V2TransactionBuilder {
    fn default() -> Self { V2TransactionBuilder::new() }
}

/// FeePolicy is data optionally included in V2TransactionBuilder to allow easier fee calculation.
/// Sia fee calculation can be complex in comparison to a typical UTXO protocol because the fee paid
/// to the miner is not simply the sum of the inputs minus the sum of the outputs. Instead, the
/// miner fee is a distinct field within the transaction, `miner_fee`. This `miner_fee` field is part
/// of signature calculation. As a result, you can build a transaction, produce signatures and preimages
/// for the inputs only to find out that the miner_fee hastings/byte rate is lower than expected.
/// Therefore a precise hastings/byte calculation requires correctly estimating the size of all
/// satisfied inputs prior to producing signatures.
#[derive(Clone, Debug)]
pub enum FeePolicy {
    HastingsPerByte(Currency),
    HastingsFixed(Currency),
}
