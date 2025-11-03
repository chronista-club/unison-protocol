# 🎵 Unison Protocol

*次世代型の型安全通信プロトコルフレームワーク*

[![Crates.io](https://img.shields.io/crates/v/unison.svg)](https://crates.io/crates/unison)
[![Documentation](https://docs.rs/unison/badge.svg)](https://docs.rs/unison)
[![Build Status](https://github.com/chronista-club/unison/workflows/CI/badge.svg)](https://github.com/chronista-club/unison/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)



## 📌 概要

**Unison Protocol** は、KDL (KDL Document Language) ベースの型安全な通信プロトコルフレームワークです。QUICトランスポートを活用し、高速・安全・拡張可能な分散システムとリアルタイムアプリケーションの構築を支援します。

### 🎯 主要機能

- **型安全な通信**: KDLスキーマベースの自動コード生成により、コンパイル時の型チェックを実現
- **超低レイテンシー**: QUIC (HTTP/3) トランスポートによる次世代の高速通信
- **IPv6専用設計**: 最新のネットワーク標準であるIPv6のみをサポート（バグ削減とシンプルな実装）
- **組み込みセキュリティ**: TLS 1.3完全暗号化と開発用証明書の自動生成
- **CGP (Context-Generic Programming)**: 拡張可能なコンポーネントベースアーキテクチャ
- **完全非同期**: Rust 2024エディション + Tokioによる最新の非同期実装
- **双方向ストリーミング**: QUICベースの全二重通信によるリアルタイムデータ転送
- **スキーマファースト**: プロトコル定義駆動開発による一貫した実装
- **ゼロコピー通信**: rkyvベースの効率的なパケットシリアライゼーション
- **自動圧縮**: 2KB以上のペイロードをzstd Level 1で自動圧縮

## 🚀 クイックスタート

### インストール

```toml
[dependencies]
unison = "^0.1"
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
use unison::{ProtocolServer, NetworkError};
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

    // QUICサーバーの起動（IPv6）
    server.listen("[::1]:8080").await?;
    Ok(())
}
```

#### 3. クライアント実装

```rust
use unison::ProtocolClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ProtocolClient::new();

    // サーバーへの接続（IPv6）
    client.connect("[::1]:8080").await?;

    // メソッド呼び出し
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
unison/
├── 🎯 コア層
│   ├── parser/          # KDLスキーマパーサー
│   ├── codegen/        # コードジェネレーター (Rust/TypeScript)
│   ├── types/          # 基本型定義
│   └── packet/         # UnisonPacket型定義
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

#### 3. **UnisonPacket** - ゼロコピー効率的パケット型

```rust
pub struct UnisonPacket<T: Payloadable> {
    // rkyv + zstd による効率的なシリアライゼーション
    // 2KB以上のペイロードは自動圧縮
    // CRC32チェックサム付き
}

impl<T: Payloadable> UnisonPacket<T> {
    pub fn builder() -> UnisonPacketBuilder<T>;
    pub fn from_bytes(data: Bytes) -> Result<Self, PacketError>;
    pub fn extract_payload(&self) -> Result<T, PayloadError>;
}
```

#### 4. **CGP Context** - 拡張可能なコンテキスト

```rust
pub struct CgpProtocolContext<T, R, H> {
    transport: T,      // トランスポート層
    registry: R,       // サービスレジストリ
    handlers: H,       // メッセージハンドラー
}
```

## 📊 パフォーマンス

### 特徴

- **超低レイテンシ**: QUICによる高速通信
- **高スループット**: マルチストリーム並列処理
- **効率的**: ゼロコピーデシリアライゼーション
- **省リソース**: 最適化されたCPU/メモリ使用率

*ベンチマーク結果は実測後に掲載予定*

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

### UnisonPacketによる効率的な通信

```rust
use unison::packet::{UnisonPacket, Payloadable};

// カスタムペイロード定義
#[derive(Archive, Serialize, Deserialize, Debug)]
struct MyPayload {
    message: String,
    timestamp: i64,
    data: Vec<u8>,
}

impl Payloadable for MyPayload {}

// パケットの送信
let payload = MyPayload {
    message: "Hello".to_string(),
    timestamp: 1234567890,
    data: vec![1, 2, 3, 4, 5],
};

let packet = UnisonPacket::builder()
    .payload(payload)
    .priority(5)
    .build()?;

// バイト配列への変換（自動圧縮付き）
let bytes = packet.to_bytes()?;
stream.send_bytes(bytes).await?;

// パケットの受信（ゼロコピーデシリアライゼーション）
let received_bytes = stream.receive_bytes().await?;
let received_packet = UnisonPacket::<MyPayload>::from_bytes(received_bytes)?;
let received_payload = received_packet.extract_payload()?;
```

### カスタムハンドラー実装

```rust
use unison::context::{Handler, HandlerRegistry};

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
use unison::network::UnisonStream;

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

- [APIリファレンス](https://docs.rs/unison)
- [プロトコル仕様](PROTOCOL_SPEC.md)
- [アーキテクチャガイド](docs/ja/architecture.md)
- [パケットモジュール仕様](docs/ja/packet.md)
- [コントリビューションガイド](CONTRIBUTING.ja.md)

## 🛠️ 開発

### ビルド要件

- Rust 1.70 以上
- Tokio 1.40 以上
- OpenSSL または BoringSSL (QUIC用)

### 開発環境のセットアップ

```bash
# リポジトリのクローン
git clone https://github.com/chronista-club/unison
cd unison

# macOSの場合: LLDリンカーをインストール（テスト実行に必要）
brew install lld

# 依存関係のインストール
cargo build

# 開発サーバーの起動
cargo run --example unison_ping_server

# テストの実行
cargo test
```

> **macOS開発者向けの注意**: macOSの標準リンカーには制限があるため、テストを実行するには`lld`リンカーが必要です。`brew install lld`でインストール後、プロジェクトルートに`.cargo/config.toml`ファイルを作成して以下の設定を追加してください：
> 
> ```toml
> [target.aarch64-apple-darwin]
> linker = "clang"
> rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/ld64.lld"]
> ```
> 
> **注**: `.cargo/config.toml`はローカル開発環境専用の設定ファイルです（`.gitignore`に含まれています）。CI環境では不要です。

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

[GitHub](https://github.com/chronista-club/unison) | [Crates.io](https://crates.io/crates/unison) | [Discord](https://discord.gg/unison)