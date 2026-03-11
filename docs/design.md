# neco 設計メモ

## LM Studio API

- **プロトコル**: OpenAI 互換 API（Chat Completions 形式）を使用する。
- **モデル指定**: neco 側ではモデルを指定しない。LM Studio 側でロードされているモデルに任せる。

- **base URL**: 環境変数 `LM_STUDIO_BASE_URL` で指定（未設定時は `http://localhost:1234/v1`）。

## その他

- コマンドパラメータ・コマンド生成ロジック・返答 JSON フォーマットは仕様書 `neco.md` に従う。
