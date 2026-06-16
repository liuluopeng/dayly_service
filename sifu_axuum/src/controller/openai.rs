use crate::config::env::OpenAiConfig;
use axum::body::Body;
use axum::{Extension, Json, Router, routing::post};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use futures::stream::{self, Stream, StreamExt};
use hyper::body::Body as HyperBody;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    session_id: Option<Uuid>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    index: u32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

async fn chat_completion(
    Extension(openai_config): Extension<OpenAiConfig>,
    Extension(pool): Extension<PgPool>,
    Json(mut request): Json<OpenAiRequest>,
) -> Result<Json<ApiResponse<Value>>, ApiError> {
    if openai_config.openai_api_key.is_empty() {
        return Err(ApiError::bad_request(ApiError::OPENAI_KEY_MISSING, "OpenAI API Key 未配置"));
    }

    debug!(
        "___________________________________________________________________{:?}",
        request
    );

    // 注意：用户消息已经在前端通过 add_message API 保存，这里不再重复保存
    // 只保存 AI 回复

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/v1/chat/completions",
            openai_config.openai_base_url
        ))
        .header(
            "Authorization",
            format!("Bearer {}", openai_config.openai_api_key),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!("OpenAI API request error: {:?}", e);
            ApiError::Internal(e.to_string())
        })?;

    let response_json = response.json::<Value>().await.map_err(|e| {
        error!("OpenAI API response parse error: {:?}", e);
        ApiError::Internal(e.to_string())
    })?;

    // 保存助手回复到会话
    if let Some(session_id) = request.session_id {
        if let Some(choices) = response_json.get("choices") {
            if let Some(first_choice) = choices.as_array().and_then(|arr| arr.first()) {
                if let Some(message) = first_choice.get("message") {
                    if let Some(role) = message.get("role").and_then(|r| r.as_str()) {
                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                            // 提取reasoning_content字段（如果存在）
                            let think = message.get("reasoning_content").and_then(|t| t.as_str());

                            // 提取cite字段（如果存在）
                            let cite = message.get("cite").cloned();

                            sqlx::query(
                                r#"INSERT INTO openai_messages (session_id, role, content, think, cite) VALUES ($1, $2, $3, $4, $5)"#
                            )
                            .bind(&session_id)
                            .bind(role)
                            .bind(content)
                            .bind(think)
                            .bind(cite)
                            .execute(&pool)
                            .await
                            .ok();
                        }
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::ok(response_json)))
}

// 流式传输版本
async fn chat_completion_stream(
    Extension(openai_config): Extension<OpenAiConfig>,
    Extension(pool): Extension<PgPool>,
    Json(mut request): Json<OpenAiRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    if openai_config.openai_api_key.is_empty() {
        return Err(ApiError::bad_request(ApiError::OPENAI_KEY_MISSING, "OpenAI API Key 未配置"));
    }

    debug!(
        "___________________________________________________________________{:?}",
        request
    );

    // 强制设置为流式传输
    request.stream = Some(true);

    // 处理session_id，确保能正确获取
    let session_id = match request.session_id {
        Some(id) => id,
        None => {
            debug!("No session_id in request, using default");
            Uuid::new_v4()
        }
    };
    debug!("Session ID to use: {:?}", session_id);

    let client = Client::new();
    let response = client
        .post(format!(
            "{}/v1/chat/completions",
            openai_config.openai_base_url
        ))
        .header(
            "Authorization",
            format!("Bearer {}", openai_config.openai_api_key),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!("OpenAI API request error: {:?}", e);
            ApiError::Internal(e.to_string())
        })?;

    // 直接返回原始响应的状态和头部
    let status = response.status();
    let mut builder = axum::response::Response::builder().status(status);

    // 复制头部
    for (name, value) in response.headers() {
        builder = builder.header(name, value);
    }

    // 手动处理流，同时收集AI回复
    let pool_clone = pool.clone();
    let session_id_clone = session_id.clone();

    let body = Body::from_stream(futures::stream::try_unfold(
        (response, String::new(), pool_clone, session_id_clone),
        |(mut response, mut full_content, pool, session_id)| async move {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    full_content.push_str(&chunk_str);
                    Ok(Some((chunk, (response, full_content, pool, session_id))))
                }
                Ok(None) => {
                    // 流结束，保存AI回复到数据库
                    debug!("Stream ended, saving AI response to database");
                    // 直接使用session_id，因为Uuid总是有效的
                    if !session_id.to_string().is_empty() {
                        debug!("Session ID: {}", session_id);
                        // 解析完整内容，提取AI回复
                        let lines: Vec<&str> = full_content.split('\n').collect();
                        let mut assistant_content = String::new();
                        let mut assistant_think = String::new();
                        let mut assistant_cite: Option<Value> = None;

                        for line in lines {
                            if line.starts_with("data: ") && !line.contains("[DONE]") {
                                let data = line.trim_start_matches("data: ");
                                debug!("Raw data from API: {}", data);
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                    debug!("Parsed JSON: {}", json);
                                    // 提取content
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        assistant_content.push_str(content);
                                    }
                                    // 提取reasoning_content (DeepSeek Reasoner使用这个字段名)
                                    if let Some(think) =
                                        json["choices"][0]["delta"]["reasoning_content"].as_str()
                                    {
                                        debug!("Found think content: {}", think);
                                        assistant_think.push_str(think);
                                    }
                                    // 提取cite（通常在最后一个chunk中）
                                    if let Some(cite) =
                                        json["choices"][0]["delta"]["cite"].as_object()
                                    {
                                        debug!("Found cite data: {:?}", cite);
                                        assistant_cite = Some(Value::Object(cite.clone()));
                                    }
                                }
                            }
                        }

                        debug!("AI response content: {}", assistant_content);
                        debug!("AI response think: {}", assistant_think);
                        if !assistant_content.is_empty() {
                            // 保存到数据库
                            debug!("Saving AI response to database");
                            match sqlx::query(
                                "INSERT INTO openai_messages (session_id, role, content, think, cite, created_at) 
                                 VALUES ($1, $2, $3, $4, $5, NOW())"
                            )
                            .bind(&session_id)
                            .bind("assistant")
                            .bind(&assistant_content)
                            .bind(if assistant_think.is_empty() { None } else { Some(&assistant_think) })
                            .bind(assistant_cite)
                            .execute(&pool)
                            .await {
                                Ok(result) => debug!("Saved successfully, rows affected: {}", result.rows_affected()),
                                Err(e) => error!("Failed to save: {:?}", e),
                            }
                        } else {
                            debug!("No assistant content to save");
                        }
                    } else {
                        debug!("Session ID is empty");
                    }
                    Ok(None)
                }
                Err(e) => Err(e),
            }
        },
    ));

    Ok(builder.body(body).unwrap())
}

pub fn openai_routes() -> Router {
    Router::new()
        .route("/chat/completions", post(chat_completion))
        .route("/chat/completions/stream", post(chat_completion_stream))
}
