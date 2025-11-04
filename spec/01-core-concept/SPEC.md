# spec/01: Unison Protocol - Core Network Concept

## 概要

Unison Protocolは、スケーラブルで高性能な分散ネットワークを実現するため、3層アーキテクチャとQUICトランスポートを採用しています。本仕様書では、ネットワーク層のコアコンセプトと設計思想を定義します。

## 1. 設計思想

### 1.1 目標

- **最高速レベルのネットワーク性能**: QUICの特性を最大限に活用
- **スケーラビリティ**: 数千〜数万のノードをサポート
- **柔軟性**: Hub無しでも動作可能な自律分散型
- **プライベート性**: 独自のIPv6ユーザネットワーク構築

### 1.2 QUICの採用理由

QUICを採用した理由は、**単一のコネクションで複数の通信パターンを効率的に実現できる**ことにあります：

- **双方向ストリーム**: 優先度付き制御メッセージ、RPC、リアルタイム通信
- **データグラム**: 大量データ配信、メディアストリーミング、低レイテンシー通信
- **マルチプレクシング**: Head-of-Line Blocking を回避した並行通信
- **0-RTT接続**: 再接続時のレイテンシー削減
- **組み込みTLS 1.3**: セキュアな通信がデフォルト

この特性により、Unisonは**単一のQUICコネクション上で、多様な通信要求を最適に処理**できます。

## 2. ネットワークトポロジー

### 2.1 3層アーキテクチャ

```
                    ┌──────────┐
                    │   Root   │ ← ネットワーク全体の起点（単一）
                    └──────────┘
                    ↗    ↑    ↖
                   /     |     \
                  /      |      \
            ┌─────┐  ┌─────┐  ┌─────┐
            │ Hub │←→│ Hub │←→│ Hub │ ← 集約・中継ノード（相互接続）
            └─────┘  └─────┘  └─────┘
             ↗  ↑              ↗  ↑
            /   |             /   |
      ┌───┐ ┌───┐       ┌───┐ ┌───┐
      │Agt│ │Agt│       │Agt│ │Agt│ ← 末端ノード（Hub無しでも動作可能）
      └───┘ └───┘       └───┘ └───┘
```

### 2.2 ノードの役割

#### Agent (末端ノード)
- **役割**: アプリケーションの実行環境
- **特徴**:
  - Hub無しでも動作可能（スタンドアロンモード）
  - Hubを通じてネットワークに参加
- **責務**:
  - 起動時にHubを発見・接続
  - Hubが見つからない場合、Hub serviceを起動
  - アプリケーションロジックの実行
  - ローカルリソースの管理

#### Hub (集約・中継ノード)
- **役割**: Agentの集約とメッセージルーティング
- **特徴**:
  - Hub同士で相互接続（メッシュトポロジー）
  - Agentからの接続を受け入れ
  - 独立したサービスプロセスとして動作
- **責務**:
  - 複数Agentからの接続を管理（推奨: 数十〜数百Agent/Hub）
  - Rootへの接続を確立・維持
  - Agent ⇔ Root間のメッセージルーティング
  - Hub間のメッセージ転送
  - 負荷分散とフェイルオーバー

#### Root (ルートノード)
- **役割**: ネットワーク全体の管理・調整
- **特徴**:
  - ネットワークごとに**常に1つ**のRoot
  - 複数のHubから接続を受け入れ（推奨: 数十〜数百Hub/Root）
- **責務**:
  - Hubからの接続を管理
  - グローバルな状態管理
  - ノードディレクトリの管理
  - network_idの発行と管理
  - 認証・認可（オプション）

### 2.3 スケーラビリティ戦略

3層アーキテクチャにより、階層的なスケーリングを実現：

```
1 Root = 100 Hubs (目安)
1 Hub  = 100 Agents (目安)
───────────────────────────
合計   = 10,000 Agents
```

実際のスケールは、ネットワーク条件とハードウェアスペックに依存します。

## 3. 起動シーケンス

### 3.1 Agentの起動フロー

```
┌─────────────┐
│ Agent起動   │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│ Hubディスカバリー   │ ← mDNS → Bootstrap nodes
└──────┬──────────────┘
       │
       ├─ Hub発見 ──────→ 接続
       │
       └─ Hub未発見 ───→ Hub service起動 ──→ 接続
                                           │
                                           └→ スタンドアロンモードで動作可能
```

**詳細**:
1. Agent起動
2. Hubディスカバリー実行（後述のディスカバリー機構）
3. 分岐:
   - **Hubが見つかる**: Hubへ接続し、ネットワークに参加
   - **Hubが見つからない**:
     - Hub serviceを独立プロセスとして起動
     - 起動したHubに接続
     - または、スタンドアロンモードで動作

### 3.2 Hubの起動フロー

```
┌─────────────┐
│ Hub起動     │
└──────┬──────┘
       │
       ▼
┌──────────────────────┐
│ Agent接続待ち受け    │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Rootディスカバリー   │ ← バックグラウンド実行
└──────┬───────────────┘
       │
       ├─ Root発見 ─────→ 接続 ──→ ネットワーク参加
       │
       └─ Root未発見 ───→ 孤立モードで動作
```

**詳細**:
1. Hub service起動
2. Agentからの接続を待ち受け開始
3. Rootディスカバリー実行（バックグラウンド）
4. 分岐:
   - **Rootが見つかる**: Rootに接続し、ネットワークに参加
   - **Rootが見つからない**: 孤立モードで動作（後でRootが現れたら接続）
5. 他のHubを発見した場合、Hub間接続を確立

### 3.3 Rootの起動フロー

```
┌─────────────┐
│ Root起動    │
└──────┬──────┘
       │
       ▼
┌──────────────────────┐
│ Hub接続待ち受け      │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ network_id発行開始   │
└──────────────────────┘
```

**詳細**:
1. Root起動
2. Hubからの接続を待ち受け開始
3. network_idの発行準備
4. ノードディレクトリの初期化

## 4. ディスカバリー機構

### 4.1 Hub Discovery（Agentから実行）

**優先順位**:
1. **mDNS (ローカル優先)**:
   - 同一ネットワーク内のHubを高速発見
   - サービス名: `_unison-hub._udp.local`
   - レイテンシー: < 100ms

2. **Bootstrap Nodes (グローバル)**:
   - 設定ファイルまたは環境変数で指定された既知のHub
   - フォールバック用のグローバルHub
   - レイテンシー: ネットワーク状況に依存

**動作**:
- mDNSで3秒以内に応答がない場合、Bootstrap Nodesを試行
- 複数のHubが見つかった場合、最も近い（レイテンシーが低い）Hubを選択

### 4.2 Root Discovery（Hubから実行）

**方式**:
1. **設定ファイル指定**:
   - `unison.toml`または環境変数`UNISON_ROOT_ADDR`
   - 明示的なRoot指定

2. **Well-known Endpoints**:
   - デフォルトのRootエンドポイント
   - 例: `root.unison.network:8080`

3. **DNS レコード**:
   - `_unison-root._udp.<domain>`

**動作**:
- 設定ファイル → DNS → Well-known の順で試行
- Rootが見つからない場合、定期的に再試行（exponential backoff）

### 4.3 Hub-Hub Discovery

**方式**:
- Rootからの通知: Rootに接続した際に、他のHubのリストを取得
- mDNS: ローカルネットワーク内の他のHubを発見
- ゴシッププロトコル: Hub間で相互に情報を交換

**動作**:
- 新しいHubを発見したら、直接接続を試行
- 接続が確立できない場合（NAT/Firewall）、Root経由のルーティング

## 5. Network IDとIPv6プライベートネットワーク

### 5.1 Network ID

**概念**:
- Unisonネットワークを識別する一意のID
- Rootが発行・管理
- 128ビットのUUID形式

**用途**:
- ネットワークの分離（複数の独立したUnisonネットワークを運用可能）
- IPv6アドレスの生成
- 認証・認可のスコープ

### 5.2 IPv6プライベートユーザネットワーク

**設計**:
- **ULA (Unique Local Address)** を使用: `fd00::/8`
- **アドレス構造**:
  ```
  fd00:network_id:node_type:node_id::

  fd00 : Unique Local Address prefix
  network_id : Network IDから生成（40bit）
  node_type : ノードタイプ（Root=0, Hub=1, Agent=2）
  node_id : ノード固有ID（64bit）
  ```

**例**:
```
Root:  fd00:1234:5678:0000:0000:0000:0000:0001
Hub:   fd00:1234:5678:0001:0000:0000:0000:0042
Agent: fd00:1234:5678:0002:0000:0000:0000:1337
```

**利点**:
- **完全なプライベートネットワーク**: 外部から隔離
- **グローバル到達性**: Unison内部では完全にルーティング可能
- **衝突回避**: Network IDによる一意性保証
- **セキュリティ**: ネットワーク境界の明確化

### 5.3 アドレス割り当てフロー

```
Agent起動
  ↓
Hub接続
  ↓
Hubを経由してRootに登録要求
  ↓
Root: network_idを含むIPv6アドレスを発行
  ↓
Agent: 発行されたIPv6アドレスを受け取り、Unisonネットワーク内で使用
```

## 6. QUIC通信戦略

### 6.1 ストリームインデックスの予約

QUICコネクション内のストリームは、以下のように用途別に予約されます：

```
Stream Index 1-99:   システム予約（Unison Protocol内部使用）
Stream Index 100以上: ユーザ自由使用（アプリケーション定義）
```

#### システム予約ストリーム（1-99）

| Stream Index | 用途 | 説明 |
|-------------|------|------|
| 1 | **接続確立制御** | ノード間接続時の初期ハンドシェイク・認証 |
| 2 | ハートビート/Keepalive | 接続維持と死活監視 |
| 3 | ノード情報交換 | メタデータ、capability negotiation |
| 4-9 | 制御メッセージ | 設定変更、状態通知など |
| 10-19 | ルーティング制御 | Hub/Root間のルーティングテーブル更新 |
| 20-29 | 認証・認可 | 権限管理、トークン更新 |
| 30-99 | 将来の拡張用 | システム機能の追加に予約 |

#### Stream 1: 接続確立制御ストリーム

**特別な役割**:
- ノード間のQUICコネクション確立時に、**最初に開かれる双方向ストリーム**
- システムレベルのハンドシェイクと初期化を担当

**特性**:
- **双方向**: 両ノードが相互に送受信可能
- **順不同**: メッセージは到着順で処理（順序依存なし）
- **安定**: コネクション存続中は常に開いた状態を維持
- **高優先度**: 他のストリームより優先的に処理

**用途の詳細**:
1. **プロトコルバージョンネゴシエーション**
   - Unisonプロトコルバージョンの交換
   - サポート機能の確認

2. **ノード認証**
   - ノードIDの交換
   - Network ID検証
   - 証明書検証（オプション）

3. **初期設定交換**
   - サポートするストリームインデックス範囲
   - 最大メッセージサイズ
   - 優先度設定

4. **接続確立完了通知**
   - ハンドシェイク完了の相互確認
   - 通常通信開始の合図

**メッセージフォーマット例**:
```rust
// Stream 1で交換されるメッセージ（概念図）
struct ConnectionHandshake {
    protocol_version: String,      // "unison/0.1.0"
    node_id: [u8; 32],             // ノード固有ID
    network_id: [u8; 16],          // Network UUID
    node_type: NodeType,           // Agent/Hub/Root
    capabilities: Vec<String>,     // サポート機能リスト
    ipv6_addr: Ipv6Addr,          // Unison IPv6アドレス
}
```

### 6.2 UnisonPacket - QUICストリーム上のデータ単位

QUICストリーム上でやり取りされるデータの基本単位として、**UnisonPacket**を定義します。

#### 構造

```
┌─────────────────────────────────────┐
│         UnisonPacket                 │
├─────────────────────────────────────┤
│  PacketHeader (48 bytes, 固定長)     │ ← ゼロアロケーションで読み取り
│  ├─ version: u8                     │
│  ├─ packet_type: u8                  │
│  ├─ flags: u16                      │
│  ├─ payload_length: u32             │
│  ├─ compressed_length: u32          │
│  ├─ sequence_number: u64            │
│  ├─ timestamp: u64                  │
│  ├─ stream_id: u64                  │
│  ├─ message_id: u64                 │
│  └─ response_to: u64                │
├─────────────────────────────────────┤
│  Payload (可変長)                    │
│  └─ 圧縮 or 非圧縮 (rkyv形式)       │
└─────────────────────────────────────┘
```

#### 設計思想

**ゼロアロケーション原則**:
- 固定長ヘッダー（48 bytes）により、メモリコピーなしで直接読み取り
- `#[repr(C)]`による明確なメモリレイアウト
- バイト配列からヘッダー構造体へ直接キャスト可能

**型安全性**:
- ジェネリクス `UnisonPacket<T: Payloadable>` による型安全なペイロード
- rkyvによるゼロコピーデシリアライゼーション

**自動最適化**:
- 2KB以上のペイロードは自動的にzstd Level 1で圧縮
- 圧縮効果がない場合は非圧縮のまま送信

#### PacketHeader フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| version | u8 | プロトコルバージョン |
| packet_type | u8 | フレームタイプ（Data/Control/Heartbeat/etc） |
| flags | u16 | フラグビット（圧縮/優先度） |
| payload_length | u32 | 元のペイロード長 |
| compressed_length | u32 | 圧縮後のサイズ（0=非圧縮） |
| sequence_number | u64 | シーケンス番号 |
| timestamp | u64 | タイムスタンプ（μs） |
| stream_id | u64 | QUICストリームID（1-99: システム, 100+: ユーザ） |
| message_id | u64 | メッセージID（このメッセージの一意な識別子） |
| response_to | u64 | 応答先メッセージID（0=Request/Oneway, >0=Response） |

#### 使用例

```rust
use unison::frame::{UnisonPacket, StringPayload};

// フレーム作成
let payload = StringPayload::from_string("Hello, Unison!");
let frame = UnisonPacket::builder()
    .with_stream_id(100)
    .with_sequence(1)
    .build(payload)?;

// QUICストリームで送信
let bytes = frame.to_bytes();
send_stream.write_all(&bytes).await?;

// 受信側：ゼロアロケーションで読み取り
let bytes = recv_stream.read_to_end().await?;
let frame = UnisonPacket::<StringPayload>::from_bytes(&bytes)?;

// ヘッダーのみ読み取り（ペイロード未処理）
let header = frame.header()?;
println!("Stream: {}, Seq: {}", header.stream_id, header.sequence_number);

// 必要に応じてペイロードを取得
let payload = frame.payload()?;
```

#### メッセージタイプの識別

UnisonPacketは、`message_id`と`response_to`フィールドを使って、3種類のメッセージタイプを識別します：

| メッセージタイプ | message_id | response_to | 説明 |
|---------------|-----------|------------|------|
| **Request** | > 0 | = 0 | リクエストメッセージ（応答を期待） |
| **Response** | > 0 | > 0 | レスポンスメッセージ（`response_to`が応答先のRequestのmessage_id） |
| **Oneway** | = 0 | = 0 | 一方向メッセージ（応答不要） |

**使用例**:

```rust
// Request: message_id=123, response_to=0
let request = UnisonPacket::builder()
    .with_message_id(123)
    .with_response_to(0)
    .build(payload)?;

assert!(request.header()?.is_request());

// Response: message_id=456, response_to=123 (Requestのmessage_idを参照)
let response = UnisonPacket::builder()
    .with_message_id(456)
    .with_response_to(123)  // 元のRequestのID
    .build(payload)?;

assert!(response.header()?.is_response());
println!("Response to request ID: {}", response.header()?.response_to);

// Oneway: message_id=0, response_to=0
let oneway = UnisonPacket::builder()
    .with_message_id(0)
    .with_response_to(0)
    .build(payload)?;

assert!(oneway.header()?.is_oneway());
```

**利点**:
- **ヘッダーだけで判別可能**: ペイロードを読む前にメッセージタイプを識別
- **Request-Response紐付け**: `response_to`でどのRequestへの応答かを追跡
- **トレーシング**: `message_id`でメッセージフロー全体を追跡可能

#### ストリームとの関係

- **各QUICストリームは、複数のUnisonPacketを連続して送信可能**
- Stream 1（接続確立制御）では、ハンドシェイク用の特殊なFrameをやり取り
- Stream 2-99（システム）では、制御用のFrameをやり取り
- Stream 100+（ユーザ）では、アプリケーション定義のFrameをやり取り
- **Request/Response**: 同じストリーム上でやり取りすることを推奨（順序保証のため）
- **Oneway**: 任意のストリームで送信可能

### 6.3 ストリームとデータグラムの使い分け

QUICの2つの通信チャネルを、用途に応じて使い分けます：

#### 双方向ストリーム（Bidirectional Streams）

**用途**:
- **システムストリーム (1-99)**: 制御メッセージ、認証、設定
- **ユーザストリーム (100+)**: RPC、ファイル転送、状態同期

**特徴**:
- 順序保証
- 信頼性（再送保証）
- フロー制御
- 優先度設定可能

**優先度設計**:
```
Priority 0 (最高): Stream 1（接続確立制御）
Priority 1:        システムストリーム（2-99）
Priority 2:        高優先度ユーザストリーム
Priority 3:        通常ユーザストリーム
Priority 4-7:      アプリケーション定義
```

#### データグラム（Unreliable Datagrams）

**用途**:
- メディアストリーミング（音声、動画）
- リアルタイムゲーム通信
- センサーデータ配信
- ログ配信（ベストエフォート）

**特徴**:
- 順序保証なし
- 再送なし
- 最低レイテンシー
- 大量データ配信に最適

### 6.2 コネクション管理

**単一コネクション原則**:
- ノード間は原則として**1つのQUICコネクション**のみ確立
- そのコネクション上で、複数のストリームとデータグラムを多重化

**利点**:
- コネクション確立コストの削減
- Head-of-Line Blockingの回避（ストリーム独立性）
- 効率的な帯域利用
- NAT/Firewallトラバーサルの簡素化

### 6.3 パフォーマンス最適化

- **0-RTT再接続**: セッション再開時のレイテンシー削減
- **コネクションマイグレーション**: IPアドレス変更時も接続維持
- **適応的フロー制御**: ネットワーク状況に応じた動的調整
- **BBR輻輳制御**: 最大スループット実現
- **Early Data**: TLSハンドシェイク中のデータ送信

## 7. 障害時の動作

### 7.1 Hub障害

**Agentの動作**:
1. 接続断を検知
2. 他のHubへの再接続を試行（Hubディスカバリー再実行）
3. Hub未発見の場合、新しいHub serviceを起動

**他のHubの動作**:
- 障害Hubに接続していたAgentを受け入れ
- Rootに障害を報告

### 7.2 Root障害

**重大な障害**: Rootは単一なので、ネットワーク全体に影響

**Hubの動作**:
1. Root切断を検知
2. 孤立モードに移行
3. Root再接続を定期的に試行
4. Hub間の直接通信を維持（既存の接続は継続）

**復旧時**:
- Rootが復旧したら、Hubは自動的に再接続
- ネットワーク状態の再同期

**将来の拡張**: Root冗長化（Hot Standby）の検討

### 7.3 Agent障害

- Hubは接続断を検知し、リソースをクリーンアップ
- 他のノードへの影響は最小限

## 8. セキュリティ

### 8.1 トランスポートレベル

- **TLS 1.3**: QUICに組み込み済み
- **前方秘匿性**: セッションキーの定期更新
- **証明書検証**: 相互認証（オプション）

### 8.2 ネットワークレベル

- **Network ID検証**: 不正なネットワークからの接続を拒否
- **ノード認証**: Rootによる認証（実装依存）
- **アクセス制御**: Rootがポリシーを管理

## 9. 今後の拡張

### 9.1 短期計画
- [ ] NAT traversal / ホールパンチング実装
- [ ] Root冗長化（Hot Standby）
- [ ] DHT-based discovery
- [ ] メトリクス収集・監視システム

### 9.2 長期計画
- [ ] マルチリージョンRoot（地理的分散）
- [ ] エッジコンピューティング対応
- [ ] P2P直接通信の最適化（Hubバイパス）
- [ ] カスタムルーティングポリシー

## 10. 関連ドキュメント

### 仕様書
- [spec/02: RPCプロトコル](../02-protocol-rpc/SPEC.md) - KDLベースRPC層

### 設計ドキュメント
- [アーキテクチャ設計](../../design/architecture.md) - 全体アーキテクチャの実装詳細
- [パケット実装仕様](../../design/packet.md) - UnisonPacket実装詳細

### 実装ガイド
- [Quinn APIガイド](../../guides/quinn-stream-api.md) - QUIC実装の使い方

### 実装
- [unison-protocol実装](../../crates/unison-protocol/) - パケット実装
- [unison-network実装](../../crates/unison-network/) - ネットワーク層実装

---

**仕様バージョン**: 0.1.0-draft
**最終更新**: 2025-11-05
**ステータス**: Draft
