//! `is_script` が true のとき `TMP_DIR`/`file_name` にスクリプトを書き出す

use std::fs;
use std::path::Path;

use crate::core::response::parse_content;

/// 環境変数 `TMP_DIR`。未定義時は Windows なら `C:\tmp`、Linux 等は `/tmp`。
#[must_use]
pub fn resolve_tmp_dir() -> String {
    std::env::var("TMP_DIR").unwrap_or_else(|_| default_tmp_dir())
}

#[cfg(windows)]
fn default_tmp_dir() -> String {
    r"C:\tmp".to_string()
}

#[cfg(not(windows))]
fn default_tmp_dir() -> String {
    "/tmp".to_string()
}

/// content をパースし、`is_script` が true なら `command_or_script` を
/// `{TMP_DIR}/{file_name}` に書き出す。パース失敗や `is_script` が false なら何もしない。
/// 書き出した場合はそのファイルパスを `Ok(Some(path))` で返す。
///
/// # Errors
/// ファイル書き出しに失敗した場合にエラーを返す。
pub fn write_script_if_needed(content: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(resp) = parse_content(content) else {
        return Ok(None);
    };
    if !resp.is_script || resp.file_name.is_empty() {
        return Ok(None);
    }
    let tmp = resolve_tmp_dir();
    let path = Path::new(&tmp).join(&resp.file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, resp.command_or_script)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_resolve_tmp_dir_uses_env() {
        env::set_var("TMP_DIR", "/custom/tmp");
        let d = resolve_tmp_dir();
        env::remove_var("TMP_DIR");
        assert_eq!(d, "/custom/tmp");
    }

    #[test]
    fn test_write_script_if_needed_ignores_non_json() {
        assert!(write_script_if_needed("not json").unwrap().is_none());
    }

    #[test]
    fn test_write_script_if_needed_ignores_is_script_false() {
        let j = r#"{"command_or_script":"ls","file_name":"","is_script":false,"from_cache":false,"description":""}"#;
        assert!(write_script_if_needed(j).unwrap().is_none());
    }

    #[test]
    fn test_write_script_if_needed_writes_file_and_returns_path() {
        let tmp = std::env::temp_dir();
        let sub = tmp.join("neco_script_test");
        let _ = fs::remove_dir_all(&sub);
        fs::create_dir_all(&sub).unwrap();
        env::set_var("TMP_DIR", sub.to_str().unwrap());
        let j = "{\"command_or_script\":\"#!/bin/bash\\necho hi\",\"file_name\":\"hello.sh\",\"is_script\":true,\"from_cache\":false,\"description\":\"\"}";
        let out = write_script_if_needed(j).unwrap();
        let path = sub.join("hello.sh");
        assert!(out.as_ref().unwrap().contains("hello.sh"));
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("echo hi"));
        let _ = fs::remove_dir_all(&sub);
        env::remove_var("TMP_DIR");
    }
}
