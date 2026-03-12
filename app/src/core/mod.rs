//! シェル判定とプロンプト生成・スクリプト書き出し

mod prompt;
mod response;
mod script;
pub(crate) mod shell;

pub use prompt::{build_messages, SYSTEM_MESSAGE};
pub use response::{parse_content, NecoResponse};
pub use script::{resolve_tmp_dir, write_script_if_needed};
pub(crate) use shell::shell_display_name;
pub use shell::{Shell, ShellDetector, SysinfoShellDetector, UnsupportedShellError};
