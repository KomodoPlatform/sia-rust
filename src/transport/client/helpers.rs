use super::{ApiClient, ApiClientError};
use crate::transport::endpoints::{AddressBalanceRequest, ConsensusTipRequest, GetAddressUtxosRequest, AddressBalanceResponse};
use crate::types::{Address, Currency, SiacoinElement};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientHelpersError {
    #[error("ApiClientHelpersError::SelectOutputs: insufficent amount, available: {:?} required: {:?}", .available, .required)]
    SelectOutputs {
        available: Currency,
        required: Currency,
    },
    #[error("ApiClientHelpersError::ApiClientError: {0}")]
    ApiClientError(#[from] ApiClientError),
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

    async fn get_unspent_outputs(&self, address: &Address, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<SiacoinElement>, ApiClientError> {
        self.dispatcher(GetAddressUtxosRequest { address: address.clone(), limit, offset }).await
    }

    /// Fetches unspent outputs for the given address and attempts to select a subset of outputs
    /// whose total value is at least `total_amount`. The outputs are sorted from largest to smallest to minimize
    /// the number of outputs selected. The function returns a vector of the selected outputs and the difference between
    /// the total value of the selected outputs and the required amount, aka the change.
    /// # Arguments
    ///
    /// * `unspent_outputs` - A vector of `SiacoinElement`s representing unspent Siacoin outputs.
    /// * `total_amount` - The total amount (in u128) required for the selection.
    ///
    /// # Returns
    ///
    /// This function returns `Result<(Vec<SiacoinElement>, Currency), ApiClientHelpersError>`:
    /// * `Ok((Vec<SiacoinElement>, Currency))` - A tuple containing, a vector of the selected unspent outputs and the change amount.
    /// * `Err(MmError<SelectOutputsError>)` - An error is returned if the available outputs cannot meet the required amount or a transport error is encountered.
    async fn select_unspent_outputs(&self, address: &Address, total_amount: Currency) -> Result<(Vec<SiacoinElement>, Currency), ApiClientHelpersError> {
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
}