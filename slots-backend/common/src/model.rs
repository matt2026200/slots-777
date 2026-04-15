use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct WsRequest {
    pub request_id: String,
    pub action: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct WsResponse<T> {
    pub request_id: String,
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}
