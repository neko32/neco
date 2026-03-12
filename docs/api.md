# neco API 仕様

## CLI インターフェース

### コマンド

```bash
neco "<コマンドの説明>" [オプション]
```

### 引数

| パラメータ | 説明 | 必須/任意 | デフォルト |
|-----------|------|-----------|------------|
| 位置引数 | 実行したいコマンドの説明 | **必須** | なし（省略時は exit 1） |
| `-t`, `--temperature` | LM Studio API に渡す temperature | 任意 | 0.0 |

### 使用例

```bash
# temperature デフォルト（0.0）
neco "/home/sanomaru の全ファイル名を取得"

# temperature 0.5 を指定
neco -t 0.5 "/home/sanomaru の全ファイル名を取得"
neco --temperature 0.5 "/home/sanomaru の全ファイル名を取得"
```

### 終了コード

- `0`: 成功（生成された JSON を標準出力に表示）
- `1`: エラー（引数不足・サポート外シェル・API エラー等。メッセージは標準エラー出力）

### スクリプトの自動生成（issue #2）

API の返す JSON で `is_script` が `true` の場合、`command_or_script` の内容を **`$TMP_DIR` / `file_name`** に書き出してファイルを作成する。

- **TMP_DIR**: 環境変数 `TMP_DIR` を参照。未定義時は Windows は `C:\tmp`、Linux 等は `/tmp` をデフォルトとする。
- ディレクトリが存在しない場合は作成してから書き出す。
- JSON のパースに失敗した場合や `is_script` が `false` の場合は何もしない。
- **標準出力**: ファイルを書き出した場合は、そのファイルパスを 1 行で標準出力に出力してから、従来どおり JSON を出力する。

---

## 外部 API（LM Studio）

neco は LM Studio の **OpenAI 互換 API** を使用する。

- **エンドポイント**: `POST {base_url}/chat/completions`
- **デフォルト base URL**: `http://localhost:1234/v1`（環境変数 `LM_STUDIO_BASE_URL` で上書き可能）
- **モデル**: リクエストでは指定しない。LM Studio 側でロードされているモデルを使用する。
- **リクエスト**: Chat Completions 形式（`messages`, `temperature` を送信。`model` は省略可）
- **レスポンス**: `choices[0].message.content` をそのまま標準出力に表示する（仕様書の JSON 形式を期待）

詳細は [design.md](design.md) を参照。
