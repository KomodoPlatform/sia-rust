use crate::transport::client::{ApiClientError, Body, EndpointSchema, EndpointSchemaBuilder, SchemaMethod};
use crate::types::{Address, BlockID, Event, Hash256, Currency, SiacoinElement, V1Transaction, V2Transaction};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ENDPOINT_ADDRESSES_BALANCE: &str = "api/addresses/{address}/balance";
const ENDPOINT_ADDRESSES_EVENTS: &str = "api/addresses/{address}/events";
const ENDPOINT_ADDRESSES_UTXOS_SIACOIN: &str = "api/addresses/{address}/outputs/siacoin";
const ENDPOINT_CONSENSUS_TIP: &str = "api/consensus/tip";
const ENDPOINT_EVENTS: &str = "api/events/{txid}";
const ENDPOINT_TXPOOL_BROADCAST: &str = "api/txpool/broadcast";
const ENDPOINT_TXPOOL_FEE: &str = "api/txpool/fee";
const ENDPOINT_TXPOOL_TRANSACTIONS: &str = "api/txpool/transactions";

pub trait SiaApiRequest: Send {
    type Response: DeserializeOwned;

    // Applicable for requests that return HTTP 204 No Content
    fn is_empty_response() -> Option<Self::Response> { None }

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError>;
}

/// Represents the request-response pair for fetching the current consensus tip of the Sia network.
///
/// # Walletd Endpoint
/// `GET /consensus/tip`
///
/// # Description
/// Returns the current consensus tip of the Sia network. The consensus tip includes the current block's height
/// and its block ID, representing the latest state of the blockchain.
///
/// # Response
/// - The response is a `ConsensusTipResponse`, which contains the block's height and ID.
///   This corresponds to the `types.ChainIndex` type in Go.
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L158)
/// - [Go Source for the ChainIndex Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L194)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ConsensusTipRequest;

impl SiaApiRequest for ConsensusTipRequest {
    type Response = ConsensusTipResponse;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        Ok(EndpointSchemaBuilder::new(ENDPOINT_CONSENSUS_TIP.to_owned(), SchemaMethod::Get).build())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ConsensusTipResponse {
    pub height: u64,
    pub id: BlockID,
}

/// Represents the request-response pair for fetching the balance of an individual address.
///
/// # Walletd Endpoint
/// `GET /addresses/:addr/balance`
///
/// # Description
/// Retrieves the balance of the specified address. The behavior of this endpoint depends on the index mode:
/// - **Personal Index Mode**: The address must be associated with an existing wallet.
/// - **Full Index Mode**: The balance of any address can be checked.
///
/// # Fields
/// - `address`: The address for which to fetch the balance. In Go, this corresponds to `types.Address`.
///   - [Go Source for Address Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L165)
///
/// # Response
/// - The response provides two fields, `siacoins` and `immature_siacoins`, each representing the balance in Siacoins.
///   These are string representations of the `Currency` type in Go.
///   - [Go Source for Currency Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/currency.go#L26)
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L752)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AddressBalanceRequest {
    pub address: Address,
}

impl SiaApiRequest for AddressBalanceRequest {
    type Response = AddressBalanceResponse;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        let mut path_params = HashMap::new();
        path_params.insert("address".to_owned(), self.address.to_string());

        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_ADDRESSES_BALANCE.to_owned(), SchemaMethod::Get)
                .path_params(path_params) // Set the path parameters for the address
                .build(),
        )
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AddressBalanceResponse {
    pub siacoins: Currency,
    #[serde(rename = "immatureSiacoins")]
    pub immature_siacoins: Currency,
}

/// Represents the request-response pair for fetching a specific event by transaction ID (txid).
///
/// # Walletd Endpoint
/// `GET /events/:id`
///
/// # Description
/// Fetches an event based on the provided transaction ID (txid).
///
/// # Fields
/// - `txid`: The transaction ID for which to fetch the event. In Go, this corresponds to `types.Hash256`.
///   - [Go Source for Hash256](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L63)
///
/// # Response
/// - The response is an `Event` in Rust, corresponding to `types.Event` in Go.
///   - [Go Source for Event](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/wallet/wallet.go#L14)
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/134a28b063df60a687899ac33aa373bf461480bc/api/server.go#L828)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GetEventRequest {
    pub txid: Hash256,
}

impl SiaApiRequest for GetEventRequest {
    type Response = Event;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        // Create the path_params HashMap to substitute {txid} in the path schema
        let mut path_params = HashMap::new();
        path_params.insert("txid".to_owned(), self.txid.to_string());

        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_EVENTS.to_owned(), SchemaMethod::Get)
                .path_params(path_params) // Set the path params containing the txid
                .build(),
        )
    }
}

/// Represents the request-response pair for fetching events for a specific address.
///
/// # Walletd Endpoint
/// `GET /addresses/:addr/events`
///
/// # Fields
/// - `addr`: (`types.Address` in Go) the address for which events are fetched.
/// - `limit`: (`i64` in Go) optional limit for the number of results.
/// - `offset`: (`i64` in Go) optional offset for paginated results.
///
/// # Response
/// - `[]types.Event` in Go corresponds to `Vec<Event>` in Rust.
///   - An event represents an on-chain event capable of influencing the state of a wallet.
///   - As per comments in the Go source: "Events can either be created by sending Siacoins between
///     addresses or they can be created by consensus (e.g. a miner payout, a siafund claim, or a contract)."
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/134a28b063df60a687899ac33aa373bf461480bc/api/server.go#L761)
/// - [Go Source for the Event Object](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/wallet/wallet.go#L14)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AddressesEventsRequest {
    pub address: Address,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl SiaApiRequest for AddressesEventsRequest {
    type Response = Vec<Event>;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        let mut path_params = HashMap::new();
        path_params.insert("address".to_owned(), self.address.to_string());

        let mut query_params = HashMap::new();
        if let Some(limit) = self.limit {
            query_params.insert("limit".to_owned(), limit.to_string());
        }
        if let Some(offset) = self.offset {
            query_params.insert("offset".to_owned(), offset.to_string());
        }

        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_ADDRESSES_EVENTS.to_owned(), SchemaMethod::Get)
                .path_params(path_params) // Set the path params containing the address
                .query_params(query_params) // Set the query params for limit and offset
                .build(),
        )
    }
}

pub type AddressesEventsResponse = Vec<Event>;

/// Represents the request-response pair for getting Siacoin UTXOs owned by a specific address.
///
/// # Walletd Endpoint
/// `GET /addresses/:addr/outputs/siacoin`
///
/// # Description
/// Fetches any Siacoin unspent transaction outputs (UTXOs) owned by the specified address.
///
/// # Fields
/// - `address`: The address for which to fetch UTXOs. In Go, this corresponds to `types.Address`.
///   - [Go Source for Address Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L165)
/// - `limit`: An optional limit on the number of results. Corresponds to `i64` in Go.
/// - `offset`: An optional offset for paginated results. Corresponds to `i64` in Go.
///
/// # Response
/// - The response is a `Vec<SiacoinElement>` in Rust, corresponding to `[]types.SiacoinElement` in Go.
///   - [Go Source for SiacoinElement Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L614)
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L795)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GetAddressUtxosRequest {
    pub address: Address,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub type GetAddressUtxosResponse = Vec<SiacoinElement>;

impl SiaApiRequest for GetAddressUtxosRequest {
    type Response = GetAddressUtxosResponse;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        let mut path_params = HashMap::new();
        path_params.insert("address".to_owned(), self.address.to_string());

        let mut query_params = HashMap::new();
        if let Some(limit) = self.limit {
            query_params.insert("limit".to_owned(), limit.to_string());
        }
        if let Some(offset) = self.offset {
            query_params.insert("offset".to_owned(), offset.to_string());
        }

        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_ADDRESSES_UTXOS_SIACOIN.to_owned(), SchemaMethod::Get)
                .path_params(path_params) // Set the path params containing the address
                .query_params(query_params) // Set the query params for limit and offset
                .build(),
        )
    }
}

/// Represents the request-response pair for broadcasting transactions.
///
/// # Walletd Endpoint
/// `POST /txpool/broadcast`
///
/// # Description
/// Used for broadcasting transactions to the network. The request body consists of two arrays:
/// - `transactions`: an array of V1 transactions.
/// - `v2transactions`: an array of V2 transactions.
///
/// # Request Body
/// The body is structured as follows:
/// ```json
/// {
///   "transactions": [],
///   "v2transactions": []
/// }
/// ```
///
/// # Response
/// - The response is `HTTP 204 NO CONTENT`, which is represented by `EmptyResponse` in Rust.
///   This indicates that the request was successful but there is no response body.
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L293)
/// - [Go Source for the V1Transaction Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L390)
/// - [Go Source for the V2Transaction Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/types.go#L649)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TxpoolBroadcastRequest {
    pub transactions: Vec<V1Transaction>,
    pub v2transactions: Vec<V2Transaction>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EmptyResponse;

impl SiaApiRequest for TxpoolBroadcastRequest {
    type Response = EmptyResponse;

    fn is_empty_response() -> Option<Self::Response> { Some(EmptyResponse) }

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        // Serialize the transactions into a JSON body
        let body = serde_json::to_value(self).map_err(ApiClientError::Serde)?;
        let body = body.to_string();
        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_TXPOOL_BROADCAST.to_owned(), SchemaMethod::Post)
                .body(Body::Utf8(body)) // Set the JSON body for the POST request
                .build(),
        )
    }
}

/// Represents the request-response pair for fetching the current fee to broadcast a transaction.
///
/// # Walletd Endpoint
/// `GET /txpool/fee`
///
/// # Description
/// Returns the current fee to broadcast a transaction. The fee is the number of Hastings per byte.
/// To calculate how much a transaction will cost to broadcast, take its encoded size and multiply it
/// by the returned value.
///
/// Most transactions are less than 1000 bytes, so using 1000 bytes as a constant size will work for
/// most transactions.
///
/// # Response
/// - The response is a `types.Currency` from the Go codebase, represented as a `String` in Rust.
///   This value represents the number of Hastings per byte. Hastings is the smallest unit in Sia,
///   similar to Satoshis in Bitcoin.
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L289)
/// - [Go Source for the Currency Type](https://github.com/SiaFoundation/core/blob/300042fd2129381468356dcd87c5e9a6ad94c0ef/types/currency.go#L26)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TxpoolFeeRequest;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TxpoolFeeResponse(pub Currency);

impl SiaApiRequest for TxpoolFeeRequest {
    type Response = TxpoolFeeResponse;

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_TXPOOL_FEE.to_owned(), SchemaMethod::Get).build(), // No path_params, query_params, or body needed for this request
        )
    }
}

/// Represents the request-response pair for fetching all transactions in the transaction pool.
///
/// # Walletd Endpoint
/// `GET /txpool/transactions`
///
/// # Description
/// Returns all transactions currently in the transaction pool. This includes transactions not associated
/// with any registered wallet.
///
/// # Response
/// - This request returns `HTTP 204 NO CONTENT` in Go, which is represented by `EmptyResponse` in Rust.
///
/// # References
/// - [Go Source for the HTTP Endpoint](https://github.com/SiaFoundation/walletd/blob/6ff23fe34f6fa45a19bfb6e4bacc8a16d2c48144/api/server.go#L282C18-L282C43)
///
/// This type is ported from the Go codebase, representing the equivalent request-response pair in Rust.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TxpoolTransactionsRequest;

impl SiaApiRequest for TxpoolTransactionsRequest {
    type Response = EmptyResponse;

    fn is_empty_response() -> Option<Self::Response> { Some(EmptyResponse) }

    fn to_endpoint_schema(&self) -> Result<EndpointSchema, ApiClientError> {
        Ok(
            EndpointSchemaBuilder::new(ENDPOINT_TXPOOL_TRANSACTIONS.to_owned(), SchemaMethod::Get).build(), // No path_params, query_params, or body needed for this request
        )
    }
}
