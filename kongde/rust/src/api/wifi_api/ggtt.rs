use flutter_rust_bridge::frb;

use crate::api::{logger_bridge::log_to_dart, wifi_api::init::{get_client_clone, init_client, set_client_token}};

pub use common::api::{
    base::ApiError,
    ggtt::{SearchRequest, search_ggtt_code},
};

pub use my_type::model::ggtt::GgttCode;

// 把dart给的code变成 rust的reqest 结构体

#[frb(mirror(GgttCode))]
pub struct _GgttCode {
    pub id: i32,
    pub char: String,
    pub code_86: String,
    pub has_diagram: bool,
    pub svg1: Option<String>,
    pub svg2: Option<String>,
    pub svg3: Option<String>,
    pub svg4: Option<String>,
}

#[frb(mirror(SearchRequest))]
pub struct _SearchRequest {
    pub search: String,
}

pub async fn search_ggtt_code_for_dart(search: String) -> Result<GgttCode, ApiError> {
    let client = get_client_clone().map_err(|e| ApiError::Internal(e.to_string()))?;
    let req = SearchRequest { search: search.clone() };
    match search_ggtt_code(&client, req).await {
        Ok(res) => {
            if let Some(ggtt_code) = res.data {
                log_to_dart(format!("GGTT 查询成功: {}", search));
                Ok(ggtt_code)
            } else {
                Err(ApiError::Internal("No data found in response".to_string()))
            }
        }
        Err(err) => Err(err),
    }
}
