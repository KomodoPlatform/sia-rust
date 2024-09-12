use crate::http::endpoints::{AddressBalanceRequest, AddressBalanceResponse, ConsensusTipRequest, SiaApiRequest};
use crate::types::Address;
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use url::Url;
use reqwest::{Client as ReqwestClient};

use crate::http::client::{ApiClient, ApiClientHelpers, ApiClientError, ClientConf, ProcessDataRequest};
use core::time::Duration;

#[derive(Clone)]
pub struct NativeClient {
    pub client: ReqwestClient,
    pub url: Url,
}

pub struct NativeDataRequest(pub reqwest::Request);

impl ProcessDataRequest for NativeDataRequest {
    fn to_data_request(&self, _endpoint_id: &EndpointIdentifier) -> Result<reqwest::RequestBuilder, ApiClientError> {
        Ok(self.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl ApiClient for NativeClient {
    type DataRequest = NativeDataRequest;

    async fn new(conf: ClientConf) -> Result<Self, ApiClientError> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Basic {}", BASE64.encode(format!(":{}", conf.password)));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|e| ApiClientError::BuildError(e.to_string()))?,
        );

        let timeout = conf.timeout.unwrap_or(10);
        let client = ReqwestClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(ApiClientError::ReqwestError)?;


        let ret = NativeClient { client, url: conf.url };
        ret.dispatcher(ConsensusTipRequest).await?;
        Ok(ret)
    }

    fn server_url(&self) -> &Url {
        &self.url
    }

    async fn dispatcher<R: SiaApiRequest>(&self, request: R) -> Result<R::Response, ApiClientError> {
        let req = request.to_data_request<DataRequest>(&self.client, &self.url)?;
        let response = self
            .client
            .execute(req)
            .await
            .map_err(ApiClientError::ReqwestError)?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response
                .json::<R::Response>()
                .await
                .map_err(ApiClientError::ReqwestError)?),
            reqwest::StatusCode::NO_CONTENT => {
                R::is_empty_response().ok_or(ApiClientError::UnexpectedEmptyResponse {
                    expected_type: std::any::type_name::<R::Response>().to_string(),
                })
            },
            _ => Err(ApiClientError::UnexpectedHttpStatus(response.status())),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl ApiClientHelpers for NativeClient {
    async fn current_height(&self) -> Result<u64, ApiClientError> {
        Ok(self.dispatcher(ConsensusTipRequest).await?.height)
    }

    async fn address_balance(&self, address: Address) -> Result<AddressBalanceResponse, ApiClientError> {
        self.dispatcher(AddressBalanceRequest { address }).await
    }
}