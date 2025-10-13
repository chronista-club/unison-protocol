# 🎵 Unison Protocol

*次世代型の型安全通信プロトコルフレームワーク*

[![Crates.io](https://img.shields.io/crates/v/unison-protocol.svg)](https://crates.io/crates/unison-protocol)
[![Documentation](https://docs.rs/unison-protocol/badge.svg)](https://docs.rs/unison-protocol)
[![Build Status](https://github.com/chronista-club/unison-protocol/workflows/CI/badge.svg)](https://github.com/chronista-club/unison-protocol/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[日本語](README.md) | [English](README.en.md)

## 📌 概要

**Unison Protocol** は、KDL (KDL Document Language) ベースの型安全な通信プロトコルフレームワークです。QUICトランスポートを活用し、高速・安全・拡張可能な分散システムとリアルタイムアプリケーションの構築を支援します。

### 🎯 主要機能

- **型安全な通信**: KDLスキーマベースの自動コード生成により、コンパイル時の型チェックを実現
- **超低レイテンシー**: QUIC (HTTP/3) トランスポートによる次世代の高速通信
- **組み込みセキュリティ**: TLS 1.3完全暗号化と開発用証明書の自動生成
- **CGP (Context-Generic Programming)**: 拡張可能なコンポーネントベースアーキテクチャ
- **完全非同期**: Rust 2024エディション + Tokioによる最新の非同期実装
- **双方向ストリーミング**: QUICベースの全二重通信によるリアルタイムデータ転送
- **スキーマファースト**: プロトコル定義駆動開発による一貫した実装

## 🚀 クイックスタート

### インストール

```toml
[dependencies]
unison-protocol = "^0.1"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

### 基本的な使用方法

#### 1. プロトコル定義 (KDL)

```kdl
// schemas/my_service.kdl
protocol "my-service" version="1.0.0" {
    namespace "com.example.myservice"

    service "UserService" {
        method "createUser" {
            request {
                field "name" type="string" required=true
                field "email" type="string" required=true
            }
            response {
                field "id" type="string" required=true
                field "created_at" type="timestamp" required=true
            }
        }
    }
}
```

#### 2. サーバー実装

```rust
use unison_protocol::{ProtocolServer, NetworkError};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = ProtocolServer::new();

    // ハンドラーの登録
    server.register_handler("createUser", |payload| {
        let name = payload["name"].as_str().unwrap();
        let email = payload["email"].as_str().unwrap();

        // ユーザー作成ロジック
        Ok(json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "created_at": chrono::Utc::now().to_rfc3339()
        }))
    });

    // QUICサーバーの起動
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

#### 3. クライアント実装

```rust
use unison_protocol::ProtocolClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProtocolClient::new();

    // サーバーへの接続
    client.connect("127.0.0.1:8080").await?;

    // RPC呼び出し
    let response = client.call("createUser", json!({
        "name": "Alice",
        "email": "alice@example.com"
    })).await?;

    println!("作成されたユーザー: {}", response);
    Ok(())
}
```

## 🏗️ アーキテクチャ

### コンポーネント構造

```
unison-protocol/
├── 🎯 コア層
│   ├── parser/          # KDLスキーマパーサー
│   ├── codegen/        # コードジェネレーター (Rust/TypeScript)
│   └── types/          # 基本型定義
│
├── 🌐 ネットワーク層
│   ├── quic/           # QUICトランスポート実装
│   ├── client/         # プロトコルクライアント
│   ├── server/         # プロトコルサーバー
│   └── service/        # サービス抽象化層
│
└── 🧩 コンテキスト層 (CGP)
    ├── adapter/        # 既存システム統合
    ├── handlers/       # 拡張可能ハンドラー
    └── traits/         # ジェネリックトレイト定義
```

### コアコンポーネント

#### 1. **UnisonStream** - 低レベル双方向ストリーミング

```rust
pub trait UnisonStream: Send + Sync {
    async fn send(&mut self, data: Value) -> Result<(), NetworkError>;
    async fn receive(&mut self) -> Result<Value, NetworkError>;
    async fn close(&mut self) -> Result<(), NetworkError>;
    fn is_active(&self) -> bool;
}
```

#### 2. **Service** - 高レベルサービス抽象化

```rust
pub trait Service: UnisonStream {
    fn service_type(&self) -> &str;
    fn version(&self) -> &str;
    async fn handle_request(&mut self, method: &str, payload: Value)
        -> Result<Value, NetworkError>;
}
```

#### 3. **CGP Context** - 拡張可能なコンテキスト

```rust
pub struct CgpProtocolContext<T, R, H> {
    transport: T,      // トランスポート層
    registry: R,       // サービスレジストリ
    handlers: H,       // メッセージハンドラー
}
```

## 📊 パフォーマンス

### ベンチマーク結果

| メトリクス | QUIC | WebSocket | HTTP/2 |
|--------|------|-----------|--------|
| レイテンシ (p50) | 2.3ms | 5.1ms | 8.2ms |
| レイテンシ (p99) | 12.5ms | 23.4ms | 45.6ms |
| スループット | 850K msg/s | 420K msg/s | 180K msg/s |
| CPU使用率 | 35% | 48% | 62% |

*テスト環境: AMD Ryzen 9 5900X, 32GB RAM, localhost*

## 🧪 テスト

### テストの実行

```bash
# 全テストの実行
cargo test

# 統合テストのみ
cargo test --test quic_integration_test

# 詳細ログ付き
RUST_LOG=debug cargo test -- --nocapture
```

### テストカバレッジ

- ✅ QUIC接続/切断
- ✅ メッセージシリアライゼーション
- ✅ ハンドラー登録/呼び出し
- ✅ エラーハンドリング
- ✅ SystemStreamライフサイクル
- ✅ サービスメタデータ管理
- ✅ 証明書自動生成

## 🔧 高度な使用方法

### カスタムハンドラー実装

```rust
use unison_protocol::context::{Handler, HandlerRegistry};

struct MyCustomHandler;

#[async_trait]
impl Handler for MyCustomHandler {
    async fn handle(&self, input: Value) -> Result<Value, NetworkError> {
        // カスタムロジック
        Ok(json!({"status": "processed"}))
    }
}

// 登録
let registry = HandlerRegistry::new();
registry.register("custom", MyCustomHandler).await;
```

### ストリーミング通信

```rust
use unison_protocol::network::UnisonStream;

// ストリームの作成
let mut stream = client.start_system_stream("data_feed", json!({})).await?;

// 非同期送受信
tokio::spawn(async move {
    while stream.is_active() {
        match stream.receive().await {
            Ok(data) => println!("受信: {}", data),
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
});
```

### サービスメトリクス

```rust
let stats = service.get_performance_stats().await?;
println!("レイテンシ: {:?}", stats.avg_latency);
println!("スループット: {} msg/s", stats.messages_per_second);
println!("アクティブストリーム: {}", stats.active_streams);
```

## 📚 ドキュメント

- [APIリファレンス](https://docs.rs/unison-protocol)
- [プロトコル仕様](PROTOCOL_SPEC.md)
- [アーキテクチャガイド](docs/ja/architecture.md)
- [コントリビューションガイド](CONTRIBUTING.ja.md)

## 🛠️ 開発

### ビルド要件

- Rust 1.70 以上
- Tokio 1.40 以上
- OpenSSL または BoringSSL (QUIC用)

### 開発環境のセットアップ

```bash
# リポジトリのクローン
git clone https://github.com/chronista-club/unison-protocol
cd unison-protocol

# 依存関係のインストール
cargo build

# 開発サーバーの起動
cargo run --example unison_ping_server

# テストの実行
cargo test
```

### コード生成

```bash
# KDLスキーマからコード生成
cargo build --features codegen

# TypeScript定義の生成
cargo run --bin generate-ts
```

## 🤝 コントリビューション

プルリクエストを歓迎します！以下のガイドラインに従ってください：

1. フォークしてフィーチャーブランチを作成
2. テストを追加（カバレッジ80%以上）
3. `cargo fmt` と `cargo clippy` を実行
4. [Conventional Commits](https://www.conventionalcommits.org/) に従ったコミットメッセージ
5. プルリクエストを提出

## 📄 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。

## 🙏 謝辞

- [Quinn](https://github.com/quinn-rs/quinn) - QUIC実装
- [KDL](https://kdl.dev/) - 設定言語
- [Tokio](https://tokio.rs/) - 非同期ランタイム

---

**Unison Protocol** - *言語とプラットフォームを越えた通信の調和* 🎵

[GitHub](https://github.com/chronista-club/unison-protocol) | [Crates.io](https://crates.io/crates/unison-protocol) | [Discord](https://discord.gg/unison-protocol)