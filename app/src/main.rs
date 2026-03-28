//! neco CLI エントリポイント

use clap::Parser;
use neco::api::ReqwestLmStudioClient;
use neco::core::SysinfoShellDetector;
use std::process;

/// コマンドの説明を受け取り、LM Studio でコマンド／スクリプトを生成して JSON で返す
#[derive(Parser, Debug)]
#[command(name = "neco", about, version)]
struct Args {
    /// 実行したいことの説明（必須）
    #[arg(required = true)]
    command_description: Option<String>,

    /// モデルの temperature（LM Studio API に渡す）
    #[arg(long, short = 't', default_value = "0.0")]
    temperature: f64,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let command_description = args.command_description.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "コマンドの説明を指定してください。",
        )
    })?;

    let detector = SysinfoShellDetector;
    let client = ReqwestLmStudioClient::new();
    let runtime = tokio::runtime::Runtime::new()?;
    let (content, shell) = runtime.block_on(neco::generate(
        &command_description,
        args.temperature,
        &detector,
        &client,
    ))?;

    // is_script が true なら TMP_DIR/{file_name} にスクリプトを書き出す（issue #2）
    if let Some(path) = neco::core::write_script_if_needed(&content)? {
        println!("{path}");
    }

    // runn: NEKOKAN_TMP_DIR/runn.{cmd|ps1|bash} に command_or_script を書き出す（issue #6）
    if let Some(path) = neco::core::write_runn_script(&content, &shell)? {
        println!("{path}");
    }

    // 仕様: API が返す JSON をそのままコンソールに表示
    println!("{content}");
    Ok(())
}
