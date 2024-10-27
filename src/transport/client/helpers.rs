use super::{ApiClient, ApiClientError};
use crate::transport::endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest,
                                  GetAddressUtxosRequest, GetEventRequest};
use crate::types::{Address, Currency, Event, EventDataWrapper, PublicKey, SiacoinElement, SiacoinOutputId,
                   SpendPolicy, TransactionId, V2TransactionBuilder};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientHelpersError {
    #[error(
        "ApiClientHelpersError::SelectOutputs: insufficent amount, available: {available:?} required: {required:?}"
    )]
    SelectOutputs { available: Currency, required: Currency },
    #[error("ApiClientHelpersError::ApiClientError: {0}")]
    ApiClientError(#[from] ApiClientError),
}

#[derive(Debug, Error)]
pub enum UtxoFromTxidError {
    #[error("utxo_from_txid: failed to fetch event {0}")]
    FetchEvent(ApiClientError),
    #[error("utxo_from_txid: invalid event variant {0:?}")]
    EventVariant(Event),
    #[error("utxo_from_txid: output index out of bounds txid: {txid:?} index: {index:?}")]
    OutputIndexOutOfBounds { txid: TransactionId, index: u32 },
    #[error("utxo_from_txid: get_unspent_outputs helper failed {0}")]
    FetchUtxos(ApiClientError),
    #[error("utxo_from_txid: output not found txid: {txid:?} index: {index:?}")]
    NotFound { txid: TransactionId, index: u32 },
    #[error("utxo_from_txid: found duplicate utxo txid: {txid:?} index: {index:?}")]
    DuplicateUtxoFound { txid: TransactionId, index: u32 },
}

/// Helper methods for the ApiClient trait
/// These generally provide higher level functionality than the base ApiClient trait
/// This crate is focused on catering to the Komodo Defi Framework integration
#[async_trait]
pub trait ApiClientHelpers: ApiClient {
    async fn current_height(&self) -> Result<u64, ApiClientError> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, ApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }

    async fn get_unspent_outputs(
        &self,
        address: &Address,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SiacoinElement>, ApiClientError> {
        self.dispatcher(GetAddressUtxosRequest {
            address: address.clone(),
            limit,
            offset,
        })
        .await
    }

    /// Fetches unspent outputs for the given address and attempts to select a subset of outputs
    /// whose total value is at least `total_amount`. The outputs are sorted from largest to smallest to minimize
    /// the number of outputs selected. The function returns a vector of the selected outputs and the difference between
    /// the total value of the selected outputs and the required amount, aka the change.
    /// # Arguments
    ///
    /// * `unspent_outputs` - A vector of `SiacoinElement`s representing unspent Siacoin outputs.
    /// * `total_amount` - The total amount required for the selection. Should generally be the sum of the outputs and miner fee.
    ///
    /// # Returns
    ///
    /// This function returns `Result<(Vec<SiacoinElement>, Currency), ApiClientHelpersError>`:
    /// * `Ok((Vec<SiacoinElement>, Currency))` - A tuple containing, a vector of the selected unspent outputs and the change amount.
    /// * `Err(ApiClientHelpersError)` - An error is returned if the available outputs cannot meet the required amount or a transport error is encountered.
    async fn select_unspent_outputs(
        &self,
        address: &Address,
        total_amount: Currency,
    ) -> Result<(Vec<SiacoinElement>, Currency), ApiClientHelpersError> {
        let mut unspent_outputs = self.get_unspent_outputs(address, None, None).await?;

        // Sort outputs from largest to smallest
        unspent_outputs.sort_by(|a, b| b.siacoin_output.value.0.cmp(&a.siacoin_output.value.0));

        let mut selected = Vec::new();
        let mut selected_amount = 0;

        // Select outputs until the total amount is reached
        for output in unspent_outputs {
            selected_amount += *output.siacoin_output.value;
            selected.push(output);

            if selected_amount >= *total_amount {
                break;
            }
        }

        if selected_amount < *total_amount {
            return Err(ApiClientHelpersError::SelectOutputs {
                available: selected_amount.into(),
                required: total_amount.into(),
            });
        }
        let change = selected_amount as u128 - *total_amount;

        Ok((selected, change.into()))
    }

    /// Fund a transaction with utxos from the given address.
    /// This should generally be used only after all outputs and miner_fee have been added to the builder.
    /// Assumes no file contracts or resolutions. This is a helper designed for Komodo DeFi Framework.
    /// Adds inputs from the given address until the total amount from outputs and miner_fee is reached.
    /// Adds the change amount to the transaction outputs
    /// See `select_unspent_outputs` for more details on UTXO selection.
    /// # Arguments
    /// * `tx_builder` - A mutable reference to a `V2TransactionBuilder.
    /// * `public_key` - The public key of the address to spend utxos from.
    /// # Returns
    /// * `Ok(())` - The transaction builder has been successfully funded
    /// * `Err(ApiClientHelpersError)` - An error is returned if the available outputs cannot meet
    ///     the required amount or a transport error is encountered.
    // Alright TODO - move V2TransactionBuilder to a separate module then move this logic to a
    // method of V2TransactionBuilder to allow chaining. It was included here because V2TransactionBuilder
    // is currently inside the transaction module which is generally meant for consensnus related types.
    // It would not be appropriate to include ApiClient-related code in transaction.rs
    async fn fund_tx_single_source(
        &self,
        tx_builder: &mut V2TransactionBuilder,
        public_key: &PublicKey,
    ) -> Result<(), ApiClientHelpersError> {
        let address = public_key.address();
        let outputs_total: Currency = tx_builder.siacoin_outputs.iter().map(|output| output.value).sum();

        // select utxos from public key's address that total at least the sum of outputs and miner fee
        let (selected_utxos, change) = self
            .select_unspent_outputs(&address, outputs_total + tx_builder.miner_fee)
            .await?;

        // add selected utxos as inputs to the transaction
        for utxo in &selected_utxos {
            tx_builder.add_siacoin_input(utxo.clone(), SpendPolicy::PublicKey(public_key.clone()));
        }

        if change > Currency::DUST {
            // add change as an output
            tx_builder.add_siacoin_output((address, change).into());
        }

        Ok(())
    }

    /// Fetches a SiacoinElement(a UTXO) from a TransactionId and Index
    /// Walletd doesn't currently offer an easy way to fetch the SiacoinElement type needed to build
    /// SiacoinInputs.
    async fn utxo_from_txid(&self, txid: &TransactionId, vout_index: u32) -> Result<SiacoinElement, UtxoFromTxidError> {
        let output_id = SiacoinOutputId::new(txid.clone(), vout_index);

        // fetch the Event via /api/events/{txid}
        let event = self
            .dispatcher(GetEventRequest { txid: txid.clone() })
            .await
            .map_err(|e| UtxoFromTxidError::FetchEvent(e))?;

        // check that the fetched event is V2Transaction
        let tx = match event.data {
            EventDataWrapper::V2Transaction(tx) => tx,
            _ => return Err(UtxoFromTxidError::EventVariant(event)),
        };

        // check that the output index is within bounds
        if tx.siacoin_outputs.len() <= (vout_index as usize) {
            return Err(UtxoFromTxidError::OutputIndexOutOfBounds {
                txid: txid.clone(),
                index: vout_index,
            });
        }

        let output_address = tx.siacoin_outputs[vout_index as usize].address.clone();

        // fetch unspent outputs of the address
        let address_utxos = self
            .get_unspent_outputs(&output_address, None, None)
            .await
            .map_err(|e| UtxoFromTxidError::FetchUtxos(e))?;

        // filter the utxos to find any matching the expected SiacoinOutputId
        let filtered_utxos: Vec<SiacoinElement> = address_utxos
            .into_iter()
            .filter(|element| element.state_element.id == output_id.clone().into())
            .collect();

        // ensure only one utxo was found
        match filtered_utxos.len() {
            1 => Ok(filtered_utxos[0].clone()),
            0 => Err(UtxoFromTxidError::NotFound {
                txid: txid.clone(),
                index: vout_index,
            }),
            _ => Err(UtxoFromTxidError::DuplicateUtxoFound {
                txid: txid.clone(),
                index: vout_index,
            }),
        }
    }
}
