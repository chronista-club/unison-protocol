//! よく使用される型と関数のprelude
//!
//! このモジュールは、Unison Protocolで頻繁に使用される型やトレイトを
//! 一括でインポートできるようにします。
//!
//! # 使用例
//!
//! ```rust
//! use unison_protocol::prelude::*;
//! ```

// パーサー関連
pub use crate::parser::{SchemaParser, ParsedSchema};

// コードジェネレータ関連
pub use crate::codegen::{RustGenerator, TypeScriptGenerator, CodeGenerator};

// ネットワーク関連
pub use crate::network::{
    ProtocolClient, ProtocolServer,
    UnisonClient, UnisonServer, UnisonServerExt
};

// エラー型
pub use crate::parser::ParseError as UnisonParseError;
pub use crate::network::NetworkError as UnisonNetworkError;

// メインエントリポイント
pub use crate::UnisonProtocol;
