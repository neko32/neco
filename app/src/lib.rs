//! neco ライブラリルート（CLI とテストから利用）

pub mod api;
pub mod core;

use api::LmStudioClient;
use core::{build_messages, Shell, ShellDetector};

/// コマンド説明とオプションから LM Studio で生成した JSON 文字列と、判定したシェルを返す。
/// テスト可能にするため、シェル判定と API クライアントを注入する。
///
/// # Errors
/// 説明が空・サポート外シェル・API エラー時にエラーを返す。
pub async fn generate<D, C>(
    command_description: &str,
    temperature: f64,
    detector: &D,
    client: &C,
) -> Result<(String, Shell), Box<dyn std::error::Error>>
where
    D: ShellDetector,
    C: LmStudioClient,
{
    let command_description = command_description.trim();
    if command_description.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "コマンドの説明が空です。",
        )
        .into());
    }
    let shell = detector
        .detect()
        .map_err(Box::<dyn std::error::Error>::from)?;
    let (system, user) = build_messages(&shell, command_description);
    let content = client
        .chat(&system, &user, temperature)
        .await
        .map_err(Box::<dyn std::error::Error>::from)?;
    Ok((content, shell))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::lm_studio::MockLmStudioClient;
    use crate::core::shell::tests::MockShellDetector;
    use crate::core::Shell;

    #[tokio::test]
    async fn test_generate_returns_client_response() {
        let detector = MockShellDetector(Some(Shell::Bash));
        let client = MockLmStudioClient {
            response: r#"{"command_or_script":"ls","description":"list"}"#.to_string(),
        };
        let (out, shell) = generate("ファイル一覧", 0.0, &detector, &client)
            .await
            .unwrap();
        assert!(matches!(shell, Shell::Bash));
        assert!(out.contains("command_or_script"));
        assert!(out.contains("ls"));
    }

    #[tokio::test]
    async fn test_generate_empty_description_errors() {
        let detector = MockShellDetector(Some(Shell::Bash));
        let client = MockLmStudioClient {
            response: String::new(),
        };
        let res = generate("  ", 0.0, &detector, &client).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("空です"));
    }

    #[tokio::test]
    async fn test_generate_unsupported_shell_errors() {
        let detector = MockShellDetector(None);
        let client = MockLmStudioClient {
            response: String::new(),
        };
        let res = generate("ls", 0.0, &detector, &client).await;
        assert!(res.is_err());
    }
}
