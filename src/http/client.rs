use crate::http::endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};
#[cfg(target_arch = "wasm32")]
use crate::http::wasm::FetchError;
use crate::types::Address;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use derive_more::Display;
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use url::Url;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))] use core::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{Client, Error as ReqwestError};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SiaHttpConf {
    pub url: Url,
    pub password: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct HttpClient {
    pub headers: HeaderMap,
}

#[cfg(not(target_arch = "wasm32"))]
pub type HttpClient = Client;

#[derive(Clone, Debug)]
pub struct SiaApiClient {
    client: HttpClient,
    conf: SiaHttpConf,
}

// TODO clean up reqwest errors
// update reqwest to latest for `.with_url()` method
#[derive(Debug, Display, Error)]
pub enum SiaApiClientError {
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
    ReqwestError(#[from] ReqwestError),
}

/// Implements the methods for sending specific requests and handling their responses.
impl SiaApiClient {
    /// Constructs a new instance of the API client using the provided base URL and password for authentication.
    pub async fn new(conf: SiaHttpConf) -> Result<Self, SiaApiClientError> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Basic {}", BASE64.encode(format!(":{}", conf.password)));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|e| SiaApiClientError::BuildError(e.to_string()))?,
        );

        #[cfg(not(target_arch = "wasm32"))]
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(SiaApiClientError::ReqwestError)?;

        #[cfg(target_arch = "wasm32")]
        let client = HttpClient { headers };

        let ret = SiaApiClient { client, conf };
        ret.dispatcher(ConsensusTipRequest).await?;
        Ok(ret)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, SiaApiClientError>
    where
        Self: Send,
    {
        let _req = request.to_http_request(&self.client, &self.conf.url)?;
        todo!()
    }

    /// General method for dispatching requests, handling routing and response parsing.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, SiaApiClientError> {
        let req = request.to_http_request(&self.client, &self.conf.url)?;
        let response = self
            .client
            .execute(req)
            .await
            .map_err(SiaApiClientError::ReqwestError)?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response
                .json::<R::Response>()
                .await
                .map_err(SiaApiClientError::ReqwestError)?),
            reqwest::StatusCode::NO_CONTENT => {
                R::is_empty_response().ok_or(SiaApiClientError::UnexpectedEmptyResponse {
                    expected_type: std::any::type_name::<R::Response>().to_string(),
                })
            },
            _ => Err(SiaApiClientError::UnexpectedHttpStatus(response.status())),
        }
    }

    pub async fn current_height(&self) -> Result<u64, SiaApiClientError> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    pub async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, SiaApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}

#[cfg(test)]
mod wasm_tests {
    use super::*;
    use common::log::info;
    use common::log::wasm_log::register_wasm_log;
    use once_cell::sync::Lazy;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    static CONF: Lazy<SiaHttpConf> = Lazy::new(|| SiaHttpConf {
        url: Url::parse("https://sia-walletd.komodo.earth/").unwrap(),
        password: "password".to_string(),
    });

    fn init_test_env() { register_wasm_log(); }

    #[wasm_bindgen_test]
    async fn test_dispatcher_invalid_base_url() {
        let bad_conf = SiaHttpConf {
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
