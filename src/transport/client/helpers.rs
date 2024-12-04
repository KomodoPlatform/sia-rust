use super::ApiClient;
use crate::transport::endpoints::{AddressBalanceRequest, AddressBalanceResponse, AddressesEventsRequest,
                                  ConsensusIndexRequest, ConsensusTipRequest, ConsensusTipstateRequest,
                                  ConsensusTipstateResponse, ConsensusUpdatesRequest, ConsensusUpdatesResponse,
                                  GetAddressUtxosRequest, GetEventRequest, TxpoolBroadcastRequest,
                                  TxpoolTransactionsRequest};
use crate::types::{Address, Currency, Event, EventDataWrapper, Hash256, PublicKey, SiacoinElement, SiacoinOutputId,
                   SpendPolicy, TransactionId, V2Transaction, V2TransactionBuilder};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtxoFromTxidError<ClientError> {
    #[error("ApiClientHelpers::utxo_from_txid: failed to fetch event {0}")]
    FetchEvent(#[from] ClientError),
    #[error("ApiClientHelpers::utxo_from_txid: invalid event variant {0:?}")]
    EventVariant(Event),
    #[error("ApiClientHelpers::utxo_from_txid: output index out of bounds txid: {txid} index: {index}")]
    OutputIndexOutOfBounds { txid: TransactionId, index: u32 },
    #[error("ApiClientHelpers::utxo_from_txid: get_unspent_outputs helper failed {0}")]
    FetchUtxos(#[from] GetUnspentOutputsError<ClientError>),
    #[error("ApiClientHelpers::utxo_from_txid: output not found txid: {txid} index: {index}")]
    NotFound { txid: TransactionId, index: u32 },
    #[error("ApiClientHelpers::utxo_from_txid: found duplicate utxo txid: {txid} index: {index}")]
    DuplicateUtxoFound { txid: TransactionId, index: u32 },
}

#[derive(Debug, Error)]
pub enum GetTransactionError<ClientError> {
    #[error("ApiClientHelpers::get_transaction: failed to fetch event {0}")]
    FetchEvent(#[from] ClientError),
    #[error("ApiClientHelpers::get_transaction: unexpected variant error {0:?}")]
    EventVariant(EventDataWrapper),
}

#[derive(Debug, Error)]
pub enum GetUnconfirmedTransactionError<ClientError> {
    #[error("ApiClientHelpers::get_unconfirmed_transaction: failed to fetch mempool {0}")]
    FetchMempool(#[from] ClientError),
}

#[derive(Debug, Error)]
pub enum FundTxSingleSourceError<ClientError> {
    #[error("ApiClientHelpers::fund_tx_single_source: failed to select Utxos: {0}")]
    SelectUtxos(#[from] SelectUtxosError<ClientError>),
}

#[derive(Debug, Error)]
pub enum GetEventError<ClientError> {
    #[error("ApiClientHelpers::get_event failed to fetch event: {0}")]
    FetchEvent(#[from] ClientError),
}

#[derive(Debug, Error)]
pub enum GetAddressEventsError<ClientError> {
    #[error("ApiClientHelpers::get_address_events failed: {0}")]
    FetchAddressEvents(#[from] ClientError),
}

#[derive(Debug, Error)]
pub enum BroadcastTransactionError<ClientError> {
    #[error("ApiClientHelpers::broadcast_transaction: broadcast failed: {0}")]
    BroadcastTx(#[from] ClientError),
}

#[derive(Debug, Error)]
pub enum SelectUtxosError<ClientError> {
    #[error(
        "ApiClientHelpers::select_unspent_outputs: insufficent funds, available: {available:?} required: {required:?}"
    )]
    Funding { available: Currency, required: Currency },
    #[error("ApiClientHelpers::select_unspent_outputs: failed to fetch UTXOs: {0}")]
    GetUnspentOutputs(#[from] GetUnspentOutputsError<ClientError>),
}

#[derive(Debug, Error)]
pub enum GetMedianTimestampError<ClientError> {
    #[error("ApiClientHelpers::get_median_timestamp: failed to fetch consensus tipstate: {0}")]
    FetchTipstate(#[from] ClientError),
    #[error(
        r#"ApiClientHelpers::get_median_timestamp: expected 11 timestamps in response: {0:?}.
           The walletd state is likely corrupt as it is evidently reporting a chain height of less
           than 11 blocks."#
    )]
    TimestampVecLen(ConsensusTipstateResponse),
}

#[derive(Debug, Error)]
pub enum GetUnspentOutputsError<ClientError> {
    #[error("ApiClientHelpers::get_unspent_outputs: failed to fetch UTXOs {0}")]
    FetchUtxos(#[from] ClientError),
}

/// ApiClientHelpers implements client agnostic helper methods catering to the Komodo Defi Framework
/// integration. These methods provide higher level functionality than the base ApiClient trait.
/// Clients can generally implement this as simply as `impl ApiClientHelpers for Client {}`.
#[async_trait]
pub trait ApiClientHelpers: ApiClient {
    async fn current_height(&self) -> Result<u64, Self::Error> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, Self::Error> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }

    async fn get_unspent_outputs(
        &self,
        address: &Address,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SiacoinElement>, GetUnspentOutputsError<Self::Error>> {
        Ok(self
            .dispatcher(GetAddressUtxosRequest {
                address: address.clone(),
                limit,
                offset,
            })
            .await?)
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
    ) -> Result<(Vec<SiacoinElement>, Currency), SelectUtxosError<Self::Error>> {
        let mut unspent_outputs = self.get_unspent_outputs(address, None, None).await?;

        // Sort outputs from largest to smallest
        unspent_outputs.sort_by(|a, b| b.siacoin_output.value.0.cmp(&a.siacoin_output.value.0));

        let mut selected = Vec::new();
        let mut selected_amount = Currency::ZERO;

        // Select outputs until the total amount is reached
        for output in unspent_outputs {
            selected_amount += output.siacoin_output.value;
            selected.push(output);

            if selected_amount >= total_amount {
                break;
            }
        }

        if selected_amount < total_amount {
            return Err(SelectUtxosError::Funding {
                available: selected_amount,
                required: total_amount,
            })?;
        }
        let change = selected_amount - total_amount;

        Ok((selected, change))
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
    ) -> Result<(), FundTxSingleSourceError<Self::Error>> {
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
    async fn utxo_from_txid(
        &self,
        txid: &TransactionId,
        vout_index: u32,
    ) -> Result<SiacoinElement, UtxoFromTxidError<Self::Error>> {
        let output_id = SiacoinOutputId::new(txid.clone(), vout_index);

        // fetch the Event via /api/events/{txid}
        let event = self.dispatcher(GetEventRequest { txid: txid.clone() }).await?;

        // check that the fetched event is V2Transaction
        let tx = match event.data {
            EventDataWrapper::V2Transaction(tx) => tx,
            _ => return Err(UtxoFromTxidError::<Self::Error>::EventVariant(event)),
        };

        // check that the output index is within bounds
        if tx.siacoin_outputs.len() <= (vout_index as usize) {
            return Err(UtxoFromTxidError::<Self::Error>::OutputIndexOutOfBounds {
                txid: txid.clone(),
                index: vout_index,
            });
        }

        let output_address = tx.siacoin_outputs[vout_index as usize].address.clone();

        // fetch unspent outputs of the address
        let address_utxos = self.get_unspent_outputs(&output_address, None, None).await?;

        // filter the utxos to find any matching the expected SiacoinOutputId
        let filtered_utxos: Vec<SiacoinElement> = address_utxos
            .into_iter()
            .filter(|element| element.id == output_id)
            .collect();

        // ensure only one utxo was found
        match filtered_utxos.len() {
            1 => Ok(filtered_utxos[0].clone()),
            0 => Err(UtxoFromTxidError::<Self::Error>::NotFound {
                txid: txid.clone(),
                index: vout_index,
            }),
            _ => Err(UtxoFromTxidError::<Self::Error>::DuplicateUtxoFound {
                txid: txid.clone(),
                index: vout_index,
            }),
        }
    }

    async fn get_event(&self, event_id: &Hash256) -> Result<Event, GetEventError<Self::Error>> {
        Ok(self.dispatcher(GetEventRequest { txid: event_id.clone() }).await?)
    }

    async fn get_address_events(&self, address: Address) -> Result<Vec<Event>, GetAddressEventsError<Self::Error>> {
        let request = AddressesEventsRequest {
            address,
            limit: None,
            offset: None,
        };
        Ok(self.dispatcher(request).await?)
    }

    /// Fetch a v2 transaction from the blockchain
    // FIXME Alright - this should return a Result<Option<V2Transaction>, HelperError> to allow for
    // logic to handle the case where the transaction is not found in the blockchain
    // ApiClientError must be refactored to allow this
    async fn get_transaction(&self, txid: &TransactionId) -> Result<V2Transaction, GetTransactionError<Self::Error>> {
        let event = self.dispatcher(GetEventRequest { txid: txid.clone() }).await?;
        match event.data {
            EventDataWrapper::V2Transaction(tx) => Ok(tx),
            wrong_variant => Err(GetTransactionError::<Self::Error>::EventVariant(wrong_variant)),
        }
    }

    /// Fetch a v2 transaction from the transaction pool / mempool
    /// Returns Ok(None) if the transaction is not found in the mempool
    async fn get_unconfirmed_transaction(
        &self,
        txid: &TransactionId,
    ) -> Result<Option<V2Transaction>, GetUnconfirmedTransactionError<Self::Error>> {
        let found_in_mempool = self
            .dispatcher(TxpoolTransactionsRequest)
            .await?
            .v2transactions
            .into_iter()
            .find(|tx| tx.txid() == *txid);
        Ok(found_in_mempool)
    }

    /// Get the median timestamp of the chain's last 11 blocks
    /// This is used in the evaluation of SpendPolicy::After
    async fn get_median_timestamp(&self) -> Result<u64, GetMedianTimestampError<Self::Error>> {
        let tipstate = self.dispatcher(ConsensusTipstateRequest).await?;

        // This can happen if the chain has less than 11 blocks
        // We assume the chain is at least 11 blocks long for this helper.
        if tipstate.prev_timestamps.len() != 11 {
            return Err(GetMedianTimestampError::<Self::Error>::TimestampVecLen(tipstate));
        }

        let median_timestamp = tipstate.prev_timestamps[5];
        Ok(median_timestamp.timestamp() as u64)
    }

    async fn broadcast_transaction(&self, tx: &V2Transaction) -> Result<(), BroadcastTransactionError<Self::Error>> {
        let request = TxpoolBroadcastRequest {
            transactions: vec![],
            v2transactions: vec![tx.clone()],
        };

        self.dispatcher(request).await?;
        Ok(())
    }

    async fn get_consensus_updates(
        &self,
        begin_height: u64,
    ) -> Result<ConsensusUpdatesResponse, GetConsensusUpdatesError<Self::Error>> {
        let index_request = ConsensusIndexRequest { height: begin_height };
        let chain_index = self
            .dispatcher(index_request)
            .await
            .map_err(|e| GetConsensusUpdatesError::<Self::Error>::FetchIndex(e))?;

        let updates_request = ConsensusUpdatesRequest {
            height: chain_index.height,
            block_hash: chain_index.id,
            limit: None,
        };

        self.dispatcher(updates_request)
            .await
            .map_err(|e| GetConsensusUpdatesError::<Self::Error>::FetchUpdates(e))
    }

    /// Find the transaction that spent the given utxo
    /// Scans the blockchain starting from `begin_height`
    /// Returns Ok(None) if the utxo has not been spent
    async fn find_where_utxo_spent(
        &self,
        output_id: &SiacoinOutputId,
        begin_height: u64,
    ) -> Result<Option<V2Transaction>, FindWhereUtxoSpentError<Self::Error>> {
        let updates = self.get_consensus_updates(begin_height).await?;

        // find the update that has the provided `output_id`` in its "spent" field
        let update_option = updates
            .applied
            .into_iter()
            .find(|applied_update| applied_update.update.spent.contains(&output_id.0));

        // If no update with the output_id was found, return Ok(None) indicating no error occured,
        // but the spend transaction has not been found
        let update = match update_option {
            Some(update) => update,
            None => return Ok(None),
        };

        // scan the block indicated by the update to find the transaction that spent the utxo
        let tx = update
            .block
            .v2
            .transactions
            .into_iter()
            .find(|tx| tx.siacoin_inputs.iter().any(|input| input.parent.id == *output_id))
            .ok_or(FindWhereUtxoSpentError::SpendNotInBlock { id: output_id.clone() })?;

        Ok(Some(tx))
    }
}

#[derive(Debug, Error)]
pub enum FindWhereUtxoSpentError<ClientError> {
    // Boxed to allow HelperError to be held within itself
    #[error("ApiClientHelpers::find_where_utxo_spent: failed to fetch consensus updates {0}")]
    FetchUpdates(#[from] GetConsensusUpdatesError<ClientError>),
    #[error("ApiClientHelpers::find_where_utxo_spent: SiacoinOutputId:{id} was not spent in the expected block")]
    SpendNotInBlock { id: SiacoinOutputId },
}

#[derive(Debug, Error)]
pub enum GetConsensusUpdatesError<ClientError> {
    #[error("ApiClientHelpers::get_consensus_updates_since_height: failed to fetch ChainIndex {0}")]
    FetchIndex(ClientError),
    #[error("ApiClientHelpers::get_consensus_updates_since_height: failed to fetch updates {0}")]
    FetchUpdates(ClientError),
}
