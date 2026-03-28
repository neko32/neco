//! シェル判定とプロンプト生成・スクリプト書き出し

mod prompt;
mod response;
mod runn;
mod script;
pub(crate) mod shell;

pub use prompt::{build_messages, SYSTEM_MESSAGE};
pub use response::{parse_content, NecoResponse};
pub use runn::{resolve_nekokan_tmp_dir, runn_basename, write_runn_script};
pub use script::{resolve_tmp_dir, write_script_if_needed};
pub(crate) use shell::shell_display_name;
pub use shell::{Shell, ShellDetector, SysinfoShellDetector, UnsupportedShellError};
