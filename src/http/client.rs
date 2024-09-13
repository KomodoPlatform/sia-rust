use crate::http::endpoints::{AddressBalanceResponse, SiaApiRequest};

use crate::types::Address;
use async_trait::async_trait;
use derive_more::Display;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

#[cfg(not(target_arch = "wasm32"))] pub mod native;

#[cfg(not(target_arch = "wasm32"))]
use reqwest::Error as ReqwestError;

#[cfg(target_arch = "wasm32")] use crate::http::wasm::FetchError;

pub struct EndpointSchema {
    // FIXME path_schema can probably be &'static str
    pub path_schema: String,                           // The endpoint path template
    pub path_params: Option<HashMap<String, String>>,  // Optional parameters to replace in the path
    pub query_params: Option<HashMap<String, String>>, // Optional query parameters
    pub method: http::Method,                          // The HTTP method (e.g., GET, POST)
    pub body: Body,                                    // Optional body for POST and POST-like requests
}

pub struct EndpointSchemaBuilder {
    path_schema: String,
    path_params: Option<HashMap<String, String>>,
    query_params: Option<HashMap<String, String>>,
    method: http::Method,
    body: Body,
}

impl EndpointSchemaBuilder {
    pub fn new(path_schema: String, method: http::Method) -> Self {
        Self {
            path_schema,
            path_params: None,
            query_params: None,
            method,
            body: Body::None,
        }
    }

    pub fn path_params(mut self, path_params: HashMap<String, String>) -> Self {
        self.path_params = Some(path_params);
        self
    }

    pub fn query_params(mut self, query_params: HashMap<String, String>) -> Self {
        self.query_params = Some(query_params);
        self
    }

    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn build(self) -> EndpointSchema {
        EndpointSchema {
            path_schema: self.path_schema,
            path_params: self.path_params,
            query_params: self.query_params,
            method: self.method,
            body: self.body,
        }
    }
}

pub enum Body {
    Utf8(String),
    Json(JsonValue),
    Bytes(Vec<u8>),
    None,
}

impl EndpointSchema {
    // Safely build the URL using percent-encoding for path params
    pub fn build_url(&self, base_url: &Url) -> Result<Url, ApiClientError> {
        let mut path = self.path_schema.clone();

        // Replace placeholders in the path with encoded values if path_params are provided
        if let Some(params) = &self.path_params {
            for (key, value) in params {
                let encoded_value = utf8_percent_encode(value, NON_ALPHANUMERIC).to_string();
                path = path.replace(&format!("{{{}}}", key), &encoded_value); // Use {} for parameters
            }
        }

        // Combine base_url with the constructed path
        let mut url = base_url.join(&path).map_err(ApiClientError::UrlParse)?;

        // Add query parameters if any
        if let Some(query_params) = &self.query_params {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query_params {
                let encoded_value = utf8_percent_encode(value, NON_ALPHANUMERIC).to_string();
                pairs.append_pair(key, &encoded_value);
            }
        }

        Ok(url)
    }
}

#[async_trait]
pub trait ApiClient: Clone {
    type Request;
    type Response;

    async fn new(conf: ClientConf) -> Result<Self, ApiClientError>
    where
        Self: Sized;

    fn process_schema(&self, schema: EndpointSchema) -> Result<Self::Request, ApiClientError>;

    fn to_data_request<R: SiaApiRequest>(&self, request: R) -> Result<Self::Request, ApiClientError>;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, ApiClientError>;

    // A generic dispatcher should be possible if Execute::Response is a serde deserializable type
    async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, ApiClientError>;
}

#[async_trait]
pub trait ApiClientHelpers {
    async fn current_height(&self) -> Result<u64, ApiClientError>;

    async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, ApiClientError>;
}

#[cfg(not(target_arch = "wasm32"))]
// FIXME this can add a generic argument to allow client specific configs
// pub client_specific_config: Option<T>,
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientConf {
    pub url: Url,
    pub password: String, // FIXME must be Option
    pub timeout: Option<u64>,
}

// TODO clean up reqwest errors
// update reqwest to latest for `.with_url()` method
#[derive(Debug, Display, Error)]
pub enum ApiClientError {
    BuildError(String),
    UrlParse(#[from] url::ParseError),
    UnexpectedHttpStatus(http::StatusCode),
    SerializationError(#[from] serde_json::Error),
    UnexpectedEmptyResponse {
        expected_type: String,
    },
    #[cfg(target_arch = "wasm32")]
    WasmFetchError(#[from] FetchError),
    #[cfg(not(target_arch = "wasm32"))]
    ReqwestError(#[from] ReqwestError), // FIXME remove this; it should be generalized enough to not need arch-specific error types
}

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use common::log::info;
    use common::log::wasm_log::register_wasm_log;
    use once_cell::sync::Lazy;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    static CONF: Lazy<ClientConf> = Lazy::new(|| ClientConf {
        url: Url::parse("https://sia-walletd.komodo.earth/").unwrap(),
        password: "password".to_string(),
    });

    fn init_test_env() { register_wasm_log(); }

    #[wasm_bindgen_test]
    async fn test_dispatcher_invalid_base_url() {
        let bad_conf = ClientConf {
            url: Url::parse("http://user:password@example.com").unwrap(),
            password: "password".to_string(),
        };

        let client = SiaApiClient::new(bad_conf).await.unwrap();
    }

    #[wasm_bindgen_test]
    async fn test_sia_wasm_client_wip() {
        use crate::http::endpoints::TxpoolBroadcastRequest;
        use crate::transaction::V2Transaction;
        init_test_env();
        let client = SiaApiClient::new(CONF.clone()).await.unwrap();

        let tx_str = r#"
        {
            "siacoinInputs": [
                {
                "parent": {
                    "id": "h:27248ab562cbbee260e07ccae87c74aae71c9358d7f91eee25837e2011ce36d3",
                    "leafIndex": 21867,
                    "merkleProof": [
                    "h:ac2fdcbed40f103e54b0b1a37c20a865f6f1f765950bc6ac358ff3a0e769da50",
                    "h:b25570eb5c106619d4eef5ad62482023df7a1c7461e9559248cb82659ebab069",
                    "h:baa78ec23a169d4e9d7f801e5cf25926bf8c29e939e0e94ba065b43941eb0af8",
                    "h:239857343f2997462bed6c253806cf578d252dbbfd5b662c203e5f75d897886d",
                    "h:ad727ef2112dc738a72644703177f730c634a0a00e0b405bd240b0da6cdfbc1c",
                    "h:4cfe0579eabafa25e98d83c3b5d07ae3835ce3ea176072064ea2b3be689e99aa",
                    "h:736af73aa1338f3bc28d1d8d3cf4f4d0393f15c3b005670f762709b6231951fc"
                    ],
                    "siacoinOutput": {
                    "value": "772999980000000000000000000",
                    "address": "addr:1599ea80d9af168ce823e58448fad305eac2faf260f7f0b56481c5ef18f0961057bf17030fb3"
                    },
                    "maturityHeight": 0
                },
                "satisfiedPolicy": {
                    "policy": {
                    "type": "pk",
                    "policy": "ed25519:968e286ef5df3954b7189c53a0b4b3d827664357ebc85d590299b199af46abad"
                    },
                    "signatures": [
                    "sig:7a2c332fef3958a0486ef5e55b70d2a68514ff46d9307a85c3c0e40b76a19eebf4371ab3dd38a668cefe94dbedff2c50cc67856fbf42dce2194b380e536c1500"
                    ]
                }
                }
            ],
            "siacoinOutputs": [
                {
                "value": "2000000000000000000000000",
                "address": "addr:1d9a926b1e14b54242375c7899a60de883c8cad0a45a49a7ca2fdb6eb52f0f01dfe678918204"
                },
                {
                "value": "770999970000000000000000000",
                "address": "addr:1599ea80d9af168ce823e58448fad305eac2faf260f7f0b56481c5ef18f0961057bf17030fb3"
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
        let resp = client.dispatcher(req).await.unwrap();
    }
}
