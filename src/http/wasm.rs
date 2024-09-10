use web_sys::RequestMode;
use std::collections::HashMap;

enum FetchMethod {
    Get,
    Post,
}

enum RequestBody {
    Utf8(String),
    Bytes(Vec<u8>),
}

pub struct FetchRequest {
    uri: String,
    method: FetchMethod,
    headers: HashMap<String, String>,
    body: Option<RequestBody>,
    mode: Option<RequestMode>,
}

impl FetchRequest {
    pub fn get(uri: &str) -> FetchRequest {
        FetchRequest {
            uri: uri.to_owned(),
            method: FetchMethod::Get,
            headers: HashMap::new(),
            body: None,
            mode: None,
        }
    }

    pub fn post(uri: &str) -> FetchRequest {
        FetchRequest {
            uri: uri.to_owned(),
            method: FetchMethod::Post,
            headers: HashMap::new(),
            body: None,
            mode: None,
        }
    }
}