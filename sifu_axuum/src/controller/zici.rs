use crate::zici_db;
use axum::Json;
use axum::extract::Query;
use common::api::base::{ApiError, ApiResponse, ApiResult};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CharsQuery {
    pub grade: Option<i64>,
    pub term: Option<i64>,
}

#[derive(Deserialize)]
pub struct WordsQuery {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct FrequencyQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct HanziSvgQuery {
    pub char: Option<String>,
}

pub async fn get_chars(Query(q): Query<CharsQuery>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let chars = zici_db::zici_chars(q.grade.unwrap_or(1), q.term.unwrap_or(1));
    let data: Vec<String> = chars.chars().map(|c| c.to_string()).collect();
    Ok(Json(ApiResponse::ok(data)))
}

pub async fn get_words(
    Query(q): Query<WordsQuery>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(100).clamp(1, 500);
    let (words, total) = zici_db::zici_words(q.search.as_deref().unwrap_or(""), page, page_size);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "data": words,
        "total": total,
        "page": page,
        "page_size": page_size,
    }))))
}

pub async fn get_word_frequency(
    Query(q): Query<FrequencyQuery>,
) -> ApiResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let items = zici_db::zici_word_frequency(q.search.as_deref().unwrap_or(""), limit);
    let data: Vec<serde_json::Value> = items
        .into_iter()
        .map(|(word, pinyin, frequency, explanation)| {
            serde_json::json!({
                "word": word,
                "pinyin": pinyin,
                "frequency": frequency,
                "explanation": explanation,
            })
        })
        .collect();
    Ok(Json(ApiResponse::ok(data)))
}

pub async fn get_pinyin_data() -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let (initials, finals, combos) = zici_db::pinyin_combos();
    let initial_list: Vec<serde_json::Value> = initials
        .into_iter()
        .map(|(pinyin, char, example)| serde_json::json!({"pinyin": pinyin, "char": char, "example": example}))
        .collect();
    let final_list: Vec<serde_json::Value> = finals
        .into_iter()
        .map(|(pinyin, char, example)| serde_json::json!({"pinyin": pinyin, "char": char, "example": example}))
        .collect();
    let combo_map: serde_json::Map<String, serde_json::Value> = {
        let mut map = serde_json::Map::new();
        for (initial, final_py, combo) in combos {
            map.entry(initial)
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .map(|m| m.insert(final_py, serde_json::Value::String(combo)));
        }
        map
    };
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "initials": initial_list,
        "finals": final_list,
        "combos": serde_json::Value::Object(combo_map),
    }))))
}

pub async fn get_hanzi_svg(
    Query(q): Query<HanziSvgQuery>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let char = q.char.unwrap_or_default();
    let data = match zici_db::hanzi_svg(&char) {
        Some(strokes) => match serde_json::from_str::<Vec<String>>(&strokes) {
            Ok(paths) => serde_json::json!({ "char": char, "strokes": paths }),
            Err(_) => serde_json::json!({ "char": char, "strokes": [] }),
        },
        None => serde_json::json!({ "char": char, "strokes": [] }),
    };
    Ok(Json(ApiResponse::ok(data)))
}

pub fn zici_routes() -> axum::Router {
    axum::Router::new()
        .route("/chars", axum::routing::get(get_chars))
        .route("/words", axum::routing::get(get_words))
        .route("/word-frequency", axum::routing::get(get_word_frequency))
        .route("/hanzi/svg", axum::routing::get(get_hanzi_svg))
        .route("/pinyin", axum::routing::get(get_pinyin_data))
}

#[allow(unused)]
fn _unused(e: ApiError) -> ApiError {
    e
}
