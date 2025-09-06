# GameCenter API リファレンス (詳細版)

このドキュメントは、GameCenter ゲーム機本体が提供するAPIの詳細なリファレンスです。
ゲームソフト開発者およびゲーム機本体の開発者が、APIの正確な仕様を理解し、実装時の誤りを減らすことを目的としています。

## 1. 共通事項

- **ゲーム機本体ベースURL:** `http://localhost:3000`
- **認証:** 現在、認証メカニズムは実装されていません。
- **エラーレスポンス:** API呼び出しでエラーが発生した場合、以下のJSON形式でエラーが返されます。
  ```json
  {"type": "error", "message": "エラー内容"}
  ```
- **`room_id` の重要性:**
  ゲーム機本体とゲームソフト間のHTTP API通信では、すべてのリクエストボディに `room_id` を含める必要があります。これにより、どのルームに対する操作かを識別します。

## 2. WebSocket API (プレイヤー ↔ ゲーム機本体)

クライアント（フロントエンド）とゲーム機本体間の主要な通信プロトコルです。リアルタイムなプレイヤーアクションの受付と、ゲーム状態の同期に使用されます。

### 2.1 クライアント → サーバー

| メッセージタイプ | 説明 | パラメータ | 備考 |
| :--- | :--- | :--- | :--- |
| `create_room` | 新しいルームを作成します。 | `display_name: string` (プレイヤーの表示名) | ルーム作成者は自動的にホストになります。 |
| `join_room` | 既存のルームに参加します。 | `room_id: string` (参加するルームのID), `display_name: string` (プレイヤーの表示名) | |
| `select_game` | ルームのホストがプレイするゲームを選択します。 | `game_id: string` (選択するゲームの識別子) | ホスト権限が必要です。 |
| `start_game` | ホストがゲームの開始を宣言します。 | なし | ホスト権限が必要です。ゲーム機本体は選択されたゲームの `/game/init` APIを呼び出します。 | 
| `realtime_action` | リアルタイム性が要求されるアクションを送信します。 | `action: object` (後述の `ClientRealtimeAction` 形式) | | 
| `update_settings` | (未実装) ルームの設定を変更します。 | `settings: object` (変更する設定項目) | | 

#### `realtime_action` の `action` オブジェクト詳細

| `action.type` | 説明 | パラメータ | 備考 | 
| :--- | :--- | :--- | :--- |
| `first_press` | 早押しボタンが押されたことを通知します。 | なし | サーバー側で `first_press` が有効な場合のみ処理されます。 | 
| `turn_action` | (未実装) ターン制ゲームにおけるプレイヤーのアクションを通知します。 | `action_type: string` (アクションの種類), `data: object` (アクションの詳細データ) | | 

### 2.2 サーバー → クライアント

| メッセージタイプ | 説明 | パラメータ | 備考 | 
| :--- | :--- | :--- | :--- |
| `room_update` | ルーム全体の最新状態を通知します。 | `room: object` (後述の `GameRoom` 構造体) | プレイヤーの参加/退出、ゲーム選択、フェーズ変更、判定結果など、状態変化のたびにこのメッセージで最新の状態が全プレイヤーに通知されます。 | 
| `game_state` | ゲーム固有のUI更新（例: 回答者表示）など、特定のフェーズにおける状態を通知します。 | `phase: string` (現在のゲームフェーズ), `data: object` (フェーズに応じたUIデータ) | | 
| `broadcast_event` | ゲームソフトからの要求に基づき、問題表示など、全プレイヤーに同時に通知したいイベントを配信します。 | `event: string` (イベントの種類), `data: object` (イベントの詳細データ) | | 
| `error` | 操作の失敗などを通知します。 | `message: string` (エラー内容) | | 
| `available_games` | (未実装) 利用可能なゲームの一覧を通知します。 | `games: array<GameInfo>` | | 
| `timer_update` | (未実装) タイマーの残り時間などを定期的に通知します。 | `timer_id: string`, `remaining: number` (残り時間) | | 

#### `room_update` の `room` オブジェクト詳細 (`GameRoom` 構造体)

```json
{
  "id": "string",                    // 5桁のルームID
  "host_id": "string",               // ホストプレイヤーのUUID
  "players": [
    { "id": "string", "display_name": "string" } // 参加プレイヤーのリスト
  ],
  "settings": {
    "max_players": "number",         // 参加人数上限
    "room_liberation_time": "number",// ルーム解放時間（分）
    "progression_rule": "string"     // 進行ルール（例: "単独進行"）
  },
  "selected_game": {
    "id": "string",              // ゲーム識別子
    "title": "string",           // 表示名
    "description": "string",     // 説明文
    "min_players": "number",     // 最小人数
    "max_players": "number",     // 最大人数
    "recommended_players": "number", // 推奨人数
    "api_endpoint": "string",    // ゲームソフトAPIのURL
    "settings_schema": {}        // 設定項目定義 (JSON Schema)
  } | null, // 選択されたゲーム情報、未選択の場合はnull
  "state": "string",               // ルームの状態 ("Waiting", "InGame", "Finished")
  "realtime_state": {
    "current_turn": "string" | null, // 現在の手番プレイヤーのUUID
    "turn_order": "array<string>",   // ターン順序のプレイヤーUUIDリスト
    "active_timers": {},             // アクティブなタイマー (キー: timer_id, 値: Timerオブジェクト) - 現在は空オブジェクト
    "first_press_winner": "string" | null, // 早押し勝者のUUID
    "game_phase": "string"           // ゲームの現在のフェーズ（例: "lobby", "answering"）
  },
  "created_at": "string",          // ルーム作成日時 (ISO 8601形式)
  "last_activity": "string"        // 最終アクティビティ日時 (ISO 8601形式)
}
```

## 3. HTTP API (ゲーム機本体 ↔ ゲームソフト)

ゲーム機本体とゲームソフト間の連携インターフェースです。ゲームのロジックはゲームソフト側で実装し、リアルタイム処理や状態同期はゲーム機本体が担当します。

### 3.1 ゲーム機本体 → ゲームソフト

ゲーム機本体がゲームソフトのAPIを呼び出します。

#### `POST /game/init`
- **説明:** ゲーム開始時に、参加プレイヤー情報などを渡してゲームの初期化を要求します。
- **リクエストボディ:**
  ```json
  {
    "room_id": "string",
    "players": [
      { "id": "string", "display_name": "string" }
    ],
    "settings": {} // ルーム設定
  }
  ```
- **レスポンス:**
  ```json
  {
    "status": "string" // 例: "initialized", "success"
  }
  ```

#### `POST /game/event`
- **説明:** プレイヤーのリアルタイムアクション（早押しなど）が発生したことをゲームソフトに通知します。
- **リクエストボディ:**
  ```json
  {
    "room_id": "string",
    "data": {
      "action": "string", // 例: "first_press"
      "winner": "string"  // 例: "player_id" (first_pressの場合)
    }
  }
  ```
- **レスポンス:**
  ```json
  {
    "status": "string" // 例: "event_handled"
  }
  ```

#### `POST /game/timer_expired` (未実装)
- **説明:** タイマーが満了した際にゲームソフトに通知します。

### 3.2 ゲームソフト → ゲーム機本体

ゲームソフトがゲーム機本体のAPIを呼び出します。

#### `POST /realtime/broadcast`
- **説明:** 全プレイヤーにイベントを同時配信するよう要求します。
- **リクエストボディ:**
  ```json
  {
    "room_id": "string",
    "event_type": "string", // 例: "question_display"
    "data": {} // イベントの詳細データ
  }
  ```
- **レスポンス:** `HTTP 200 OK` またはエラーコード

#### `POST /realtime/enable_action`
- **説明:** 早押しなどのリアルタイムアクションの受付状態を制御するよう要求します。
- **リクエストボディ:**
  ```json
  {
    "room_id": "string",
    "action_type": "string", // 例: "first_press"
    "enabled": "boolean"     // trueで有効化、falseで無効化
  }
  ```
- **レスポンス:** `HTTP 200 OK` またはエラーコード

#### `POST /game/next_phase`
- **説明:** ゲームの次のフェーズへの移行と、それに伴うUIの更新を要求します。
- **リクエストボディ:**
  ```json
  {
    "room_id": "string",
    "phase": "string", // 新しいフェーズ名
    "ui_data": {}      // フェーズに応じたUIデータ
  }
  ```
- **レスポンス:** `HTTP 200 OK` またはエラーコード

#### `POST /realtime/start_timer` (未実装)
- **説明:** サーバーサイドでタイマーを開始するよう要求します。

#### `GET /game/info` (ゲームソフト側実装)
- **説明:** ゲーム機本体がゲーム情報を取得するために、ゲームソフトが実装すべきAPIです。

#### `GET /game/settings` (ゲームソフト側実装)
- **説明:** ゲーム機本体がゲーム固有の設定項目定義を取得するために、ゲームソフトが実装すべきAPIです。

#### `GET /game/health` (推奨)
- **説明:** ゲームソフトのヘルスチェック用APIです。

#### `POST /game/cleanup` (推奨)
- **説明:** ゲーム終了時のクリーンアップ処理をゲームソフトに要求します。
