use crate::http::client::{ApiClient, ApiClientError, ApiClientHelpers, Body, EndpointSchema, SchemaMethod};
use crate::http::endpoints::{ConsensusTipRequest, SiaApiRequest};

use async_trait::async_trait;
use http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

pub mod wasm_fetch;
use wasm_fetch::{Body as FetchBody, FetchMethod, FetchRequest, FetchResponse};

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

    async fn new(conf: Self::Conf) -> Result<Self, ApiClientError> {
        let client = Client {
            base_url: conf.server_url,
            headers: conf.headers,
        };
        // Ping the server with ConsensusTipRequest to check if the client is working
        client.dispatcher(ConsensusTipRequest).await?;
        Ok(client)
    }

    fn process_schema(&self, schema: EndpointSchema) -> Result<Self::Request, ApiClientError> {
        let url = schema.build_url(&self.base_url)?;
        let method = match schema.method {
            SchemaMethod::Get => FetchMethod::Get,
            SchemaMethod::Post => FetchMethod::Post,
            _ => return Err(ApiClientError::FixmePlaceholder("Unsupported method".to_string())),
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

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, ApiClientError> {
        request
            .execute()
            .await
            .map_err(|e| ApiClientError::FixmePlaceholder(format!("FIXME {}", e)))
    }

    // Dispatcher function that converts the request and handles execution
    async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, ApiClientError> {
        let request = self.to_data_request(request)?; // Convert request to data request

        // Execute the request
        let response = self.execute_request(request).await?;

        match response.status {
            StatusCode::OK => {
                let response_body = match response.body {
                    Some(FetchBody::Json(body)) => serde_json::from_value(body).map_err(ApiClientError::Serde)?,
                    Some(FetchBody::Utf8(body)) => serde_json::from_str(&body).map_err(ApiClientError::Serde)?,
                    _ => {
                        return Err(ApiClientError::FixmePlaceholder(
                            "Unsupported body type in response".to_string(),
                        ))
                    },
                };
                Ok(response_body)
            },
            StatusCode::NO_CONTENT => {
                if let Some(resp_type) = R::is_empty_response() {
                    Ok(resp_type)
                } else {
                    Err(ApiClientError::UnexpectedEmptyResponse {
                        expected_type: std::any::type_name::<R::Response>().to_string(),
                    })
                }
            },
            status => {
                // Extract the body, using the Display implementation of Body or an empty string
                let body = response
                    .body
                    .map(|b| format!("{}", b)) // Use Display trait to format Body
                    .unwrap_or_else(|| "".to_string()); // If body is None, use an empty string
    
                Err(ApiClientError::UnexpectedHttpStatus {
                    status,
                    body,
                })
            }
        }
    }
}

// Implement the optional helper methods for ExampleClient
// Just this is needed to implement the `ApiClientHelpers` trait
// unless custom implementations for the traits methods are needed
#[async_trait]
impl ApiClientHelpers for Client {}

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use blake2b_simd::Hash;
    use once_cell::sync::Lazy;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;
    use log::info;

    wasm_bindgen_test_configure!(run_in_browser);

    static CONF: Lazy<Conf> = Lazy::new(|| Conf {
        server_url: Url::parse("https://sia-walletd.komodo.earth/").unwrap(),
        headers: HashMap::new(),
    });

    #[wasm_bindgen_test]
    async fn test_sia_wasm_client_wip() {
        use crate::http::endpoints::TxpoolBroadcastRequest;
        use crate::transaction::V2Transaction;
        let client = Client::new(CONF.clone()).await.unwrap();

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
