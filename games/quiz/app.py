import os
from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

# ゲーム機本体のAPIエンドポイント
GAME_ENGINE_API_BASE = os.environ.get("GAME_ENGINE_API_BASE", "http://localhost:3000")

# 簡単なインメモリ状態管理
game_state = {}

def send_to_game_engine(endpoint, data):
    """ゲーム機本体にリクエストを送信するヘルパー関数"""
    try:
        url = f"{GAME_ENGINE_API_BASE}{endpoint}"
        print(f"Sending request to {url} with data: {data}")
        response = requests.post(url, json=data)
        response.raise_for_status()
        print(f"Successfully sent request to {endpoint}")
    except requests.exceptions.RequestException as e:
        print(f"Error sending request to {endpoint}: {e}")

@app.route("/game/init", methods=["POST"])
def init_game():
    """ゲーム初期化"""
    data = request.json
    room_id = data.get("room_id")
    if not room_id:
        return "room_id is required", 400

    print(f"Initializing game for room {room_id}")
    game_state[room_id] = {"winner": None, "question": "Rustで最も人気のあるWebフレームワークは？"}

    # 1. 問題を全員に表示するようリクエスト
    send_to_game_engine("/realtime/broadcast", {
        "room_id": room_id,
        "event_type": "question_display",
        "data": {"question": game_state[room_id]["question"]}
    })

    # 2. 早押しアクションを有効にするようリクエスト
    send_to_game_engine("/realtime/enable_action", {
        "room_id": room_id,
        "action_type": "first_press",
        "enabled": True
    })
    
    return jsonify({"status": "initialized"})

@app.route("/game/event", methods=["POST"])
def handle_event():
    """リアルタイム判定結果の処理"""
    data = request.json
    room_id = data.get("room_id")
    event_data = data.get("data", {})
    
    if not room_id:
        return "room_id is required", 400

    print(f"Received event for room {room_id}: {data}")

    if event_data.get("action") == "first_press":
        winner_id = event_data.get("winner")
        if winner_id:
            game_state.setdefault(room_id, {})["winner"] = winner_id
            print(f"Winner in room {room_id} is {winner_id}")

            # 回答フェーズに移行するようリクエスト
            send_to_game_engine("/game/next_phase", {
                "room_id": room_id,
                "phase": "answering",
                "ui_data": {
                    "message": f"Player {winner_id} is answering!",
                    "winner": winner_id
                }
            })

    return jsonify({"status": "event_handled"})

@app.route("/game/timer_expired", methods=["POST"])
def timer_expired():
    """タイマー満了時の処理"""
    data = request.json
    room_id = data.get("room_id")
    print(f"Timer expired for room {room_id}: {data}")
    return jsonify({"status": "timer_handled"})

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5001, debug=True)
