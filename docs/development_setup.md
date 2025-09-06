# GameCenter 開発環境セットアップガイド

このドキュメントは、GameCenter プロジェクトの開発環境を構築するための詳細な手順を提供します。

## 1. 前提条件

開発を開始する前に、以下のツールがシステムにインストールされていることを確認してください。

- **Git:** ソースコード管理のため。
  - [Git 公式サイト](https://git-scm.com/)
- **Rust:** Game Engine の開発のため。
  - [rustup (Rust インストーラ)](https://rustup.rs/)
  - インストール後、`rustup update` を実行して最新の状態に保つことを推奨します。
- **Node.js & npm:** Frontend の開発のため。
  - [Node.js 公式サイト](https://nodejs.org/)
  - LTS (Long Term Support) バージョンのインストールを推奨します。
- **Python & uv:** サンプルゲームの開発のため。
  - [Python 公式サイト](https://www.python.org/)
  - [uv (Python パッケージインストーラ)](https://github.com/astral-sh/uv)
  - `uv` は `pip install uv` または `brew install uv` (macOS) でインストールできます。

## 2. リポジトリのクローン

まず、プロジェクトのリポジトリをローカルマシンにクローンします。

```bash
git clone [リポジトリのURL]
cd RustGameCenter
```

## 3. 各コンポーネントのセットアップ

プロジェクトルート (`RustGameCenter/`) から、各コンポーネントのディレクトリに移動してセットアップを行います。

### 3.1 Game Engine (Rust)

Game Engine は Rust で書かれており、依存関係の解決とビルドは Cargo が行います。

```bash
cd game-engine

# 依存関係をダウンロードし、プロジェクトをビルドします。
# 初回ビルドには時間がかかる場合があります。
cargo build

# 開発サーバーを起動する際は以下を実行します。
# cargo run
```

### 3.2 Sample Game (Python)

サンプルゲームは Python で書かれており、`uv` を使用して仮想環境と依存パッケージを管理します。

```bash
bash
cd games/quiz

# 仮想環境を作成します。(.venv ディレクトリが作成されます)
uv venv .venv

# 仮想環境を有効化します。
# Windows の場合は `.\.venv\Scripts\activate` を実行します。
source .venv/bin/activate

# 依存パッケージをインストールします。
uv pip install -r requirements.txt

# ゲームサーバーを起動する際は以下を実行します。
# python app.py
```

### 3.3 Frontend (Svelte)

フロントエンドは SvelteKit で書かれており、npm を使用して依存パッケージを管理します。

```bash
cd frontend

# 依存パッケージをインストールします。
npm install

# 開発サーバーを起動する際は以下を実行します。
# npm run dev
```

## 4. IDEの推奨設定

Visual Studio Code (VS Code) の使用を推奨します。以下の拡張機能をインストールすると、開発体験が向上します。

- **Rust Analyzer:** Rust コードの補完、エラーチェック、リファクタリングなど。
- **Svelte for VS Code:** Svelte コンポーネントのシンタックスハイライト、補完など。
- **Python:** Python コードの補完、デバッグ、フォーマットなど。

## 5. 一般的な開発ワークフロー

1.  **コードの変更:** 各コンポーネントのソースコードを編集します。
2.  **ビルド/実行:**
    -   Rust: `cargo run` (変更を自動検出し再ビルド・実行)
    -   Python: `python app.py` (変更を自動検出し再起動)
    -   Svelte: `npm run dev` (変更を自動検出しブラウザに反映)
3.  **テスト:** (テストが実装されている場合)
    -   Rust: `cargo test`
    -   Python: `pytest` (別途インストールが必要)
    -   Svelte: `npm test` (別途設定が必要)

## 6. トラブルシューティング

セットアップや開発中に問題が発生した場合は、`docs/troubleshooting.md` を参照してください。
