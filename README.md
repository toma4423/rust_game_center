# Rust GameCenter

これは、設計書に基づいて開発されたリアルタイム低遅延ゲームプラットフォームです。
Rust製のゲーム機本体（バックエンド）、多言語対応のゲームソフト、Svelte製のフロントエンドで構成されています。

## ✨ 特徴

- **リアルタイム通信:** WebSocketを利用し、2-6msの低遅延な通信を実現します。
- **汎用判定エンジン:** 早押し判定やタイマー制御など、リアルタイム性が求められる処理をゲーム機本体が担当します。
- **多言語対応:** ゲームロジックはHTTP APIを介して連携するため、Python, Go, TypeScriptなど、好きな言語で開発できます。

## 🏛️ アーキテクチャ

本プロジェクトは、以下の3つの主要コンポーネントで構成されています。

1.  **Game Engine (`/game-engine`)**
    -   **役割:** プロジェクトの中核。リアルタイム処理、ルーム管理、クライアントとのWebSocket通信、ゲームソフトとのHTTP API通信を担当します。
    -   **技術スタック:** Rust, Axum, Tokio, Tokio-Tungstenite

2.  **Game Software (`/games`)**
    -   **役割:** 個別のゲームルールやロジックを実装します。HTTP APIを介してゲーム機本体と連携します。
    -   **技術スタック:** 任意の言語（サンプルとしてPython + Flaskを使用）

3.  **Frontend (`/frontend`)**
    -   **役割:** プレイヤーが操作するWebクライアント。ゲーム機本体とWebSocketで通信します。
    -   **技術スタック:** Svelte, SvelteKit, TypeScript

### コンポーネント連携図

```mermaid
graph TD
    subgraph Players
        A[Player 1 Browser]
        B[Player 2 Browser]
    end

    subgraph Game Software
        D(Python/Flask Server)
    end

    C[Game Engine (Rust Server)]

    A -- WebSocket --> C
    B -- WebSocket --> C
    C -- HTTP API --> D
    D -- HTTP API --> C
```

## 📁 ディレクトリ構造

```
.
├── game-engine/    # Rust製ゲーム機本体
├── games/
│   └── quiz/       # Python製サンプルゲーム（早押しクイズ）
├── frontend/       # Svelte製フロントエンド
└── README.md
```

## 🚀 起動方法 (開発環境)

各コンポーネントを起動するには、3つのターミナルが必要です。

### 1. Game Engine (Rust)

ゲーム機本体のサーバーを起動します。

```bash
# 1. ディレクトリに移動
cd game-engine

# 2. サーバーを起動
cargo run
```
> サーバーは `http://localhost:3000` で起動します。

### 2. Sample Game (Python)

サンプルとして実装されている早押しクイズゲームを起動します。

```bash
# 1. ディレクトリに移動
cd games/quiz

# 2. 仮想環境の作成 (初回のみ)
# このPCにはuvがインストールされていることを前提としています
uv venv .venv

# 3. 仮想環境の有効化
source .venv/bin/activate

# 4. 依存パッケージのインストール (初回のみ)
uv pip install -r requirements.txt

# 5. ゲームサーバーを起動
python app.py
```
> サーバーは `http://localhost:5001` で起動します。

### 3. Frontend (Svelte)

プレイヤーがアクセスするWebクライアントを起動します。

```bash
# 1. ディレクトリに移動
cd frontend

# 2. 依存パッケージのインストール (初回のみ)
npm install

# 3. 開発サーバーを起動
npm run dev
```
> サーバーは `http://localhost:5173` (または別のポート) で起動します。ターミナルの指示に従ってください。

### 4. 動作確認

すべてのサーバーが起動したら、WebブラウザでフロントエンドのURL（例: `http://localhost:5173`）にアクセスしてください。
複数のブラウザやタブを開くことで、複数人でのプレイをシミュレートできます。

## 📝 API概要

### クライアント ↔ ゲーム機 (WebSocket)

-   **Client → Server**
    -   `create_room`: ルームを作成
    -   `join_room`: ルームに参加
    -   `select_game`: ゲームを選択 (ホストのみ)
    -   `start_game`: ゲームを開始 (ホストのみ)
    -   `realtime_action`: 早押しなどのリアルタイムアクションを送信
-   **Server → Client**
    -   `room_update`: ルーム全体の最新情報を通知
    -   `game_state`: ゲームのフェーズやUIデータを通知
    -   `broadcast_event`: ゲーム固有のイベントを通知
    -   `error`: エラーを通知

### ゲーム機 ↔ ゲームソフト (HTTP)

-   **Game Engine → Game Software**
    -   `POST /game/init`: ゲームの初期化を要求
    -   `POST /game/event`: リアルタイムアクションの結果を通知
-   **Game Software → Game Engine**
    -   `POST /realtime/broadcast`: 全員へのイベント同時配信を要求
    -   `POST /realtime/enable_action`: リアルタイムアクションの受付開始/終了を要求
    -   `POST /game/next_phase`: ゲームの次のフェーズへの移行を要求
# rust_game_center
# rust_game_center
