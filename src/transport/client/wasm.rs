use crate::transport::client::{ApiClient, ApiClientHelpers, Body, EndpointSchema, EndpointSchemaError, SchemaMethod};
use crate::transport::endpoints::{ConsensusTipRequest, SiaApiRequest, SiaApiRequestError};

use async_trait::async_trait;
use http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

pub mod wasm_fetch;
use wasm_fetch::{Body as FetchBody, FetchError, FetchMethod, FetchRequest, FetchResponse};

pub mod error {
    use super::*;
    use crate::transport::client::helpers::generic_errors::*;

    pub type BroadcastTransactionError = BroadcastTransactionErrorGeneric<ClientError>;
    pub type UtxoFromTxidError = UtxoFromTxidErrorGeneric<ClientError>;
    pub type GetUnconfirmedTransactionError = GetUnconfirmedTransactionErrorGeneric<ClientError>;
    pub type GetMedianTimestampError = GetMedianTimestampErrorGeneric<ClientError>;
    pub type FindWhereUtxoSpentError = FindWhereUtxoSpentErrorGeneric<ClientError>;
    pub type FundTxSingleSourceError = FundTxSingleSourceErrorGeneric<ClientError>;
    pub type GetConsensusUpdatesError = GetConsensusUpdatesErrorGeneric<ClientError>;
    pub type GetUnspentOutputsError = GetUnspentOutputsErrorGeneric<ClientError>;
    pub type CurrentHeightError = CurrentHeightErrorGeneric<ClientError>;
    pub type SelectUtxosError = SelectUtxosErrorGeneric<ClientError>;
    pub type GetTransactionError = GetTransactionErrorGeneric<ClientError>;

    /// An error that may occur when using the `WasmClient`.
    /// Each variant is used exactly once and represents a unique logical path in the code.
    #[derive(Debug, Error)]
    pub enum ClientError {
        #[error("WasmClient::new: Failed to ping server with ConsensusTipRequest: {0}")]
        PingServer(Box<ClientError>),
        #[error("WasmClient::process_schema: failed to build url: {0}")]
        SchemaBuildUrl(#[from] EndpointSchemaError),
        #[error("WasmClient::process_schema: unsupported EndpointSchema.method: {0:?}")]
        SchemaUnsupportedMethod(EndpointSchema),
        #[error("WasmClient::dispatcher: Failed to generate EndpointSchema from SiaApiRequest: {0}")]
        DispatcherGenerateSchema(#[from] SiaApiRequestError),
        #[error("WasmClient::dispatcher: process_schema failed: {0}")]
        DispatcherProcessSchema(Box<ClientError>),
        #[error("WasmClient::dispatcher: Failed to execute request: {0}")]
        DispatcherExecuteRequest(#[from] FetchError),
        #[error("WasmClient::dispatcher: expected utf-8 or JSON in response body, found octet-stream: {0:?}")]
        DispatcherUnexpectedBodyBytes(Vec<u8>),
        #[error("WasmClient::dispatcher: expected utf-8 or JSON in response body, found empty body")]
        DispatcherUnexpectedBodyEmpty,
        #[error("WasmClient::dispatcher: failed to deserialize response body from JSON: {0}")]
        DispatcherDeserializeBodyJson(serde_json::Error),
        #[error("WasmClient::dispatcher: failed to deserialize response body from string: {0}")]
        DispatcherDeserializeBodyUtf8(serde_json::Error),
        #[error("WasmClient::dispatcher: unexpected HTTP status:{status} body:{body:?}")]
        DispatcherUnexpectedHttpStatus {
            status: StatusCode,
            body: Option<FetchBody>,
        },
        #[error("WasmClient::dispatcher: Expected:{expected_type} found 204 No Content")]
        DispatcherUnexpectedEmptyResponse { expected_type: String },
    }
}

use error::*;

#[derive(Clone)]
pub struct Client {
    pub base_url: Url,
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Conf {
    pub server_url: Url,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[async_trait]
impl ApiClient for Client {
    type Request = FetchRequest;
    type Response = FetchResponse;
    type Conf = Conf;
    type Error = ClientError;

    async fn new(conf: Self::Conf) -> Result<Self, Self::Error> {
        let client = Client {
            base_url: conf.server_url,
            headers: conf.headers,
        };
        // Ping the server with ConsensusTipRequest to check if the client is working
        client
            .dispatcher(ConsensusTipRequest)
            .await
            .map_err(|e| ClientError::PingServer(Box::new(e)))?;
        Ok(client)
    }

    fn process_schema(&self, schema: EndpointSchema) -> Result<Self::Request, Self::Error> {
        let url = schema.build_url(&self.base_url)?;
        let method = match schema.method {
            SchemaMethod::Get => FetchMethod::Get,
            SchemaMethod::Post => FetchMethod::Post,
            _ => return Err(ClientError::SchemaUnsupportedMethod(schema.clone())),
        };
        let body = match schema.body {
            Body::Utf8(body) => Some(FetchBody::Utf8(body)),
            Body::Json(body) => Some(FetchBody::Json(body)),
            Body::Bytes(body) => Some(FetchBody::Bytes(body)),
            Body::None => None,
        };
        Ok(FetchRequest {
            uri: url,
            method,
            headers: self.headers.clone(),
            body,
        })
    }

    // Dispatcher function that converts the request and handles execution
    async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, Self::Error> {
        // Generate EndpointSchema from the SiaApiRequest
        let schema = request.to_endpoint_schema()?;

        // Convert the SiaApiRequest to FetchRequest
        let request = self
            .process_schema(schema)
            .map_err(|e| ClientError::DispatcherProcessSchema(Box::new(e)))?;

        // Execute the FetchRequest
        let response = request.execute().await?;

        match response.status {
            // Deserialize the response body if 200 OK
            StatusCode::OK => {
                let response_body = match response.body {
                    Some(FetchBody::Json(body)) => {
                        serde_json::from_value(body).map_err(ClientError::DispatcherDeserializeBodyJson)?
                    },
                    Some(FetchBody::Utf8(body)) => {
                        serde_json::from_str(&body).map_err(ClientError::DispatcherDeserializeBodyUtf8)?
                    },
                    Some(FetchBody::Bytes(bytes)) => return Err(ClientError::DispatcherUnexpectedBodyBytes(bytes)),
                    None => return Err(ClientError::DispatcherUnexpectedBodyEmpty),
                };
                Ok(response_body)
            },
            // Return an EmptyResponse if 204 NO CONTENT
            StatusCode::NO_CONTENT => {
                if let Some(resp_type) = R::is_empty_response() {
                    Ok(resp_type)
                } else {
                    Err(ClientError::DispatcherUnexpectedEmptyResponse {
                        expected_type: std::any::type_name::<R::Response>().to_string(),
                    })
                }
            },
            // Handle unexpected HTTP statuses eg, 400, 404, 500
            status => Err(ClientError::DispatcherUnexpectedHttpStatus {
                status,
                body: response.body,
            }),
        }
    }
}

// Just this is needed to implement the `ApiClientHelpers` trait
// unless custom implementations for the traits methods are needed
#[async_trait]
impl ApiClientHelpers for Client {}

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use once_cell::sync::Lazy;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    static CONF: Lazy<Conf> = Lazy::new(|| Conf {
        server_url: Url::parse("https://sia-walletd.komodo.earth/").unwrap(),
        headers: HashMap::new(),
    });

    // #[ignore] -- FIXME Alright must use docker container or mock server
    // #[wasm_bindgen_test]
    async fn test_sia_wasm_client_client_error() {
        use crate::transport::endpoints::TxpoolBroadcastRequest;
        use crate::types::V2Transaction;
        let client = Client::new(CONF.clone()).await.unwrap();

        let tx_str = r#"
        {
            "siacoinInputs": [
                {
                "parent": {
                    "id": "27248ab562cbbee260e07ccae87c74aae71c9358d7f91eee25837e2011ce36d3",
                    "leafIndex": 21867,
                    "merkleProof": [
                    "ac2fdcbed40f103e54b0b1a37c20a865f6f1f765950bc6ac358ff3a0e769da50",
                    "b25570eb5c106619d4eef5ad62482023df7a1c7461e9559248cb82659ebab069",
                    "baa78ec23a169d4e9d7f801e5cf25926bf8c29e939e0e94ba065b43941eb0af8",
                    "239857343f2997462bed6c253806cf578d252dbbfd5b662c203e5f75d897886d",
                    "ad727ef2112dc738a72644703177f730c634a0a00e0b405bd240b0da6cdfbc1c",
                    "4cfe0579eabafa25e98d83c3b5d07ae3835ce3ea176072064ea2b3be689e99aa",
                    "736af73aa1338f3bc28d1d8d3cf4f4d0393f15c3b005670f762709b6231951fc"
                    ],
                    "siacoinOutput": {
                    "value": "772999980000000000000000000",
                    "address": "1599ea80d9af168ce823e58448fad305eac2faf260f7f0b56481c5ef18f0961057bf17030fb3"
                    },
                    "maturityHeight": 0
                },
                "satisfiedPolicy": {
                    "policy": {
                    "type": "pk",
                    "policy": "ed25519:968e286ef5df3954b7189c53a0b4b3d827664357ebc85d590299b199af46abad"
                    },
                    "signatures": [
                    "7a2c332fef3958a0486ef5e55b70d2a68514ff46d9307a85c3c0e40b76a19eebf4371ab3dd38a668cefe94dbedff2c50cc67856fbf42dce2194b380e536c1500"
                    ]
                }
                }
            ],
            "siacoinOutputs": [
                {
                "value": "2000000000000000000000000",
                "address": "1d9a926b1e14b54242375c7899a60de883c8cad0a45a49a7ca2fdb6eb52f0f01dfe678918204"
                },
                {
                "value": "770999970000000000000000000",
                "address": "1599ea80d9af168ce823e58448fad305eac2faf260f7f0b56481c5ef18f0961057bf17030fb3"
                }
            ],
            "minerFee": "10000000000000000000"
        }
        "#;
        let tx: V2Transaction = serde_json::from_str(tx_str).unwrap();
        let req = TxpoolBroadcastRequest {
            transactions: vec![],
            v2transactions: vec![tx],
        };
        match client.dispatcher(req).await.expect_err("Expected HTTP 400 error") {
            ClientError::DispatcherUnexpectedHttpStatus {
                status: StatusCode::BAD_REQUEST,
                body: _,
            } => (),
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
