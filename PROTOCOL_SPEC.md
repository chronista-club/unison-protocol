# 🎵 Unison Protocol 技術仕様書

**バージョン**: 1.0.0  
**日付**: 2025-01-04  
**ステータス**: ドラフト

## 目次

- [概要](#概要)
- [目標と設計原則](#目標と設計原則)
- [プロトコル構造](#プロトコル構造)
- [コアプロトコルメッセージ](#コアプロトコルメッセージ)
- [スキーマ定義言語](#スキーマ定義言語)
- [ネットワークプロトコル](#ネットワークプロトコル)
- [コード生成](#コード生成)
- [セキュリティ考慮事項](#セキュリティ考慮事項)
- [パフォーマンス特性](#パフォーマンス特性)
- [バージョニングと互換性](#バージョニングと互換性)
- [実装ガイドライン](#実装ガイドライン)
- [将来の機能拡張](#将来の機能拡張)
- [付録](#付録)

## 概要

Unison Protocolは、クライアントとサーバー間でのリアルタイム双方向通信を実現するために設計された、KDLベースの型安全通信フレームワークです。このプロトコルは、強力な型安全性と包括的なエラーハンドリングを維持しながら、複数のプログラミング言語向けの自動コード生成を可能にします。

## 目標と設計原則

### 主要目標

1. **型安全性**: 対応する全ての言語でコンパイル時・実行時の型チェックを保証
2. **開発者体験**: 包括的なエラーメッセージを持つ、シンプルで直感的なAPI の提供
3. **多言語サポート**: 複数プログラミング言語向けのイディオマティックなコード生成
4. **リアルタイム通信**: 低レイテンシーの双方向通信をサポート
5. **拡張性**: 新しいメソッド、型、プロトコルの簡単な追加を可能にする

### 設計原則

- **スキーマファースト**: プロトコル定義が実装を牽引し、逆ではない
- **非同期優先**: async/awaitパターンを基盤として構築
- **エラー耐性**: 包括的なエラーハンドリングと回復メカニズム
- **トランスポート非依存**: 複数のトランスポートレイヤー（WebSocket、TCP等）をサポート
- **バージョン互換性**: 前方・後方互換性をサポート

## プロトコル構造

### 階層構造

```
Protocol（プロトコル）
├── Metadata（メタデータ） (name, version, namespace, description)
├── Types（型） (カスタム型定義)
├── Messages（メッセージ） (構造化データ定義)
└── Services（サービス）
    └── Methods（メソッド）
```

### プロトコル定義フォーマット

Unison Protocolは、スキーマ定義にKDL（KDL Document Language）を使用します：

```kdl
protocol "service-name" version="1.0.0" {
    namespace "com.example.service"
    description "サービス説明"
    
    // 型定義
    // メッセージ定義
    // サービス定義
}
```

## コアプロトコルメッセージ

### UnisonMessage

全てのUnison Protocol通信における標準メッセージ形式：

```rust
struct UnisonMessage {
    id: String,           // 一意メッセージ識別子
    method: String,       // RPCメソッド名
    payload: JsonValue,   // JSON形式のメソッドパラメータ
    timestamp: DateTime,  // メッセージ作成タイムスタンプ
    version: String,      // プロトコルバージョン（デフォルト: "1.0.0"）
}
```

### UnisonResponse

標準レスポンス形式：

```rust
struct UnisonResponse {
    id: String,                    // 対応するリクエストメッセージID
    success: bool,                 // 操作成功インジケーター
    payload: Option<JsonValue>,    // JSON形式のレスポンスデータ
    error: Option<String>,         // 操作失敗時のエラーメッセージ
    timestamp: DateTime,           // レスポンス作成タイムスタンプ
    version: String,               // プロトコルバージョン
}
```

### UnisonError

構造化されたエラー情報：

```rust
struct UnisonError {
    code: String,                  // エラーコード識別子
    message: String,               // 人間が読めるエラーメッセージ
    details: Option<JsonValue>,    // 追加のエラーコンテキスト
    timestamp: DateTime,           // エラー発生タイムスタンプ
}
```

## スキーマ定義言語

### 基本型

Unison Protocolは以下の基本型をサポートします：

| 型 | 説明 | Rustマッピング | TypeScriptマッピング |
|------|-------------|--------------|---------------------|
| `string` | UTF-8テキスト | `String` | `string` |
| `number` | 数値 | `f64` | `number` |
| `bool` | 真偽値 | `bool` | `boolean` |
| `timestamp` | ISO-8601日時 | `DateTime<Utc>` | `string` |
| `json` | 任意のJSON | `serde_json::Value` | `any` |
| `array` | アイテムのリスト | `Vec<T>` | `T[]` |

### フィールド修飾子

- `required=true`: フィールドが必須（デフォルト: false）
- `default=value`: オプションフィールドのデフォルト値
- `description="text"`: フィールドドキュメンテーション

### スキーマ例

```kdl
protocol "user-management" version="1.0.0" {
    namespace "com.example.users"
    description "ユーザー管理サービス"
    
    message "User" {
        description "ユーザーアカウント情報"
        field "id" type="string" required=true description="一意のユーザー識別子"
        field "username" type="string" required=true description="ユーザーログイン名"
        field "email" type="string" required=true description="ユーザーメールアドレス"
        field "created_at" type="timestamp" required=true description="アカウント作成時刻"
        field "is_active" type="bool" required=false default=true description="アカウント有効ステータス"
    }
    
    service "UserService" {
        description "ユーザーアカウント管理操作"
        
        method "create_user" {
            description "新しいユーザーアカウントを作成"
            request {
                field "username" type="string" required=true
                field "email" type="string" required=true
                field "password" type="string" required=true
            }
            response {
                field "user" type="User" required=true
                field "session_token" type="string" required=true
            }
        }
        
        method "get_user" {
            description "IDによってユーザー情報を取得"
            request {
                field "user_id" type="string" required=true
            }
            response {
                field "user" type="User" required=true
            }
        }
        
        method "list_users" {
            description "オプションフィルタリング付きでユーザーをリスト"
            request {
                field "filter" type="string" required=false
                field "limit" type="number" required=false default=50
                field "offset" type="number" required=false default=0
            }
            response {
                field "users" type="array" item_type="User" required=true
                field "total_count" type="number" required=true
            }
        }
    }
}
```

## ネットワークプロトコル

### トランスポートレイヤー

Unison Protocolはトランスポート非依存ですが、主にWebSocket通信用に設計されています：

- **WebSocket**: リアルタイム双方向通信（主要）
- **TCP**: 直接ソケット通信（計画中）
- **HTTP**: リクエスト・レスポンスパターン（計画中）

### メッセージフロー

1. **接続確立**
   - クライアントがサーバーへの接続を開始
   - バージョンネゴシエーション用のオプションハンドシェイク交換

2. **メソッド呼び出し**
   - クライアントがメソッド名とパラメータと共に`UnisonMessage`を送信
   - サーバーがリクエストを処理し、`UnisonResponse`を送信
   - エラーは`success: false`の`UnisonResponse`として返される

3. **接続管理**
   - 接続ヘルスのためのハートビート/ping メカニズム
   - 接続の正常な切断ハンドリング

### エラーハンドリング

#### クライアントサイドエラー
- 接続失敗
- タイムアウトエラー
- シリアライゼーション/デシリアライゼーションエラー
- プロトコルバージョン不整合

#### サーバーサイドエラー
- メソッドが見つからない
- 無効なパラメータ
- 処理失敗
- リソース制限

#### エラーレスポンス形式

```json
{
  "id": "request-message-id",
  "success": false,
  "error": "メソッドが見つかりません: unknown_method",
  "timestamp": "2025-01-04T10:30:00Z",
  "version": "1.0.0"
}
```

## コード生成

### Rustコード生成

生成されるRustコードには以下が含まれます：

- **型定義**: Serde注釈付きの構造体
- **クライアントトレイト**: 各サービスメソッドの非同期メソッド
- **サーバートレイト**: メソッドのハンドラー登録
- **検証**: リクエスト/レスポンス検証ロジック

生成コードの例：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user: User,
    pub session_token: String,
}

#[async_trait]
pub trait UserServiceClient {
    async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse, NetworkError>;
    async fn get_user(&self, user_id: String) -> Result<User, NetworkError>;
    async fn list_users(&self, filter: Option<String>, limit: Option<i32>, offset: Option<i32>) -> Result<ListUsersResponse, NetworkError>;
}
```

### TypeScriptコード生成（計画中）

生成されるTypeScriptコードには以下が含まれる予定です：

- **インターフェース定義**: すべての型のTypeScriptインターフェース
- **クライアントクラス**: Promiseベースのクライアント実装
- **型ガード**: 実行時型検証
- **エラー型**: 構造化されたエラーハンドリング

## セキュリティ考慮事項

### 認証と認可

- プロトコルレベルの認証は未指定（トランスポートレイヤーの責任）
- カスタムハンドラーを通じたサービスレベルの認可
- アプリケーション固有トークンを通じたセッション管理

### 入力検証

- 必須フィールドの自動検証
- 全パラメータの型チェック
- ハンドラー実装を通じたカスタム検証

### トランスポートセキュリティ

- 本番使用にはTLS/WSSを推奨
- 証明書検証とピン留め
- 接続暗号化と完全性

## パフォーマンス特性

### メッセージサイズ

- JSONベースのシリアライゼーション
- 典型的なメッセージオーバーヘッド: 100-200バイト
- ペイロードサイズはトランスポートレイヤーによって制限

### レイテンシー

- WebSocket: サブミリ秒のプロトコルオーバーヘッド
- ネットワークレイテンシーが全体的なパフォーマンスを決定
- 非同期処理によってブロッキング操作を排除

### スループット

- トランスポートレイヤーとハンドラー実装によって制限
- 非同期ランタイムを通じた同時リクエストハンドリング
- 高負荷シナリオ向けの接続プール

## バージョニングと互換性

### プロトコルバージョニング

- セマンティックバージョニング（MAJOR.MINOR.PATCH）
- プロトコル定義で指定されるバージョン
- ハンドシェイク時のバージョンネゴシエーション

### 後方互換性

- 新しいオプションフィールド: 互換
- 新しい必須フィールド: 破壊的変更
- 新しいメソッド: 互換
- メソッドシグネチャーの変更: 破壊的変更

### 前方互換性

- デシリアライゼーション時に不明フィールドは無視
- 不明メソッドは「メソッドが見つかりません」エラーを返す
- バージョン不整合ハンドリング

## 実装ガイドライン

### クライアント実装

1. **接続管理**: 自動再接続、接続プール
2. **リクエスト相関**: メッセージIDを使ったリクエストとレスポンスのマッチング
3. **エラーハンドリング**: 適切なエラー伝播とユーザーフィードバック
4. **タイムアウトハンドリング**: リクエストタイムアウトとリトライロジック

### サーバー実装

1. **ハンドラー登録**: 型安全なハンドラー登録
2. **同時処理**: 非同期リクエスト処理
3. **リソース管理**: 接続制限とクリーンアップ
4. **ログとモニタリング**: リクエスト/レスポンスログとメトリクス

## 将来の機能拡張

### 計画中の機能

- **ストリーミングサポート**: サーバー送信イベントと双方向ストリーミング
- **スキーマ進化**: 実行時スキーマ更新とマイグレーション
- **圧縮**: 大きなペイロード向けのメッセージ圧縮
- **バッチ操作**: 単一リクエストでの複数操作

### 言語サポート拡張

- TypeScript クライアント・サーバー生成の完成

## 付録

### 参考資料

- [KDL仕様](https://kdl.dev/)
- [WebSocketプロトコル (RFC 6455)](https://tools.ietf.org/html/rfc6455)
- [JSONスキーマ](https://json-schema.org/)

### 変更ログ

| バージョン | 日付 | 変更内容 |
|---------|------|---------|
| 1.0.0 | 2025-01-04 | 初期仕様 |

---

*この仕様書は生きたドキュメントであり、プロトコルの進化と共に更新されます。*