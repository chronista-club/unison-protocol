//! # Unison Protocol
//!
//! **Unison Protocol** は、KDLベースの型安全な通信フレームワークで、
//! 複数言語向けの自動コード生成によるシームレスな分散型ノード間通信を実現します。
//!
//! ## 機能
//!
//! - **型安全な通信**: KDLプロトコル定義からの自動コード生成
//! - **多言語サポート**: Rust、TypeScriptなど複数言語向けのノード実装コード生成
//! - **QUICプロトコル**: 高速で信頼性の高い双方向ストリーム通信
//! - **スキーマ検証**: コンパイル時およびランタイムでのプロトコル検証
//! - **非同期ファースト**: async/awaitサポートを基盤から組み込み
//!
//! ## クイックスタート
//!
//! ```rust,no_run
//! # use anyhow::Result;
//! # #[tokio::main]
//! # async fn main() -> Result<()> {
//! use unison_protocol::{UnisonProtocol, UnisonServer, UnisonServerExt, NetworkError};
//!
//! // プロトコルスキーマを読み込み
//! let mut protocol = UnisonProtocol::new();
//! // protocol.load_schema(include_str!("../schemas/ping_pong.kdl"))?;
//!
//! // サーバーを作成
//! let mut server = protocol.create_server();
//! server.register_handler("ping", |payload| {
//!     // pingリクエストを処理
//!     Ok(serde_json::json!({"message": "pong"})) as Result<serde_json::Value, NetworkError>
//! });
//! // server.listen("127.0.0.1:8080").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## コア概念
//!
//! - **Protocol**: サービス、メッセージ、型を定義するトップレベルコンテナ
//! - **Service**: リクエスト/レスポンス定義を持つRPCメソッドの集合
//! - **Message**: 型付きフィールドを持つ構造化データ型
//! - **Method**: サービス内の個別RPCエンドポイント
//!
//! ## 生成コード
//!
//! プロトコル定義は、ビルドプロセス中に自動的に強く型付けされた
//! 分散ノード実装コードにコンパイルされます。

pub mod codegen;
pub mod network;
pub mod parser;

// プロトコル定義のコアモジュール
pub mod core;

// フレーム層モジュール
pub mod frame;

// CGPベースのコンテキストモジュール
pub mod context;

// よく使用される型と関数のprelude
pub mod prelude;

// 生成コードの再エクスポート
pub mod generated {
    // build.rsによって生成される
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

// preludeの型を内部で使用
use codegen::{CodeGenerator, RustGenerator, TypeScriptGenerator};
use parser::{ParseError as UnisonParseError, ParsedSchema, SchemaParser};

// よく使用されるトレイトとクライアント/サーバーの再エクスポート
pub use network::{
    NetworkError, ProtocolClient, ProtocolServer, UnisonClient, UnisonServer, UnisonServerExt,
};

/// Unison Protocolのメインエントリポイント
pub struct UnisonProtocol {
    schemas: Vec<ParsedSchema>,
    parser: SchemaParser,
}

impl UnisonProtocol {
    /// 新しいUnison Protocolインスタンスを作成
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            parser: SchemaParser::new(),
        }
    }

    /// KDL文字列からプロトコルスキーマを読み込み
    pub fn load_schema(&mut self, schema: &str) -> Result<(), UnisonParseError> {
        let parsed = self.parser.parse(schema)?;
        self.schemas.push(parsed);
        Ok(())
    }

    /// 読み込んだスキーマからRustコードを生成
    pub fn generate_rust_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = RustGenerator::new();
        let type_registry = crate::parser::TypeRegistry::new(); // 一時的な空のレジストリ
        let mut code = String::new();

        for schema in &self.schemas {
            code.push_str(&generator.generate(schema, &type_registry)?);
            code.push('\n');
        }

        Ok(code)
    }

    /// 読み込んだスキーマからTypeScriptコードを生成
    pub fn generate_typescript_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let generator = TypeScriptGenerator::new();
        let type_registry = crate::parser::TypeRegistry::new(); // 一時的な空のレジストリ
        let mut code = String::new();

        for schema in &self.schemas {
            code.push_str(&generator.generate(schema, &type_registry)?);
            code.push('\n');
        }

        Ok(code)
    }

    /// 新しいUnisonクライアントを作成
    pub fn create_client(&self) -> Result<ProtocolClient, anyhow::Error> {
        ProtocolClient::new_default()
    }

    /// 新しいUnisonサーバーを作成
    pub fn create_server(&self) -> ProtocolServer {
        ProtocolServer::new()
    }
}

impl Default for UnisonProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unison_protocol_creation() {
        let protocol = UnisonProtocol::new();
        assert_eq!(protocol.schemas.len(), 0);
    }

    #[test]
    fn test_parse_schema() {
        let schema = r#"
protocol "test" version="1.0.0" {
    service "TestService" {
        method "test_method" {
            request {
                field "id" type="string"
            }
            response {
                field "id" type="string"
            }
        }
    }
}
        "#;

        let mut protocol = UnisonProtocol::new();
        let result = protocol.load_schema(schema);
        if let Err(e) = &result {
            eprintln!("パースエラー: {:?}", e);
        }
        assert!(result.is_ok());
        assert_eq!(protocol.schemas.len(), 1);
    }

    #[test]
    fn test_client_server_creation() {
        let protocol = UnisonProtocol::new();
        let _client = protocol.create_client().unwrap();
        let _server = protocol.create_server();
        // パニックが発生しなければテスト成功
    }
}
