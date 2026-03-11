//! シェル判定とプロンプト生成

mod prompt;
pub(crate) mod shell;

pub use prompt::{build_messages, SYSTEM_MESSAGE};
pub(crate) use shell::shell_display_name;
pub use shell::{Shell, ShellDetector, SysinfoShellDetector, UnsupportedShellError};
