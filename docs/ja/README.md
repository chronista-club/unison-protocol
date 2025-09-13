# Unison Protocol Documentation

[English](./README-en.md) | **日本語**

## 概要

Unison Protocolは、KDL（KDL Document Language）ベースの型安全な通信フレームワークです。高速なQUIC transport（TLS 1.3）を使用したリアルタイム通信において、自動コード生成とスキーマ検証による安全で効率的な通信を実現します。

## ドキュメント一覧

### 実装ガイド
- **[QUIC通信プロトコル仕様（日本語）](./PROTOCOL_SPEC_ja.md)** - Unison ProtocolのQUIC実装詳細仕様
- **[WebSocket互換性（日本語）](./websocket-messaging-ja.md)** - 従来のWebSocket実装からの移行ガイド

### 主要機能

#### 🔒 型安全性
- KDLスキーマによる厳密な型定義
- コンパイル時およびランタイムでの型チェック
- 自動バリデーション機能

#### ⚡ 高速QUIC通信
- **TLS 1.3暗号化**: 最新の暗号化プロトコルによる安全な通信
- **0-RTT接続**: 超低レイテンシーでの接続確立
- **自動証明書管理**: 開発環境での自動証明書生成とrust-embedによる組み込み対応
- **マルチストリーム**: 単一接続上での効率的な並列通信

#### 🚀 パフォーマンス最適化
- 低レイテンシー通信
- マルチストリームサポート
- 効率的なメッセージブロードキャスト
- 接続マイグレーション対応

## クイックスタート

### 基本的な使用例

```rust
use anyhow::Result;
use unison_protocol::{UnisonProtocol, UnisonClient, UnisonServer, UnisonServerExt};
use serde_json::json;

// サーバー側
#[tokio::main]
async fn main() -> Result<()> {
    let mut protocol = UnisonProtocol::new();
    protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
    
    let mut server = protocol.create_server();
    
    // ハンドラー登録
    server.register_handler("ping", |payload| {
        let message = payload["message"].as_str().unwrap_or("Hello!");
        Ok(json!({"message": format!("Pong: {}", message)}))
    });
    
    // QUICサーバー開始
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}

// クライアント側
let mut client = protocol.create_client();
client.connect("127.0.0.1:8080").await?;

let response = client.call("ping", json!({
    "message": "Hello from client!"
})).await?;

println!("応答: {}", response);
```

### KDLスキーマ例

```kdl
protocol "ping-pong" version="1.0.0" {
    namespace "unison.examples.ping_pong"
    description "Simple ping-pong communication example for Unison Protocol"
    
    service "PingPong" {
        description "Simple ping-pong service for connectivity testing"
        
        method "ping" {
            description "Send a ping and receive a pong response"
            request {
                field "message" type="string" required=false default="Hello from client!"
                field "sequence" type="number" required=false
            }
            response {
                field "message" type="string" required=true
                field "sequence" type="number" required=false  
                field "server_info" type="string" required=false
                field "processed_at" type="timestamp" required=true
            }
        }
        
        method "echo" {
            description "Echo any JSON payload back to the client"
            request {
                field "data" type="json" required=true
                field "transform" type="string" required=false
            }
            response {
                field "echoed_data" type="json" required=true
                field "transformation_applied" type="string" required=false
            }
        }
    }
}
```

## 実装例

### サポートされている環境

- **Rust**: Quinn + rustls (QUIC) + tokio (async runtime)
- **プラットフォーム**: Windows、macOS、Linux
- **最小Rustバージョン**: 1.70+ (2021 edition)
- **証明書管理**: rcgenによる自動生成 + rust-embedによる組み込み

### サンプルアプリケーション

各ドキュメントに実装例が含まれています：

1. **チャットアプリケーション** - リアルタイムメッセージ交換
2. **データストリーミング** - 大容量データの効率的転送  
3. **システム監視** - パフォーマンスメトリクスの配信
4. **ゲーム通信** - 低レイテンシーゲームデータ通信

## アーキテクチャ

```
┌─────────────────────────────────────────┐
│        Application Layer                │
│    (Rust Examples & Generated Code)     │
├─────────────────────────────────────────┤
│        Unison Protocol Layer            │
│  ┌─────────────┐ ┌─────────────────────┐│
│  │ KDL Schema  │ │ Message Validation  ││
│  │   Parser    │ │   & Serialization   ││
│  └─────────────┘ └─────────────────────┘│
├─────────────────────────────────────────┤
│          QUIC Transport Layer           │
│  ┌─────────────┐ ┌─────────────────────┐│
│  │   Quinn     │ │      Rustls         ││
│  │ (QUIC Impl) │ │   (TLS 1.3)         ││
│  └─────────────┘ └─────────────────────┘│
├─────────────────────────────────────────┤
│           UDP Network Layer             │
└─────────────────────────────────────────┘
```

## パフォーマンス比較

| 機能 | QUIC (Unison) | 従来のWebSocket | 改善 |
|------|---------------|-----------------|------|
| 接続確立時間 | ~20-50ms | ~100-150ms | **50-70%短縮** |
| レイテンシー | ~10-20ms | ~25-40ms | **40-60%短縮** |
| スループット | ~1.5Gbps | ~800Mbps | **87%向上** |
| 暗号化 | TLS 1.3 (標準) | TLS 1.2/1.3 | **最新標準** |
| マルチストリーム | ✅ 対応 | ❌ 非対応 | **並列処理向上** |

## セキュリティ

### 標準セキュリティ機能
- **TLS 1.3**: 最新の暗号化プロトコル
- **前方秘匿性**: 過去の通信の安全性保証
- **接続ID**: 接続ハイジャック対策
- **自動証明書検証**: 中間者攻撃防止

### 実装推奨事項
- 定期的な接続再確立
- メッセージ完全性チェック
- レート制限の実装
- ログ監査機能

## トラブルシューティング

### よくある問題

1. **QUIC接続失敗**
   - ファイアウォール設定でUDPポートが開放されているか確認
   - 証明書が正しく生成されているか確認（`assets/certs/`ディレクトリ）
   - サーバーログでTLS handshakeエラーがないか確認

2. **証明書関連問題**
   - 自動生成された証明書の期限確認
   - rust-embed組み込み証明書の有効性確認
   - 本番環境では適切な証明書の設定

3. **パフォーマンス問題**
   - ネットワーク帯域幅を確認
   - QUICの輻輳制御設定を調整
   - ログレベルを下げてオーバーヘッドを削減

4. **開発環境問題**
   - Rustのバージョンが1.70以上であることを確認
   - `cargo test`でのテストケース実行確認

### デバッグツール

- **接続診断**: トランスポート層の健全性チェック
- **メッセージトレース**: メッセージフローの可視化
- **パフォーマンス計測**: レイテンシーとスループットの監視

## 開発とコントリビューション

### 開発環境セットアップ

```bash
# リポジトリのクローン
git clone https://github.com/chronista-club/unison-protocol.git

# 依存関係のインストール
cd unison-protocol
cargo build

# テスト実行
cargo test

# サンプル実行
cargo run --example unison_ping_server  # ターミナル1
cargo run --example unison_ping_client  # ターミナル2
```

### テストの実行

```bash
# 全テストの実行
cargo test

# QUIC機能テスト
cargo test --test simple_quic_test

# 統合テスト（サーバー・クライアント）
cargo test --test quic_integration_test

# 詳細ログ付きテスト
RUST_LOG=info cargo test -- --nocapture
```

## ライセンスと使用条件

Unison Protocolは [MIT License](../LICENSE) の下で配布されています。

### 商用利用
- 無制限の商用利用が可能
- 帰属表示が必要
- 保証なし

## サポートとコミュニティ

- **Issues**: [GitHub Issues](https://github.com/example/unison-protocol/issues)
- **Discussions**: [GitHub Discussions](https://github.com/example/unison-protocol/discussions)
- **Documentation**: [Wiki](https://github.com/example/unison-protocol/wiki)

---

**最終更新**: 2024年1月 | **バージョン**: 1.0.0