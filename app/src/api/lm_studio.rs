//! `OpenAI` 互換 Chat Completions で LM Studio を呼び出す

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// チャット完了 1 回分のリクエスト（OpenAI 互換。model は LM Studio に任せるためダミー可）
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    /// LM Studio は自前でロードしたモデルを使うため、空やダミーでよい
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// レスポンス（必要な部分だけ）
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub content: Option<String>,
}

/// クライアントのエラー
#[derive(Debug)]
pub enum LmStudioError {
    Request(reqwest::Error),
    Status(reqwest::StatusCode, String),
    Decode(serde_json::Error),
}

impl std::fmt::Display for LmStudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LmStudioError::Request(e) => write!(f, "LM Studio リクエストエラー: {e}"),
            LmStudioError::Status(code, body) => {
                write!(f, "LM Studio エラー ({code}): {body}")
            }
            LmStudioError::Decode(e) => write!(f, "LM Studio レスポンス解析エラー: {e}"),
        }
    }
}

impl std::error::Error for LmStudioError {}

/// LM Studio を呼び出すトレイト（テスト・モック用）
#[async_trait]
pub trait LmStudioClient: Send + Sync {
    async fn chat(
        &self,
        system: &str,
        user: &str,
        temperature: f64,
    ) -> Result<String, LmStudioError>;
}

/// リクエスト body を組み立てる（テスト・モック用に pub）
pub fn chat_completion_request(
    system: &str,
    user: &str,
    temperature: f64,
) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: None,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ],
        temperature,
    }
}

fn default_base_url() -> String {
    std::env::var("LM_STUDIO_BASE_URL").unwrap_or_else(|_| "http://localhost:1234/v1".to_string())
}

/// reqwest を使った実装
pub struct ReqwestLmStudioClient {
    base_url: String,
    client: reqwest::Client,
}

impl ReqwestLmStudioClient {
    #[must_use]
    pub fn new() -> Self {
        Self::with_base_url(default_base_url())
    }

    #[must_use]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    async fn chat_impl(
        &self,
        system: &str,
        user: &str,
        temperature: f64,
    ) -> Result<String, LmStudioError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = chat_completion_request(system, user, temperature);
        let res = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(LmStudioError::Request)?;

        let status = res.status();
        let text = res.text().await.map_err(LmStudioError::Request)?;
        if !status.is_success() {
            return Err(LmStudioError::Status(status, text));
        }

        let parsed: ChatCompletionResponse =
            serde_json::from_str(&text).map_err(LmStudioError::Decode)?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or("")
            .to_string();
        Ok(content)
    }
}

impl Default for ReqwestLmStudioClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LmStudioClient for ReqwestLmStudioClient {
    async fn chat(
        &self,
        system: &str,
        user: &str,
        temperature: f64,
    ) -> Result<String, LmStudioError> {
        self.chat_impl(system, user, temperature).await
    }
}

/// テスト用: 固定レスポンスを返すモック
#[cfg(test)]
pub struct MockLmStudioClient {
    pub response: String,
}

#[cfg(test)]
#[async_trait]
impl LmStudioClient for MockLmStudioClient {
    async fn chat(
        &self,
        _system: &str,
        _user: &str,
        _temperature: f64,
    ) -> Result<String, LmStudioError> {
        Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_reqwest_client_calls_mock_server() {
        let server = MockServer::start().await;
        let body = r#"{"choices":[{"message":{"content":"{\"command_or_script\":\"ls\"}"}}]}"#;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
        let base = format!("{}/v1", server.uri());
        let client = ReqwestLmStudioClient::with_base_url(base);
        let out = client.chat("sys", "user", 0.0).await.unwrap();
        assert!(out.contains("command_or_script"));
        assert!(out.contains("ls"));
    }

    #[tokio::test]
    async fn test_reqwest_client_returns_status_error_on_4xx() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;
        let base = format!("{}/v1", server.uri());
        let client = ReqwestLmStudioClient::with_base_url(base);
        let res = client.chat("sys", "user", 0.0).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("500") || err.to_string().contains("internal"));
    }

    #[test]
    fn test_chat_completion_request_structure() {
        let req = chat_completion_request("sys", "user", 0.5);
        assert_eq!(req.model, None);
        assert!((req.temperature - 0.5).abs() < f64::EPSILON);
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, "system");
        assert_eq!(req.messages[0].content, "sys");
        assert_eq!(req.messages[1].role, "user");
        assert_eq!(req.messages[1].content, "user");
    }

    #[test]
    fn test_request_serialize_no_model() {
        let req = chat_completion_request("s", "u", 0.0);
        let j = serde_json::to_string(&req).unwrap();
        assert!(!j.contains("\"model\""));
        assert!(j.contains("\"temperature\":0.0"));
    }

    #[test]
    fn test_lm_studio_error_display() {
        let e = LmStudioError::Status(reqwest::StatusCode::BAD_REQUEST, "bad".to_string());
        let s = e.to_string();
        assert!(s.contains("BAD_REQUEST") || s.contains("400"));
        assert!(s.contains("bad"));
    }
}
