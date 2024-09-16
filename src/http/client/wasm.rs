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
