use crate::http_endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};
use crate::types::Address;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use derive_more::Display;
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use url::Url;

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
#[derive(Debug, Display)]
pub enum SiaApiClientError {
    BuildError(String),
    UrlParse(url::ParseError),
    UnexpectedHttpStatus(u16),
    SerializationError(serde_json::Error),
    UnexpectedEmptyResponse {
        expected_type: String,
    },
    #[cfg(target_arch = "wasm32")]
    FetchError(String),
    #[cfg(not(target_arch = "wasm32"))]
    ReqwestError(ReqwestError),
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
        let req = request.to_http_request(&self.client, &self.conf.url)?;
        let (status, response_string) = req
            .request_str()
            .await
            .map_err(|e| SiaApiClientError::FetchError(e.to_string()))?;

        match status.as_u16() {
            200 => Ok(serde_json::from_str(&response_string).map_err(SiaApiClientError::SerializationError)?),
            204 => Ok(
                R::is_empty_response().ok_or(SiaApiClientError::UnexpectedEmptyResponse {
                    expected_type: std::any::type_name::<R::Response>().to_string(),
                })?,
            ),
            _ => Err(SiaApiClientError::UnexpectedHttpStatus(status.as_u16())),
        }
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
            _ => Err(SiaApiClientError::UnexpectedHttpStatus(response.status().as_u16())),
        }
    }

    pub async fn current_height(&self) -> Result<u64, SiaApiClientError> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    pub async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, SiaApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}
