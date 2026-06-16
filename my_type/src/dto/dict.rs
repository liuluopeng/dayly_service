use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictSearchResult {
    pub source: String,
    pub word: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictExactResponse {
    pub 柯林斯: Option<super::super::model::dict::DictWord>,
    pub 朗文: Option<super::super::model::dict::DictWord>,
    pub 现代汉语词典: Option<super::super::model::dict::DictWord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictCandidateResponse {
    pub 柯林斯: Vec<super::super::model::dict::DictWord>,
    pub 朗文: Vec<super::super::model::dict::DictWord>,
    pub 现代汉语词典: Vec<super::super::model::dict::DictWord>,
}

#[derive(Debug, Deserialize)]
pub struct DictSearchQuery {
    pub search: String,
}

#[derive(Debug, Deserialize)]
pub struct RecentHistoryQuery {
    pub limit: i64,
}

crate::impl_display!(self, DictSearchResult,
    "词语" => self.word,
    "来源" => self.source,
    "解释" => self.explanation
);

crate::impl_display!(self, DictExactResponse,
    "柯林斯"       => if self.柯林斯.is_some() { "✓" } else { "✗" },
    "朗文"         => if self.朗文.is_some() { "✓" } else { "✗" },
    "现代汉语词典" => if self.现代汉语词典.is_some() { "✓" } else { "✗" }
);

crate::impl_display!(self, DictCandidateResponse,
    "柯林斯"       => format!("{} 条", self.柯林斯.len()),
    "朗文"         => format!("{} 条", self.朗文.len()),
    "现代汉语词典" => format!("{} 条", self.现代汉语词典.len())
);