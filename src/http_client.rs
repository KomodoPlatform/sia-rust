use crate::http_endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};
use crate::types::Address;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use core::fmt::Display;
#[cfg(not(target_arch = "wasm32"))] use core::time::Duration;
use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Error as ReqwestError, Request, Url};
use serde::de::DeserializeOwned;
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

/// Generic function to fetch data from a URL and deserialize it into a specified type.
async fn fetch_and_parse<T: DeserializeOwned>(client: &Client, request: Request) -> Result<T, SiaApiClientError> {
    let url = request.url().clone();
    let fetched = client.execute(request).await.map_err(|e| {
        SiaApiClientError::ReqwestFetchError(ReqwestErrorWithUrl {
            error: e,
            url: url.clone(),
        })
    })?;

    let status = fetched.status().as_u16();
    let response_text = match status {
        200 | 500 => fetched.text().await.map_err(|e| {
            SiaApiClientError::ReqwestParseInvalidEncodingError(
                ReqwestErrorWithUrl {
                    error: e,
                    url: url.clone(),
                }
                .to_string(),
            )
        })?,
        s => return Err(SiaApiClientError::UnexpectedHttpStatus(s)),
    };

    if status == 500 {
        return Err(SiaApiClientError::ApiInternalError(response_text));
    }

    let json: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        SiaApiClientError::ReqwestParseInvalidJsonError(format!(
            "Failed to parse response as JSON. Response: '{}'. Error: {}",
            response_text, e
        ))
    })?;

    let parsed: T = serde_json::from_value(json.clone()).map_err(|e| {
        SiaApiClientError::ReqwestParseUnexpectedTypeError(format!(
            "JSON response does not match the expected type '{:?}'. Response: '{}'. Error: {}",
            std::any::type_name::<T>(),
            json.to_string(),
            e
        ))
    })?;

    Ok(parsed)
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
            .map_err(|e| {
                SiaApiClientError::ReqwestTlsError(ReqwestErrorWithUrl {
                    error: e,
                    url: conf.url.clone(),
                })
            })?;
        let ret = SiaApiClient { client, conf };
        ret.dispatcher(ConsensusTipRequest).await?;
        Ok(ret)
    }

    /// General method for dispatching requests, handling routing and response parsing.
    pub async fn dispatcher<R: SiaApiRequest + Send>(&self, request: R) -> Result<R::Response, SiaApiClientError> {
        let req = request.to_http_request(&self.client, &self.conf.url)?;
        fetch_and_parse::<R::Response>(&self.client, req).await
    }

    pub async fn current_height(&self) -> Result<u64, SiaApiClientError> {
        let response = self.dispatcher(ConsensusTipRequest).await?;
        Ok(response.height)
    }

    pub async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, SiaApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}
