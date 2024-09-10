use http::{HeaderMap, StatusCode};
use js_sys::Uint8Array;
use derive_more::Display;
use std::collections::HashMap;
use web_sys::{Request as JsRequest, Response as JsResponse, RequestInit, Window, WorkerGlobalScope};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// This is loosely based on the "mm2_net" crate found within Komodo DeFi Framework.

/// Get only the first line of the error.
/// Generally, the `JsValue` error contains the stack trace of an error.
/// This function cuts off the stack trace.
pub fn stringify_js_error(error: &JsValue) -> String {
    format!("{:?}", error)
        .lines()
        .next()
        .map(|e| e.to_owned())
        .unwrap_or_default()
}

#[derive(Debug, Display)]
pub enum FetchError {
    #[display(fmt = "Error deserializing '{}' response: {}", uri, error)]
    ErrorDeserializing { uri: String, error: String },
    #[display(fmt = "Transport '{}' error: {}", uri, error)]
    Transport { uri: String, error: String },
    InvalidStatusCode(http::status::InvalidStatusCode),
    InvalidHeadersInResponse(String),
    #[display(fmt = "Internal error: {}", _0)]
    Internal(String),
}

impl From<http::status::InvalidStatusCode> for FetchError {
    fn from(e: http::status::InvalidStatusCode) -> Self {
        FetchError::InvalidStatusCode(e)
    }
}

/// The result containing either a pair of (HTTP status code, body) or a stringified error.
pub type FetchResult<T> = Result<(StatusCode, T), FetchError>;

enum FetchMethod {
    Get,
    Post,
}

impl FetchMethod {
    fn as_str(&self) -> &'static str {
        match self {
            FetchMethod::Get => "GET",
            FetchMethod::Post => "POST",
        }
    }
}

pub enum Body {
    Utf8(String),
    Bytes(Vec<u8>),
}

impl Body {
    fn into_js_value(self) -> JsValue {
        match self {
            Body::Utf8(string) => JsValue::from_str(&string),
            Body::Bytes(bytes) => {
                let js_array = Uint8Array::from(bytes.as_slice());
                js_array.into()
            },
        }
    }
}

pub struct FetchResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>
}

impl FetchResponse {
    pub async fn from_js_response(response: JsResponse) -> Result<Self, FetchError> {

        let _status = StatusCode::from_u16(response.status()).map_err(FetchError::InvalidStatusCode)?;
        todo!()
         // TODO need to bump web-sys version to use .entires()
        // let headers = response.headers().entries();

        // let mut header_map = HashMap::new();
        // for header in headers {
        //     let header = header?;
        //     let key = header.get(0).as_string().map_err(|e| FetchError::InvalidHeadersInResponse("key is not utf-8".to_string()))?;
        //     let value = header.get(1).as_string().map_err(|e| FetchError::InvalidHeadersInResponse("value is not utf-8".to_string()))?;
        //     header_map.insert(key, value);
        // }

        // let body = None; // TODO
        // Ok(FetchResponse {
        //     status,
        //     headers: header_map,
        //     body: None,
        // })
    }
}

pub struct FetchRequest {
    uri: String,
    method: FetchMethod,
    headers: HashMap<String, String>,
    body: Option<Body>
}

impl FetchRequest {
    pub fn get(uri: &str) -> FetchRequest {
        FetchRequest {
            uri: uri.to_owned(),
            method: FetchMethod::Get,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn post(uri: &str) -> FetchRequest {
        FetchRequest {
            uri: uri.to_owned(),
            method: FetchMethod::Post,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn body_utf8(mut self, body: String) -> FetchRequest {
        self.body = Some(Body::Utf8(body));
        self
    }

    pub fn header_map(mut self, header_map: HeaderMap) -> FetchRequest {
        for (key, value) in header_map.iter() {
            if let Ok(val) = value.to_str() {
                self.headers.insert(key.as_str().to_owned(), val.to_owned());
            }
        }
        self
    }

    pub async fn fetch(request: Self) -> Result<FetchResponse, FetchError> {
        let uri = request.uri;

        let mut req_init = RequestInit::new();
        req_init.method(request.method.as_str());
        req_init.body(request.body.map(Body::into_js_value).as_ref());

        let js_request = JsRequest::new_with_str_and_init(&uri, &req_init)
            .map_err(|e| FetchError::Internal(stringify_js_error(&e)))?;


        for (hkey, hval) in request.headers {
            js_request
                .headers()
                .set(&hkey, &hval)
                .map_err(|e| FetchError::Internal(stringify_js_error(&e)))?;
        }

        let request_promise = compatible_fetch_with_request(&js_request)?;

        let future = JsFuture::from(request_promise);
        let resp_value = future.await.map_err(|e| FetchError::Transport {
            uri: uri.clone(),
            error: stringify_js_error(&e),
        })?;
        let js_response: JsResponse = match resp_value.dyn_into() {
            Ok(res) => res,
            Err(origin_val) => {
                let error = format!("Error casting {:?} to 'JsResponse'", origin_val);
                return Err(FetchError::Internal(error));
            },
        };
        let _status = StatusCode::from_u16(js_response.status()).map_err(FetchError::InvalidStatusCode)?;
        let _headers = js_response.headers();

        let fetch_response = FetchResponse::from_js_response(js_response).await?;
        Ok(fetch_response)
    }
}

/// This function is a wrapper around the `fetch_with_request`, providing compatibility across
/// different execution environments, such as window and worker.
fn compatible_fetch_with_request(js_request: &web_sys::Request) -> Result<js_sys::Promise, FetchError> {
    let global = js_sys::global();

    if let Some(scope) = global.dyn_ref::<Window>() {
        return Ok(scope.fetch_with_request(js_request));
    }

    if let Some(scope) = global.dyn_ref::<WorkerGlobalScope>() {
        return Ok(scope.fetch_with_request(js_request));
    }

    Err(FetchError::Internal("Unknown WASM environment.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_build_js_request_ok() {
        let uri = "http://example.com";
        let mut req_init = RequestInit::new();
        req_init.method("GET");
        let js_request = JsRequest::new_with_str_and_init(&uri, &req_init).unwrap();
    }

    // further unit tests could be implemented for the Err case based on the spec
    // https://fetch.spec.whatwg.org/#dom-request
    #[wasm_bindgen_test]
    async fn test_build_js_request_err_invalid_uri() {
        let uri = "http://user:password@example.com";
        let mut req_init = RequestInit::new();
        req_init.method("GET");
        let err = JsRequest::new_with_str_and_init(&uri, &req_init).map_err(|e| FetchError::Internal(stringify_js_error(&e))).unwrap_err();
        assert!(err.to_string().contains("Request cannot be constructed from a URL that includes credentials"));
    }

}