//! LM Studio が返す JSON（content 文字列）のパース

use serde::Deserialize;

/// API の content に含まれる JSON の形（仕様書どおり）
#[derive(Debug, Deserialize)]
pub struct NecoResponse {
    pub command_or_script: String,
    pub file_name: String,
    pub is_script: bool,
    #[serde(default)]
    pub from_cache: bool,
    pub description: String,
}

/// Markdown の ```json ... ``` または ``` ... ``` を外して中身を返す。
#[must_use]
pub fn strip_markdown_code_block(content: &str) -> &str {
    let content = content.trim();
    if !content.starts_with("```") {
        return content;
    }
    let after_fence = content[3..].trim_start();
    let after_lang = if after_fence.len() >= 4 && after_fence[..4].eq_ignore_ascii_case("json") {
        after_fence[4..].trim_start()
    } else {
        after_fence
    };
    let end = after_lang
        .rfind("\n```")
        .unwrap_or_else(|| after_lang.find("```").unwrap_or(after_lang.len()));
    after_lang[..end].trim()
}

/// content 文字列を JSON としてパースする。失敗したら `None`。
/// ```json ... ``` で囲まれていれば除去してからパースする。
#[must_use]
pub fn parse_content(content: &str) -> Option<NecoResponse> {
    let stripped = strip_markdown_code_block(content);
    serde_json::from_str(stripped).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_command() {
        let j = r#"{"command_or_script":"ls -la","file_name":"","is_script":false,"from_cache":false,"description":"一覧"}"#;
        let r = parse_content(j).unwrap();
        assert_eq!(r.command_or_script, "ls -la");
        assert!(r.file_name.is_empty());
        assert!(!r.is_script);
    }

    #[test]
    fn test_parse_content_script() {
        let j = "{\"command_or_script\":\"#!/bin/bash\\necho 1\",\"file_name\":\"run.sh\",\"is_script\":true,\"from_cache\":false,\"description\":\"スクリプト\"}";
        let r = parse_content(j).unwrap();
        assert!(r.command_or_script.contains("echo 1"));
        assert_eq!(r.file_name, "run.sh");
        assert!(r.is_script);
    }

    #[test]
    fn test_parse_content_invalid_returns_none() {
        assert!(parse_content("not json").is_none());
        assert!(parse_content("").is_none());
    }

    #[test]
    fn test_parse_content_strips_markdown_code_block() {
        let wrapped = r#"```json
{"command_or_script":"ls","file_name":"a.ps1","is_script":true,"from_cache":false,"description":""}
```"#;
        let r = parse_content(wrapped).unwrap();
        assert_eq!(r.file_name, "a.ps1");
        assert!(r.is_script);
    }

    #[test]
    fn test_strip_markdown_code_block_plain_unchanged() {
        let s = r#"{"x":1}"#;
        assert_eq!(strip_markdown_code_block(s), s);
    }
}
