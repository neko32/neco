//! 親プロセスに基づくシェル判定

use std::process;

use sysinfo::{Pid, System};

/// サポートするシェル種別（プロンプト用の表示名に変換する）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shell {
    WindowsCmd,
    PowerShell,
    Bash,
}

/// シェル判定のためのトレイト（テスト・モック用）
pub trait ShellDetector: Send + Sync {
    /// # Errors
    /// 親プロセスがサポート外のシェルの場合 `UnsupportedShellError` を返す。
    fn detect(&self) -> Result<Shell, UnsupportedShellError>;
}

/// サポート外シェルで実行されたときのエラー
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedShellError {
    pub parent_name: String,
}

impl std::fmt::Display for UnsupportedShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "サポートしていないシェルです。親プロセス: {}",
            self.parent_name
        )
    }
}

impl std::error::Error for UnsupportedShellError {}

/// 親プロセス名からサポートするシェルを判定する。未対応なら `None`。
#[must_use]
pub fn shell_from_parent_name(name: &str) -> Option<Shell> {
    let name = name.to_lowercase();
    if name == "cmd.exe" {
        return Some(Shell::WindowsCmd);
    }
    if name == "powershell.exe" {
        return Some(Shell::PowerShell);
    }
    if name == "wsl.exe" || name == "bash" || name.starts_with("bash") {
        return Some(Shell::Bash);
    }
    None
}

/// プロンプト用の表示名を返す（仕様書どおり）
#[must_use]
pub fn shell_display_name(shell: &Shell) -> &'static str {
    match shell {
        Shell::WindowsCmd => "windowsコマンドプロンプト",
        Shell::PowerShell => "powershell",
        Shell::Bash => "bash",
    }
}

/// cargo / cargo.exe は中間プロセスなので、その親をたどる対象にする
fn is_cargo_process_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n == "cargo.exe" || n == "cargo"
}

/// sysinfo を使った実装
pub struct SysinfoShellDetector;

/// 親プロセスをたどる上限（cargo 経由でシェルを探すため）
const MAX_ANCESTOR_DEPTH: u32 = 10;

impl ShellDetector for SysinfoShellDetector {
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn detect(&self) -> Result<Shell, UnsupportedShellError> {
        let pid = process::id();
        let pid = Pid::from_u32(pid);
        let mut sys = System::new();
        sys.refresh_processes();

        let mut current =
            sys.process(pid)
                .and_then(|p| p.parent())
                .ok_or_else(|| UnsupportedShellError {
                    parent_name: "（取得できませんでした）".to_string(),
                })?;

        // cargo / cargo.exe の場合はさらに親をたどってシェルを判定する
        for _ in 0..MAX_ANCESTOR_DEPTH {
            let proc = match sys.process(current) {
                Some(p) => p,
                None => {
                    break;
                }
            };
            let name = proc.name().to_lowercase();
            if is_cargo_process_name(&name) {
                if let Some(ancestor) = proc.parent() {
                    current = ancestor;
                    continue;
                }
            }
            match shell_from_parent_name(&name) {
                Some(shell) => return Ok(shell),
                None => {
                    return Err(UnsupportedShellError {
                        parent_name: proc.name().to_string(),
                    });
                }
            }
        }

        Err(UnsupportedShellError {
            parent_name: "（シェルを特定できませんでした）".to_string(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// テスト用: 固定のシェルを返すモック
    pub struct MockShellDetector(pub Option<Shell>);

    impl ShellDetector for MockShellDetector {
        fn detect(&self) -> Result<Shell, UnsupportedShellError> {
            self.0.clone().ok_or_else(|| UnsupportedShellError {
                parent_name: "mock_unsupported".to_string(),
            })
        }
    }

    #[test]
    fn test_shell_display_name() {
        assert_eq!(
            shell_display_name(&Shell::WindowsCmd),
            "windowsコマンドプロンプト"
        );
        assert_eq!(shell_display_name(&Shell::PowerShell), "powershell");
        assert_eq!(shell_display_name(&Shell::Bash), "bash");
    }

    #[test]
    fn test_mock_detector_returns_shell() {
        let d = MockShellDetector(Some(Shell::Bash));
        assert_eq!(d.detect().unwrap(), Shell::Bash);
    }

    #[test]
    fn test_mock_detector_returns_unsupported() {
        let d = MockShellDetector(None);
        let e = d.detect().unwrap_err();
        assert_eq!(e.parent_name, "mock_unsupported");
        assert!(e.to_string().contains("サポートしていない"));
    }

    #[test]
    fn test_unsupported_shell_error_display() {
        let e = UnsupportedShellError {
            parent_name: "zsh".to_string(),
        };
        assert!(e.to_string().contains("zsh"));
    }

    #[test]
    fn test_shell_from_parent_name_cmd() {
        assert_eq!(shell_from_parent_name("cmd.exe"), Some(Shell::WindowsCmd));
        assert_eq!(shell_from_parent_name("CMD.EXE"), Some(Shell::WindowsCmd));
    }

    #[test]
    fn test_shell_from_parent_name_powershell() {
        assert_eq!(
            shell_from_parent_name("powershell.exe"),
            Some(Shell::PowerShell)
        );
    }

    #[test]
    fn test_shell_from_parent_name_bash() {
        assert_eq!(shell_from_parent_name("bash"), Some(Shell::Bash));
        assert_eq!(shell_from_parent_name("wsl.exe"), Some(Shell::Bash));
        assert_eq!(shell_from_parent_name("bash-5.2"), Some(Shell::Bash));
    }

    #[test]
    fn test_shell_from_parent_name_unsupported() {
        assert_eq!(shell_from_parent_name("zsh"), None);
        assert_eq!(shell_from_parent_name("fish"), None);
    }

    #[test]
    fn test_is_cargo_process_name() {
        assert!(is_cargo_process_name("cargo.exe"));
        assert!(is_cargo_process_name("cargo"));
        assert!(is_cargo_process_name("Cargo.EXE"));
        assert!(!is_cargo_process_name("powershell.exe"));
        assert!(!is_cargo_process_name("cargo-build"));
    }

    /// 実環境で detect を実行する（親プロセスがサポート外なら Err、それ以外は Ok）。
    /// カバレッジ用。結果は環境依存のため Ok/Err のどちらでも許容する。
    #[test]
    fn test_sysinfo_detect_runs_without_panic() {
        let detector = SysinfoShellDetector;
        let result = detector.detect();
        match &result {
            Ok(shell) => {
                assert!(matches!(
                    shell,
                    Shell::WindowsCmd | Shell::PowerShell | Shell::Bash
                ));
            }
            Err(e) => {
                assert!(!e.parent_name.is_empty());
            }
        }
    }
}
