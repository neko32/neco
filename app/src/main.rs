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
    let content = runtime.block_on(neco::generate(
        &command_description,
        args.temperature,
        &detector,
        &client,
    ))?;

    // 仕様: API が返す JSON をそのままコンソールに表示
    println!("{content}");
    Ok(())
}
