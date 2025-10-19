# AI Context - Unison Protocol

このドキュメントは、AIアシスタント（GitHub Copilot、Claude、ChatGPT等）がUnison Protocolプロジェクトを理解し、効果的な支援を提供するためのコンテキスト情報です。

## プロジェクト概要

**Unison Protocol**は、KDLベースの型安全な通信プロトコルフレームワークです。QUICトランスポートを活用し、高速・安全・拡張可能な分散システムの構築を支援します。

### 主要な技術選択
- **言語**: Rust 2024 Edition (MSRV: 1.85)
- **トランスポート**: QUIC (quinn)
- **シリアライゼーション**: rkyv（ゼロコピー）、serde_json（互換性）
- **圧縮**: zstd Level 1（2KB以上で自動適用）
- **スキーマ定義**: KDL (KDL Document Language)
- **非同期ランタイム**: Tokio

### 設計原則
1. **型安全性優先**: コンパイル時の型チェックを最大限活用
2. **ゼロコピー最適化**: メモリアロケーションを最小限に
3. **自動圧縮**: 帯域効率を透過的に最適化
4. **拡張可能**: トレイトベースの抽象化で柔軟な拡張
5. **日本語ファースト**: ドキュメント・コメントは日本語で統一

## アーキテクチャ

### レイヤー構造

```
┌─────────────────────────────────────┐
│     アプリケーション層              │
│  (Services, Handlers, Business Logic)│
├─────────────────────────────────────┤
│        プロトコル層                  │
│  (Schema Parser, Code Generation)    │
├─────────────────────────────────────┤
│        パケット層                    │
│  (UnisonPacket, Serialization)       │
├─────────────────────────────────────┤
│      トランスポート層                │
│  (QUIC, Stream Management)           │
└─────────────────────────────────────┘
```

### モジュール構成

```
src/
├── packet/         # バイナリパケット層（新規追加）
│   ├── mod.rs     # UnisonPacket<T>実装
│   ├── header.rs  # 48バイト固定長ヘッダー
│   ├── flags.rs   # 16ビットフラグ管理
│   ├── payload.rs # Payloadableトレイト
│   └── serialization.rs # rkyv/zstd処理
│
├── network/       # ネットワーク層
│   ├── quic.rs   # QUICトランスポート
│   ├── client.rs # クライアント実装
│   ├── server.rs # サーバー実装
│   └── service.rs # サービス抽象化
│
├── parser/        # KDLスキーマパーサー
├── codegen/       # コード生成（Rust/TypeScript）
├── context/       # CGPコンテキスト
└── core/         # コア型定義
```

## 主要な型とトレイト

### UnisonPacket<T: Payloadable>

```rust
// ジェネリックパケット型
pub struct UnisonPacket<T> where T: Payloadable {
    raw_data: Bytes,
    _phantom: PhantomData<T>,
}

// ビルダーパターンでの構築
let packet = UnisonPacket::builder()
    .with_stream_id(123)
    .with_sequence(1)
    .with_checksum()
    .build(payload)?;
```

### Payloadableトレイト

```rust
pub trait Payloadable: Archive + Sized + Serialize<AllocSerializer<256>> {
    fn to_bytes(&self) -> Result<Bytes, PayloadError>;
    fn from_bytes(bytes: &Bytes) -> Result<Self, PayloadError>;
    fn from_bytes_zero_copy(bytes: &[u8]) -> Result<&Self::Archived, PayloadError>;
}
```

### PacketFlags

```rust
// ビットフラグによるパケット制御
pub struct PacketFlags(u16);
// COMPRESSED(0x0001), ENCRYPTED(0x0002), FRAGMENTED(0x0004)...
```

## コーディングガイドライン

### Rust特有の注意点

1. **ライフタイム管理**
   - ゼロコピー実装では借用チェッカーに注意
   - `'static`は避け、適切なライフタイムを明示

2. **エラーハンドリング**
   - `Result`型を積極的に使用
   - `thiserror`でエラー型を定義
   - パニックは避け、エラーを適切に伝播

3. **非同期処理**
   - `async`/`await`を使用
   - `tokio::spawn`での並行処理
   - `Arc<RwLock<T>>`での状態共有

4. **最適化**
   - `#[inline]`は慎重に使用
   - ジェネリクスでコードの重複を避ける
   - `Box`/`Arc`は必要最小限に

### 命名規則

- **構造体/トレイト**: PascalCase（例: `UnisonPacket`）
- **関数/メソッド**: snake_case（例: `to_bytes`）
- **定数**: SCREAMING_SNAKE_CASE（例: `COMPRESSION_THRESHOLD`）
- **型パラメータ**: 単一大文字（例: `T: Payloadable`）

### ドキュメント

```rust
/// UnisonPacketのヘッダー構造
/// 
/// 固定長48バイトのヘッダーで、パケットのメタデータを格納します。
pub struct UnisonPacketHeader {
    /// プロトコルバージョン（現在: 0x01）
    pub version: u8,
    // ...
}
```

## よくある実装パターン

### 1. ビルダーパターン

```rust
impl<T: Payloadable> UnisonPacketBuilder<T> {
    pub fn new() -> Self { /* ... */ }
    pub fn with_stream_id(mut self, id: u64) -> Self {
        self.header.stream_id = id;
        self
    }
    pub fn build(self, payload: T) -> Result<UnisonPacket<T>> { /* ... */ }
}
```

### 2. ゼロコピーデシリアライゼーション

```rust
// rkyvを使用した効率的な読み取り
let archived = rkyv::check_archived_root::<T>(bytes)?;
// archivedは元のbytesを参照（コピーなし）
```

### 3. 自動圧縮

```rust
if payload_size >= COMPRESSION_THRESHOLD {
    let compressed = zstd::encode_all(&payload, COMPRESSION_LEVEL)?;
    if compressed.len() < payload_size {
        // 圧縮が効果的な場合のみ使用
        return compressed;
    }
}
```

## テスト戦略

### ユニットテスト
- 各モジュールに`#[cfg(test)]`セクション
- `cargo test`で全テスト実行
- `pretty_assertions`で見やすいアサーション

### 統合テスト
- `tests/`ディレクトリに配置
- 実際のネットワーク通信をシミュレート
- ラウンドトリップテストで完全性確認

### ベンチマーク
- `criterion`を使用
- `benches/`ディレクトリに配置
- 圧縮効率、シリアライゼーション速度を測定

## 依存関係の理由

| クレート | バージョン | 用途 | 選定理由 |
|---------|-----------|------|----------|
| quinn | 0.11 | QUICトランスポート | 最も成熟したRust QUIC実装 |
| rkyv | 0.7 | シリアライゼーション | ゼロコピーで最高速 |
| zstd | 0.13 | 圧縮 | リアルタイム圧縮に最適 |
| bytes | 1.10 | バッファ管理 | 効率的なバイト操作 |
| tokio | 1.40 | 非同期ランタイム | デファクトスタンダード |
| serde | 1.0 | JSON互換性 | エコシステム標準 |

## パフォーマンス目標

### レイテンシー
- パケット処理: < 1μs（非圧縮）
- 圧縮（2KB）: < 5μs
- QUIC接続確立: < 100ms

### スループット
- 単一ストリーム: > 1Gbps
- 並行ストリーム: > 10,000/秒
- パケット処理: > 1M packets/秒

### メモリ使用量
- パケットヘッダー: 48 bytes固定
- ゼロコピー読み取り: 追加アロケーションなし
- バッファプール: 事前割り当て

## デバッグのヒント

### ログ出力
```bash
# 詳細ログ有効化
RUST_LOG=debug cargo run

# 特定モジュールのみ
RUST_LOG=unison_protocol::packet=trace cargo run
```

### よくある問題

1. **rkyvのバージョン不一致**
   - `AllocSerializer<256>`の定数ジェネリクスに注意
   - `check_bytes`機能フラグが必要

2. **ライフタイムエラー**
   - ゼロコピー実装では参照の有効期間に注意
   - バッファの所有権を適切に管理

3. **非同期デッドロック**
   - `RwLock`の取得順序を一定に
   - `.await`を跨いだロック保持を避ける

## 貢献ガイドライン

### コミットメッセージ
```
feat: 新機能追加
fix: バグ修正
docs: ドキュメント更新
test: テスト追加・修正
refactor: リファクタリング
perf: パフォーマンス改善
chore: ビルド・ツール関連
```

### プルリクエスト
1. `feature/`ブランチで開発
2. `cargo fmt`でフォーマット
3. `cargo clippy`で警告解消
4. テストカバレッジ80%以上
5. 日本語でPR説明記載

## 今後の開発方向

### 短期（v0.2.0）
- [ ] パケットフラグメンテーション実装
- [ ] ベンチマークスイート追加
- [ ] エラーリカバリー強化

### 中期（v0.5.0）
- [ ] 暗号化サポート（AES-GCM）
- [ ] TypeScriptバインディング生成
- [ ] メトリクス・監視機能

### 長期（v1.0.0）
- [ ] プロダクション対応
- [ ] クラスター機能
- [ ] 負荷分散・フェイルオーバー

## AIへの推奨事項

### コード生成時の注意
1. **必ずエラーハンドリングを含める**
2. **ライフタイムは明示的に指定**
3. **ドキュメントコメントは日本語で**
4. **テストコードも同時に生成**
5. **パフォーマンスを意識した実装**

### レビュー時の観点
1. **メモリ安全性**: 不正な参照やリークがないか
2. **エラー処理**: パニックせずResult型で処理
3. **並行性**: データ競合やデッドロックのリスク
4. **効率性**: 不要なアロケーションやコピー
5. **可読性**: 適切な抽象化と命名

### 質問への回答方針
1. **具体的なコード例を提示**
2. **パフォーマンスへの影響を説明**
3. **代替案がある場合は比較**
4. **既存コードとの整合性を重視**
5. **Rustのベストプラクティスに従う**