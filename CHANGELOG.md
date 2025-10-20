# 変更履歴

このプロジェクトの主要な変更はこのファイルに記録されます。

フォーマットは [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に基づいており、
このプロジェクトは [セマンティックバージョニング](https://semver.org/lang/ja/) に準拠しています。

## [0.1.0-alpha3] - 2025-10-21

### 追加
- 新しい`frame`モジュールの実装
  - `UnisonFrame`構造体でヘッダー、ペイロード、フラグ、設定を統合管理
  - `RkyvPayload`によるゼロコピーシリアライゼーション
  - Zstd圧縮とCRC32チェックサム機能
  - フレームベースの通信プロトコル
- `AI_CONTEXT.md`を追加してプロジェクトコンテキストを整理
- `docs/ja/packet.md`を追加してパケット仕様を文書化

### 変更
- パーサーをknuffelに完全移行
  - KDLスキーマパーシングをknuffelベースに統一
  - インラインメソッド定義をサポート（`MethodMessage`型）
- ネットワーク層を`UnisonFrame<RkyvPayload<ProtocolMessage>>`を使用するように統合
- `packetモジュールをframeモジュールにリネーム
- テストコードを`new_with_json()`メソッドに統一
- WebSocketモジュールを削除（QUICに集中）

### 改善
- CI/CDの強化
  - Windows環境でのPDB制限エラーを回避（codegen-units増加）
  - macOS環境でのリンカーシンボル長制限に対応
  - Clippy警告を修正してCI通過を実現
- ドキュメント整理
  - 英語版ドキュメントを削除して日本語版に集約
  - 不要なファイルを削除（CONTRIBUTING.ja.md、SECURITY.ja.md等）
- 依存関係の更新
  - MSRV（Minimum Supported Rust Version）を1.85に更新
  - `cargo-deny` 0.18フォーマットに対応
  - knuffelをフォーク版（chronista-club/knuffel）に変更

### 修正
- パケットビルダーでチェックサムが正しく有効化されるように修正
- CI環境でのリンカーエラーを修正
- フォーマットとベンチマークのAPIミスマッチを修正
- スキーマパーステストを簡略化

## [0.1.0] - 2025-01-05

### 追加
- 🎵 QUICトランスポートを採用したUnison Protocolの初期リリース
- 型安全な通信のためのKDLベースのスキーマ定義システム
- 超低遅延トランスポートを備えたQUICクライアントとサーバー実装
- 包括的な型検証とコード生成を備えたスキーマパーサー
- Quinn + rustlsを使用したTLS 1.3対応の最新QUICトランスポート層
- 自動証明書生成とプロダクション用rust-embedサポート
- コアプロトコル型: `UnisonMessage`, `UnisonResponse`, `NetworkError`
- `UnisonClient`, `UnisonServer`, `UnisonServerExt` トレイトによるネットワーク抽象化
- 完全なドキュメントとQUICプロトコル仕様
- 実装例:
  - `unison_ping_server.rs` - ハンドラー登録機能を備えたQUICベースのping-pongサーバー
  - `unison_ping_client.rs` - レイテンシ測定付き高性能QUICクライアント
- スキーマ定義:
  - `unison_core.kdl` - コアUnisonプロトコルスキーマ
  - `ping_pong.kdl` - 複数メソッドを含むping-pongプロトコル例
  - `diarkis_devtools.kdl` - 開発ツール用の高度なプロトコル
- 包括的なテストスイート:
  - `simple_quic_test.rs` - QUIC機能と証明書テスト
  - `quic_integration_test.rs` - 完全なクライアント・サーバー統合テスト
- `build.rs`による自動証明書生成ビルドシステム
- オープンソース配布用MITライセンス

### 機能
- **型安全性**: KDLスキーマによるコンパイル時と実行時のプロトコル検証
- **QUICトランスポート**: TLS 1.3暗号化による超低遅延通信
- **マルチストリームサポート**: 単一接続での効率的な並列通信
- **ゼロコンフィギュレーション**: 開発環境用の自動証明書生成
- **プロダクション対応**: バイナリ内の組み込み証明書用rust-embedサポート
- **スキーマ検証**: 包括的な検証を備えたKDLベースのプロトコル定義
- **コード生成**: 自動クライアント/サーバーコード生成（Rust完成、TypeScript予定）
- **非同期ファースト**: 高性能非同期I/Oとfutures用にtokioで構築
- **包括的テスト**: 完全なクライアント・サーバーシナリオの単一プロセス統合テスト
- **開発者体験**: tracingによるリッチなログ、エラー処理、デバッグサポート

### 技術詳細
- **コア依存関係**: 
  - `quinn` 0.11+ - QUICプロトコル実装
  - `rustls` 0.23+ - ring暗号によるTLS 1.3暗号化
  - `tokio` 1.40+ - フル機能付き非同期ランタイム
  - `kdl` 4.6+ - スキーマ解析と検証
  - `serde` 1.0+ - derive機能付きJSONシリアライゼーション
  - `rcgen` 0.13+ - 自動証明書生成
  - `rust-embed` 8.5+ - バイナリへの証明書埋め込み
  - `Cargo.toml`に完全な依存関係リストと機能
- **ビルドシステム**: 証明書自動生成とコード生成を備えたカスタムビルドスクリプト
- **テスト**: 包括的なユニットテスト、QUIC統合テスト、パフォーマンス検証
- **ドキュメント**: 完全なAPIドキュメント、使用例、QUICプロトコル仕様
- **セキュリティ**: デフォルトでTLS 1.3、自動証明書管理、セキュアなデフォルト設定

### リポジトリ構造
```
unison-protocol/
├── .github/workflows/ci.yml    # GitHub Actions CI with Rust matrix testing
├── .gitignore                  # Git ignore rules
├── Cargo.toml                  # Rust package with QUIC dependencies
├── LICENSE                     # MIT License
├── README.md                   # Updated QUIC-focused documentation
├── CHANGELOG.md                # This file
├── build.rs                    # Build script with certificate generation
├── src/                        # Source code
│   ├── lib.rs                  # Library entry point with QUIC exports
│   ├── core/                   # Core protocol types and traits
│   ├── parser/                 # KDL schema parsing with validation
│   ├── codegen/                # Code generation for Rust and TypeScript
│   └── network/                # QUIC implementation
│       ├── mod.rs              # Network traits and error types
│       ├── client.rs           # QUIC client implementation
│       ├── server.rs           # QUIC server with handler registration
│       └── quic.rs             # QUIC transport with Quinn/rustls
├── assets/                     # Build-time generated assets
│   └── certs/                  # Auto-generated QUIC certificates
│       ├── cert.pem            # Server certificate
│       └── private_key.der     # Private key
├── schemas/                    # Protocol schema definitions
│   ├── unison_core.kdl         # Core protocol schema
│   ├── ping_pong.kdl           # Example ping-pong with multiple methods
│   └── diarkis_devtools.kdl    # Advanced development tools protocol
├── tests/                      # Integration tests
│   ├── simple_quic_test.rs     # QUIC functionality tests
│   └── quic_integration_test.rs # Full client-server integration
├── examples/                   # Usage examples
│   ├── unison_ping_server.rs   # QUIC server with handler registration
│   └── unison_ping_client.rs   # QUIC client with performance metrics
└── docs/                       # Documentation
    ├── README.md               # Japanese documentation
    ├── README-en.md            # English documentation  
    └── PROTOCOL_SPEC_ja.md     # QUIC protocol specification
```

### パフォーマンス特性
- **接続**: 超高速接続確立
- **レイテンシ**: 超低遅延通信
- **スループット**: マルチストリーミングによる高スループット
- **セキュリティ**: TLS 1.3暗号化とforward secrecy
- **リソース**: CPU/メモリ使用量の最適化

### 今後の予定（ロードマップ）
- [ ] crates.ioへ `unison-protocol` v0.1.0 として公開
- [ ] WebTransport APIサポート付きTypeScript/JavaScriptコード生成
- [ ] aioquic統合によるPythonバインディング
- [ ] quic-go統合によるGoバインディング
- [ ] カスタムバリデータによる拡張スキーマ検証
- [ ] パフォーマンスベンチマークと最適化分析
- [ ] ロードバランシングとコネクションマイグレーション機能
- [ ] 大規模データ転送のためのストリーミングサポート

### 移行に関する注意
これはQUICトランスポートを主要プロトコルとした初期の独立リリースです。このフレームワークは、優れたパフォーマンスとセキュリティ特性を活用し、QUIC通信専用に設計されています。

### 既知の問題
- 本番環境での証明書検証には適切なCA署名済み証明書が必要
- 一部の企業ファイアウォールはQUICに必要なUDPトラフィックをブロックする可能性
- WebTransport APIのサポートはブラウザにより異なる（Chrome 97+、Firefox実験的）

### コミュニティとサポート
- GitHub Issues: バグ報告と機能リクエスト
- GitHub Discussions: コミュニティサポートと質問  
- ドキュメント: `docs/` ディレクトリ内の包括的なガイド
- 例: `examples/` 内の本番対応サーバー/クライアント実装