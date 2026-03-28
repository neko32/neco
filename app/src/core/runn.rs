//! `runn` 用スクリプト書き出し（issue #6）

use std::fs;
use std::path::Path;

use crate::core::response::parse_content;
use crate::core::Shell;

/// 環境変数 `NEKOKAN_TMP_DIR`。未定義時は Windows なら `C:\tmp`、Linux 等は `/tmp`。
#[must_use]
pub fn resolve_nekokan_tmp_dir() -> String {
    std::env::var("NEKOKAN_TMP_DIR").unwrap_or_else(|_| default_nekokan_tmp_dir())
}

#[cfg(windows)]
fn default_nekokan_tmp_dir() -> String {
    r"C:\tmp".to_string()
}

#[cfg(not(windows))]
fn default_nekokan_tmp_dir() -> String {
    "/tmp".to_string()
}

/// Unix では書き出した `runn` ファイルに実行権（`0o755`）を付与する。Windows では何もしない。
#[allow(clippy::unnecessary_wraps)] // Unix では `io::Result` が必要なため、Windows でも型を揃える
fn set_runn_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// シェル種別に応じた `runn` ファイル名（`runn.cmd` / `runn.ps1` / `runn.bash`）。
#[must_use]
pub fn runn_basename(shell: &Shell) -> &'static str {
    match shell {
        Shell::WindowsCmd => "runn.cmd",
        Shell::PowerShell => "runn.ps1",
        Shell::Bash => "runn.bash",
    }
}

/// パースに成功したら `command_or_script` を `{NEKOKAN_TMP_DIR}/{runn.*}` に書き出す。
/// パース失敗や本文が空なら何もしない。書き出した場合はパスを `Ok(Some(path))` で返す。
///
/// # Errors
/// ディレクトリ作成・ファイル書き込みに失敗した場合にエラーを返す。
pub fn write_runn_script(
    content: &str,
    shell: &Shell,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(resp) = parse_content(content) else {
        return Ok(None);
    };
    if resp.command_or_script.trim().is_empty() {
        return Ok(None);
    }
    let dir = resolve_nekokan_tmp_dir();
    let path = Path::new(&dir).join(runn_basename(shell));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, resp.command_or_script)?;
    set_runn_executable(&path)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Shell;
    use std::env;
    use std::sync::Mutex;

    static NEKOKAN_TMP_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_runn_basename() {
        assert_eq!(runn_basename(&Shell::WindowsCmd), "runn.cmd");
        assert_eq!(runn_basename(&Shell::PowerShell), "runn.ps1");
        assert_eq!(runn_basename(&Shell::Bash), "runn.bash");
    }

    #[test]
    fn test_resolve_nekokan_tmp_dir_env() {
        let _guard = NEKOKAN_TMP_TEST_LOCK.lock().unwrap();
        env::set_var("NEKOKAN_TMP_DIR", "/x/y");
        let d = resolve_nekokan_tmp_dir();
        env::remove_var("NEKOKAN_TMP_DIR");
        assert_eq!(d, "/x/y");
    }

    #[test]
    fn test_write_runn_script_writes_and_returns_path() {
        let _guard = NEKOKAN_TMP_TEST_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("neco_runn_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        env::set_var("NEKOKAN_TMP_DIR", tmp.to_str().unwrap());
        let j = r#"{"command_or_script":"echo ok","file_name":"","is_script":false,"from_cache":false,"description":""}"#;
        let out = write_runn_script(j, &Shell::Bash).unwrap().unwrap();
        assert!(out.contains("runn.bash"));
        let body = std::fs::read_to_string(tmp.join("runn.bash")).unwrap();
        assert_eq!(body, "echo ok");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(tmp.join("runn.bash"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }
        let _ = std::fs::remove_dir_all(&tmp);
        env::remove_var("NEKOKAN_TMP_DIR");
    }

    #[test]
    fn test_write_runn_script_skips_invalid_json() {
        assert!(write_runn_script("not json", &Shell::Bash)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_write_runn_script_skips_empty_command() {
        let j = r#"{"command_or_script":"   ","file_name":"","is_script":false,"from_cache":false,"description":""}"#;
        assert!(write_runn_script(j, &Shell::Bash).unwrap().is_none());
    }
}
