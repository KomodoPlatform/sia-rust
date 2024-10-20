use super::{ApiClient, ApiClientError};
use crate::transport::endpoints::{AddressBalanceRequest, ConsensusTipRequest, GetAddressUtxosRequest, AddressBalanceResponse};
use crate::types::{Address, Currency, SiacoinElement};
use async_trait::async_trait;


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

    async fn select_utxos(&self, address: &Address, amount: Currency) -> Result<Vec<SiacoinElement>, ApiClientError> {
        let _utxos = self.get_unspent_outputs(address, None, None).await?;
        todo!()
    }
}