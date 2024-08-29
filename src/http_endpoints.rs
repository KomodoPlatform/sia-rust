use crate::http_client::SiaApiClientError;
use crate::transaction::{SiacoinElement, V1Transaction, V2Transaction};
use crate::types::{Address, BlockID, Currency, Event, H256};
use reqwest::{Client, Method, Request, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

const ENDPOINT_CONSENSUS_TIP: &str = "api/consensus/tip";

pub trait SiaApiRequest {
    type Response: SiaApiResponse + DeserializeOwned;

    fn to_http_request(&self, client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError>;
}

/// Marker trait for Sia API responses.
///
/// This trait is used to indicate that a type can be used as a response
pub trait SiaApiResponse {}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConsensusTipRequest;

impl SiaApiRequest for ConsensusTipRequest {
    type Response = ConsensusTipResponse;

    fn to_http_request(&self, _client: &Client, base_url: &Url) -> Result<reqwest::Request, SiaApiClientError> {
        let endpoint_url = base_url
            .join(ENDPOINT_CONSENSUS_TIP)
            .map_err(SiaApiClientError::UrlParse)?;

        let request = reqwest::Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

/// The current consensus tip of the Sia network.
/// It's a ChainIndex pairing a block's height with its ID.
/// https://github.com/SiaFoundation/core/blob/4e46803f702891e7a83a415b7fcd7543b13e715e/types/types.go#L181
#[derive(Deserialize, Serialize, Debug)]
pub struct ConsensusTipResponse {
    pub height: u64,
    pub id: BlockID,
}

impl SiaApiResponse for ConsensusTipResponse {}

/// GET /addresses/:addr/balance
#[derive(Deserialize, Serialize, Debug)]
pub struct AddressBalanceRequest {
    pub address: Address,
}

impl SiaApiRequest for AddressBalanceRequest {
    type Response = AddressBalanceResponse;

    fn to_http_request(&self, _client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = format!("api/addresses/{}/balance", self.address);
        let endpoint_url = base_url.join(&endpoint_path).map_err(SiaApiClientError::UrlParse)?;

        let request = Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

/// The balance response of for a Sia address.
/// https://github.com/SiaFoundation/walletd/blob/9574e69ff0bf84de1235b68e78db2a41d5e27516/api/api.go#L36
/// https://github.com/SiaFoundation/walletd/blob/9574e69ff0bf84de1235b68e78db2a41d5e27516/wallet/wallet.go#L25
#[derive(Deserialize, Serialize, Debug)]
pub struct AddressBalanceResponse {
    pub siacoins: Currency,
    #[serde(rename = "immatureSiacoins")]
    pub immature_siacoins: Currency,
    pub siafunds: u64,
}

impl SiaApiResponse for AddressBalanceResponse {}

/// GET /events/:id
#[derive(Deserialize, Serialize, Debug)]
pub struct EventsTxidRequest {
    pub txid: H256,
}

impl SiaApiRequest for EventsTxidRequest {
    type Response = EventsTxidResponse;

    fn to_http_request(&self, _client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = format!("api/events/{}", self.txid);
        let endpoint_url = base_url.join(&endpoint_path).map_err(SiaApiClientError::UrlParse)?;

        let request = Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

#[derive(Deserialize, Serialize)]
pub struct EventsTxidResponse(pub Event);

impl SiaApiResponse for EventsTxidResponse {}

/// GET /addresses/:addr/events
#[derive(Deserialize, Serialize, Debug)]
pub struct AddressesEventsRequest {
    pub address: Address,
}

impl SiaApiRequest for AddressesEventsRequest {
    type Response = Vec<Event>;

    fn to_http_request(&self, _client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = format!("api/addresses/{}/events", self.address);
        let endpoint_url = base_url.join(&endpoint_path).map_err(SiaApiClientError::UrlParse)?;

        let request = Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

pub type AddressesEventsResponse = Vec<Event>;

impl SiaApiResponse for Vec<Event> {}

/// The request to get the unspent transaction outputs (UTXOs) for a Sia address.
/// GET /addresses/:addr/outputs/siacoin
#[derive(Deserialize, Serialize, Debug)]
pub struct AddressUtxosRequest {
    pub address: Address,
}

pub type AddressUtxosResponse = Vec<SiacoinElement>;

impl SiaApiResponse for AddressUtxosResponse {}

impl SiaApiRequest for AddressUtxosRequest {
    type Response = AddressUtxosResponse;

    fn to_http_request(&self, _client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = format!("api/addresses/{}/outputs/siacoin", self.address);
        let endpoint_url = base_url.join(&endpoint_path).map_err(SiaApiClientError::UrlParse)?;

        let request = Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

/// POST /txpool/broadcast
#[derive(Deserialize, Serialize, Debug)]
pub struct TxpoolBroadcastRequest {
    pub transactions: Vec<V1Transaction>,
    pub v2transactions: Vec<V2Transaction>,
}

impl SiaApiRequest for TxpoolBroadcastRequest {
    type Response = EmptyResponse;

    fn to_http_request(&self, client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = "api/txpool/broadcast";
        let endpoint_url = base_url.join(endpoint_path).map_err(SiaApiClientError::UrlParse)?;

        let json_body = serde_json::to_string(self).map_err(SiaApiClientError::SerializationError)?;

        let request = client
            .post(endpoint_url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json_body)
            .build()
            .map_err(SiaApiClientError::ReqwestError)?;
        Ok(request)
    }
}

/// GET /txpool/fee
#[derive(Deserialize, Serialize, Debug)]
pub struct TxpoolFeeRequest;

#[derive(Deserialize, Serialize, Debug)]
pub struct TxpoolFeeResponse(pub Currency);

impl SiaApiResponse for TxpoolFeeResponse {}

impl SiaApiRequest for TxpoolFeeRequest {
    type Response = TxpoolFeeResponse;

    fn to_http_request(&self, client: &Client, base_url: &Url) -> Result<Request, SiaApiClientError> {
        let endpoint_path = "api/txpool/fee";
        let endpoint_url = base_url.join(endpoint_path).map_err(SiaApiClientError::UrlParse)?;
        let request = Request::new(Method::GET, endpoint_url);
        Ok(request)
    }
}

#[derive(Serialize, Debug, Default)]
pub struct EmptyResponse;

impl<'de> Deserialize<'de> for EmptyResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(EmptyResponse)
        } else {
            Err(serde::de::Error::custom("expected an empty string"))
        }
    }
}

impl SiaApiResponse for EmptyResponse {}
