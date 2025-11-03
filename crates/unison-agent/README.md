# 🤖 Unison Agent

**Claude Agent SDK**のRust実装を**Unisonプロトコル**上で動作させるためのライブラリ

## 📌 概要

Unison Agentは、[claude-agent-sdk](https://crates.io/crates/claude-agent-sdk)をベースに、Unisonプロトコルと統合可能な形でClaude AIエージェント機能を提供します。

### 主要機能

- ✅ **シンプルなクエリAPI**: ワンショットクエリで簡単にClaude AIと対話
- ✅ **対話型クライアント**: ステートフルな会話セッション
- ✅ **バッチ処理**: 複数のクエリを効率的に処理
- ✅ **完全非同期**: Tokioベースの非同期実装
- ✅ **型安全**: Rustの型システムによる安全な実装
- ✅ **エラーハンドリング**: 包括的なエラー処理

## 🚀 クイックスタート

### 依存関係

```toml
[dependencies]
unison-agent = "0.1"
tokio = { version = "1.40", features = ["full"] }
```

### 基本的な使用方法

#### 1. シンプルなクエリ

```rust
use unison_agent::AgentClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AgentClient::new();
    let response = client.query("What is 2 + 2?").await?;
    println!("Response: {}", response);
    Ok(())
}
```

#### 2. 対話型チャット

```rust
use unison_agent::client::InteractiveClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InteractiveClient::new().await?;
    
    // メッセージを送信
    client.send_message("Hello, Claude!").await?;
    
    // レスポンスを受信
    let responses = client.receive_response().await?;
    for response in responses {
        println!("{}", response);
    }
    
    Ok(())
}
```

#### 3. バッチクエリ

```rust
use unison_agent::AgentClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AgentClient::new();
    
    let prompts = vec![
        "What is the capital of Japan?",
        "What is 10 + 15?",
        "Name a programming language.",
    ];
    
    let responses = client.batch_query(prompts).await?;
    for (i, response) in responses.iter().enumerate() {
        println!("Response {}: {}", i + 1, response);
    }
    
    Ok(())
}
```

## 📚 Examples

プロジェクトには以下のサンプルコードが含まれています:

```bash
# シンプルなクエリ
cargo run --example simple_query

# 対話型チャット
cargo run --example interactive_chat

# バッチクエリ
cargo run --example batch_query
```

## 🏗️ アーキテクチャ

```
unison-agent/
├── src/
│   ├── lib.rs          # ライブラリのエントリポイント
│   ├── agent.rs        # AgentClient実装
│   ├── client.rs       # InteractiveClient実装
│   └── error.rs        # エラー型定義
└── examples/           # 使用例
    ├── simple_query.rs
    ├── interactive_chat.rs
    └── batch_query.rs
```

### コンポーネント

#### `AgentClient`
- シンプルなワンショットクエリに使用
- ステートレスな操作
- 軽量で高速

#### `InteractiveClient`
- 対話型の会話セッション
- ステートフルな操作
- 複数ターンの会話に対応

## 🔧 開発

### ビルド

```bash
# ライブラリのビルド
cargo build

# テストの実行
cargo test

# ドキュメントの生成
cargo doc --open
```

### 前提条件

- **Rust**: 1.85以上
- **Node.js**: Claude Code CLIの実行に必要
- **Claude Code CLI**: 
  ```bash
  npm install -g @anthropic-ai/claude-code
  ```

### 環境変数

Claude APIキーの設定:

```bash
export ANTHROPIC_API_KEY=your_api_key_here
```

## 🧪 テスト

```bash
# 全テストの実行
cargo test

# 詳細ログ付き
RUST_LOG=debug cargo test -- --nocapture
```

## 📖 ドキュメント

- [Claude Agent SDK (Rust)](https://docs.rs/claude-agent-sdk)
- [Unison Protocol](../../README.md)
- [API Documentation](https://docs.rs/unison-agent)

## 🤝 コントリビューション

プルリクエストを歓迎します！

1. フォークしてフィーチャーブランチを作成
2. テストを追加
3. `cargo fmt` と `cargo clippy` を実行
4. プルリクエストを提出

## 📄 ライセンス

MIT License - 詳細は [LICENSE](../../LICENSE) ファイルを参照

## 🙏 謝辞

- [claude-agent-sdk](https://crates.io/crates/claude-agent-sdk) - Rust Claude Agent SDK実装
- [Anthropic](https://www.anthropic.com/) - Claude AI
- [Unison Protocol](../../README.md) - 通信プロトコルフレームワーク
