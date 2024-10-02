use crate::transport::endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};

use crate::types::Address;
use async_trait::async_trait;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;

#[cfg(not(target_arch = "wasm32"))] pub mod native;
#[cfg(target_arch = "wasm32")] pub mod wasm;

// FIXME remove these client specific error types
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Error as ReqwestError;

#[cfg(target_arch = "wasm32")] use wasm::wasm_fetch::FetchError;

// Client implementation is generalized
// This allows for different client implementations (e.g., WebSocket, libp2p, etc.)
// Any client implementation must implement the ApiClient trait and optionally ApiClientHelpers
#[async_trait]
pub trait ApiClient: Clone {
    type Request;
    type Response;
    type Conf;

    async fn new(conf: Self::Conf) -> Result<Self, ApiClientError>
    where
        Self: Sized;

    fn process_schema(&self, schema: EndpointSchema) -> Result<Self::Request, ApiClientError>;

    fn to_data_request<R: SiaApiRequest>(&self, request: R) -> Result<Self::Request, ApiClientError> {
        self.process_schema(request.to_endpoint_schema()?)
    }

    // TODO this can have a default implementation if an associated type can provide .execute()
    // eg self.client().execute(request).await.map_err(Self::ClientError)
    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, ApiClientError>;

    // TODO default implementation should be possible if Execute::Response is a serde deserializable type
    async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, ApiClientError>;
}

#[async_trait]
pub trait ApiClientHelpers: ApiClient {
    async fn current_height(&self) -> Result<u64, ApiClientError> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, ApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("BuildError error: {0}")]
    BuildError(String),
    #[error("FixmePlaceholder error: {0}")]
    FixmePlaceholder(String), // FIXME this entire enum needs refactoring to not use client-specific error types
    #[error("UrlParse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("UnexpectedHttpStatus error: status:{status} body:{body}")]
    UnexpectedHttpStatus { status: http::StatusCode, body: String },
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("UnexpectedEmptyResponse error: {expected_type}")]
    UnexpectedEmptyResponse { expected_type: String },
    #[error("WasmFetchError error: {0}")]
    #[cfg(target_arch = "wasm32")]
    WasmFetchError(#[from] FetchError),
    #[error("ReqwestError error: {0}")]
    #[cfg(not(target_arch = "wasm32"))]
    ReqwestError(#[from] ReqwestError), // FIXME remove this; it should be generalized enough to not need arch-specific error types
}

// Not all client implementations will have an exact equivalent of HTTP methods
// However, the client implementation should be able to map the HTTP methods to its own methods
pub enum SchemaMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl From<SchemaMethod> for http::Method {
    fn from(method: SchemaMethod) -> Self {
        match method {
            SchemaMethod::Get => http::Method::GET,
            SchemaMethod::Post => http::Method::POST,
            SchemaMethod::Put => http::Method::PUT,
            SchemaMethod::Delete => http::Method::DELETE,
        }
    }
}

pub struct EndpointSchema {
    pub path_schema: String, // The endpoint path template (e.g., /api/transactions/{id})
    pub path_params: Option<HashMap<String, String>>, // Optional parameters to replace in the path (e.g., /{key} becomes /value)
    pub query_params: Option<HashMap<String, String>>, // Optional query parameters to add to the URL (e.g., ?key=value)
    pub method: SchemaMethod,                         // The method (e.g., Get, Post, Put, Delete)
    pub body: Body,                                   // Optional body for POST and POST-like requests
}

pub struct EndpointSchemaBuilder {
    path_schema: String,
    path_params: Option<HashMap<String, String>>,
    query_params: Option<HashMap<String, String>>,
    method: SchemaMethod,
    body: Body,
}

impl EndpointSchemaBuilder {
    pub fn new(path_schema: String, method: SchemaMethod) -> Self {
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
        let mut path = self.path_schema.to_string();

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
