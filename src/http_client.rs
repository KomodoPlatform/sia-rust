use crate::http_endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};
use crate::types::Address;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
#[cfg(not(target_arch = "wasm32"))] use core::time::Duration;
use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Error as ReqwestError, Url};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SiaHttpConf {
    pub url: Url,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct SiaApiClient {
    client: Client,
    conf: SiaHttpConf,
}

// TODO clean up reqwest errors
// update reqwest to latest for `.with_url()` method
#[derive(Debug, Display)]
pub enum SiaApiClientError {
    Timeout(String),
    BuildError(String),
    ServerUnreachable(String),
    ReqwestError(ReqwestError),
    UrlParse(url::ParseError),
    UnexpectedHttpStatus(u16),
    ApiInternalError(String),
    SerializationError(serde_json::Error),
    UnexpectedEmptyResponse { expected_type: String },
}

impl From<SiaApiClientError> for String {
    fn from(e: SiaApiClientError) -> Self { format!("{:?}", e) }
}

/// Implements the methods for sending specific requests and handling their responses.
impl SiaApiClient {
    /// Constructs a new instance of the API client using the provided base URL and password for authentication.
    pub async fn new(conf: SiaHttpConf) -> Result<Self, SiaApiClientError> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Basic {}", BASE64.encode(format!(":{}", conf.password)));
        headers.insert(
            AUTHORIZATION,
            // This error does not require a test case as it is impossible to trigger in practice
            // the from_str method can only return Err if the str is invalid ASCII
            // the encode() method can only return valid ASCII
            HeaderValue::from_str(&auth_value).map_err(|e| SiaApiClientError::BuildError(e.to_string()))?,
        );
        //let proxy = Proxy::http("http://127.0.0.1:8080").unwrap(); TODO remove debugging code
        let client_builder = Client::builder()
            //.proxy(proxy)
            .default_headers(headers);

        #[cfg(not(target_arch = "wasm32"))]
        // TODO make this configurable and add timeout for wasm using `fetch_and_parse`
        let client_builder = client_builder.timeout(Duration::from_secs(10));

        let client = client_builder
            .build()
            // covering this with a unit test seems to require altering the system's ssl certificates
            .map_err(SiaApiClientError::ReqwestError)?;
        let ret = SiaApiClient { client, conf };
        ret.dispatcher(ConsensusTipRequest).await?;
        Ok(ret)
    }

    /// General method for dispatching requests, handling routing and response parsing.
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
                if let Some(empty_response) = R::is_empty_response() {
                    Ok(empty_response)
                } else {
                    Err(SiaApiClientError::UnexpectedEmptyResponse {
                        expected_type: std::any::type_name::<R::Response>().to_string(),
                    })
                }
            },
            _ => Err(SiaApiClientError::UnexpectedHttpStatus(response.status().as_u16())),
        }
    }

    pub async fn current_height(&self) -> Result<u64, SiaApiClientError> {
        let response = self.dispatcher(ConsensusTipRequest).await?;
        Ok(response.height)
    }

    pub async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, SiaApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}
