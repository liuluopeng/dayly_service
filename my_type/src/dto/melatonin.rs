use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ScanMelatoninQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct MelatoninListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ActorMovieQuery {
    pub actor: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanMelatoninResult {
    pub scanned: u32,
    pub added: u32,
}

crate::impl_display!(self, ScanMelatoninResult,
    "扫描数" => self.scanned,
    "新增数" => self.added
);
