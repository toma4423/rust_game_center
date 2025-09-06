# GameCenter トラブルシューティングガイド

このドキュメントは、GameCenter の開発中や運用中に発生しうる一般的な問題とその解決策をまとめたものです。
問題解決の助けとなることを目的としています。

## 1. サーバーが起動しない

### 1.1 Game Engine (Rust) が起動しない

- **エラーメッセージ:** `error: could not compile ...`
  - **原因:** コードにコンパイルエラーがあります。
  - **解決策:** エラーメッセージをよく読み、指示に従ってコードを修正してください。Rustコンパイラのエラーメッセージは非常に詳細です。
- **エラーメッセージ:** `Address already in use` または `Port already in use`
  - **原因:** 別のプロセスが `3000` 番ポートを使用しています。
  - **解決策:** 以下のいずれかを試してください。
    - 別のターミナルで実行中のGame Engineプロセスを終了する。
    - ポートを使用している他のアプリケーションを特定し、終了する（例: `lsof -i :3000`）。
    - `main.rs` でGame Engineが使用するポート番号を変更する。
- **エラーメッセージ:** `No such file or directory` (特に `Cargo.toml` 関連)
  - **原因:** `cargo run` コマンドを `game-engine` ディレクトリの外部で実行しています。
  - **解決策:** `cd game-engine` で `game-engine` ディレクトリに移動してから `cargo run` を実行してください。

### 1.2 Sample Game (Python) が起動しない

- **エラーメッセージ:** `ModuleNotFoundError: No module named 'flask'` など
  - **原因:** 依存パッケージがインストールされていません。
  - **解決策:** `games/quiz` ディレクトリで `uv venv .venv` (仮想環境作成) と `uv pip install -r requirements.txt` (パッケージインストール) を実行したか確認してください。また、`source .venv/bin/activate` で仮想環境が有効になっているか確認してください。
- **エラーメッセージ:** `Address already in use` または `Port already in use`
  - **原因:** 別のプロセスが `5001` 番ポートを使用しています。
  - **解決策:** 別のターミナルで実行中のPythonゲームプロセスを終了するか、ポートを使用している他のアプリケーションを特定し、終了してください。

### 1.3 Frontend (Svelte) が起動しない

- **エラーメッセージ:** `command not found: npm` または `npm: command not found`
  - **原因:** Node.jsとnpmがインストールされていません。
  - **解決策:** [Node.js 公式サイト](https://nodejs.org/) からインストールしてください。
- **エラーメッセージ:** `Cannot find package.json` または `Error: ENOENT: no such file or directory, open '.../package.json'`
  - **原因:** `npm install` または `npm run dev` コマンドを `frontend` ディレクトリの外部で実行しています。
  - **解決策:** `cd frontend` で `frontend` ディレクトリに移動してからコマンドを実行してください。
- **エラーメッセージ:** `Address already in use` または `Port already in use`
  - **原因:** 別のプロセスが `5173` 番ポート（Viteのデフォルト）を使用しています。
  - **解決策:** 別のターミナルで実行中のフロントエンドプロセスを終了するか、ポートを使用している他のアプリケーションを特定し、終了してください。

## 2. WebSocket接続ができない

- **ブラウザのコンソールにエラーが表示される:**
  - **`WebSocket connection to 'ws://127.0.0.1:3000/ws' failed:`**
    - **原因:** Game Engineが起動していないか、指定されたアドレス/ポートでリッスンしていません。
    - **解決策:** Game Engineが `http://localhost:3000` で起動していることを確認してください。Game Engineのターミナルで `listening on 127.0.0.1:3000` のログが出ているか確認します。
  - **`CORS policy: No 'Access-Control-Allow-Origin' header is present on the requested resource.`**
    - **原因:** フロントエンドとGame Engineが異なるオリジン（ドメイン、ポート）で動作しているため、CORSポリシーに違反しています。
    - **解決策:** Game Engineの `main.rs` にCORSミドルウェアが正しく設定されているか確認してください。現在の実装では `tower_http::cors::Any` を使用しており、通常は問題ありません。
- **ファイアウォール:**
  - **原因:** OSやネットワークのファイアウォールが、指定されたポートへの接続をブロックしている可能性があります。
  - **解決策:** ファイアウォールの設定を確認し、`3000` 番ポートへのアクセスを許可してください。

## 3. ゲームソフトが連携しない (HTTP API)

- **Game Engineのログに `Failed to notify game software` エラーが出る:**
  - **原因:** Pythonゲームサーバーが起動していないか、指定されたアドレス/ポートでリッスンしていません。
  - **解決策:** Pythonゲームサーバーが `http://localhost:5001` で起動していることを確認してください。Pythonゲームのターミナルで `Running on http://127.0.0.1:5001` のログが出ているか確認します。
- **Pythonゲームのログに `Error sending request to ...` エラーが出る:**
  - **原因:** Game Engineが起動していないか、指定されたアドレス/ポートでリッスンしていません。
  - **解決策:** Game Engineが `http://localhost:3000` で起動していることを確認してください。
- **APIリクエストが正しく処理されない:**
  - **原因:** リクエストボディの形式がAPI仕様と一致していません。特に `room_id` の有無や、`data` フィールドの構造を確認してください。
  - **解決策:** `docs/api_reference.md` と `docs/ゲーム開発者向けガイドライン.md` を参照し、リクエストボディのJSON形式が正しいか確認してください。
- **`room_id` の不一致:**
  - **原因:** ゲーム機本体とゲームソフト間でやり取りされる `room_id` が一致していません。これは通常、ゲーム機本体がゲームソフトに `room_id` を渡す際に発生します。
  - **解決策:** `POST /game/init` や `POST /game/event` のリクエストボディに正しい `room_id` が含まれているか、ゲームソフト側で正しく受け取って処理しているか確認してください。

## 4. ルームの状態が更新されない / ブロードキャストされない

- **クライアントのUIが更新されない:**
  - **原因:** `room_update` メッセージがクライアントに届いていないか、クライアント側で正しく処理されていません。
  - **解決策:** ブラウザの開発者ツール（F12）のネットワークタブでWebSocket通信を確認し、`room_update` メッセージが受信されているか確認してください。受信されている場合、フロントエンドの `+page.svelte` で `socket.onmessage` が正しく `roomState` を更新しているか確認してください。
- **Game Engineのログに `No subscribers in room ...` 警告が出る:**
  - **原因:** そのルームに接続しているクライアントがいないか、クライアントがブロードキャストチャネルを購読できていません。
  - **解決策:** クライアントがルームに参加しているか確認してください。また、`websocket/mod.rs` の `handle_client_message` で `new_rx` が正しく `room_rx` に設定されているか確認してください。

## 5. 早押しが動作しない

- **早押しボタンを押しても何も起こらない:**
  - **原因:** ゲームソフトが `POST /realtime/enable_action` を呼び出して `first_press` を有効化していません。
  - **解決策:** Pythonゲームのログで `Action 'first_press' in room ... set to True` のログが出ているか確認してください。
- **早押しボタンを押しても勝者が決まらない:**
  - **原因:** ゲーム機本体が `first_press` アクションを処理できていないか、ゲームソフトに結果を通知できていません。
  - **解決策:** Game Engineのログで `First press by ...` のログが出ているか確認してください。また、`Failed to notify game software` エラーが出ていないか確認してください。

---

**問題が解決しない場合:**

上記の解決策を試しても問題が解決しない場合は、以下の情報を含めて開発チームに問い合わせてください。

- 発生している問題の具体的な内容と再現手順
- 関連するコンポーネント（Game Engine, Python Game, Frontend）のログ出力
- 試した解決策とその結果
- 使用しているOS、Node.js/Python/Rustのバージョン