# 🎵 Unison Protocol

*複数言語・プラットフォーム間での通信の調和*

[![Crates.io](https://img.shields.io/crates/v/unison-protocol.svg)](https://crates.io/crates/unison-protocol)
[![Documentation](https://docs.rs/unison-protocol/badge.svg)](https://docs.rs/unison-protocol)
[![Build Status](https://github.com/chronista-club/unison-protocol/workflows/CI/badge.svg)](https://github.com/chronista-club/unison-protocol/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Unison Protocol** は、KDLベースの型安全通信フレームワークです。複数のプログラミング言語に対応した自動コード生成により、シームレスなクライアント・サーバー間通信を実現します。

## 目次

- [特徴](#-特徴)
- [クイックスタート](#-クイックスタート)
- [プロジェクト構造](#-プロジェクト構造)
- [サンプル](#-サンプル)
- [テスト](#-テスト)
- [統合](#-統合)
- [アーキテクチャの特徴](#️-アーキテクチャの特徴)
- [技術仕様詳細](#-技術仕様詳細)
- [セキュリティ考慮事項](#-セキュリティ考慮事項)
- [パフォーマンス特性](#-パフォーマンス特性)
- [コントリビューション](#-コントリビューション)
- [今後の計画](#-今後の計画)
- [参考資料](#-参考資料)
- [ライセンス](#-ライセンス)

## ✨ 特徴

- **🎯 型安全通信**: KDLプロトコル定義からの自動コード生成
- **🌐 多言語サポート**: Rust、TypeScript等の複数言語での自動クライアント・サーバーコード生成
- **⚡ QUICベース**: 多重化を伴う超低遅延双方向通信
- **🔍 スキーマ検証**: コンパイル時・実行時の両方でプロトコル検証
- **🚀 非同期優先**: async/await パターンをベースとした設計
- **📚 豊富なプロトコル定義**: サービス、メッセージ、メソッド、複雑型のサポート
- **🔧 開発者フレンドリー**: 包括的なエラーハンドリングを備えたシンプルなAPI

## 🚀 クイックスタート

### 1. プロトコル定義の作成

KDLスキーマファイル（例：`my_protocol.kdl`）を作成します：

```kdl
protocol "my-service" version="1.0.0" {
    namespace "my.service"
    description "私の素晴らしいサービスプロトコル"
    
    service "UserService" {
        method "create_user" {
            description "新しいユーザーを作成"
            request {
                field "name" type="string" required=true
                field "age" type="number" required=false
            }
            response {
                field "user_id" type="string" required=true
                field "message" type="string" required=true
            }
        }
    }
}
```

### 2. サーバー実装

```rust
use unison_protocol::{UnisonProtocol, UnisonServer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("my_protocol.kdl"))?;
    
    let mut server = protocol.create_server();
    server.register_handler("create_user", |payload| {
        let name = payload["name"].as_str().unwrap_or("Anonymous");
        let user_id = uuid::Uuid::new_v4().to_string();
        
        Ok(json!({
            "user_id": user_id,
            "message": format!("ようこそ、{}さん！", name)
        }))
    });
    
    println!("🎵 Unison Protocolサーバーが 127.0.0.1:8080 (QUIC) で起動しました");
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}
```

### 3. クライアント実装

```rust
use unison_protocol::{UnisonProtocol, UnisonClient};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("my_protocol.kdl"))?;
    
    let mut client = protocol.create_client();
    client.connect("127.0.0.1:8080").await?;
    
    let response = client.call("create_user", json!({
        "name": "Alice",
        "age": 30
    })).await?;
    
    println!("レスポンス: {}", response);
    client.disconnect().await?;
    Ok(())
}
```

## 📂 プロジェクト構造

```
unison/
├── src/
│   ├── lib.rs              # メインライブラリインターフェース
│   ├── core/               # コアプロトコル型
│   │   └── mod.rs          # UnisonMessage, UnisonResponse 等
│   ├── parser/             # KDLスキーマパーサー
│   │   ├── mod.rs          # パーサーインターフェース
│   │   ├── schema.rs       # スキーマ解析ロジック
│   │   └── types.rs        # 型定義
│   ├── codegen/            # コードジェネレーター
│   │   ├── mod.rs          # ジェネレーターインターフェース
│   │   ├── rust.rs         # Rustコード生成
│   │   └── typescript.rs   # TypeScriptコード生成
│   └── network/            # ネットワーク実装
│       ├── mod.rs          # ネットワークトレイトとエラー
│       ├── client.rs       # クライアント実装
│       ├── server.rs       # サーバー実装
│       └── quic.rs         # QUICトランスポート層
├── schemas/                # プロトコル定義
│   ├── unison_core.kdl     # コアUnison Protocol
│   ├── ping_pong.kdl       # サンプル ping-pong プロトコル
│   └── diarkis_devtools.kdl # Diarkis DevTools プロトコル
├── examples/               # 使用例
│   ├── unison_ping_server.rs # Unisonサーバーサンプル
│   └── unison_ping_client.rs # Unisonクライアントサンプル
├── Cargo.toml              # Rust依存関係
└── README.md               # このファイル
```

## 🔧 サンプル

リポジトリには、使用開始に役立つ複数のサンプルが含まれています：

### Ping-Pongサーバーの実行
```bash
cargo run --example unison_ping_server
```

### Ping-Pongクライアントの実行
```bash
# 別のターミナルで
cargo run --example unison_ping_client
```

### 利用可能なテストメソッド

ping-pongサンプルは、以下のUnison Protocolメソッドを実演します：

- **`ping`**: メッセージエコーを使った基本的なリクエスト・レスポンス
- **`echo`**: オプション変換付きの任意JSONデータエコー
- **`get_server_time`**: サーバータイムスタンプと稼働時間を取得

## 🧠 テスト

テストスイートの実行：

```bash
# 全テストの実行
cargo test

# ログ付きで実行
RUST_LOG=info cargo test

# 特定テストの実行
cargo test unison_protocol_tests
```

## 🔌 統合

### インストール

`Cargo.toml`に追加：

```toml
[dependencies]
unison-protocol = "0.1"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
quinn = "0.11"  # QUIC通信用
```

または、cargo経由でインストール：

```bash
cargo add unison-protocol
```

### 言語サポートロードマップ

- ✅ **Rust**: async/await完全サポート
- 🚧 **TypeScript**: コード生成（開発中）

## 🏗️ アーキテクチャの特徴

### 設計原則

Unison Protocolは以下の原則に基づいて設計されています：

- **スキーマ駆動開発**: プロトコル定義が実装を牽引
- **型安全性**: コンパイル時・実行時の両方で型チェック
- **非同期優先**: async/awaitパターンを基盤とした設計
- **QUIC最適化**: HTTP/3の基盤技術による高速で信頼性の高い通信

### KDL（KDL Document Language）採用の利点

1. **人間に読みやすい**: JSONやYAMLよりも構造化された記述が可能
2. **コメント対応**: プロトコル定義内での詳細な説明記述
3. **階層構造**: 複雑なプロトコル定義を明確に表現
4. **設定指向**: 設定ファイル形式としての最適化

### メッセージフロー

```
クライアント                    サーバー
    |                            |
    |-- UnisonMessage ---------->|
    |   (method, payload)        |
    |                            |-- ハンドラー処理
    |                            |
    |<-- UnisonResponse ---------|
    |   (success, payload/error) |
```

## 📋 技術仕様詳細

### コアメッセージ型

**UnisonMessage** - 全ての通信の標準メッセージ形式：
```rust
struct UnisonMessage {
    id: String,           // 一意メッセージ識別子
    method: String,       // RPCメソッド名
    payload: JsonValue,   // メソッドパラメータ（JSON形式）
    timestamp: DateTime,  // メッセージ作成タイムスタンプ
    version: String,      // プロトコルバージョン（デフォルト："1.0.0"）
}
```

**UnisonResponse** - 標準レスポンス形式：
```rust
struct UnisonResponse {
    id: String,                    // 対応するリクエストメッセージID
    success: bool,                 // 操作成功フラグ
    payload: Option<JsonValue>,    // レスポンスデータ（JSON形式）
    error: Option<String>,         // 操作失敗時のエラーメッセージ
    timestamp: DateTime,           // レスポンス作成タイムスタンプ
    version: String,               // プロトコルバージョン
}
```

### 型システム

Unison Protocolは以下の基本型をサポート：

| 型 | 説明 | Rustマッピング | TypeScriptマッピング |
|------|-------------|--------------|---------------------|
| `string` | UTF-8テキスト | `String` | `string` |
| `number` | 数値 | `f64` | `number` |
| `bool` | 真偽値 | `bool` | `boolean` |
| `timestamp` | ISO-8601日時 | `DateTime<Utc>` | `string` |
| `json` | 任意JSON | `serde_json::Value` | `any` |
| `array` | 配列 | `Vec<T>` | `T[]` |

### エラーハンドリング

包括的なエラーハンドリングシステム：

- **接続エラー**: ネットワーク関連の問題
- **プロトコルエラー**: バージョン不整合、無効なメッセージ形式
- **アプリケーションエラー**: ビジネスロジック関連のエラー
- **検証エラー**: 型チェック、必須フィールドの不備

## 🔒 セキュリティ考慮事項

### 推奨セキュリティ実践

1. **トランスポートセキュリティ**: 本番環境ではTLS/WSSの使用を推奨
2. **入力検証**: 全ての外部入力の自動検証
3. **認証・認可**: アプリケーションレベルでの実装
4. **レート制限**: DoS攻撃対策

### セキュリティ機能

- 必須フィールドの自動検証
- 全パラメータの型チェック
- カスタム検証のサポート

## 🚀 パフォーマンス特性

### 最適化ポイント

- **超低レイテンシー**: QUIC使用による0-RTT接続と多重化によるヘッドオブラインブロッキング解消
- **高スループット**: 非同期ランタイムによる同時リクエスト処理
- **効率的なシリアライゼーション**: JSON形式での軽量なメッセージ交換

### ベンチマーク指標

- メッセージオーバーヘッド: 100-200バイト（典型的）
- 同時接続サポート: トランスポートレイヤーに依存
- メモリ使用量: 効率的な非同期処理による最適化

## 🤝 コントリビューション

コントリビューションを歓迎します！以下のガイドラインに従ってください：

1. リポジトリをフォークする
2. フィーチャーブランチを作成（`git checkout -b feature/amazing-feature`）
3. 変更をコミット（`git commit -m 'Add amazing feature'`）
4. ブランチにプッシュ（`git push origin feature/amazing-feature`）
5. プルリクエストを作成

### 開発環境セットアップ

```bash
# リポジトリのクローン
git clone https://github.com/chronista-club/unison-protocol.git
cd unison-protocol

# 依存関係のインストール
cargo build

# テストの実行
cargo test

# サンプルの実行
cargo run --example unison_ping_server
```

## 📈 今後の計画

### 予定機能

- **ストリーミングサポート**: Server-sent eventsと双方向ストリーミング
- **スキーマ進化**: 実行時スキーマ更新とマイグレーション
- **圧縮**: 大きなペイロードに対するメッセージ圧縮
- **バッチ操作**: 単一リクエストでの複数操作

### 言語サポート拡張

- TypeScript クライアント・サーバー生成の完成

## 📚 参考資料

- [KDL仕様](https://kdl.dev/)
- [WebSocketプロトコル (RFC 6455)](https://tools.ietf.org/html/rfc6455)
- [JSONスキーマ](https://json-schema.org/)

## 📄 ライセンス

このプロジェクトは [MITライセンス](LICENSE) の下でライセンスされています。

---

**Unison Protocol** - *複数言語・プラットフォーム間での通信の調和* 🎵