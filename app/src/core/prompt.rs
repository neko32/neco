//! ユーザ／システムメッセージの組み立て

use crate::core::shell_display_name;
use crate::core::Shell;

/// 仕様書の system メッセージ（固定）
pub const SYSTEM_MESSAGE: &str = r#"あなたはWindowsとLinuxのコマンドラインのエキスパートです。ユーザが実行したいことを伝えてきた場合、それを正確に実行するコマンドを作成しユーザに返します。もしユーザのコマンドの説明を正確に実行するのに一行で十分でない場合はis_scriptフラグをtrueにしてスクリプトの内容をユーザに返してください。file_nameは英数字及びファイルで許可される記号、descriptionは日本語で返します。

返事のフォーマットは以下の通りです

{
  "command_or_script": "{あなたが生成したコマンドまたはスクリプトの中身}",
  "file_name": "{もしそのコマンドをファイルにするとした場合、適切なファイル名}",
  "is_script": true or false,
  "from_cache": 今のところは常にfalse,
  "description": "コマンドの詳細な説明"
}
"#;

/// ユーザメッセージのテンプレート（先頭）。末尾に改行 + コマンドの説明を付与する。
fn user_message_prefix(shell: &Shell) -> String {
    let name = shell_display_name(shell);
    format!("以下の仕様を満たす{name}用のコマンドまたはファイルを生成してください。\n\n")
}

/// user と system メッセージを組み立てる。返すのは (system, user) の順。
#[must_use]
pub fn build_messages(shell: &Shell, command_description: &str) -> (String, String) {
    let system = SYSTEM_MESSAGE.to_string();
    let user = user_message_prefix(shell) + command_description;
    (system, user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Shell;

    #[test]
    fn test_build_messages_user_contains_shell_and_description() {
        let (system, user) = build_messages(&Shell::Bash, "ファイル一覧を出して");
        assert!(system.contains("エキスパート"));
        assert!(user.contains("bash用のコマンド"));
        assert!(user.contains("ファイル一覧を出して"));
    }

    #[test]
    fn test_build_messages_powershell() {
        let (_, user) = build_messages(&Shell::PowerShell, "dir");
        assert!(user.contains("powershell用のコマンド"));
        assert!(user.contains("dir"));
    }

    #[test]
    fn test_system_message_contains_json_format() {
        assert!(SYSTEM_MESSAGE.contains("command_or_script"));
        assert!(SYSTEM_MESSAGE.contains("file_name"));
        assert!(SYSTEM_MESSAGE.contains("is_script"));
        assert!(SYSTEM_MESSAGE.contains("from_cache"));
        assert!(SYSTEM_MESSAGE.contains("description"));
    }
}
